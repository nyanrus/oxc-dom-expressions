use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_traverse::traverse_mut;

fn main() {
    let source_text = r#"
const App = () => <div id="main">
  <h1>Hello World</h1>
</div>;
    "#;

    let allocator = Allocator::default();
    let source_type = SourceType::default().with_jsx(true).with_module(true);
    let ret = Parser::new(&allocator, source_text, source_type).parse();

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

    let options = DomExpressionsOptions::new("solid-js/web");
    let mut transformer = DomExpressions::new(&allocator, options);

    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());

    let output = Codegen::new().build(&program).code;
    println!("Transformed output:");
    println!("{}", output);
    
    // Check that the helper is injected
    if output.contains("$template") && output.contains("$clone") && output.contains("$bind") {
        println!("\n✅ Success! Helper functions are injected.");
    } else {
        println!("\n❌ Error: Helper functions not found in output.");
    }
    
    // Check that it imports from solid-js/web
    if output.contains("solid-js/web") {
        println!("✅ Success! Imports from solid-js/web.");
    } else {
        println!("❌ Error: Doesn't import from solid-js/web.");
    }
}
