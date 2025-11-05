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
//! 2. **Traverse**: The AST is traversed bottom-up using the Traverse trait
//! 3. **Template Building**: JSX elements → HTML templates + dynamic slots
//! 4. **AST-Based Code Generation**: Generate runtime calls using AstBuilder (no string manipulation)
//! 5. **Optimization**: Deduplicate templates and collect statistics
//!
//! ### Code Generation Philosophy
//!
//! This transformer follows Oxc's best practices for AST transformation:
//! - **Manual AST Construction**: All code is generated using `AstBuilder`, never through string concatenation
//! - **Type Safety**: The AST API ensures type-safe code generation
//! - **Single Pass**: All transformations happen in one traversal for maximum performance
//! - **Node Replacement**: Transform JSX by replacing nodes in `exit_*` methods
//! - **Statement Insertion**: Insert new statements by manipulating the statements vector
//!
//! For more details on the AST-based approach, see the [`transform`] module documentation.
//!
//! ## Modules
//!
//! ### Core Modules
//!
//! - [`transform`]: Modern transformation logic using declarative $bind API
//! - [`compat2`]: Babel-compatible transformation (legacy format)
//! - [`template`]: Template string generation and dynamic slot tracking
//! - [`utils`]: Utility functions for component detection, event handling, etc.
//!
//! ### Configuration
//!
//! - [`options`]: Configuration options (re-exported as [`DomExpressionsOptions`])
//!
//! ### Compatibility
//!
//! - [`compat`]: Compatibility layer for babel-plugin-jsx-dom-expressions
//!   - Output normalization for fixture test compatibility
//!   - Import ordering to match babel plugin output
//!
//! ### Optimization
//!
//! - [`opt`]: Optimization and minimization utilities
//!   - Template deduplication and statistics
//!   - HTML minimization
//!   - Static expression evaluation
//!
//! ### Internal Modules
//!
//! - [`html_subset_parser`]: HTML parsing for template generation

pub mod compat;
pub mod compat2;
pub mod html_subset_parser;
#[cfg(feature = "opt")]
pub mod opt;
mod options;
pub mod template;
mod transform;
pub mod utils;

#[cfg(feature = "opt")]
pub use opt::{Optimization, OptimizationKind, TemplateOptimizer, TemplateStats};
pub use options::{DomExpressionsOptions, GenerateMode};
pub use transform::DomExpressions;
pub use compat2::DomExpressionsCompat2;

#[cfg(test)]
mod tests;
