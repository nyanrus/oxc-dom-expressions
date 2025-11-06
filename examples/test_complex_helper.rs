use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_traverse::traverse_mut;

fn main() {
    let source_text = r#"
const view = ({ item }) => {
  const itemId = item.id;
  return <tr class={itemId === selected() ? "danger" : ""}>
    <td class="col-md-1">{itemId}</td>
    <td class="col-md-4">
      <a onclick={e => select(item, e)}>{item.label}</a>
    </td>
  </tr>;
};
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
    
    // Verify key features
    let checks = vec![
        ("Helper $template", output.contains("function $template")),
        ("Helper $clone", output.contains("function $clone")),
        ("Helper $bind", output.contains("function $bind")),
        ("Import from runtime", output.contains("solid-js/web")),
        ("Template creation", output.contains("const _tmpl$")),
        ("Clone usage", output.contains("$clone(_tmpl$)")),
    ];
    
    println!("\nVerification:");
    for (check_name, passed) in checks {
        if passed {
            println!("✅ {}", check_name);
        } else {
            println!("❌ {}", check_name);
        }
    }
}
