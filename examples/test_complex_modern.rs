use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_traverse::traverse_mut;

fn main() {
    println!("=== Modern Universal Output ===\n");
    
    let source_text = r#"
const TodoItem = ({ item, onDelete }) => {
  return <tr class="item">
    <td>{item.id}</td>
    <td>{item.label}</td>
    <td>
      <button onClick={(e) => onDelete(item, e)}>Delete</button>
    </td>
  </tr>;
};
    "#;

    let allocator = Allocator::default();
    let source_type = SourceType::default().with_jsx(true).with_module(true);
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let mut program = ret.program;
    let semantic = SemanticBuilder::new().build(&program).semantic;
    let scoping = semantic.into_scoping();

    let options = DomExpressionsOptions::new("solid-js/web");
    let mut transformer = DomExpressions::new(&allocator, options);
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());

    let output = Codegen::new().build(&program).code;
    println!("{}", output);
}
