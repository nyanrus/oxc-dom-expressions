//! Main transformer for DOM expressions

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};
use std::collections::{HashMap, HashSet};

use crate::options::DomExpressionsOptions;
use crate::template::{build_template, Template};

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
        }
    }

    /// Get the current options
    pub fn options(&self) -> &DomExpressionsOptions {
        &self.options
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

    fn exit_program(&mut self, _program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a>) {
        // Exit point for the transformation
        // In a full implementation, we would:
        // 1. Add all collected imports to the program
        // 2. Add template variable declarations at the top level
        // 3. Add delegateEvents call if needed
        
        // For now, we just track what would be added
        if self.options.delegate_events && !self.delegated_events.is_empty() {
            self.add_import("delegateEvents");
        }
    }

    fn enter_jsx_element(&mut self, elem: &mut JSXElement<'a>, _ctx: &mut TraverseCtx<'a>) {
        // Handle JSX elements
        // Build a template from the JSX element
        let template = build_template(elem);
        
        // Get or create template variable
        let _template_var = self.get_template_var(&template.html);
        
        // Track which imports are needed based on dynamic slots
        if !template.dynamic_slots.is_empty() {
            self.add_import("insert");
        }
        
        // Store the template for later code generation
        self.templates.push(template);
        
        // Note: In a full implementation, we would:
        // 1. Replace the JSX element with generated code
        // 2. Create an IIFE that clones the template
        // 3. Add code to set up dynamic bindings
        // 4. Handle event handlers
    }

    fn enter_jsx_fragment(&mut self, _frag: &mut JSXFragment<'a>, _ctx: &mut TraverseCtx<'a>) {
        // Handle JSX fragments
        // Fragments are converted to arrays in Solid
        // For now, just track that we encountered one
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
