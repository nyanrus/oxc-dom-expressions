//! Template and IIFE generation
//!
//! This module contains methods for generating templates and IIFEs:
//! - Template creation and caching
//! - IIFE (Immediately Invoked Function Expression) generation  
//! - Element declarations and references

use oxc_allocator::Box;
use oxc_allocator::Vec as OxcVec;
use oxc_ast::ast::*;
use oxc_span::SPAN;

use crate::template::{SlotType, Template};

use super::DomExpressions;

impl<'a> DomExpressions<'a> {
    /// Create a template call expression
    pub(super) fn create_template_call(
        &self,
        template_var: &'a str,
    ) -> Box<'a, CallExpression<'a>> {
        use crate::options::GenerateMode;
        use oxc_ast::ast::*;

        let is_ssr = self.options.generate == GenerateMode::Ssr;

        if is_ssr {
            let ssr_fn = IdentifierReference {
                span: SPAN,
                name: oxc_span::Atom::from("_$ssr"),
                reference_id: None.into(),
            };

            let template_ident = IdentifierReference {
                span: SPAN,
                name: oxc_span::Atom::from(template_var),
                reference_id: None.into(),
            };

            let mut args = OxcVec::new_in(self.allocator);
            args.push(Argument::Identifier(Box::new_in(
                template_ident,
                self.allocator,
            )));

            let call_expr = CallExpression {
                span: SPAN,
                arguments: args,
                callee: Expression::Identifier(Box::new_in(ssr_fn, self.allocator)),
                optional: false,
                type_arguments: None,
                pure: false,
            };

            Box::new_in(call_expr, self.allocator)
        } else {
            let callee_ident = IdentifierReference {
                span: SPAN,
                name: oxc_span::Atom::from(template_var),
                reference_id: None.into(),
            };
            let callee = Expression::Identifier(Box::new_in(callee_ident, self.allocator));

            let call_expr = CallExpression {
                span: SPAN,
                arguments: OxcVec::new_in(self.allocator),
                callee,
                optional: false,
                type_arguments: None,
                pure: false,
            };

            Box::new_in(call_expr, self.allocator)
        }
    }

    /// Create an IIFE that clones template and applies dynamic bindings
    pub(super) fn create_template_iife_from_expressions(
        &mut self,
        expressions: Vec<Expression<'a>>,
        template: &Template,
        template_var: &str,
    ) -> Box<'a, CallExpression<'a>> {
        use oxc_ast::ast::*;

        let mut body_stmts = OxcVec::new_in(self.allocator);

        let (root_var, elem_decls, path_to_var) =
            self.create_element_declarations(template, template_var);
        body_stmts.push(elem_decls);

        let runtime_stmts = self.create_runtime_calls_from_expressions(
            &expressions,
            template,
            &root_var,
            &path_to_var,
        );
        body_stmts.extend(runtime_stmts);

        let return_stmt = self.create_return_statement(&root_var);
        body_stmts.push(return_stmt);

        let func_body = FunctionBody {
            span: SPAN,
            directives: OxcVec::new_in(self.allocator),
            statements: body_stmts,
        };

        let arrow_fn = ArrowFunctionExpression {
            span: SPAN,
            expression: false,
            r#async: false,
            params: Box::new_in(
                FormalParameters {
                    span: SPAN,
                    kind: FormalParameterKind::ArrowFormalParameters,
                    items: OxcVec::new_in(self.allocator),
                    rest: None,
                },
                self.allocator,
            ),
            body: Box::new_in(func_body, self.allocator),
            type_parameters: None,
            return_type: None,
            scope_id: None.into(),
            pure: false,
            pife: false,
        };

        let call_expr = CallExpression {
            span: SPAN,
            callee: Expression::ArrowFunctionExpression(Box::new_in(arrow_fn, self.allocator)),
            arguments: OxcVec::new_in(self.allocator),
            optional: false,
            type_arguments: None,
            pure: false,
        };

        Box::new_in(call_expr, self.allocator)
    }

