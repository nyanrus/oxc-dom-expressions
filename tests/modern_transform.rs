//! Test to verify modern transform output

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_traverse::traverse_mut;

#[test]
fn test_modern_transform_basic() {
    let source_text = r#"const hello = <div>Hello World</div>;"#;

    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, SourceType::jsx()).parse();
    let mut program = ret.program;

    let semantic = SemanticBuilder::new().build(&program).semantic;
    let scoping = semantic.into_scoping();

    let options = DomExpressionsOptions::new("solid-js/web");
    let mut transformer = DomExpressions::new(&allocator, options);
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());

    let output = Codegen::new().build(&program).code;
    println!("Output:\n{}", output);

    // Check that it contains modern format functions
    assert!(output.contains("$template"));
    assert!(output.contains("$clone"));
    assert!(output.contains("_tmpl$"));
}

#[test]
fn test_modern_transform_with_attributes() {
    let source_text = r#"const template = <div id="main" class="test">Content</div>;"#;

    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, SourceType::jsx()).parse();
    let mut program = ret.program;

    let semantic = SemanticBuilder::new().build(&program).semantic;
    let scoping = semantic.into_scoping();

    let options = DomExpressionsOptions::new("solid-js/web");
    let mut transformer = DomExpressions::new(&allocator, options);
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());

    let output = Codegen::new().build(&program).code;
    println!("Output:\n{}", output);

    // Check that it contains the import
    assert!(output.contains("import"));
    assert!(output.contains("solid-runtime/polyfill"));
}
