use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_ast::ast::*;

fn main() {
    let source = r#"<span> {greeting} {name} </span>"#;
    
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::jsx()).parse();
    
    if let Some(Statement::ExpressionStatement(stmt)) = ret.program.body.first() {
        if let Expression::JSXElement(elem) = &stmt.expression {
            println!("Number of children: {}", elem.children.len());
            for (i, child) in elem.children.iter().enumerate() {
                match child {
                    JSXChild::Text(text) => println!("  {}: Text({:?})", i, text.value.as_str()),
                    JSXChild::ExpressionContainer(_) => println!("  {}: Expression", i),
                    _ => println!("  {}: Other", i),
                }
            }
        }
    }
}
