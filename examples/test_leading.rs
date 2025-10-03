use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions, GenerateMode};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_traverse::traverse_mut;

fn main() {
    let source = r#"const leadingExpr = <span>{greeting} John</span>;"#;
    
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::jsx()).parse();
    
    if !ret.errors.is_empty() {
        println!("Parse errors:");
        for e in &ret.errors {
            println!("  {}", e);
        }
        return;
    }
    
    let mut program = ret.program;
    
    let semantic = SemanticBuilder::new()
        .build(&program)
        .semantic;
    let scoping = semantic.into_scoping();
    
    let options = DomExpressionsOptions::new("r-dom")
        .with_delegate_events(true)
        .with_generate(GenerateMode::Dom);
    
    let mut transformer = DomExpressions::new(&allocator, options);
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());
    
    let generated = Codegen::new().build(&program).code;
    
    println!("Generated code:");
    println!("{}", generated);
}
