//! Phase 4 demonstration example
//!
//! This example demonstrates the optimization features implemented in Phase 4:
//! - Template deduplication
//! - Static analysis
//! - Performance statistics
//! - SSR mode optimizations

use oxc_allocator::Allocator;
use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions, GenerateMode};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_traverse::traverse_mut;

fn main() {
    println!("=== Phase 4: Optimization Demo ===\n");

    // Example 1: Template Deduplication
    println!("Example 1: Template Deduplication");
    println!("Input:  Multiple identical templates");
    demonstrate_deduplication();

    println!("\n---\n");

    // Example 2: Mixed Templates
    println!("Example 2: Mixed Unique and Duplicate Templates");
    println!("Input:  Some duplicates, some unique");
    demonstrate_mixed_templates();

    println!("\n---\n");

    // Example 3: Static vs Dynamic Analysis
    println!("Example 3: Static vs Dynamic Template Analysis");
    println!("Input:  Mix of static and dynamic templates");
    demonstrate_static_vs_dynamic();

    println!("\n---\n");

    // Example 4: Space Savings
    println!("Example 4: Space Savings from Deduplication");
    println!("Input:  Large repeated templates");
    demonstrate_space_savings();

    println!("\n---\n");

    // Example 5: SSR Mode Optimization
    println!("Example 5: SSR Mode with Optimization");
    println!("Input:  Templates in SSR mode");
    demonstrate_ssr_optimization();

    println!("\n=== Phase 4 Features Summary ===");
    println!("✅ Template deduplication - Reuse identical templates");
    println!("✅ Static analysis - Identify static vs dynamic templates");
    println!("✅ Performance metrics - Track space savings and efficiency");
    println!("✅ SSR mode support - Optimizations work in SSR mode");
    println!("✅ Optimization reporting - Detailed statistics available");
}

fn demonstrate_deduplication() {
    let source = r#"
        const button1 = <button class="primary">Click</button>;
        const button2 = <button class="primary">Click</button>;
        const button3 = <button class="primary">Click</button>;
        const button4 = <button class="primary">Click</button>;
        const button5 = <button class="primary">Click</button>;
    "#;

    let (stats, _reused) = analyze_source(source, DomExpressionsOptions::default());

    println!("Analysis:");
    println!("  Total templates encountered: {}", stats.total_templates);
    println!("  Unique templates: {}", stats.unique_templates);
    println!("  Templates reused: {}", stats.reused_templates);
    println!(
        "  Deduplication ratio: {:.1}%",
        stats.deduplication_ratio() * 100.0
    );
    println!("\nExpected output:");
    println!("  - Only 1 template variable created (_tmpl$)");
    println!("  - All 5 buttons clone from the same template");
    println!("  - Significant memory savings");
}

fn demonstrate_mixed_templates() {
    let source = r#"
        const div1 = <div class="box">Content</div>;
        const div2 = <div class="box">Content</div>;
        const span1 = <span>Text</span>;
        const p1 = <p>Paragraph</p>;
        const div3 = <div class="box">Content</div>;
    "#;

    let (stats, reused) = analyze_source(source, DomExpressionsOptions::default());

    println!("Analysis:");
    println!("  Total templates encountered: {}", stats.total_templates);
    println!("  Unique templates: {}", stats.unique_templates);
    println!("  Templates reused: {}", stats.reused_templates);
    println!(
        "  Deduplication ratio: {:.1}%",
        stats.deduplication_ratio() * 100.0
    );
    println!("\nReused templates:");
    for (html, count) in reused {
        println!(
            "  - Used {} times: {}",
            count,
            html.chars().take(50).collect::<String>()
        );
    }
    println!("\nExpected output:");
    println!("  - 3 template variables created");
    println!("  - div.box template used 3 times");
    println!("  - span and p templates used once each");
}

