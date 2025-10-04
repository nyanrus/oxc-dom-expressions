//! # oxc-dom-expressions
//!
//! A drop-in replacement for babel-plugin-jsx-dom-expressions for Solid.js, implemented in Rust using the oxc compiler toolchain.
//!
//! This plugin transforms JSX into efficient DOM operations for fine-grained reactive libraries like Solid.js.
//!
//! ## Features
//!
//! - ✅ Template-based DOM generation with automatic deduplication
//! - ✅ Dynamic content insertion with _$insert runtime calls
//! - ✅ Dynamic attributes with _$setAttribute wrapped in _$effect
//! - ✅ Component transformation to _$createComponent calls
//! - ✅ Fragment transformation to arrays
//! - ✅ Event delegation support
//! - ✅ Performance optimization and statistics
//! - 🚧 SSR mode (in progress)
//! - 🚧 Hydratable mode (in progress)
//!
//! ## Example
//!
//! ```rust,ignore
//! use oxc_allocator::Allocator;
//! use oxc_parser::Parser;
//! use oxc_semantic::SemanticBuilder;
//! use oxc_span::SourceType;
//! use oxc_traverse::traverse_mut;
//! use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions, GenerateMode};
//! use oxc_codegen::Codegen;
//!
//! let source_text = r#"
//!   const App = () => <div id="main">
//!     <h1>{title}</h1>
//!     <p class={className}>{content}</p>
//!   </div>;
//! "#;
//!
//! let allocator = Allocator::default();
//! let ret = Parser::new(&allocator, source_text, SourceType::jsx()).parse();
//! let mut program = ret.program;
//!
//! let semantic = SemanticBuilder::new().build(&program).semantic;
//! let scoping = semantic.into_scoping();
//!
//! let options = DomExpressionsOptions::new("solid-js/web")
//!     .with_delegate_events(true)
//!     .with_generate(GenerateMode::Dom);
//!
//! let mut transformer = DomExpressions::new(&allocator, options);
//! traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());
//!
//! let output = Codegen::new().build(&program).code;
//! println!("{}", output);
//! ```
//!
//! ## Architecture
//!
//! The transformation happens in several phases:
//!
//! 1. **Parse**: JSX source code is parsed into an AST
//! 2. **Traverse**: The AST is traversed bottom-up
//! 3. **Template Building**: JSX elements → HTML templates + dynamic slots
//! 4. **Code Generation**: Generate runtime calls for dynamic content
//! 5. **Optimization**: Deduplicate templates and collect statistics
//!
//! ## Modules
//!
//! - `transform`: Main transformation logic and AST traversal
//! - [`template`]: Template string generation and dynamic slot tracking
//! - [`codegen`]: Code generation utilities for runtime calls
//! - `options`: Configuration options (re-exported as [`DomExpressionsOptions`])
//! - [`optimizer`]: Template optimization and statistics
//! - [`utils`]: Utility functions for component detection, event handling, etc.
//! - [`compat`]: Compatibility layer for babel-plugin-jsx-dom-expressions

pub mod codegen;
pub mod compat;
pub mod html_subset_parser;
pub mod optimizer;
mod options;
pub mod template;
pub mod template_minimizer;
mod transform;
pub mod utils;

pub use optimizer::{Optimization, OptimizationKind, TemplateOptimizer, TemplateStats};
pub use options::{DomExpressionsOptions, GenerateMode};
pub use transform::DomExpressions;

#[cfg(test)]
mod tests;
