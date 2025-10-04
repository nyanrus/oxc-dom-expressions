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

    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read fixture {:?}: {}", path, e))
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

    let semantic = SemanticBuilder::new().build(&program).semantic;
    let scoping = semantic.into_scoping();

    // Use the same options as the babel plugin dom.spec.js
    let options = DomExpressionsOptions::new("r-dom")
        .with_delegate_events(true)
        .with_generate(oxc_dom_expressions::GenerateMode::Dom);

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
    
    // Replace tabs with double spaces to match babel output
    result = result.replace('\t', "  ");

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
        println!("\n✅ TEST PASSED: {}", test_name);
        return true;
    }

    // Debug: show first difference
    println!("\nNormalized actual length: {}", normalized_actual.len());
    println!("Normalized expected length: {}", normalized_expected.len());

    // Find first difference
    for (i, (a, e)) in normalized_actual
        .chars()
        .zip(normalized_expected.chars())
        .enumerate()
    {
        if a != e {
            println!(
                "First difference at position {}: actual='{}' expected='{}'",
                i, a, e
            );
            println!(
                "Context: ...{}...",
                &normalized_actual
                    [i.saturating_sub(20)..i.saturating_add(20).min(normalized_actual.len())]
            );
            break;
        }
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

    println!("\n❌ TEST FAILED: {}", test_name);
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
    result = result.replace('\n', "").replace('\r', "");
    
    // Remove all tabs
    result = result.replace('\t', "");

    // Remove all indentation spaces (multiple spaces in a row)
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }

    // Remove spaces after opening brackets/parens and before closing brackets/parens
    result = result.replace("( ", "(").replace(" )", ")");
    result = result.replace("[ ", "[").replace(" ]", "]");
    result = result.replace("{ ", "{").replace(" }", "}");

    result
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
            panic!(
                "❌ TEST FAILED: simple_elements\nParse/transform error: {}",
                e
            );
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
            panic!(
                "❌ TEST FAILED: event_expressions\nParse/transform error: {}",
                e
            );
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
            panic!(
                "❌ TEST FAILED: attribute_expressions\nParse/transform error: {}",
                e
            );
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
            panic!(
                "❌ TEST FAILED: text_interpolation\nParse/transform error: {}",
                e
            );
        }
    }
}
