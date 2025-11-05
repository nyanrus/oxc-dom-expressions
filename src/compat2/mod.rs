//! Babel-compatible transformer for DOM expressions (compat2)
//!
//! This module implements the babel-plugin-jsx-dom-expressions compatible JSX to DOM transformation.
//! It transforms JSX syntax into optimized DOM manipulation code using a template-based approach.
//! This is the legacy/compatibility format that matches the original babel plugin output.
//!
//! # AST-Based Code Generation
//!
//! This transformer follows Oxc's recommended practices for AST transformation:
//!
//! ## Core Principles
//!
//! - **Manual AST Construction**: All code is generated using `AstBuilder` through the allocator
//! - **No String Manipulation**: Code is never generated via string concatenation or formatting
//! - **Type Safety**: The AST API ensures type-safe and correct code generation
//! - **Single Pass**: All transformations happen in one traversal for maximum performance
//!
//! ## Code Injection Patterns
//!
//! ### 1. Node Replacement
//! In `exit_*` methods, JSX nodes are replaced with generated AST nodes:
//! ```rust,ignore
//! fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
//!     if let Expression::JSXElement(jsx_elem) = expr {
//!         *expr = self.transform_jsx_element(jsx_elem, ctx);
//!     }
//! }
//! ```
//!
//! ### 2. Statement Insertion
//! New statements are created and inserted into blocks:
//! ```rust,ignore
//! let insert_stmt = self.create_insert_call(element, expression);
//! statements.push(insert_stmt);
//! ```
//!
//! ## Example AST Construction
//!
//! All AST nodes are constructed using the `AstBuilder` API:
//! ```rust,ignore
//! // Creating a call expression: _$insert(element, value)
//! let call_expr = CallExpression {
//!     span: SPAN,
//!     callee: Expression::Identifier(Box::new_in(
//!         IdentifierReference {
//!             span: SPAN,
//!             name: Atom::from("_$insert"),
//!             reference_id: None.into(),
//!         },
//!         self.allocator,
//!     )),
//!     arguments: args,
//!     optional: false,
//!     type_arguments: None,
//!     pure: false,
//! };
//! ```
//!
//! # Module Organization
//!
//! The transform module is split into several sub-modules for better organization:
//!
//! - **mod.rs** (this file) - Core struct, state management, and public API
//! - **events.rs** - Event handling transformations (delegation, listeners, etc.)
//! - **attributes.rs** - Attribute transformations (style, class, etc.)
//! - **templates.rs** - Template and IIFE generation
//! - **components.rs** - Component and fragment transformations  
//! - **codegen.rs** - AST-based code generation helpers (imports, declarations, etc.)
//! - **traverse_impl.rs** - Traverse trait implementation
//!
//! # Transformation Flow
//!
//! 1. **Parse**: JSX is parsed into an AST by the oxc parser
//! 2. **Traverse**: The transformer traverses the AST bottom-up
//! 3. **Template Building**: JSX elements are converted to HTML templates with dynamic slots
//! 4. **AST Generation**: Generate runtime calls using AstBuilder for dynamic content
//! 5. **Output**: Emit optimized JavaScript with template literals and runtime library calls

use oxc_allocator::Allocator;
use std::collections::{HashMap, HashSet};

use crate::opt::{TemplateOptimizer, TemplateStats};
use crate::options::DomExpressionsOptions;
use crate::template::Template;

// Sub-modules containing impl blocks for DomExpressions
mod attributes;
mod codegen;
mod components;
mod events;
mod templates;
mod traverse_impl;

/// The babel-compatible DOM expressions transformer (compat2)
pub struct DomExpressionsCompat2<'a> {
    pub(super) allocator: &'a Allocator,
    pub(super) options: DomExpressionsOptions,
    /// Collection of templates generated during transformation
    pub(super) templates: Vec<Template>,
    /// Map of template HTML to variable name for deduplication
    pub(super) template_map: HashMap<String, String>,
    /// Counter for generating unique template variable names
    pub(super) template_counter: usize,
    /// Counter for generating unique element variable names
    pub(super) element_counter: usize,
    /// Whether we've generated the first root element (for _el$ vs _el$N)
    pub(super) first_root_generated: bool,
    /// List of required imports (preserves insertion order)
    pub(super) required_imports: Vec<String>,
    /// Set of events that need delegation
    pub(super) delegated_events: HashSet<String>,
    /// Optimizer for template analysis
    pub(super) optimizer: TemplateOptimizer,
}

impl<'a> DomExpressionsCompat2<'a> {
    /// Create a new babel-compatible DOM expressions transformer
    pub fn new(allocator: &'a Allocator, options: DomExpressionsOptions) -> Self {
        Self {
            allocator,
            options,
            templates: Vec::new(),
            template_map: HashMap::new(),
            template_counter: 0,
            element_counter: 0, // Start at 0
            first_root_generated: false,
            required_imports: Vec::new(),
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
    pub(super) fn generate_template_var(&mut self) -> String {
        use crate::compat::template_var_name;
        self.template_counter += 1;
        template_var_name(self.template_counter)
    }

    /// Get or create a template variable for given HTML
    pub(super) fn get_template_var(&mut self, html: &str) -> String {
        if let Some(var) = self.template_map.get(html) {
            var.clone()
        } else {
            let var = self.generate_template_var();
            self.template_map.insert(html.to_string(), var.clone());
            var
        }
    }

    /// Add a required import (preserves insertion order)
    pub(super) fn add_import(&mut self, name: &str) {
        if !self.required_imports.contains(&name.to_string()) {
            self.required_imports.push(name.to_string());
        }
    }

    /// Add an event that needs delegation
    pub(super) fn add_delegated_event(&mut self, event: &str) {
        // Events should be normalized to lowercase for delegation
        let lowercase_event = event.to_lowercase();
        self.delegated_events.insert(lowercase_event);
    }

    /// Generate a unique element variable name
    pub(super) fn generate_element_var(&mut self) -> String {
        use crate::compat::element_var_name;
        self.element_counter += 1;
        element_var_name(self.element_counter)
    }

    /// Generate root element variable name
    /// First root in file is "_el$", subsequent are numbered
    pub(super) fn generate_root_element_var(&mut self) -> String {
        if !self.first_root_generated {
            self.first_root_generated = true;
            "_el$".to_string()
        } else {
            self.generate_element_var()
        }
    }
}
