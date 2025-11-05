//! Optimization and minimization utilities
//!
//! This module contains code focused on optimizing and minifying the transformer output.
//! These are not core transformation features but performance enhancements.
//!
//! ## Modules
//!
//! - **optimizer**: Template deduplication and static analysis
//! - **minimizer**: Template HTML minimization and whitespace handling
//! - **evaluator**: Static expression evaluation for compile-time optimization
//!
//! ## Usage
//!
//! ```rust,ignore
//! use oxc_dom_expressions::opt::{TemplateOptimizer, minimize_template};
//!
//! // Optimize templates
//! let mut optimizer = TemplateOptimizer::new();
//! optimizer.record_template(template);
//! let stats = optimizer.get_stats();
//!
//! // Minimize template HTML
//! let minimized = minimize_template(&html, &options);
//! ```

pub mod evaluator;
pub mod minimizer;
pub mod optimizer;

pub use evaluator::{evaluate_expression, EvaluatedValue, EvaluationResult};
pub use minimizer::minimize_template;
pub use optimizer::{Optimization, OptimizationKind, TemplateOptimizer, TemplateStats};
