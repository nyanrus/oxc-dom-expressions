//! Test suite based on babel-plugin-jsx-dom-expressions fixtures
//!
//! This test suite uses the same test fixtures as the original babel plugin
//! to ensure compatibility and correctness.

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_traverse::traverse_mut;
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::PathBuf;

/// Helper function to load a fixture file
fn load_fixture(category: &str, filename: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push("dom");
    path.push(category);
    path.push(filename);
    
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {:?}: {}", path, e))
}

/// Test helper to transform JSX code and return the generated output
fn transform_jsx(source: &str) -> Result<String, String> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::jsx()).parse();
    
    if !ret.errors.is_empty() {
        let errors: Vec<_> = ret.errors.iter().map(|e| e.to_string()).collect();
        return Err(format!("Parse errors: {}", errors.join(", ")));
    }
    
    let mut program = ret.program;
    
    let semantic = SemanticBuilder::new()
        .build(&program)
        .semantic;
    let scoping = semantic.into_scoping();
    
    // Use the same options as the babel plugin dom.spec.js
    let options = DomExpressionsOptions::new("r-dom")
        .with_delegate_events(true)
        .with_generate(oxc_dom_expressions::GenerateMode::Dom);
    
    let mut transformer = DomExpressions::new(&allocator, options);
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());
    
    // Generate code from the transformed AST
    let generated = Codegen::new().build(&program).code;
    
    Ok(generated)
}

/// Compare actual output with expected output and print diff
fn compare_outputs(actual: &str, expected: &str, test_name: &str) -> bool {
    let diff = TextDiff::from_lines(expected, actual);
    
    let mut has_differences = false;
    let mut diff_output = String::new();
    
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => {
                has_differences = true;
                "- "
            }
            ChangeTag::Insert => {
                has_differences = true;
                "+ "
            }
            ChangeTag::Equal => "  ",
        };
        diff_output.push_str(&format!("{}{}", sign, change));
    }
    
    if has_differences {
        println!("\n❌ TEST FAILED: {}", test_name);
        println!("==================== DIFF ====================");
        println!("{}", diff_output);
        println!("==============================================\n");
        println!("Expected output length: {} chars", expected.len());
        println!("Actual output length: {} chars", actual.len());
        println!("\nNote: Full code generation is still in development.");
        println!("This test shows the current transformation output for comparison.\n");
        false
    } else {
        println!("\n✅ TEST PASSED: {}", test_name);
        true
    }
}

#[test]
fn test_simple_elements() {
    let code = load_fixture("simpleElements", "code.js");
    let expected = load_fixture("simpleElements", "output.js");
    
    match transform_jsx(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "simple_elements");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!("❌ TEST FAILED: simple_elements\nParse/transform error: {}", e);
        }
    }
}

#[test]
fn test_event_expressions() {
    let code = load_fixture("eventExpressions", "code.js");
    let expected = load_fixture("eventExpressions", "output.js");
    
    match transform_jsx(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "event_expressions");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!("❌ TEST FAILED: event_expressions\nParse/transform error: {}", e);
        }
    }
}

#[test]
fn test_attribute_expressions() {
    let code = load_fixture("attributeExpressions", "code.js");
    let expected = load_fixture("attributeExpressions", "output.js");
    
    match transform_jsx(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "attribute_expressions");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!("❌ TEST FAILED: attribute_expressions\nParse/transform error: {}", e);
        }
    }
}

#[test]
fn test_fragments() {
    let code = load_fixture("fragments", "code.js");
    let expected = load_fixture("fragments", "output.js");
    
    match transform_jsx(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "fragments");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!("❌ TEST FAILED: fragments\nParse/transform error: {}", e);
        }
    }
}

#[test]
fn test_text_interpolation() {
    let code = load_fixture("textInterpolation", "code.js");
    let expected = load_fixture("textInterpolation", "output.js");
    
    match transform_jsx(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "text_interpolation");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!("❌ TEST FAILED: text_interpolation\nParse/transform error: {}", e);
        }
    }
}

#[test]
fn test_components() {
    let code = load_fixture("components", "code.js");
    let expected = load_fixture("components", "output.js");
    
    match transform_jsx(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "components");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!("❌ TEST FAILED: components\nParse/transform error: {}", e);
        }
    }
}

#[test]
fn test_conditional_expressions() {
    let code = load_fixture("conditionalExpressions", "code.js");
    let expected = load_fixture("conditionalExpressions", "output.js");
    
    match transform_jsx(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "conditional_expressions");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!("❌ TEST FAILED: conditional_expressions\nParse/transform error: {}", e);
        }
    }
}

#[test]
fn test_insert_children() {
    let code = load_fixture("insertChildren", "code.js");
    let expected = load_fixture("insertChildren", "output.js");
    
    match transform_jsx(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "insert_children");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!("❌ TEST FAILED: insert_children\nParse/transform error: {}", e);
        }
    }
}

#[test]
fn test_custom_elements() {
    let code = load_fixture("customElements", "code.js");
    let expected = load_fixture("customElements", "output.js");
    
    match transform_jsx(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "custom_elements");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!("❌ TEST FAILED: custom_elements\nParse/transform error: {}", e);
        }
    }
}

#[test]
fn test_svg() {
    let code = load_fixture("SVG", "code.js");
    let expected = load_fixture("SVG", "output.js");
    
    match transform_jsx(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "svg");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!("❌ TEST FAILED: svg\nParse/transform error: {}", e);
        }
    }
}

#[test]
fn test_namespace_elements() {
    let code = load_fixture("namespaceElements", "code.js");
    let expected = load_fixture("namespaceElements", "output.js");
    
    match transform_jsx(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "namespace_elements");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!("❌ TEST FAILED: namespace_elements\nParse/transform error: {}", e);
        }
    }
}
