use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_traverse::traverse_mut;

fn main() {
    let cases = vec![
        ("trailing", r#"<span>Hello {name}</span>"#),
        ("leading", r#"<span>{greeting} John</span>"#),
        ("multi", r#"<span>{greeting} {name}</span>"#),
        ("multi_spaced", r#"<span> {greeting} {name} </span>"#),
    ];

    for (name, source) in cases {
        println!("\n=== {} ===", name);
        let full_source = format!("const el = {};", source);

        let allocator = Allocator::default();
        let ret = Parser::new(&allocator, &full_source, SourceType::jsx()).parse();

        if !ret.errors.is_empty() {
            eprintln!("Parse errors: {:?}", ret.errors);
            continue;
        }

        let mut program = ret.program;
        let semantic = SemanticBuilder::new().build(&program).semantic;
        let scoping = semantic.into_scoping();

        let options = DomExpressionsOptions::new("r-dom")
            .with_delegate_events(true)
            .with_generate(oxc_dom_expressions::GenerateMode::Dom);

        let mut transformer = DomExpressions::new(&allocator, options);
        traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());

        let generated = Codegen::new().build(&program).code;
        println!("{}", generated);
    }
}
