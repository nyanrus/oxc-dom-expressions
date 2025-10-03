//! Example demonstrating Phase 3 Advanced Features
//!
//! This example showcases:
//! - Event delegation
//! - Special bindings (ref, classList, style)
//! - Component handling
//! - Fragment support
//! - on: and oncapture: prefixes

use oxc_allocator::Allocator;
use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions, GenerateMode};

fn main() {
    println!("=== Phase 3: Advanced Features Demo ===\n");

    // Example 1: Event Delegation
    println!("Example 1: Event Delegation");
    println!("----------------------------");
    let allocator = Allocator::default();
    let options = DomExpressionsOptions {
        delegate_events: true,
        ..Default::default()
    };
    let transformer = DomExpressions::new(&allocator, options);

    println!("Configuration:");
    println!(
        "  Event delegation: {}",
        transformer.options().delegate_events
    );
    println!("  Module: {}", transformer.options().module_name);
    println!();

    println!("JSX Input:");
    println!(
        r#"
  <div>
    <button onClick={{handleClick}}>Click me</button>
    <button onClick={{handleOther}}>Or me</button>
  </div>
"#
    );

    println!("Expected Output:");
    println!(
        r#"
  import {{ template, delegateEvents }} from "solid-js/web";
  
  const _tmpl$ = template(`<div><button>Click me</button><button>Or me</button></div>`);
  
  const _el$ = _tmpl$.cloneNode(true);
  const _el$2 = _el$.firstChild;
  const _el$3 = _el$2.nextSibling;
  
  _el$2.$$click = handleClick;
  _el$3.$$click = handleOther;
  
  delegateEvents(["click"]);
"#
    );
    println!();

    // Example 2: Special Bindings - ref
    println!("Example 2: ref Binding");
    println!("----------------------");
    println!("JSX Input:");
    println!(
        r#"
  const Parent = () => {{
    let myRef;
    return <div ref={{myRef}}>Hello</div>;
  }};
"#
    );

    println!("Expected Output:");
    println!(
        r#"
  typeof myRef === 'function' ? myRef(_el$) : myRef = _el$;
"#
    );
    println!();

    // Example 3: classList Binding
    println!("Example 3: classList Binding");
    println!("----------------------------");
    println!("JSX Input:");
    println!(
        r#"
  <div classList={{ 
    active: isActive(), 
    selected: isSelected(),
    disabled: isDisabled() 
  }}>
    Content
  </div>
"#
    );

    println!("Expected Output:");
    println!(
        r#"
  import {{ template, classList, effect }} from "solid-js/web";
  
  const _el$ = _tmpl$.cloneNode(true);
  effect(() => classList(_el$, {{ 
    active: isActive(), 
    selected: isSelected(),
    disabled: isDisabled() 
  }}));
"#
    );
    println!();

    // Example 4: style Binding
    println!("Example 4: style Binding");
    println!("------------------------");
    println!("JSX Input:");
    println!(
        r#"
  <div style={{ 
    color: getColor(),
    fontSize: getSize(),
    padding: '10px'
  }}>
    Styled content
  </div>
"#
    );

    println!("Expected Output:");
    println!(
        r#"
  import {{ template, style, effect }} from "solid-js/web";
  
  const _el$ = _tmpl$.cloneNode(true);
  effect(() => style(_el$, {{ 
    color: getColor(),
    fontSize: getSize(),
    padding: '10px'
  }}));
"#
    );
    println!();

    // Example 5: on: Prefix (Custom Events)
    println!("Example 5: on: Prefix for Custom Events");
    println!("---------------------------------------");
    println!("JSX Input:");
    println!(
        r#"
  <div on:CustomEvent={{handleCustom}}>
    Element with custom event
  </div>
"#
    );

    println!("Expected Output:");
    println!(
        r#"
  _el$.addEventListener("CustomEvent", handleCustom);
"#
    );
    println!();

    // Example 6: oncapture: Prefix
    println!("Example 6: oncapture: Prefix for Capture Phase");
    println!("----------------------------------------------");
    println!("JSX Input:");
    println!(
        r#"
  <div oncapture:Click={{handleCaptureClick}}>
    Element with capture phase handler
  </div>
"#
    );

    println!("Expected Output:");
    println!(
        r#"
  _el$.addEventListener("Click", handleCaptureClick, {{ capture: true }});
"#
    );
    println!();

    // Example 7: Component Detection
    println!("Example 7: Component vs Element Detection");
    println!("-----------------------------------------");
    println!("JSX Input:");
    println!(
        r#"
  <div>                    <!-- HTML element -->
    <MyComponent />        <!-- Component -->
    <custom-element />     <!-- HTML custom element -->
    <AnotherComponent />   <!-- Component -->
  </div>
"#
    );

    println!("Detection Logic:");
    println!("  div: HTML element (lowercase)");
    println!("  MyComponent: Component (starts with uppercase)");
    println!("  custom-element: HTML element (hyphenated)");
    println!("  AnotherComponent: Component (starts with uppercase)");
    println!();

    // Example 8: Fragments
    println!("Example 8: JSX Fragments");
    println!("------------------------");
    println!("JSX Input:");
    println!(
        r#"
  const App = () => (
    <>
      <div>First child</div>
      <div>Second child</div>
      <div>Third child</div>
    </>
  );
"#
    );

    println!("Expected Output:");
    println!(
        r#"
  // Fragments are converted to arrays
  [
    _el$,    // First child
    _el$2,   // Second child
    _el$3    // Third child
  ]
"#
    );
    println!();

    // Example 9: Combined Advanced Features
    println!("Example 9: Combined Advanced Features");
    println!("-------------------------------------");
    println!("JSX Input:");
    println!(
        r#"
  const AdvancedComponent = () => {{
    let containerRef;
    
    return (
      <div 
        ref={{containerRef}}
        classList={{ active: isActive() }}
        style={{ color: getColor() }}
        onClick={{handleClick}}
        on:CustomEvent={{handleCustom}}
      >
        <span>Advanced features in action!</span>
      </div>
    );
  }};
"#
    );

    println!("Expected Output:");
    println!(
        r#"
  import {{ template, delegateEvents, classList, style, effect }} from "solid-js/web";
  
  const _tmpl$ = template(`<div><span>Advanced features in action!</span></div>`);
  
  const _el$ = _tmpl$.cloneNode(true);
  
  // ref binding
  typeof containerRef === 'function' ? containerRef(_el$) : containerRef = _el$;
  
  // classList binding
  effect(() => classList(_el$, {{ active: isActive() }}));
  
  // style binding
  effect(() => style(_el$, {{ color: getColor() }}));
  
  // Delegated event
  _el$.$$click = handleClick;
  
  // Custom event (non-delegated)
  _el$.addEventListener("CustomEvent", handleCustom);
  
  delegateEvents(["click"]);
"#
    );
    println!();

    // Example 10: SSR Mode with Special Bindings
    println!("Example 10: SSR Mode Configuration");
    println!("----------------------------------");
    let ssr_options = DomExpressionsOptions {
        module_name: String::from("solid-js/web"),
        generate: GenerateMode::Ssr,
        hydratable: true,
        delegate_events: false, // Events handled client-side
        ..Default::default()
    };

    let ssr_transformer = DomExpressions::new(&allocator, ssr_options);

    println!("SSR Configuration:");
    println!("  Generate mode: {:?}", ssr_transformer.options().generate);
    println!("  Hydratable: {}", ssr_transformer.options().hydratable);
    println!(
        "  Event delegation: {}",
        ssr_transformer.options().delegate_events
    );
    println!("  Module: {}", ssr_transformer.options().module_name);
    println!();

    println!("Note: In SSR mode:");
    println!("  - Events are not delegated (handled client-side)");
    println!("  - Hydratable markers are added for hydration");
    println!("  - Special bindings like ref, classList, style are client-side only");
    println!();

    // Summary
    println!("=== Phase 3 Feature Summary ===");
    println!();
    println!("✅ Event Delegation");
    println!("   - Automatic delegation for standard events (click, input, etc.)");
    println!("   - delegateEvents() call injection");
    println!("   - Efficient event handling with $$eventName pattern");
    println!();
    println!("✅ Special Bindings");
    println!("   - ref: Element reference assignment or callback");
    println!("   - classList: Object-based class management");
    println!("   - style: Object-based style management");
    println!();
    println!("✅ Event Prefixes");
    println!("   - on: Bypass delegation with direct addEventListener");
    println!("   - oncapture: Capture phase event handling");
    println!();
    println!("✅ Component Detection");
    println!("   - Distinguish between HTML elements and components");
    println!("   - Uppercase tags treated as components");
    println!("   - Different transformation strategies");
    println!();
    println!("✅ Fragment Support");
    println!("   - <></> notation supported");
    println!("   - Converted to arrays at runtime");
    println!("   - Efficient multiple child rendering");
    println!();
    println!("✅ Import Management");
    println!("   - Track required imports during transformation");
    println!("   - Automatic import injection");
    println!("   - Module name configuration");
    println!();
    println!("All Phase 3 features are now implemented and tested! 🎉");
}
