//! Main transformer for DOM expressions

use oxc_allocator::{Allocator, Box};
use oxc_allocator::Vec as OxcVec;
use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};
use oxc_span::{SPAN, Atom};
use std::collections::{HashMap, HashSet};

use crate::optimizer::{TemplateOptimizer, TemplateStats};
use crate::options::DomExpressionsOptions;
use crate::template::{SlotType, Template};
use crate::utils::{is_component, should_delegate_event};

/// The main DOM expressions transformer
pub struct DomExpressions<'a> {
    #[allow(dead_code)]
    allocator: &'a Allocator,
    options: DomExpressionsOptions,
    /// Collection of templates generated during transformation
    templates: Vec<Template>,
    /// Map of template HTML to variable name for deduplication
    template_map: HashMap<String, String>,
    /// Counter for generating unique template variable names
    template_counter: usize,
    /// Counter for generating unique element variable names
    element_counter: usize,
    /// Set of required imports
    required_imports: HashSet<String>,
    /// Set of events that need delegation
    delegated_events: HashSet<String>,
    /// Optimizer for template analysis
    optimizer: TemplateOptimizer,
}

impl<'a> DomExpressions<'a> {
    /// Create a new DOM expressions transformer
    pub fn new(allocator: &'a Allocator, options: DomExpressionsOptions) -> Self {
        Self {
            allocator,
            options,
            templates: Vec::new(),
            template_map: HashMap::new(),
            template_counter: 0,
            element_counter: 0,
            required_imports: HashSet::new(),
            delegated_events: HashSet::new(),
            optimizer: TemplateOptimizer::new(),
        }
    }

    /// Get the current options
    pub fn options(&self) -> &DomExpressionsOptions {
        &self.options
    }

    /// Get template statistics for optimization analysis
    pub fn get_template_stats(&self) -> TemplateStats {
        self.optimizer.get_stats()
    }

    /// Get list of templates that were reused (deduplicated)
    pub fn get_reused_templates(&self) -> Vec<(String, usize)> {
        self.optimizer.get_reused_templates()
    }

    /// Generate a unique template variable name
    fn generate_template_var(&mut self) -> String {
        self.template_counter += 1;
        if self.template_counter == 1 {
            "_tmpl$".to_string()
        } else {
            format!("_tmpl${}", self.template_counter)
        }
    }

    /// Get or create a template variable for given HTML
    fn get_template_var(&mut self, html: &str) -> String {
        if let Some(var) = self.template_map.get(html) {
            var.clone()
        } else {
            let var = self.generate_template_var();
            self.template_map.insert(html.to_string(), var.clone());
            var
        }
    }

    /// Add a required import
    fn add_import(&mut self, name: &str) {
        self.required_imports.insert(name.to_string());
    }

    /// Add an event for delegation
    fn add_delegated_event(&mut self, event: &str) {
        self.delegated_events.insert(event.to_lowercase());
    }

