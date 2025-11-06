//! Traverse trait implementation for modern transformer

use oxc_allocator::Box;
use oxc_allocator::Vec as OxcVec;
use oxc_ast::ast::*;
use oxc_span::{Atom, SPAN};
use oxc_traverse::{Traverse, TraverseCtx};

use crate::template::build_template_with_options;

use super::DomExpressions;

impl<'a> Traverse<'a, ()> for DomExpressions<'a> {
    fn exit_expression(&mut self, expr: &mut Expression<'a>, _ctx: &mut TraverseCtx<'a, ()>) {
        // Transform JSX elements to modern format
        if let Expression::JSXElement(jsx_elem) = expr {
            if let Some(transformed) = self.transform_jsx_element_modern(jsx_elem) {
                *expr = transformed;
            }
        }
    }

    fn exit_program(&mut self, program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a, ()>) {
        // Inject imports and template declarations at the top when templates exist
        // Modern approach: Just import runtime functions, use them directly
        // No complex helpers - clean, transformer-friendly, runtime-friendly
        if !self.templates.is_empty() && !self.helper_injected {
            let mut new_stmts = Vec::new();
            
            // 1. Add import statement (just runtime imports, no helper functions)
            let helper_stmts = self.create_helper_statements();
            new_stmts.extend(helper_stmts);

            // 2. Add template variable declarations
            let template_decls = self.create_template_declarations();
            new_stmts.extend(template_decls);

            // 3. Prepend new statements to the program
            let existing_stmts =
                std::mem::replace(&mut program.body, OxcVec::new_in(self.allocator));

            // Create new statement list with injected statements first
            let mut all_stmts = new_stmts;
            all_stmts.extend(existing_stmts);

            // Replace program body
            program.body = OxcVec::from_iter_in(all_stmts, self.allocator);
            
            // Mark as injected to prevent duplicates
            self.helper_injected = true;
        }
    }
}