    /// Create element reference declarations
    /// Returns (root_var_name, statement, path_to_var_map)
    pub(super) fn create_element_declarations(
        &mut self,
        template: &Template,
        template_var: &str,
    ) -> (
        String,
        Statement<'a>,
        std::collections::HashMap<Vec<String>, String>,
    ) {
        use oxc_ast::ast::*;

        // Generate root element variable (e.g., _el$)
        let root_var = self.generate_root_element_var();
        let mut path_to_var = std::collections::HashMap::new();
        let mut declarators = OxcVec::new_in(self.allocator);

        // First declarator: var _el$ = _tmpl$()
        declarators.push(self.create_root_element_declarator(&root_var, template_var));

        // Collect all paths (including intermediate paths) we need to create
        let mut all_paths = std::collections::HashSet::new();

        // Check if we have any TextContent slots - if so, always create firstChild reference
        // This matches babel plugin behavior for consistency
        let has_text_content = template
            .dynamic_slots
            .iter()
            .any(|slot| matches!(slot.slot_type, SlotType::TextContent));

        if has_text_content {
            // Always create firstChild reference for text content templates
            all_paths.insert(vec!["firstChild".to_string()]);
        }

        for slot in &template.dynamic_slots {
            // Add intermediate paths for slot path
            if !slot.path.is_empty() {
                for i in 1..=slot.path.len() {
                    all_paths.insert(slot.path[..i].to_vec());
                }
            }

            // Add intermediate paths for marker path
            if let Some(marker_path) = &slot.marker_path {
                if !marker_path.is_empty() {
                    for i in 1..=marker_path.len() {
                        all_paths.insert(marker_path[..i].to_vec());
                    }
                }
            }
        }

        // Sort paths by length to ensure we create parent references before children
        let mut sorted_paths: Vec<_> = all_paths.into_iter().collect();
        sorted_paths.sort_by_key(|path| path.len());

        // Generate element references for each path
        for path in sorted_paths {
            let elem_var = self.generate_element_var();
            path_to_var.insert(path.clone(), elem_var.clone());

            // For intermediate references, use the previous reference as base
            let base_var = if path.len() == 1 {
                &root_var
            } else {
                // Get the parent path and its variable
                let parent_path = &path[..path.len() - 1];
                path_to_var.get(parent_path).unwrap_or(&root_var)
            };

            // Create reference from parent
            let single_step_path = vec![path.last().unwrap().clone()];
            declarators.push(self.create_element_ref_declarator(
                &elem_var,
                base_var,
                &single_step_path,
            ));
        }

        let var_decl = VariableDeclaration {
            span: SPAN,
            kind: VariableDeclarationKind::Var,
            declarations: declarators,
            declare: false,
        };

        (
            root_var,
            Statement::VariableDeclaration(Box::new_in(var_decl, self.allocator)),
            path_to_var,
        )
    }

    /// Create the root element declarator: var _el$ = _tmpl$()
    pub(super) fn create_root_element_declarator(
        &self,
        root_var: &str,
        template_var: &str,
    ) -> VariableDeclarator<'a> {
        use oxc_ast::ast::*;

        let root_id = BindingPattern {
            kind: BindingPatternKind::BindingIdentifier(Box::new_in(
                BindingIdentifier {
                    span: SPAN,
                    name: Atom::from(self.allocator.alloc_str(root_var)),
                    symbol_id: None.into(),
                },
                self.allocator,
            )),
            type_annotation: None,
            optional: false,
        };

        let template_call = self.create_template_call(self.allocator.alloc_str(template_var));

