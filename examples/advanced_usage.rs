//! Advanced example showing integration with oxc parsing pipeline
//!
//! This demonstrates how oxc-dom-expressions would be used in a real transformation pipeline.

use oxc_allocator::Allocator;
use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions, GenerateMode};

fn main() {
    println!("=== Advanced oxc-dom-expressions Usage ===\n");

    // Example 1: Solid.js configuration
    println!("Example 1: Standard Solid.js configuration");
    let solid_config = DomExpressionsOptions::new("solid-js/web")
        .with_delegate_events(true)
        .with_generate(GenerateMode::Dom);

    println!("  Module: {}", solid_config.module_name);
    println!("  Generate mode: {:?}", solid_config.generate);
    println!("  Event delegation: {}", solid_config.delegate_events);
    println!("  Effect wrapper: {}", solid_config.effect_wrapper);
    println!();

    // Example 2: SSR configuration
    println!("Example 2: Server-Side Rendering configuration");
    let ssr_config = DomExpressionsOptions {
        module_name: String::from("solid-js/web"),
        generate: GenerateMode::Ssr,
        hydratable: true,
        delegate_events: false, // Events are handled client-side
        ..Default::default()
    };

    println!("  Generate mode: {:?}", ssr_config.generate);
    println!("  Hydratable: {}", ssr_config.hydratable);
    println!("  Event delegation: {}", ssr_config.delegate_events);
    println!();

    // Example 3: Custom runtime configuration
    println!("Example 3: Custom runtime with optimizations");
    let custom_config = DomExpressionsOptions {
        module_name: String::from("@my-framework/dom"),
        effect_wrapper: String::from("createEffect"),
        memo_wrapper: String::from("createMemo"),
        omit_nested_closing_tags: true,
        omit_quotes: true,
        validate: true,
        ..Default::default()
    };

    println!("  Module: {}", custom_config.module_name);
    println!("  Effect: {}", custom_config.effect_wrapper);
    println!("  Memo: {}", custom_config.memo_wrapper);
    println!("  Template optimization: enabled");
    println!();

    // Example 4: Using the transformer in a pipeline
    println!("Example 4: Integration pipeline");
    let allocator = Allocator::default();
    let options = DomExpressionsOptions::default();
    let _transformer = DomExpressions::new(&allocator, options);

    println!("  Step 1: Parse JSX with oxc_parser::Parser");
    println!("  Step 2: Build semantic model with oxc_semantic::SemanticBuilder");
    println!("  Step 3: Apply DomExpressions transformer");
    println!("  Step 4: Generate output with oxc_codegen::Codegen");
    println!();

    println!("=== Conceptual JSX Input ===");
    println!(
        r#"
const App = () => {{
  const [count, setCount] = createSignal(0);
  
  return (
    <div class="container">
      <h1>Counter: {{count()}}</h1>
      <button onClick={{() => setCount(count() + 1)}}>
        Increment
      </button>
    </div>
  );
}};
"#
    );

    println!("=== Expected Transformation ===");
    println!(
        r#"
import {{ template, insert, effect, delegateEvents }} from "solid-js/web";

const _tmpl$ = template(`<div class="container"><h1></h1><button>Increment</button></div>`);

const App = () => {{
  const [count, setCount] = createSignal(0);
  
  return (() => {{
    const _el$ = _tmpl$.cloneNode(true),
      _el$2 = _el$.firstChild,
      _el$3 = _el$2.nextSibling;
    
    insert(_el$2, count, null);
    _el$3.$$click = () => setCount(count() + 1);
    
    return _el$;
  }})();
}};

delegateEvents(["click"]);
"#
    );

    println!("\n=== Key Features Demonstrated ===");
    println!("✓ Template string generation from static HTML");
    println!("✓ Efficient cloneNode for element creation");
    println!("✓ Dynamic content insertion with insert()");
    println!("✓ Event handler delegation");
    println!("✓ Minimal DOM traversal with firstChild/nextSibling");
}
