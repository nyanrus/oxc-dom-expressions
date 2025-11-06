// Comprehensive demo showing full feature support in modern transform

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_traverse::traverse_mut;

fn main() {
    println!("=== Modern Transform - Full Feature Support ===\n");
    
    let examples = vec![
        ("Static Template", r#"const App = () => <div><h1>Hello World</h1></div>;"#),
        ("Dynamic Text", r#"const App = () => <div><h1>{message}</h1></div>;"#),
        ("Dynamic Attribute", r#"const App = () => <div id={dynamicId}>Content</div>;"#),
        ("Event Handler", r#"const App = () => <button onClick={handleClick}>Click</button>;"#),
        ("Complex", r#"
const TodoItem = ({ item }) => (
  <tr>
    <td>{item.id}</td>
    <td>{item.label}</td>
  </tr>
);"#),
    ];

    for (name, source) in examples {
        println!("### {} ###", name);
        transform_and_print(source);
        println!();
    }
}

fn transform_and_print(source_text: &str) {
    let allocator = Allocator::default();
    let source_type = SourceType::default().with_jsx(true).with_module(true);
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    
    if !ret.errors.is_empty() {
        println!("  Parse errors!");
        return;
    }
    
    let mut program = ret.program;
    let semantic = SemanticBuilder::new().build(&program).semantic;
    let scoping = semantic.into_scoping();

    let options = DomExpressionsOptions::new("solid-js/web");
    let mut transformer = DomExpressions::new(&allocator, options);
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());

    let output = Codegen::new().build(&program).code;
    
    // Print with indentation
    for line in output.lines() {
        println!("  {}", line);
    }
}