fn demonstrate_static_vs_dynamic() {
    let source = r#"
        const static1 = <div class="static">Fixed Content</div>;
        const static2 = <span>More Static</span>;
        const static3 = <p>Also Static</p>;
        const dynamic1 = <div>{count()}</div>;
        const dynamic2 = <span>{name()}</span>;
        const dynamic3 = <p>{value()}</p>;
    "#;

    let (stats, _reused) = analyze_source(source, DomExpressionsOptions::default());

    println!("Analysis:");
    println!("  Total templates: {}", stats.total_templates);
    println!("  Static templates: {}", stats.static_templates);
    println!("  Dynamic templates: {}", stats.dynamic_templates);
    println!("\nStatic templates:");
    println!("  - No runtime reactivity needed");
    println!("  - Can be aggressively cached");
    println!("  - Optimal for SSR");
    println!("\nDynamic templates:");
    println!("  - Require effect wrappers");
    println!("  - Need reactive tracking");
    println!("  - More runtime overhead");
}

fn demonstrate_space_savings() {
    let source = r#"
        const card1 = <div class="card"><h2>Title</h2><p>Content here</p><button>Action</button></div>;
        const card2 = <div class="card"><h2>Title</h2><p>Content here</p><button>Action</button></div>;
        const card3 = <div class="card"><h2>Title</h2><p>Content here</p><button>Action</button></div>;
        const card4 = <div class="card"><h2>Title</h2><p>Content here</p><button>Action</button></div>;
        const card5 = <div class="card"><h2>Title</h2><p>Content here</p><button>Action</button></div>;
        const card6 = <div class="card"><h2>Title</h2><p>Content here</p><button>Action</button></div>;
        const card7 = <div class="card"><h2>Title</h2><p>Content here</p><button>Action</button></div>;
        const card8 = <div class="card"><h2>Title</h2><p>Content here</p><button>Action</button></div>;
        const card9 = <div class="card"><h2>Title</h2><p>Content here</p><button>Action</button></div>;
        const card10 = <div class="card"><h2>Title</h2><p>Content here</p><button>Action</button></div>;
    "#;

    let (stats, _reused) = analyze_source(source, DomExpressionsOptions::default());

    println!("Analysis:");
    println!(
        "  Total HTML size (without deduplication): {} bytes",
        stats.total_html_size
    );
    println!(
        "  Deduplicated HTML size: {} bytes",
        stats.deduplicated_html_size
    );
    println!(
        "  Space saved: {} bytes ({:.1}%)",
        stats.space_saved(),
        (stats.space_saved() as f64 / stats.total_html_size as f64) * 100.0
    );
    println!(
        "  Average template size: {:.1} bytes",
        stats.average_template_size()
    );
    println!("\nBenefit:");
    println!("  - Reduces bundle size");
    println!("  - Faster parsing");
    println!("  - Better browser caching");
    println!("  - Lower memory usage");
}

fn demonstrate_ssr_optimization() {
    let source = r#"
        const page1 = <div class="page"><h1>Welcome</h1><p>Content</p></div>;
        const page2 = <div class="page"><h1>Welcome</h1><p>Content</p></div>;
        const page3 = <div class="page"><h1>Welcome</h1><p>Content</p></div>;
    "#;

    let options = DomExpressionsOptions {
        generate: GenerateMode::Ssr,
        hydratable: true,
        ..Default::default()
    };

    let (stats, _reused) = analyze_source(source, options);

    println!("Analysis (SSR Mode):");
    println!("  Mode: Server-Side Rendering");
    println!("  Hydratable: Yes");
    println!("  Total templates: {}", stats.total_templates);
    println!("  Unique templates: {}", stats.unique_templates);
    println!(
        "  Deduplication ratio: {:.1}%",
        stats.deduplication_ratio() * 100.0
    );
    println!("\nSSR Optimization Benefits:");
    println!("  - Templates deduplicated on server");
    println!("  - Smaller server-rendered HTML");
    println!("  - Faster hydration on client");
    println!("  - Reduced memory on server");
}

fn analyze_source(
    source: &str,
    options: DomExpressionsOptions,
) -> (oxc_dom_expressions::TemplateStats, Vec<(String, usize)>) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::jsx()).parse();
    let mut program = ret.program;

    let semantic = SemanticBuilder::new().build(&program).semantic;
    let scoping = semantic.into_scoping();

    let mut transformer = DomExpressions::new(&allocator, options);
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());

    let stats = transformer.get_template_stats();
    let reused = transformer.get_reused_templates();

    (stats, reused)
}
