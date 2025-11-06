//! Modern transformer for DOM expressions
//!
//! This module implements a modern, declarative JSX to DOM transformation using
//! $template, $clone, and $bind runtime functions that wrap the original solid-js/web API.

use oxc_allocator::Allocator;
use std::collections::{HashMap, HashSet};

#[cfg(feature = "opt")]
use crate::opt::{TemplateOptimizer, TemplateStats};
use crate::options::DomExpressionsOptions;
use crate::template::Template;

// Sub-modules
mod codegen;
mod helper;
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
    #[cfg(feature = "opt")]
    pub(super) optimizer: TemplateOptimizer,
    /// Whether helper functions have been injected
    pub(super) helper_injected: bool,
    /// Track which runtime functions we need to import
    pub(super) imports_needed: HashSet<String>,
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
            #[cfg(feature = "opt")]
            optimizer: TemplateOptimizer::new(),
            helper_injected: false,
            imports_needed: HashSet::new(),
        }
    }

    /// Get the current options
    pub fn options(&self) -> &DomExpressionsOptions {
        &self.options
    }

    /// Get template statistics for optimization analysis
    #[cfg(feature = "opt")]
    pub fn get_template_stats(&self) -> TemplateStats {
        self.optimizer.get_stats()
    }

    /// Get list of templates that were reused (deduplicated)
    #[cfg(feature = "opt")]
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
    
    /// Mark a runtime function as needed for import
    pub(super) fn add_import(&mut self, name: &str) {
        self.imports_needed.insert(name.to_string());
    }
}