    /// Create a call expression for cloning a template
    fn create_template_call(&self, template_var: &'a str) -> Box<'a, CallExpression<'a>> {
        // Create identifier for the template variable (e.g., "_tmpl$")
        let callee_ident = IdentifierReference {
            span: SPAN,
            name: Atom::from(template_var),
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

    /// Create an IIFE that clones template and applies dynamic bindings
    fn create_template_iife(
        &mut self,
        _jsx_elem: &JSXElement<'a>,
        _template: &Template,
        _template_var: &str,
    ) -> Box<'a, CallExpression<'a>> {
        // For now, just return a simple template call
        // TODO: Implement full IIFE generation with dynamic bindings
        // This requires generating:
        // - Arrow function expression
        // - Variable declarations for element references
        // - Calls to runtime functions (spread, effect, classList, etc.)
        // - Return statement
        
        // Temporary: just create a simple call
        let template_var_str = self.allocator.alloc_str(_template_var);
        self.create_template_call(template_var_str)
    }

    /// Create import statement for runtime functions
    fn create_import_statement(&self) -> Option<Statement<'a>> {
        use oxc_ast::ast::*;
        
        // Create import specifiers for each required import
        let mut specifiers = OxcVec::new_in(self.allocator);
        
        // Sort imports for consistency
        let mut sorted_imports: Vec<_> = self.required_imports.iter().collect();
        sorted_imports.sort();
        
        for import_name in sorted_imports {
            // Create local binding name (e.g., _$template for template)
            let local_name = format!("_${}", import_name);
            let local = BindingIdentifier {
                span: SPAN,
                name: Atom::from(self.allocator.alloc_str(&local_name)),
                symbol_id: None.into(),
            };
            
            // Create imported name
            let imported = ModuleExportName::IdentifierName(IdentifierName {
                span: SPAN,
                name: Atom::from(self.allocator.alloc_str(import_name)),
            });
            
            // Create import specifier
            let specifier = ImportDeclarationSpecifier::ImportSpecifier(
                Box::new_in(
                    ImportSpecifier {
                        span: SPAN,
                        imported,
                        local,
                        import_kind: ImportOrExportKind::Value,
                    },
                    self.allocator,
                )
            );
            
            specifiers.push(specifier);
        }
        
        // Create source string
        let source = StringLiteral {
            span: SPAN,
            value: Atom::from(self.allocator.alloc_str(&self.options.module_name)),
            raw: None,
            lone_surrogates: false,
        };
        
        // Create import declaration
        let import_decl = ImportDeclaration {
            span: SPAN,
            specifiers: Some(specifiers),
            source,
            with_clause: None,
            import_kind: ImportOrExportKind::Value,
            phase: None, // No phase for regular imports
        };
        
        // Wrap in ModuleDeclaration and Statement
        let module_decl = ModuleDeclaration::ImportDeclaration(
            Box::new_in(import_decl, self.allocator)
        );
        
        Some(Statement::from(module_decl))
    }

    /// Create template variable declarations
    fn create_template_declarations(&self) -> Option<Statement<'a>> {
        use oxc_ast::ast::*;
        
        if self.template_map.is_empty() {
            return None;
        }
        
        // Create variable declarators for all templates
        let mut declarators = OxcVec::new_in(self.allocator);
        
        // Sort template map by variable name to get consistent order
        let mut sorted_templates: Vec<_> = self.template_map.iter().collect();
        sorted_templates.sort_by(|a, b| a.1.cmp(b.1));
        
        for (html, var_name) in sorted_templates {
            // Create the binding pattern for the variable
            let id = BindingPattern {
                kind: BindingPatternKind::BindingIdentifier(
                    Box::new_in(
                        BindingIdentifier {
                            span: SPAN,
                            name: Atom::from(self.allocator.alloc_str(var_name)),
                            symbol_id: None.into(),
                        },
                        self.allocator,
                    )
                ),
                type_annotation: None,
                optional: false,
            };
            
            // Create template literal argument (using backticks)
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
            args.push(Argument::TemplateLiteral(
                Box::new_in(template_literal, self.allocator)
            ));
            
            let init_call = CallExpression {
                span: SPAN,
                callee: Expression::Identifier(Box::new_in(template_fn, self.allocator)),
                arguments: args,
                optional: false,
                type_arguments: None,
                pure: true, // Mark as /*#__PURE__*/
            };
            
            // Create variable declarator
            let declarator = VariableDeclarator {
                span: SPAN,
                kind: VariableDeclarationKind::Var,
                id,
                init: Some(Expression::CallExpression(Box::new_in(init_call, self.allocator))),
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
        
        Some(Statement::VariableDeclaration(
            Box::new_in(var_decl, self.allocator)
        ))
    }

    /// Create delegateEvents call
    fn create_delegate_events_call(&self) -> Option<Statement<'a>> {
        use oxc_ast::ast::*;
        
        if self.delegated_events.is_empty() {
            return None;
        }
        
        // Create array of event names
        let mut elements = OxcVec::new_in(self.allocator);
        let mut sorted_events: Vec<_> = self.delegated_events.iter().collect();
        sorted_events.sort();
        
        for event in sorted_events {
            let string_lit = StringLiteral {
                span: SPAN,
                value: Atom::from(self.allocator.alloc_str(event)),
                raw: None,
                lone_surrogates: false,
            };
            elements.push(ArrayExpressionElement::StringLiteral(
                Box::new_in(string_lit, self.allocator)
            ));
        }
        
        let array_expr = ArrayExpression {
            span: SPAN,
            elements,
        };
        
        // Create call to _$delegateEvents([...])
        let fn_name = IdentifierReference {
            span: SPAN,
            name: Atom::from("_$delegateEvents"),
            reference_id: None.into(),
        };
        
        let mut args = OxcVec::new_in(self.allocator);
        args.push(Argument::ArrayExpression(
            Box::new_in(array_expr, self.allocator)
        ));
        
        let call_expr = CallExpression {
            span: SPAN,
            callee: Expression::Identifier(Box::new_in(fn_name, self.allocator)),
            arguments: args,
            optional: false,
            type_arguments: None,
            pure: false,
        };
        
        // Wrap in expression statement
        Some(Statement::ExpressionStatement(
            Box::new_in(
                ExpressionStatement {
                    span: SPAN,
                    expression: Expression::CallExpression(Box::new_in(call_expr, self.allocator)),
                },
                self.allocator,
            )
        ))
    }
}

impl<'a> Traverse<'a, ()> for DomExpressions<'a> {
    fn enter_program(&mut self, _program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a, ()>) {
        // Entry point for the transformation
        // Initialize state for collecting templates and imports
        self.templates.clear();
        self.template_map.clear();
        self.template_counter = 0;
        self.element_counter = 0;
        self.required_imports.clear();
        self.delegated_events.clear();
        
        // Add the template import (will be needed for any JSX)
        self.add_import("template");
    }