        VariableDeclarator {
            span: SPAN,
            kind: VariableDeclarationKind::Var,
            id: root_id,
            init: Some(Expression::CallExpression(template_call)),
            definite: false,
        }
    }

    /// Create an element reference declarator: var _el$2 = _el$.firstChild.nextSibling
    pub(super) fn create_element_ref_declarator(
        &self,
        elem_var: &str,
        root_var: &str,
        path: &[String],
    ) -> VariableDeclarator<'a> {
        use oxc_ast::ast::*;

        let elem_id = BindingPattern {
            kind: BindingPatternKind::BindingIdentifier(Box::new_in(
                BindingIdentifier {
                    span: SPAN,
                    name: Atom::from(self.allocator.alloc_str(elem_var)),
                    symbol_id: None.into(),
                },
                self.allocator,
            )),
            type_annotation: None,
            optional: false,
        };

        // Build path expression: _el$.firstChild.nextSibling...
        let mut expr = Expression::Identifier(Box::new_in(
            IdentifierReference {
                span: SPAN,
                name: Atom::from(self.allocator.alloc_str(root_var)),
                reference_id: None.into(),
            },
            self.allocator,
        ));

        for segment in path {
            expr = Expression::StaticMemberExpression(Box::new_in(
                StaticMemberExpression {
                    span: SPAN,
                    object: expr,
                    property: IdentifierName {
                        span: SPAN,
                        name: Atom::from(self.allocator.alloc_str(segment)),
                    },
                    optional: false,
                },
                self.allocator,
            ));
        }

        VariableDeclarator {
            span: SPAN,
            kind: VariableDeclarationKind::Var,
            id: elem_id,
            init: Some(expr),
            definite: false,
        }
    }

    /// Generate unique element variable name
    pub(super) fn create_template_declarations(&self) -> Option<Statement<'a>> {
        use crate::options::GenerateMode;
        use oxc_ast::ast::*;

        if self.template_map.is_empty() {
            return None;
        }

        // Create variable declarators for all templates
        let mut declarators = OxcVec::new_in(self.allocator);

        // Sort template map by variable name to get consistent order (numerically)
        let mut sorted_templates: Vec<_> = self.template_map.iter().collect();
        sorted_templates.sort_by(|a, b| {
            // Extract the numeric part from variable names using compat naming module
            use crate::compat::naming::extract_template_counter;
            let get_num = |name: &str| -> usize { extract_template_counter(name).unwrap_or(0) };
            get_num(a.1).cmp(&get_num(b.1))
        });

        let is_ssr = self.options.generate == GenerateMode::Ssr;

        for (html, var_name) in sorted_templates {
            // Create the binding pattern for the variable
            let id = BindingPattern {
                kind: BindingPatternKind::BindingIdentifier(Box::new_in(
                    BindingIdentifier {
                        span: SPAN,
                        name: Atom::from(self.allocator.alloc_str(var_name)),
                        symbol_id: None.into(),
                    },
                    self.allocator,
                )),
                type_annotation: None,
                optional: false,
            };

            // For SSR mode, just assign string literals
            // For DOM mode, wrap in _$template() call
            let init_expr = if is_ssr {
                // SSR: just a string literal
                let string_lit = StringLiteral {
                    span: SPAN,
                    value: Atom::from(self.allocator.alloc_str(html)),
                    raw: None,
                    lone_surrogates: false,
                };
                Expression::StringLiteral(Box::new_in(string_lit, self.allocator))
            } else {
                // DOM: call _$template with template literal
                let template_element = TemplateElement {
                    span: SPAN,
                    tail: true,
                    value: TemplateElementValue {
                        raw: Atom::from(self.allocator.alloc_str(html)),
                        cooked: Some(Atom::from(self.allocator.alloc_str(html))),
                    },
                    lone_surrogates: false,
                };

                let mut elements = OxcVec::new_in(self.allocator);
                elements.push(template_element);

                let template_literal = TemplateLiteral {
                    span: SPAN,
                    quasis: elements,
                    expressions: OxcVec::new_in(self.allocator),
                };

                // Create call to _$template(...)
                let template_fn = IdentifierReference {
                    span: SPAN,
                    name: Atom::from("_$template"),
                    reference_id: None.into(),
                };

                let mut args = OxcVec::new_in(self.allocator);
                args.push(Argument::TemplateLiteral(Box::new_in(
                    template_literal,
                    self.allocator,
                )));

                let call_expr = CallExpression {
                    span: SPAN,
                    callee: Expression::Identifier(Box::new_in(template_fn, self.allocator)),
                    arguments: args,
                    optional: false,
                    type_arguments: None,
                    pure: true, // Mark as /*#__PURE__*/
                };

                Expression::CallExpression(Box::new_in(call_expr, self.allocator))
            };

            // Create variable declarator
            let declarator = VariableDeclarator {
                span: SPAN,
                kind: VariableDeclarationKind::Var,
                id,
                init: Some(init_expr),
                definite: false,
            };

            declarators.push(declarator);
        }

        // Create variable declaration
        let var_decl = VariableDeclaration {
            span: SPAN,
            kind: VariableDeclarationKind::Var,
            declarations: declarators,
            declare: false,
        };

        Some(Statement::VariableDeclaration(Box::new_in(
            var_decl,
            self.allocator,
        )))
    }
}
