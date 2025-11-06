// Demonstrates the new universal, clean modern output approach
// Philosophy: transformer-friendly and runtime-friendly with direct runtime calls

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_traverse::traverse_mut;

fn main() {
    println!("=== Modern Universal Output Demo ===\n");
    println!("The new approach: Clean, direct, universal\n");
    println!("Key principles:");
    println!("  1. No complex helper functions");
    println!("  2. Import and use runtime API directly");
    println!("  3. Transformer-friendly (easy to generate)");
    println!("  4. Runtime-friendly (fast to execute)\n");

    let source_text = r#"
const TodoItem = ({ item }) => {
  return <tr>
    <td>{item.id}</td>
    <td><a>{item.label}</a></td>
  </tr>;
};
    "#;

    let allocator = Allocator::default();
    let source_type = SourceType::default().with_jsx(true).with_module(true);
    let ret = Parser::new(&allocator, source_text, source_type).parse();

    if !ret.errors.is_empty() {
        eprintln!("Parse errors:");
        for error in &ret.errors {
            eprintln!("  {}", error);
        }
        return;
    }

    let mut program = ret.program;
    let semantic = SemanticBuilder::new().build(&program).semantic;
    let scoping = semantic.into_scoping();

    let options = DomExpressionsOptions::new("solid-js/web");
    let mut transformer = DomExpressions::new(&allocator, options);
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());

    let output = Codegen::new().build(&program).code;

    println!("=== Transformed Output ===\n");
    println!("{}", output);

    println!("\n=== Analysis ===\n");

    let checks = vec![
        ("✅ Simple import (no helpers)", output.contains("import { template as _$template }")),
        ("✅ Direct runtime usage", output.contains("_$template(")),
        ("✅ Clean template call", output.contains("_tmpl$()")),
        ("✅ No $bind helper", !output.contains("$bind")),
        ("✅ No $clone wrapper", !output.contains("$clone")),
        ("✅ Minimal output", output.len() < 500), // Should be concise
    ];

    let mut all_passed = true;
    for (description, passed) in checks {
        if passed {
            println!("{}", description);
        } else {
            println!("❌ {}", description);
            all_passed = false;
        }
    }

    if all_passed {
        println!("\n🎉 Perfect! The output is clean, direct, and universal.");
        println!("\nThis approach is:");
        println!("  • Transformer-friendly: Simple to generate");
        println!("  • Runtime-friendly: No overhead from helpers");
        println!("  • Universal: One consistent pattern");
    } else {
        println!("\n⚠️  Some checks did not pass.");
    }
}