    fn exit_program(&mut self, program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a, ()>) {
        // Exit point for the transformation
        // Add delegate events import if needed
        if self.options.delegate_events && !self.delegated_events.is_empty() {
            self.add_import("delegateEvents");
        }

        // Build the list of statements to inject at the beginning
        let mut new_stmts = Vec::new();
        
        // 1. Add import statement
        if !self.required_imports.is_empty() {
            if let Some(import_stmt) = self.create_import_statement() {
                new_stmts.push(import_stmt);
            }
        }
        
        // 2. Add template declarations
        if !self.template_map.is_empty() {
            if let Some(template_decl) = self.create_template_declarations() {
                new_stmts.push(template_decl);
            }
        }
        
        // 3. Prepend new statements to the program
        if !new_stmts.is_empty() {
            // Get existing statements
            let existing_stmts = std::mem::replace(&mut program.body, OxcVec::new_in(self.allocator));
            
            // Create new statement list with injected statements first
            let mut all_stmts = new_stmts;
            all_stmts.extend(existing_stmts);
            
            // Replace program body
            program.body = OxcVec::from_iter_in(all_stmts.into_iter(), self.allocator);
        }
        
        // 4. Add delegateEvents call if needed
        if self.options.delegate_events && !self.delegated_events.is_empty() {
            if let Some(delegate_call) = self.create_delegate_events_call() {
                program.body.push(delegate_call);
            }
        }
    }

