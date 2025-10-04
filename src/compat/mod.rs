//! Compatibility layer for babel-plugin-jsx-dom-expressions
//!
//! This module contains code that ensures compatibility with the original
//! babel-plugin-jsx-dom-expressions. It includes preprocessing and postprocessing
//! utilities to match the exact output format of the babel plugin.
//!
//! ## Purpose
//!
//! The compat module separates babel-specific transformations from the core
//! transformation logic. This makes the codebase more maintainable by clearly
//! identifying what is essential functionality vs. what is needed purely for
//! compatibility with the original babel plugin.
//!
//! ## Modules
//!
//! - `output_normalizer`: Normalizes generated code to match babel plugin format
//!   - Converts `/* @__PURE__ */` to `/*#__PURE__*/` (babel format)
//!   - Replaces tabs with double spaces (babel formatting)
//!   - Formats template variable declarations with proper line breaks
//!
//! - `import_ordering`: Defines import priority order for babel compatibility
//!   - Template/SSR imports first
//!   - Runtime functions in specific order
//!   - Unknown imports last
//!
//! - `naming`: Variable naming conventions matching babel plugin output
//!   - Template variable names: `_tmpl$`, `_tmpl$2`, `_tmpl$3`, etc.
//!   - Element variable names: `_el$1`, `_el$2`, `_el$3`, etc.
//!   - Runtime function names: `_$insert`, `_$template`, etc.
//!   - Helper functions for checking and extracting variable names
//!
//! - `constants`: Babel-specific constants for documentation and clarity
//!   - Pure comment formats
//!   - Variable prefixes
//!   - Default module names
//!   - Default wrapper function names
//!
//! ## Usage
//!
//! ```rust
//! use oxc_dom_expressions::compat::{
//!     BabelOutputNormalizer,
//!     get_import_priority,
//!     template_var_name,
//!     element_var_name,
//! };
//!
//! // Generate babel-compatible variable names
//! let tmpl_var = template_var_name(1); // "_tmpl$"
//! let elem_var = element_var_name(1); // "_el$1"
//!
//! // Normalize output for babel compatibility
//! let normalized = BabelOutputNormalizer::normalize(&generated_code);
//!
//! // Sort imports by priority
//! imports.sort_by_key(|name| get_import_priority(name));
//! ```

pub mod constants;
pub mod import_ordering;
pub mod naming;
pub mod output_normalizer;

pub use import_ordering::get_import_priority;
pub use naming::{element_var_name, runtime_function_name, template_var_name};
pub use output_normalizer::BabelOutputNormalizer;
