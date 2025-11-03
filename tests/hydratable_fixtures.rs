//! Test suite for hydratable mode based on babel-plugin-jsx-dom-expressions fixtures
//!
//! This test suite uses the hydratable test fixtures from the original babel plugin

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions, GenerateMode};
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
    path.push("hydratable");
    path.push(category);
    path.push(filename);

    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read fixture {:?}: {}", path, e))
}

/// Test helper to transform JSX code in hydratable mode
fn transform_jsx_hydratable(source: &str) -> Result<String, String> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::jsx()).parse();

    if !ret.errors.is_empty() {
        let errors: Vec<_> = ret.errors.iter().map(|e| e.to_string()).collect();
        return Err(format!("Parse errors: {}", errors.join(", ")));
    }

    let mut program = ret.program;

    let semantic = SemanticBuilder::new().build(&program).semantic;
    let scoping = semantic.into_scoping();

    // Use hydratable mode options
    let options = DomExpressionsOptions::new("r-dom").with_generate(GenerateMode::Dom);

    let mut transformer = DomExpressions::new(&allocator, options);
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());

    // Generate code from the transformed AST
    let mut generated = Codegen::new().build(&program).code;

    // Post-process to match expected format
    generated = normalize_output(&generated);

    Ok(generated)
}

/// Normalize output to match expected format from babel plugin
fn normalize_output(code: &str) -> String {
    let mut result = code.to_string();

    // Replace /* @__PURE__ */ with /*#__PURE__*/
    result = result.replace("/* @__PURE__ */", "/*#__PURE__*/");

    // Format multi-line variable declarations
    // Replace all instances of ", _tmpl$" with ",\n  _tmpl$" in the entire code
    result = result.replace(", _tmpl$", ",\n  _tmpl$");

    result
}

/// Compare actual output with expected output and print diff
fn compare_outputs(actual: &str, expected: &str, test_name: &str) -> bool {
    // Normalize both strings for comparison (remove formatting differences)
    let normalized_actual = normalize_for_comparison(actual);
    let normalized_expected = normalize_for_comparison(expected);

    if normalized_actual == normalized_expected {
        println!("\n✅ TEST PASSED: {} (Hydratable)", test_name);
        return true;
    }

    // If normalized versions don't match, show the diff
    let diff = TextDiff::from_lines(expected, actual);

    let mut diff_output = String::new();

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "- ",
            ChangeTag::Insert => "+ ",
            ChangeTag::Equal => "  ",
        };
        diff_output.push_str(&format!("{}{}", sign, change));
    }

    println!("\n❌ TEST FAILED: {} (Hydratable)", test_name);
    println!("==================== DIFF ====================");
    println!("{}", diff_output);
    println!("==============================================\n");
    println!("Expected output length: {} chars", expected.len());
    println!("Actual output length: {} chars", actual.len());

    false
}

/// Normalize code for comparison by removing insignificant whitespace
fn normalize_for_comparison(code: &str) -> String {
    let mut result = code.to_string();

    // Remove all newlines and carriage returns
    result = result.replace(['\n', '\r'], "");

    // Remove all indentation spaces (multiple spaces in a row)
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }

    // Remove spaces after opening parens and before closing parens
    result = result.replace("( ", "(").replace(" )", ")");

    result
}

#[test]
fn test_hydratable_simple_elements() {
    let code = load_fixture("simpleElements", "code.js");
    let expected = load_fixture("simpleElements", "output.js");

    match transform_jsx_hydratable(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "simple_elements");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!(
                "❌ TEST FAILED: simple_elements (Hydratable)\nParse/transform error: {}",
                e
            );
        }
    }
}

#[test]
fn test_hydratable_event_expressions() {
    let code = load_fixture("eventExpressions", "code.js");
    let expected = load_fixture("eventExpressions", "output.js");

    match transform_jsx_hydratable(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "event_expressions");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!(
                "❌ TEST FAILED: event_expressions (Hydratable)\nParse/transform error: {}",
                e
            );
        }
    }
}

#[test]
fn test_hydratable_attribute_expressions() {
    let code = load_fixture("attributeExpressions", "code.js");
    let expected = load_fixture("attributeExpressions", "output.js");

    match transform_jsx_hydratable(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "attribute_expressions");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!(
                "❌ TEST FAILED: attribute_expressions (Hydratable)\nParse/transform error: {}",
                e
            );
        }
    }
}

#[test]
fn test_hydratable_fragments() {
    let code = load_fixture("fragments", "code.js");
    let expected = load_fixture("fragments", "output.js");

    match transform_jsx_hydratable(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "fragments");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!(
                "❌ TEST FAILED: fragments (Hydratable)\nParse/transform error: {}",
                e
            );
        }
    }
}

#[test]
fn test_hydratable_text_interpolation() {
    let code = load_fixture("textInterpolation", "code.js");
    let expected = load_fixture("textInterpolation", "output.js");

    match transform_jsx_hydratable(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "text_interpolation");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!(
                "❌ TEST FAILED: text_interpolation (Hydratable)\nParse/transform error: {}",
                e
            );
        }
    }
}

#[test]
fn test_hydratable_components() {
    let code = load_fixture("components", "code.js");
    let expected = load_fixture("components", "output.js");

    match transform_jsx_hydratable(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "components");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!(
                "❌ TEST FAILED: components (Hydratable)\nParse/transform error: {}",
                e
            );
        }
    }
}

#[test]
fn test_hydratable_conditional_expressions() {
    let code = load_fixture("conditionalExpressions", "code.js");
    let expected = load_fixture("conditionalExpressions", "output.js");

    match transform_jsx_hydratable(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "conditional_expressions");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!(
                "❌ TEST FAILED: conditional_expressions (Hydratable)\nParse/transform error: {}",
                e
            );
        }
    }
}

#[test]
fn test_hydratable_insert_children() {
    let code = load_fixture("insertChildren", "code.js");
    let expected = load_fixture("insertChildren", "output.js");

    match transform_jsx_hydratable(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "insert_children");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!(
                "❌ TEST FAILED: insert_children (Hydratable)\nParse/transform error: {}",
                e
            );
        }
    }
}

#[test]
fn test_hydratable_custom_elements() {
    let code = load_fixture("customElements", "code.js");
    let expected = load_fixture("customElements", "output.js");

    match transform_jsx_hydratable(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "custom_elements");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!(
                "❌ TEST FAILED: custom_elements (Hydratable)\nParse/transform error: {}",
                e
            );
        }
    }
}

#[test]
fn test_hydratable_svg() {
    let code = load_fixture("SVG", "code.js");
    let expected = load_fixture("SVG", "output.js");

    match transform_jsx_hydratable(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "svg");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!(
                "❌ TEST FAILED: svg (Hydratable)\nParse/transform error: {}",
                e
            );
        }
    }
}

#[test]
fn test_hydratable_flags() {
    let code = load_fixture("flags", "code.js");
    let expected = load_fixture("flags", "output.js");

    match transform_jsx_hydratable(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "flags");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!(
                "❌ TEST FAILED: flags (Hydratable)\nParse/transform error: {}",
                e
            );
        }
    }
}

#[test]
fn test_hydratable_document() {
    let code = load_fixture("document", "code.js");
    let expected = load_fixture("document", "output.js");

    match transform_jsx_hydratable(&code) {
        Ok(actual) => {
            let matches = compare_outputs(&actual, &expected, "document");
            assert!(matches, "Output does not match expected result");
        }
        Err(e) => {
            panic!(
                "❌ TEST FAILED: document (Hydratable)\nParse/transform error: {}",
                e
            );
        }
    }
}
