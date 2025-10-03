//! Configuration options for the DOM expressions transformer
//!
//! This module defines the configuration options that control how JSX is transformed.
//! The options closely mirror the babel-plugin-jsx-dom-expressions configuration
//! to maintain compatibility.
//!
//! # Example
//!
//! ```rust
//! use oxc_dom_expressions::{DomExpressionsOptions, GenerateMode};
//!
//! // Create options for DOM mode with event delegation
//! let options = DomExpressionsOptions::new("solid-js/web")
//!     .with_delegate_events(true)
//!     .with_generate(GenerateMode::Dom);
//!
//! // Create options for SSR mode
//! let ssr_options = DomExpressionsOptions::new("solid-js/web")
//!     .with_generate(GenerateMode::Ssr)
//!     .with_hydratable(true);
//! ```
//!
//! # Key Options
//!
//! - **module_name**: Runtime module to import from (e.g., "solid-js/web", "r-dom")
//! - **generate**: Output mode (Dom or Ssr)
//! - **delegate_events**: Enable event delegation for better performance
//! - **hydratable**: Generate hydration markers for SSR
//! - **wrap_conditionals**: Wrap conditionals in functions for reactivity
//! - **context_to_custom_elements**: Pass context to custom elements
//! - **builtin_components**: Map of built-in component transformers
//! - **effect_wrapper**: Custom effect wrapper function name

use serde::{Deserialize, Serialize};

/// Output mode for the compiler
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GenerateMode {
    /// Standard DOM output
    Dom,
    /// Server-side rendering output
    Ssr,
}

impl Default for GenerateMode {
    fn default() -> Self {
        Self::Dom
    }
}

/// Configuration options for the DOM expressions transformer
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DomExpressionsOptions {
    /// The name of the runtime module to import methods from
    pub module_name: String,

    /// The output mode of the compiler
    pub generate: GenerateMode,

    /// Whether the output should contain hydratable markers
    pub hydratable: bool,

    /// Whether to enable automatic event delegation on camelCase
    pub delegate_events: bool,

    /// Whether smart conditional detection should be used
    pub wrap_conditionals: bool,

    /// Whether to set current render context on Custom Elements and slots
    pub context_to_custom_elements: bool,

    /// Array of Component exports from module that aren't included by default
    pub built_ins: Vec<String>,

    /// The reactive wrapper function name
    pub effect_wrapper: String,

    /// Comment decorator string that indicates a static expression
    pub static_marker: String,

    /// The memo function name
    pub memo_wrapper: String,

    /// Whether to validate HTML nesting
    pub validate: bool,

    /// Whether to remove unnecessary closing tags from the template output
    pub omit_nested_closing_tags: bool,

    /// Whether to remove tags if they are the last element
    pub omit_last_closing_tag: bool,

    /// Whether to remove quotes for HTML attributes when possible
    pub omit_quotes: bool,

    /// When set, restricts JSX transformation to files with specific import source pragma
    pub require_import_source: Option<String>,
}

impl Default for DomExpressionsOptions {
    fn default() -> Self {
        Self {
            module_name: String::from("solid-js/web"),
            generate: GenerateMode::Dom,
            hydratable: false,
            delegate_events: true,
            wrap_conditionals: true,
            context_to_custom_elements: false,
            built_ins: Vec::new(),
            effect_wrapper: String::from("effect"),
            static_marker: String::from("@once"),
            memo_wrapper: String::from("memo"),
            validate: true,
            omit_nested_closing_tags: false,
            omit_last_closing_tag: true,
            omit_quotes: true,
            require_import_source: None,
        }
    }
}

impl DomExpressionsOptions {
    /// Create a new options instance with the given module name
    pub fn new(module_name: impl Into<String>) -> Self {
        Self {
            module_name: module_name.into(),
            ..Default::default()
        }
    }

    /// Set the generate mode
    pub fn with_generate(mut self, generate: GenerateMode) -> Self {
        self.generate = generate;
        self
    }

    /// Set whether to enable event delegation
    pub fn with_delegate_events(mut self, delegate: bool) -> Self {
        self.delegate_events = delegate;
        self
    }
}
