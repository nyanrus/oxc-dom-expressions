//! Example usage of oxc-dom-expressions
//!
//! This example demonstrates how to set up and use the DOM expressions transformer.

use oxc_allocator::Allocator;
use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions};

fn main() {
    // Create an allocator for the AST
    let allocator = Allocator::default();

    // Set up default options for Solid.js
    let options = DomExpressionsOptions::default();
    println!("Default configuration:");
    println!("  Module name: {}", options.module_name);
    println!("  Effect wrapper: {}", options.effect_wrapper);
    println!("  Delegate events: {}", options.delegate_events);
    println!();

    // Create transformer with default options
    let transformer = DomExpressions::new(&allocator, options);
    println!(
        "Transformer created with module: {}",
        transformer.options().module_name
    );
    println!();

    // Example with custom configuration
    let custom_options = DomExpressionsOptions::new("custom-runtime").with_delegate_events(false);

    println!("Custom configuration:");
    println!("  Module name: {}", custom_options.module_name);
    println!("  Delegate events: {}", custom_options.delegate_events);

    let _custom_transformer = DomExpressions::new(&allocator, custom_options);
    println!();

    println!("To use this transformer in an oxc pipeline:");
    println!("1. Parse your JSX source code with oxc_parser");
    println!("2. Run semantic analysis with oxc_semantic");
    println!("3. Apply the DomExpressions transformer using oxc_traverse");
    println!("4. Generate the transformed code");
}
