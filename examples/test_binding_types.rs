use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_traverse::traverse_mut;

fn test_transformation(name: &str, source: &str, expected_features: Vec<&str>) {
    println!("\n=== Testing: {} ===", name);
    
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_jsx(true).with_module(true);
    let ret = Parser::new(&allocator, source, source_type).parse();

    if !ret.errors.is_empty() {
        println!("❌ Parse errors:");
        for error in &ret.errors {
            println!("  {}", error);
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

    let mut all_passed = true;
    for feature in expected_features {
        if output.contains(feature) {
            println!("  ✅ Contains: {}", feature);
        } else {
            println!("  ❌ Missing: {}", feature);
            all_passed = false;
        }
    }
    
    if all_passed {
        println!("✅ All checks passed for: {}", name);
    } else {
        println!("❌ Some checks failed for: {}", name);
        println!("\nOutput:\n{}", output);
    }
}

fn main() {
    println!("Testing Helper Injection with Various Binding Types\n");
    
    // Test 1: Basic template
    test_transformation(
        "Basic Template",
        r#"const App = () => <div>Hello</div>;"#,
        vec![
            "function $template",
            "function $clone",
            "function $bind",
            "import {",
            "solid-js/web",
        ],
    );

    // Test 2: Dynamic attributes
    test_transformation(
        "Dynamic Attributes",
        r#"const App = () => <div id={dynamicId} class={className}>Content</div>;"#,
        vec![
            "$template",
            "$clone",
            "const _tmpl$",
        ],
    );

    // Test 3: Event handlers
    test_transformation(
        "Event Handlers",
        r#"const App = () => <button onClick={handleClick}>Click me</button>;"#,
        vec![
            "$template",
            "$clone",
        ],
    );

    // Test 4: Ref binding
    test_transformation(
        "Ref Binding",
        r#"const App = () => { let ref; return <div ref={ref}>Content</div>; }"#,
        vec![
            "$template",
            "$clone",
        ],
    );

    // Test 5: Multiple elements
    test_transformation(
        "Multiple Elements",
        r#"const App = () => <div><h1>Title</h1><p>Paragraph</p></div>;"#,
        vec![
            "$template",
            "$clone",
            "const _tmpl$",
        ],
    );

    // Test 6: classList binding
    test_transformation(
        "classList Binding",
        r#"const App = () => <div classList={{ active: isActive(), selected: true }}>Content</div>;"#,
        vec![
            "$template",
            "$clone",
        ],
    );

    // Test 7: style binding
    test_transformation(
        "Style Binding", 
        r#"const App = () => <div style={{ color: 'red', background: bgColor() }}>Content</div>;"#,
        vec![
            "$template",
            "$clone",
        ],
    );

    // Test 8: Fragment
    test_transformation(
        "Fragment",
        r#"const App = () => <><div>First</div><div>Second</div></>;"#,
        vec![
            "$template",
            "$clone",
        ],
    );

    // Test 9: Component
    test_transformation(
        "Component",
        r#"const App = () => <MyComponent prop={value} />;"#,
        vec![
            "$template",
            "$clone",
        ],
    );

    // Test 10: Text interpolation
    test_transformation(
        "Text Interpolation",
        r#"const App = () => <div>{message}</div>;"#,
        vec![
            "$template",
            "$clone",
        ],
    );

    println!("\n=== Test Suite Complete ===");
}
