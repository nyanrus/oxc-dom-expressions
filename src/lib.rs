//! # oxc-dom-expressions
//!
//! A drop-in replacement of babel-plugin-jsx-dom-expressions for solid-js implemented with oxc in Rust.
//!
//! This plugin transforms JSX into efficient DOM operations for fine-grained reactive libraries like Solid.js.

pub mod codegen;
mod options;
pub mod optimizer;
pub mod template;
pub mod template_minimalizer;
mod transform;
pub mod utils;

pub use options::{DomExpressionsOptions, GenerateMode};
pub use optimizer::{Optimization, OptimizationKind, TemplateOptimizer, TemplateStats};
pub use transform::DomExpressions;

#[cfg(test)]
mod tests;
