//! Modern transformer for DOM expressions
//!
//! This module implements a modern, declarative JSX to DOM transformation.
//! Instead of imperative DOM manipulation, it uses a declarative binding API
//! that is more readable and maintainable.
//!
//! # Output Format
//!
//! The modern format uses three main runtime functions:
//!
//! - **$template(html)**: Parses HTML template string once at module scope
//! - **$clone(template)**: Clones the parsed template for each instance
//! - **$bind(root, path, bindings)**: Declaratively binds properties/events to element at path
//!
//! ## Example Output
//!
//! ```javascript
//! import { $template, $clone, $bind } from "solid-runtime/polyfill";
//!
//! const _tmpl$ = $template(`<div id="main"><h1 class="base"><a href="/">Welcome</a></h1></div>`);
//!
//! const template = (() => {
//!   const _root$ = $clone(_tmpl$);
//!   
//!   $bind(_root$, [0], {
//!     spread: [() => results],
//!     classList: { selected: () => unknown },
//!     style: { color: () => color }
//!   });
//!   
//!   $bind(_root$, [0, 0], {
//!     id: () => id,
//!     title: () => welcoming(),
//!     style: { "background-color": () => color(), "margin-right": "40px" },
//!     classList: { dynamic: () => dynamic(), selected: () => selected }
//!   });
//!   
//!   $bind(_root$, [0, 0, 0], {
//!     ref: (el) => link = el,
//!     classList: { "ccc ddd": true }
//!   });
//!   
//!   return _root$;
//! })();
//! ```
//!
//! # Path System
//!
//! Elements are referenced by their child index path from the root:
//! - `[0]`: First child of root
//! - `[0, 0]`: First child of first child
//! - `[0, 0, 0]`: First child's first child's first child
//!
//! # Binding Options
//!
//! The bindings object passed to `$bind` supports:
//! - **ref**: Element reference callback or variable assignment
//! - **spread**: Array of spread expressions to apply
//! - **classList**: Object mapping class names to boolean conditions
//! - **style**: Object mapping style properties to values (static or reactive)
//! - **textContent**: Text content (for textContent attribute)
//! - **innerHTML**: HTML content (for innerHTML attribute)
//! - **on:eventName**: Event handlers
//! - **attr:name**: Static attributes set via setAttribute
//! - **prop:name**: Property assignments
//! - **bool:name**: Boolean attributes
//! - **use:directive**: Custom directives
//! - Any other key: Regular attributes (reactive if value is a function)

use oxc_allocator::Allocator;
use std::collections::{HashMap, HashSet};

use crate::optimizer::{TemplateOptimizer, TemplateStats};
use crate::options::DomExpressionsOptions;
use crate::template::Template;

// Sub-modules
mod bindings;
mod codegen;
mod traverse_impl;

/// The modern DOM expressions transformer
pub struct DomExpressions<'a> {
    pub(super) allocator: &'a Allocator,
    pub(super) options: DomExpressionsOptions,
    /// Collection of templates generated during transformation
    pub(super) templates: Vec<Template>,
    /// Map of template HTML to variable name for deduplication
    pub(super) template_map: HashMap<String, String>,
    /// Counter for generating unique template variable names
    pub(super) template_counter: usize,
    /// Optimizer for template analysis
    pub(super) optimizer: TemplateOptimizer,
}

impl<'a> DomExpressions<'a> {
    /// Create a new modern DOM expressions transformer
    pub fn new(allocator: &'a Allocator, options: DomExpressionsOptions) -> Self {
        Self {
            allocator,
            options,
            templates: Vec::new(),
            template_map: HashMap::new(),
            template_counter: 0,
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
        self.template_counter += 1;
        if self.template_counter == 1 {
            "_tmpl$".to_string()
        } else {
            format!("_tmpl${}", self.template_counter)
        }
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
}
