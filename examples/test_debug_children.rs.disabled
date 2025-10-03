use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn main() {
    let source = r#"const x = <span> {greeting} {name} </span>;"#;

    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::jsx()).parse();

    // Find the JSX element
    use oxc_ast::Visit;
    struct Visitor;
    impl<'a> oxc_ast::Visit<'a> for Visitor {
        fn visit_jsx_element(&mut self, elem: &oxc_ast::ast::JSXElement<'a>) {
            println!("JSX element has {} children:", elem.children.len());
            for (i, child) in elem.children.iter().enumerate() {
                match child {
                    oxc_ast::ast::JSXChild::Text(t) => {
                        println!("  {}: Text({:?})", i, t.value.as_str());
                    }
                    oxc_ast::ast::JSXChild::ExpressionContainer(_) => {
                        println!("  {}: Expression", i);
                    }
                    oxc_ast::ast::JSXChild::Element(_) => {
                        println!("  {}: Element", i);
                    }
                    _ => {
                        println!("  {}: Other", i);
                    }
                }
            }
        }
    }

    let mut visitor = Visitor;
    visitor.visit_program(&ret.program);
}