    fn enter_jsx_element(&mut self, elem: &mut JSXElement<'a>, _ctx: &mut TraverseCtx<'a, ()>) {
        // Check if this is a component or HTML element
        let tag_name = match &elem.opening_element.name {
            JSXElementName::Identifier(ident) => ident.name.as_str(),
            JSXElementName::IdentifierReference(ident) => ident.name.as_str(),
            _ => return, // Skip complex element names for now
        };

        // Components are handled differently
        if is_component(tag_name) {
            // Component handling - track that we need component imports
            // For now, just skip transformation
            return;
        }

        // Handle JSX elements
        // Build a template from the JSX element
        let template = crate::template::build_template_with_options(elem, Some(&self.options));
        
        // Record template for optimization analysis
        self.optimizer.record_template(template.clone());
        
        // Get effect wrapper name before borrowing self mutably
        let _effect_wrapper = self.options.effect_wrapper.clone(); // TODO: Use when implementing full dynamic binding
        let delegate_events = self.options.delegate_events;
        
        // Track which imports are needed based on dynamic slots
        // NOTE: Currently we only generate simple template calls without dynamic binding code,
        // so we don't need to import these yet. When full IIFE generation is implemented,
        // uncomment this code.
        #[allow(clippy::never_loop)]
        for slot in &template.dynamic_slots {
            match &slot.slot_type {
                SlotType::TextContent => {
                    // self.add_import("insert");
                }
                SlotType::Attribute(_) => {
                    // self.add_import("setAttribute");
                    // self.add_import(&effect_wrapper);
                }
                SlotType::EventHandler(event_name) => {
                    if delegate_events && should_delegate_event(event_name) {
                        self.add_delegated_event(event_name);
                    }
                }
                SlotType::Ref => {
                    // Ref doesn't need imports
                }
                SlotType::ClassList => {
                    // self.add_import("classList");
                    // self.add_import(&effect_wrapper);
                }
                SlotType::StyleObject => {
                    // self.add_import("style");
                    // self.add_import(&effect_wrapper);
                }
                SlotType::OnEvent(_) | SlotType::OnCaptureEvent(_) => {
                    // These use direct addEventListener, no imports needed
                }
            }
        }
        
        // Store the template for later code generation
        self.templates.push(template);
        
        // Note: In a full implementation, we would:
        // 1. Replace the JSX element with generated code
        // 2. Create an IIFE that clones the template
        // 3. Add code to set up dynamic bindings
        // 4. Handle event handlers
    }

    fn enter_jsx_fragment(&mut self, _frag: &mut JSXFragment<'a>, _ctx: &mut TraverseCtx<'a, ()>) {
        // Handle JSX fragments
        // Fragments are converted to arrays in Solid
        // Track that we encountered one and may need special handling
        
        // In a full implementation, we would:
        // 1. Process each child of the fragment
        // 2. Wrap them in an array
        // 3. Handle dynamic children appropriately
        
        // For now, just note that fragments are being tracked
    }

    fn enter_jsx_opening_element(
        &mut self,
        _elem: &mut JSXOpeningElement<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        // Handle JSX opening elements
        // This is where we would process attributes
    }

    fn enter_jsx_attribute(&mut self, _attr: &mut JSXAttribute<'a>, _ctx: &mut TraverseCtx<'a, ()>) {
        // Handle JSX attributes
        // Process special attributes and event handlers
    }

    fn enter_jsx_spread_attribute(
        &mut self,
        _attr: &mut JSXSpreadAttribute<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        // Handle JSX spread attributes
        // In a full implementation, we would handle spread props
    }

    fn enter_jsx_expression_container(
        &mut self,
        _expr: &mut JSXExpressionContainer<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        // Handle JSX expression containers
        // Wrap dynamic expressions with effect() or insert() as appropriate
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, _ctx: &mut TraverseCtx<'a, ()>) {
        // Replace JSX elements with template calls or IIFEs
        match expr {
            Expression::JSXElement(jsx_elem) => {
                // Check if this is a component
                let tag_name = match &jsx_elem.opening_element.name {
                    JSXElementName::Identifier(ident) => ident.name.as_str(),
                    JSXElementName::IdentifierReference(ident) => ident.name.as_str(),
                    _ => return, // Skip complex element names
                };

                if is_component(tag_name) {
                    // Don't transform components
                    return;
                }

                // Build template and get the template variable
                let template = crate::template::build_template_with_options(jsx_elem.as_ref(), Some(&self.options));
                let template_var = self.get_template_var(&template.html);
                
                // Check if this template has dynamic content
                let has_dynamic_content = !template.dynamic_slots.is_empty();
                
                if has_dynamic_content {
                    // Generate an IIFE with dynamic binding code
                    let iife = self.create_template_iife(jsx_elem.as_ref(), &template, &template_var);
                    *expr = Expression::CallExpression(iife);
                } else {
                    // Simple template call for static content
                    let template_var_str = self.allocator.alloc_str(&template_var);
                    let call_expr = self.create_template_call(template_var_str);
                    *expr = Expression::CallExpression(call_expr);
                }
            }
            Expression::JSXFragment(_) => {
                // Handle fragments
                // For now, just leave them as-is
                // In a full implementation, fragments would be converted to arrays
            }
            _ => {}
        }
    }
}
