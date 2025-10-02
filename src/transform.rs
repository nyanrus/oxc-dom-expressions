//! Main transformer for DOM expressions

use oxc_allocator::{Allocator, Box};
use oxc_allocator::Vec as OxcVec;
use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};
use oxc_span::{SPAN, Atom};
use std::collections::{HashMap, HashSet};

use crate::optimizer::{TemplateOptimizer, TemplateStats};
use crate::options::DomExpressionsOptions;
use crate::template::{build_template, SlotType, Template};
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

    /// Create import statement for runtime functions
    fn create_import_statement(&self) -> Option<Statement<'a>> {
        // For now, return None - implementing full AST for import is complex
        // This would create: import { template as _$template, ... } from "module_name";
        None
    }

    /// Create template variable declarations
    fn create_template_declarations(&self) -> Option<Statement<'a>> {
        // For now, return None - implementing full AST for var declarations is complex
        // This would create: var _tmpl$ = _$template(`<html>`), ...
        None
    }

    /// Create delegateEvents call
    fn create_delegate_events_call(&self) -> Option<Statement<'a>> {
        // For now, return None - implementing full AST for function call is complex
        // This would create: _$delegateEvents(["click", "input"]);
        None
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
        let template = build_template(elem);
        
        // Record template for optimization analysis
        self.optimizer.record_template(template.clone());
        
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
        // Replace JSX elements with template calls
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
                let template = build_template(jsx_elem.as_ref());
                let template_var = self.get_template_var(&template.html);
                
                // Allocate the template variable string so it lives for 'a
                let template_var_str = self.allocator.alloc_str(&template_var);
                
                // Create a call expression to clone the template
                let call_expr = self.create_template_call(template_var_str);
                
                // Replace the JSX element with the call expression
                *expr = Expression::CallExpression(call_expr);
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
