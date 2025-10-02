//! Main transformer for DOM expressions

use oxc_allocator::{Allocator, CloneIn};
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_traverse::{Traverse, TraverseCtx};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use crate::optimizer::{TemplateOptimizer, TemplateStats};
use crate::options::DomExpressionsOptions;
use crate::template::{build_template, SlotType, Template};
use crate::utils::{is_component, should_delegate_event};

/// The main DOM expressions transformer
pub struct DomExpressions<'a> {
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
    /// Stores JSX elements to replace with their generated code (span -> expression)
    replacements: RefCell<HashMap<u32, Expression<'a>>>,
    /// Template declarations to add to the program
    template_declarations: RefCell<Vec<(String, String)>>,
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
            replacements: RefCell::new(HashMap::new()),
            template_declarations: RefCell::new(Vec::new()),
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
    
    /// Generate a template call expression (e.g., _tmpl$())
    fn generate_template_call(&self, template_var: String, _template: &Template, ctx: &mut TraverseCtx<'a>) -> Expression<'a> {
        // Create a call expression to the template function
        // e.g., _tmpl$()
        
        // Create the callee (the template variable reference)
        // Pass the String directly - it will be converted to Atom<'a> via IntoIn
        let callee = ctx.ast.expression_identifier_reference(SPAN, template_var);
        
        // Create an empty arguments list
        let arguments = ctx.ast.vec();
        
        // Create the call expression (None for type_parameters)
        ctx.ast.expression_call(SPAN, callee, None::<TSTypeParameterInstantiation>, arguments, false)
    }
    
    /// Create an import statement for the runtime functions (static version)
    fn create_import_statement_static(required_imports: &HashSet<String>, module_name: String, ctx: &mut TraverseCtx<'a>) -> Statement<'a> {
        // Create: import { template as _$template, ... } from "r-dom";
        
        let mut specifiers = ctx.ast.vec();
        
        // Sort imports for consistent output
        let mut imports: Vec<_> = required_imports.iter().collect();
        imports.sort();
        
        for import_name in imports {
            // Create the imported binding (e.g., "template")
            let imported = ctx.ast.module_export_name_identifier_name(SPAN, (*import_name).clone());
            
            // Create the local binding (e.g., "_$template")
            let local_name = format!("_${}", import_name);
            let local = ctx.ast.binding_identifier(SPAN, local_name);
            
            // Create the import specifier
            let specifier = ctx.ast.import_declaration_specifier_import_specifier(
                SPAN,
                imported,
                local,
                ImportOrExportKind::Value,
            );
            
            specifiers.push(specifier);
        }
        
        // Create the source (e.g., "r-dom")
        let source = ctx.ast.string_literal(SPAN, module_name, None);
        
        // Create the import declaration
        let import_decl = ctx.ast.alloc_import_declaration(
            SPAN,
            Some(specifiers),
            source,
            None,  // phase
            None::<WithClause>,  // with_clause
            ImportOrExportKind::Value,
        );
        
        Statement::ImportDeclaration(import_decl)
    }
    
    /// Create template variable declarations (static version)
    fn create_template_declarations_static(template_decls: Vec<(String, String)>, ctx: &mut TraverseCtx<'a>) -> Statement<'a> {
        // Create: var _tmpl$ = /*#__PURE__*/ _$template(`<div>`), _tmpl$2 = ...;
        
        let mut declarators = ctx.ast.vec();
        
        for (var_name, html) in template_decls {
            // Create the template call: _$template(`<div>`)
            let template_fn_expr = ctx.ast.expression_identifier_reference(SPAN, "_$template");
            
            // Create the template string argument
            let template_str = ctx.ast.string_literal(SPAN, html, None);
            let template_str_expr = Expression::StringLiteral(ctx.ast.alloc(template_str));
            
            let mut args = ctx.ast.vec();
            args.push(Argument::from(template_str_expr));
            
            // Create the call expression
            let call_expr = ctx.ast.expression_call(SPAN, template_fn_expr, None::<TSTypeParameterInstantiation>, args, false);
            
            // Wrap with /*#__PURE__*/ comment (represented as the expression itself for now)
            let init_expr = call_expr;
            
            // Create the declarator
            let id_pattern = ctx.ast.binding_pattern_kind_binding_identifier(SPAN, var_name);
            let binding_pattern = ctx.ast.binding_pattern(id_pattern, None::<TSTypeAnnotation>, false);
            
            let declarator = ctx.ast.variable_declarator(
                SPAN,
                VariableDeclarationKind::Var,
                binding_pattern,
                Some(init_expr),
                false,
            );
            
            declarators.push(declarator);
        }
        
        // Create the variable declaration statement
        let var_decl = ctx.ast.alloc_variable_declaration(
            SPAN,
            VariableDeclarationKind::Var,
            declarators,
            false,
        );
        
        Statement::VariableDeclaration(var_decl)
    }
    
    /// Create a delegateEvents call statement (static version)
    fn create_delegate_events_call_static(delegated_events: Vec<String>, ctx: &mut TraverseCtx<'a>) -> Statement<'a> {
        // Create: _$delegateEvents(["click", "input"]);
        
        let fn_expr = ctx.ast.expression_identifier_reference(SPAN, "_$delegateEvents");
        
        // Create the array of event names
        let mut elements = ctx.ast.vec();
        
        for event in delegated_events {
            let event_str = ctx.ast.string_literal(SPAN, event, None);
            let event_expr = Expression::StringLiteral(ctx.ast.alloc(event_str));
            let element = ArrayExpressionElement::from(event_expr);
            elements.push(element);
        }
        
        let array_expr = ctx.ast.expression_array(SPAN, elements, None);
        
        // Create the call
        let mut args = ctx.ast.vec();
        args.push(Argument::from(array_expr));
        
        let call_expr = ctx.ast.expression_call(SPAN, fn_expr, None::<TSTypeParameterInstantiation>, args, false);
        
        // Wrap in an expression statement
        let expr_stmt = ctx.ast.alloc_expression_statement(SPAN, call_expr);
        Statement::ExpressionStatement(expr_stmt)
    }
}

impl<'a> Traverse<'a> for DomExpressions<'a> {
    fn enter_program(&mut self, _program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
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

    fn exit_program(&mut self, program: &mut Program<'a>, ctx: &mut TraverseCtx<'a>) {
        // Exit point for the transformation
        // Add all collected imports and template declarations
        
        if self.options.delegate_events && !self.delegated_events.is_empty() {
            self.add_import("delegateEvents");
        }
        
        // Build the new statements to add
        let mut new_statements = ctx.ast.vec();
        
        // 1. Add imports
        if !self.required_imports.is_empty() {
            let import_stmt = Self::create_import_statement_static(&self.required_imports, self.options.module_name.clone(), ctx);
            new_statements.push(import_stmt);
        }
        
        // 2. Add template variable declarations
        {
            let template_decls = self.template_declarations.borrow();
            if !template_decls.is_empty() {
                // Clone the data to pass ownership
                let template_decls_owned: Vec<_> = template_decls.iter().cloned().collect();
                let template_stmt = Self::create_template_declarations_static(template_decls_owned, ctx);
                new_statements.push(template_stmt);
            }
        }
        
        // 3. Append existing program statements
        for stmt in program.body.iter() {
            new_statements.push(stmt.clone_in(self.allocator));
        }
        
        // 4. Add delegateEvents call if needed
        if self.options.delegate_events && !self.delegated_events.is_empty() {
            let mut events: Vec<_> = self.delegated_events.iter().cloned().collect();
            events.sort();
            let delegate_stmt = Self::create_delegate_events_call_static(events, ctx);
            new_statements.push(delegate_stmt);
        }
        
        // Replace program body
        program.body = new_statements;
    }

    fn exit_jsx_element(&mut self, elem: &mut JSXElement<'a>, ctx: &mut TraverseCtx<'a>) {
        // Check if this is a component or HTML element
        let tag_name = match &elem.opening_element.name {
            JSXElementName::Identifier(ident) => ident.name.as_str(),
            JSXElementName::IdentifierReference(ident) => ident.name.as_str(),
            _ => return, // Skip complex element names for now
        };

        // Components are handled differently - don't transform them
        if is_component(tag_name) {
            return;
        }

        // Handle JSX elements
        // Build a template from the JSX element
        let template = build_template(elem);
        
        // Record template for optimization analysis
        self.optimizer.record_template(template.clone());
        
        // Get or create template variable
        let template_var = self.get_template_var(&template.html);
        
        // Store template declaration
        self.template_declarations.borrow_mut().push((template_var.clone(), template.html.clone()));
        
        // Get effect wrapper name before borrowing self mutably
        let effect_wrapper = self.options.effect_wrapper.clone();
        let delegate_events = self.options.delegate_events;
        
        // Track which imports are needed based on dynamic slots
        for slot in &template.dynamic_slots {
            match &slot.slot_type {
                SlotType::TextContent => {
                    self.add_import("insert");
                }
                SlotType::Attribute(_) => {
                    self.add_import("setAttribute");
                    self.add_import(&effect_wrapper);
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
                    self.add_import("classList");
                    self.add_import(&effect_wrapper);
                }
                SlotType::StyleObject => {
                    self.add_import("style");
                    self.add_import(&effect_wrapper);
                }
                SlotType::OnEvent(_) | SlotType::OnCaptureEvent(_) => {
                    // These use direct addEventListener, no imports needed
                }
            }
        }
        
        // Store the template for later code generation
        self.templates.push(template.clone());
        
        // Generate the replacement expression - a call to the template function
        // e.g., _tmpl$()
        let replacement = self.generate_template_call(template_var, &template, ctx);
        
        // Store the replacement to apply during exit_expression
        let span_start = elem.span.start;
        self.replacements.borrow_mut().insert(span_start, replacement);
    }
    
    fn exit_expression(&mut self, expr: &mut Expression<'a>, _ctx: &mut TraverseCtx<'a>) {
        // Check if this expression is a JSX element that needs to be replaced
        if let Expression::JSXElement(jsx_elem) = expr {
            let span_start = jsx_elem.span.start;
            if let Some(replacement) = self.replacements.borrow_mut().remove(&span_start) {
                *expr = replacement;
            }
        }
    }

    fn enter_jsx_fragment(&mut self, _frag: &mut JSXFragment<'a>, _ctx: &mut TraverseCtx<'a>) {
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
        _ctx: &mut TraverseCtx<'a>,
    ) {
        // Handle JSX opening elements
        // This is where we would process attributes
    }

    fn enter_jsx_attribute(&mut self, _attr: &mut JSXAttribute<'a>, _ctx: &mut TraverseCtx<'a>) {
        // Handle JSX attributes
        // Process special attributes and event handlers
    }

    fn enter_jsx_spread_attribute(
        &mut self,
        _attr: &mut JSXSpreadAttribute<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        // Handle JSX spread attributes
        // In a full implementation, we would handle spread props
    }

    fn enter_jsx_expression_container(
        &mut self,
        _expr: &mut JSXExpressionContainer<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        // Handle JSX expression containers
        // Wrap dynamic expressions with effect() or insert() as appropriate
    }
}
