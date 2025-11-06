// Complete demonstration of the helper injection feature
// This example shows how the modern $template, $clone, $bind API
// is automatically provided by wrapping the original solid-js/web runtime

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_traverse::traverse_mut;

fn main() {
    println!("=== Modern API Helper Injection Demo ===\n");
    println!("Problem: dom-expressions library (solid-js/web) doesn't expose");
    println!("         the modern $template, $clone, $bind API.\n");
    println!("Solution: Automatically inject helper functions that wrap the");
    println!("          original runtime API in every transformed file.\n");

    let source_text = r#"
// Example component using JSX
const TodoItem = ({ item, selected, onSelect, onDelete }) => {
  return <tr class={selected ? "selected" : ""}>
    <td>{item.id}</td>
    <td>
      <a onClick={(e) => onSelect(item, e)}>{item.label}</a>
    </td>
    <td>
      <button onClick={(e) => onDelete(item, e)}>Delete</button>
    </td>
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

    // Create transformer with solid-js/web as the runtime
    let options = DomExpressionsOptions::new("solid-js/web");
    let mut transformer = DomExpressions::new(&allocator, options);

    // Transform the JSX
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());

    // Generate output code
    let output = Codegen::new().build(&program).code;

    println!("=== Transformed Output ===\n");
    println!("{}", output);

    println!("\n=== Verification ===\n");

    // Verify the output contains the expected elements
    let checks = vec![
        ("Imports from solid-js/web", output.contains("from \"solid-js/web\"")),
        ("Helper: $template function", output.contains("function $template(")),
        ("Helper: $clone function", output.contains("function $clone(")),
        ("Helper: $bind function", output.contains("function $bind(")),
        ("Wraps _template", output.contains("_template(html)")),
        ("Wraps _effect", output.contains("_effect")),
        ("Wraps _setAttribute", output.contains("_setAttribute")),
        ("Template creation", output.contains("const _tmpl$")),
        ("Uses $template", output.contains("$template(`")),
        ("Uses $clone", output.contains("$clone(_tmpl$)")),
        ("Original component preserved", output.contains("const TodoItem")),
    ];

    let mut all_passed = true;
    for (description, passed) in checks {
        if passed {
            println!("✅ {}", description);
        } else {
            println!("❌ {}", description);
            all_passed = false;
        }
    }

    println!("\n=== Summary ===\n");
    if all_passed {
        println!("✅ SUCCESS! All verifications passed.");
        println!("\nThe transformer successfully:");
        println!("  1. Imported the original runtime functions from solid-js/web");
        println!("  2. Injected helper functions ($template, $clone, $bind)");
        println!("  3. Helper functions wrap the original API (_template, _effect, etc.)");
        println!("  4. Transformed JSX code uses the modern declarative API");
        println!("  5. No separate polyfill package is needed!");
    } else {
        println!("❌ FAILED! Some verifications did not pass.");
    }
}