impl<'a> DomExpressions<'a> {
    /// Transform a JSX element to modern format with full feature support
    fn transform_jsx_element_modern(&mut self, jsx_elem: &JSXElement<'a>) -> Option<Expression<'a>> {
        use oxc_allocator::CloneIn;
        use crate::template::SlotType;
        
        // Build template from JSX
        let template = build_template_with_options(jsx_elem, Some(&self.options));

        // Get HTML (minimized if opt feature is enabled)
        #[cfg(feature = "opt")]
        let html = crate::opt::minimizer::minimize_template(&template.html, &self.options);
        #[cfg(not(feature = "opt"))]
        let html = template.html.clone();

        // Get or create template variable
        let template_var = self.get_template_var(&html);

        // Track this template in optimizer (if opt feature is enabled)
        #[cfg(feature = "opt")]
        self.optimizer.record_template(template.clone());
        
        let has_dynamic_content = !template.dynamic_slots.is_empty();
        
        if !has_dynamic_content {
            // Simple static template - just clone and return
            self.templates.push(template);
            
            let mut statements = Vec::new();
            let clone_call = self.create_clone_call(self.allocator.alloc_str(&template_var));
            let el_var = self.allocator.alloc_str("_el$");

            let declarator = VariableDeclarator {
                span: SPAN,
                kind: VariableDeclarationKind::Const,
                id: BindingPattern {
                    kind: BindingPatternKind::BindingIdentifier(Box::new_in(
                        BindingIdentifier {
                            span: SPAN,
                            name: Atom::from(el_var),
                            symbol_id: Default::default(),
                        },
                        self.allocator,
                    )),
                    type_annotation: None,
                    optional: false,
                },
                init: Some(clone_call),
                definite: false,
            };

            let mut declarators = OxcVec::new_in(self.allocator);
            declarators.push(declarator);

            statements.push(Statement::VariableDeclaration(Box::new_in(
                VariableDeclaration {
                    span: SPAN,
                    kind: VariableDeclarationKind::Const,
                    declarations: declarators,
                    declare: false,
                },
                self.allocator,
            )));

            let iife = self.create_iife(statements, el_var);
            Some(iife)
        } else {
            // Has dynamic content - extract expressions and generate binding code
            let mut expressions = Vec::new();
            self.extract_expressions_from_jsx(jsx_elem, &mut expressions);
            
            self.templates.push(template.clone());
            
            // Generate statements for the IIFE
            let mut statements = Vec::new();
            
            // const _el$ = _tmpl$();
            let clone_call = self.create_clone_call(self.allocator.alloc_str(&template_var));
            let el_var = self.allocator.alloc_str("_el$");

            let declarator = VariableDeclarator {
                span: SPAN,
                kind: VariableDeclarationKind::Const,
                id: BindingPattern {
                    kind: BindingPatternKind::BindingIdentifier(Box::new_in(
                        BindingIdentifier {
                            span: SPAN,
                            name: Atom::from(el_var),
                            symbol_id: Default::default(),
                        },
                        self.allocator,
                    )),
                    type_annotation: None,
                    optional: false,
                },
                init: Some(clone_call),
                definite: false,
            };

            let mut declarators = OxcVec::new_in(self.allocator);
            declarators.push(declarator);

            statements.push(Statement::VariableDeclaration(Box::new_in(
                VariableDeclaration {
                    span: SPAN,
                    kind: VariableDeclarationKind::Const,
                    declarations: declarators,
                    declare: false,
                },
                self.allocator,
            )));
            
            // Generate binding code for each dynamic slot
            let mut expr_index = 0;
            for slot in &template.dynamic_slots {
                if expr_index >= expressions.len() {
                    break;
                }
                
                let element_expr = self.navigate_to_element(el_var, &slot.path);
                
                match &slot.slot_type {
                    SlotType::TextContent => {
                        self.add_import("insert");
                        
                        // For text content, use the slot's path (parent element) and marker
                        let parent_expr = self.navigate_to_element(el_var, &slot.path);
                        let marker_expr = if let Some(marker_path) = &slot.marker_path {
                            Some(self.navigate_to_element(el_var, marker_path))
                        } else {
                            None
                        };
                        
                        let insert_stmt = self.create_insert_call(
                            parent_expr,
                            expressions[expr_index].clone_in(self.allocator),
                            marker_expr,
                        );
                        statements.push(insert_stmt);
                        expr_index += 1;
                    }
                    SlotType::Attribute(attr_name) => {
                        self.add_import("setAttribute");
                        self.add_import("effect");
                        let attr_stmt = self.create_set_attribute_effect(
                            element_expr,
                            self.allocator.alloc_str(attr_name),
                            expressions[expr_index].clone_in(self.allocator),
                        );
                        statements.push(attr_stmt);
                        expr_index += 1;
                    }
                    SlotType::EventHandler(event_name) => {
                        self.add_import("addEventListener");
                        let event_stmt = self.create_event_listener(
                            element_expr,
                            self.allocator.alloc_str(event_name),
                            expressions[expr_index].clone_in(self.allocator),
                        );
                        statements.push(event_stmt);
                        expr_index += 1;
                    }
                    _ => {
                        // TODO: Implement other slot types
                        expr_index += 1;
                    }
                }
            }
            
            let iife = self.create_iife(statements, el_var);
            Some(iife)
        }
    }
    
    /// Extract expressions from JSX element
    fn extract_expressions_from_jsx(&self, jsx_elem: &JSXElement<'a>, expressions: &mut Vec<Expression<'a>>) {
        use oxc_allocator::CloneIn;

        for attr in &jsx_elem.opening_element.attributes {
            if let JSXAttributeItem::Attribute(attr) = attr {
                if let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value {
                    match &container.expression {
                        JSXExpression::StringLiteral(_)
                        | JSXExpression::NumericLiteral(_)
                        | JSXExpression::EmptyExpression(_) => {}
                        expr => {
                            if let Some(expr_ref) = expr.as_expression() {
                                expressions.push(expr_ref.clone_in(self.allocator));
                            }
                        }
                    }
                }
            }
        }

        for child in &jsx_elem.children {
            self.extract_expressions_from_child(child, expressions);
        }
    }

    /// Extract expressions from a JSX child
    fn extract_expressions_from_child(&self, child: &JSXChild<'a>, expressions: &mut Vec<Expression<'a>>) {
        use oxc_allocator::CloneIn;

        match child {
            JSXChild::Element(elem) => {
                self.extract_expressions_from_jsx(elem, expressions);
            }
            JSXChild::ExpressionContainer(container) => {
                match &container.expression {
                    JSXExpression::StringLiteral(_)
                    | JSXExpression::NumericLiteral(_)
                    | JSXExpression::EmptyExpression(_) => {}
                    expr => {
                        if let Some(expr_ref) = expr.as_expression() {
                            expressions.push(expr_ref.clone_in(self.allocator));
                        }
                    }
                }
            }
            JSXChild::Text(_) | JSXChild::Fragment(_) | JSXChild::Spread(_) => {}
        }
    }
}
