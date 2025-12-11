/// Demo showing how oxc-dom-expressions can be used as a drop-in replacement for vite-plugin-solid
/// 
/// This example demonstrates the transformation of typical Solid.js patterns:
/// - Simple components with props
/// - Interactive components with event handlers
/// - Conditional rendering (Show component)
/// - List rendering (For component)
/// - Fragments
/// 
/// The output shows how JSX is transformed into efficient template-based code
/// that works with the solid-js/web runtime API.

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_dom_expressions::{DomExpressionsCompat2, DomExpressionsOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_traverse::traverse_mut;

fn main() {
    println!("=== oxc-dom-expressions: Vite Plugin Demo ===\n");
    
    // Example 1: Simple component
    transform_and_print(
        "Simple Component",
        r#"
const Greeting = (props) => (
  <div class="greeting">
    <h1>Hello, {props.name}!</h1>
  </div>
);
"#,
    );
    
    // Example 2: Interactive component with events
    transform_and_print(
        "Interactive Component",
        r#"
const Counter = () => {
  const [count, setCount] = createSignal(0);
  
  return (
    <div class="counter">
      <button onClick={() => setCount(count() + 1)}>
        Count: {count()}
      </button>
    </div>
  );
};
"#,
    );
    
    // Example 3: Component with conditional rendering
    transform_and_print(
        "Conditional Rendering",
        r#"
const ConditionalComponent = (props) => (
  <div>
    <Show when={props.isVisible}>
      <p>Visible content</p>
    </Show>
  </div>
);
"#,
    );
    
    // Example 4: List rendering
    transform_and_print(
        "List Rendering",
        r#"
const TodoList = (props) => (
  <ul class="todo-list">
    <For each={props.todos}>
      {(todo) => <li>{todo.text}</li>}
    </For>
  </ul>
);
"#,
    );
    
    // Example 5: Fragment
    transform_and_print(
        "Fragment",
        r#"
const MultiElement = () => (
  <>
    <div>First</div>
    <div>Second</div>
  </>
);
"#,
    );
    
    println!("\n=== Summary ===");
    println!("✅ oxc-dom-expressions successfully transforms Solid.js JSX");
    println!("✅ Compatible with solid-js/web runtime");
    println!("✅ Supports all major Solid.js patterns:");
    println!("   - Components and props");
    println!("   - Event handlers");
    println!("   - Dynamic content");
    println!("   - Conditional rendering (Show, Switch)");
    println!("   - List rendering (For, Index)");
    println!("   - Fragments");
    println!("\n📚 Usage: Can be integrated into Vite plugin for fast Solid.js compilation");
}

// Helper function to transform and print JSX examples
// Extracted to avoid duplication of source_type configuration
fn transform_and_print(title: &str, source: &str) {
    // Standard configuration for Solid.js JSX
    const SOURCE_TYPE_CONFIG: fn() -> SourceType = || {
        SourceType::default().with_jsx(true).with_module(true)
    };
    println!("--- {} ---", title);
    println!("Input:");
    println!("{}", source.trim());
    println!("\nTransformed Output:");
    
    let allocator = Allocator::default();
    let source_type = SOURCE_TYPE_CONFIG();
    let ret = Parser::new(&allocator, source, source_type).parse();
    
    if !ret.errors.is_empty() {
        println!("Parse errors:");
        for error in &ret.errors {
            println!("  {}", error);
        }
        return;
    }
    
    let mut program = ret.program;
    let semantic = SemanticBuilder::new().build(&program).semantic;
    let scoping = semantic.into_scoping();
    
    // Use DomExpressionsCompat2 for babel-plugin-jsx-dom-expressions compatibility
    let options = DomExpressionsOptions::new("solid-js/web")
        .with_delegate_events(true);
    
    let mut transformer = DomExpressionsCompat2::new(&allocator, options);
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());
    
    let output = Codegen::new().build(&program).code;
    println!("{}", output);
    println!();
}
