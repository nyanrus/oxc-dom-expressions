//! Performance benchmarks for oxc-dom-expressions
//!
//! These benchmarks measure the performance of template transformation and optimization

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::hint::black_box;
use oxc_allocator::Allocator;
use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions, GenerateMode};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_traverse::traverse_mut;
use oxc_semantic::SemanticBuilder;

fn transform_jsx(source: &str, options: DomExpressionsOptions) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::jsx()).parse();
    let mut program = ret.program;
    
    let semantic = SemanticBuilder::new()
        .build(&program)
        .semantic;
    let scoping = semantic.into_scoping();
    
    let mut transformer = DomExpressions::new(&allocator, options);
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());
}

fn bench_simple_element(c: &mut Criterion) {
    let source = r#"const view = <div class="test">Hello World</div>;"#;
    
    c.bench_function("simple_element", |b| {
        b.iter(|| {
            transform_jsx(black_box(source), DomExpressionsOptions::default());
        });
    });
}

fn bench_nested_elements(c: &mut Criterion) {
    let source = r#"
        const view = <div class="container">
            <header>
                <h1>Title</h1>
                <nav>
                    <a href="/">Home</a>
                    <a href="/about">About</a>
                </nav>
            </header>
            <main>
                <p>Content here</p>
            </main>
        </div>;
    "#;
    
    c.bench_function("nested_elements", |b| {
        b.iter(|| {
            transform_jsx(black_box(source), DomExpressionsOptions::default());
        });
    });
}

fn bench_dynamic_content(c: &mut Criterion) {
    let source = r#"
        const view = <div>
            <h1>{title()}</h1>
            <p>{description()}</p>
            <button onClick={handleClick}>Click {count()}</button>
        </div>;
    "#;
    
    c.bench_function("dynamic_content", |b| {
        b.iter(|| {
            transform_jsx(black_box(source), DomExpressionsOptions::default());
        });
    });
}

fn bench_template_deduplication(c: &mut Criterion) {
    let mut group = c.benchmark_group("deduplication");
    
    for template_count in [5, 10, 20, 50].iter() {
        let source = (0..*template_count)
            .map(|i| format!(r#"const view{} = <div class="repeated">Content</div>;"#, i))
            .collect::<Vec<_>>()
            .join("\n");
        
        group.bench_with_input(
            BenchmarkId::from_parameter(template_count),
            &source,
            |b, s| {
                b.iter(|| {
                    transform_jsx(black_box(s), DomExpressionsOptions::default());
                });
            },
        );
    }
    
    group.finish();
}

fn bench_special_bindings(c: &mut Criterion) {
    let source = r#"
        const view = <div
            ref={myRef}
            classList={{ active: isActive(), disabled: isDisabled() }}
            style={{ color: getColor(), fontSize: '14px' }}
            onClick={handleClick}
        >
            Content
        </div>;
    "#;
    
    c.bench_function("special_bindings", |b| {
        b.iter(|| {
            transform_jsx(black_box(source), DomExpressionsOptions::default());
        });
    });
}

fn bench_event_delegation(c: &mut Criterion) {
    let source = r#"
        const view = <div>
            <button onClick={handler1}>Button 1</button>
            <button onClick={handler2}>Button 2</button>
            <button onClick={handler3}>Button 3</button>
            <button onClick={handler4}>Button 4</button>
            <button onClick={handler5}>Button 5</button>
        </div>;
    "#;
    
    c.bench_function("event_delegation", |b| {
        b.iter(|| {
            transform_jsx(black_box(source), DomExpressionsOptions::default());
        });
    });
}

fn bench_ssr_mode(c: &mut Criterion) {
    let source = r#"
        const page = <div class="page">
            <header><h1>Welcome</h1></header>
            <main><p>Content here</p></main>
            <footer><p>Footer</p></footer>
        </div>;
    "#;
    
    let mut group = c.benchmark_group("ssr_vs_dom");
    
    group.bench_function("dom_mode", |b| {
        b.iter(|| {
            transform_jsx(
                black_box(source),
                DomExpressionsOptions::default()
            );
        });
    });
    
    group.bench_function("ssr_mode", |b| {
        b.iter(|| {
            transform_jsx(
                black_box(source),
                DomExpressionsOptions {
                    generate: GenerateMode::Ssr,
                    hydratable: true,
                    ..Default::default()
                }
            );
        });
    });
    
    group.finish();
}

fn bench_large_template(c: &mut Criterion) {
    // Generate a large template with many elements
    let elements = (0..100)
        .map(|i| format!(r#"<div class="item-{}">{}</div>"#, i, i))
        .collect::<Vec<_>>()
        .join("");
    let source = format!(r#"const view = <div>{}</div>;"#, elements);
    
    c.bench_function("large_template", |b| {
        b.iter(|| {
            transform_jsx(black_box(&source), DomExpressionsOptions::default());
        });
    });
}

fn bench_optimization_statistics(c: &mut Criterion) {
    let source = r#"
        const a = <div>Repeated</div>;
        const b = <div>Repeated</div>;
        const c = <div>Repeated</div>;
        const d = <span>Unique</span>;
        const e = <p>Another</p>;
    "#;
    
    c.bench_function("optimization_stats", |b| {
        b.iter(|| {
            let allocator = Allocator::default();
            let ret = Parser::new(&allocator, black_box(source), SourceType::jsx()).parse();
            let mut program = ret.program;
            
            let semantic = SemanticBuilder::new()
                .build(&program)
                .semantic;
            let scoping = semantic.into_scoping();
            
            let mut transformer = DomExpressions::new(&allocator, DomExpressionsOptions::default());
            traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());
            
            // Get optimization statistics
            let _stats = transformer.get_template_stats();
            let _reused = transformer.get_reused_templates();
        });
    });
}

criterion_group!(
    benches,
    bench_simple_element,
    bench_nested_elements,
    bench_dynamic_content,
    bench_template_deduplication,
    bench_special_bindings,
    bench_event_delegation,
    bench_ssr_mode,
    bench_large_template,
    bench_optimization_statistics,
);

criterion_main!(benches);
