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
//! - `import_ordering`: Defines import priority order for babel compatibility

pub mod import_ordering;
pub mod output_normalizer;

pub use import_ordering::get_import_priority;
pub use output_normalizer::BabelOutputNormalizer;
