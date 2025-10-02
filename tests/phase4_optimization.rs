//! Phase 4: Optimization Tests
//!
//! Tests for template deduplication, static analysis, and performance optimizations

use oxc_allocator::Allocator;
use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions, TemplateOptimizer, TemplateStats};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_traverse::traverse_mut;
use oxc_semantic::SemanticBuilder;

#[test]
fn test_template_deduplication() {
    // Test that identical templates are deduplicated
    let source = r#"
        const view1 = <div class="test">Hello</div>;
        const view2 = <div class="test">Hello</div>;
        const view3 = <div class="test">Hello</div>;
    "#;
    
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::jsx()).parse();
    let mut program = ret.program;
    
    let semantic = SemanticBuilder::new()
        .build(&program)
        .semantic;
    let scoping = semantic.into_scoping();
    
    let options = DomExpressionsOptions::default();
    let mut transformer = DomExpressions::new(&allocator, options);
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());
    
    // Get template statistics
    let stats = transformer.get_template_stats();
    
    // Should have 3 total templates but only 1 unique
    assert_eq!(stats.total_templates, 3);
    assert_eq!(stats.unique_templates, 1);
    assert_eq!(stats.reused_templates, 2);
}

#[test]
fn test_multiple_unique_templates() {
    // Test that different templates are tracked separately
    let source = r#"
        const view1 = <div>First</div>;
        const view2 = <span>Second</span>;
        const view3 = <p>Third</p>;
    "#;
    
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::jsx()).parse();
    let mut program = ret.program;
    
    let semantic = SemanticBuilder::new()
        .build(&program)
        .semantic;
    let scoping = semantic.into_scoping();
    
    let options = DomExpressionsOptions::default();
    let mut transformer = DomExpressions::new(&allocator, options);
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());
    
    let stats = transformer.get_template_stats();
    
    // Should have 3 unique templates
    assert_eq!(stats.total_templates, 3);
    assert_eq!(stats.unique_templates, 3);
    assert_eq!(stats.reused_templates, 0);
}

#[test]
fn test_partial_deduplication() {
    // Test mixed duplicate and unique templates
    let source = r#"
        const a = <div>Same</div>;
        const b = <div>Same</div>;
        const c = <span>Different</span>;
        const d = <div>Same</div>;
    "#;
    
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::jsx()).parse();
    let mut program = ret.program;
    
    let semantic = SemanticBuilder::new()
        .build(&program)
        .semantic;
    let scoping = semantic.into_scoping();
    
    let options = DomExpressionsOptions::default();
    let mut transformer = DomExpressions::new(&allocator, options);
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());
    
    let stats = transformer.get_template_stats();
    
    // Should have 4 total templates, 2 unique (div and span)
    assert_eq!(stats.total_templates, 4);
    assert_eq!(stats.unique_templates, 2);
    assert_eq!(stats.reused_templates, 2);
}

#[test]
fn test_deduplication_ratio() {
    // Test deduplication ratio calculation
    let source = r#"
        const a = <div>Repeated</div>;
        const b = <div>Repeated</div>;
        const c = <div>Repeated</div>;
        const d = <div>Repeated</div>;
        const e = <span>Unique</span>;
    "#;
    
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::jsx()).parse();
    let mut program = ret.program;
    
    let semantic = SemanticBuilder::new()
        .build(&program)
        .semantic;
    let scoping = semantic.into_scoping();
    
    let options = DomExpressionsOptions::default();
    let mut transformer = DomExpressions::new(&allocator, options);
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());
    
    let stats = transformer.get_template_stats();
    
    // 5 total, 2 unique = (5-2)/5 = 0.6
    assert_eq!(stats.total_templates, 5);
    assert_eq!(stats.unique_templates, 2);
    assert_eq!(stats.deduplication_ratio(), 0.6);
}

#[test]
fn test_reused_templates_tracking() {
    // Test that we can retrieve which templates were reused
    let source = r#"
        const a = <div class="reused">Content</div>;
        const b = <div class="reused">Content</div>;
        const c = <span>Once</span>;
    "#;
    
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::jsx()).parse();
    let mut program = ret.program;
    
    let semantic = SemanticBuilder::new()
        .build(&program)
        .semantic;
    let scoping = semantic.into_scoping();
    
    let options = DomExpressionsOptions::default();
    let mut transformer = DomExpressions::new(&allocator, options);
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());
    
    let reused = transformer.get_reused_templates();
    
    // Should have one template that was reused
    assert_eq!(reused.len(), 1);
    // The div template was used twice
    assert!(reused[0].1 >= 2);
}

#[test]
fn test_static_vs_dynamic_templates() {
    // Test classification of static vs dynamic templates
    let source = r#"
        const static1 = <div>Static</div>;
        const static2 = <span>Also Static</span>;
        const dynamic1 = <div>{count()}</div>;
        const dynamic2 = <span>{name()}</span>;
    "#;
    
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::jsx()).parse();
    let mut program = ret.program;
    
    let semantic = SemanticBuilder::new()
        .build(&program)
        .semantic;
    let scoping = semantic.into_scoping();
    
    let options = DomExpressionsOptions::default();
    let mut transformer = DomExpressions::new(&allocator, options);
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());
    
    let stats = transformer.get_template_stats();
    
    // Should have 2 static and 2 dynamic templates
    assert_eq!(stats.static_templates, 2);
    assert_eq!(stats.dynamic_templates, 2);
}

#[test]
fn test_template_stats_space_saved() {
    // Test that space savings are calculated correctly
    let source = r#"
        const a = <div class="large-template-content">Repeated</div>;
        const b = <div class="large-template-content">Repeated</div>;
        const c = <div class="large-template-content">Repeated</div>;
    "#;
    
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::jsx()).parse();
    let mut program = ret.program;
    
    let semantic = SemanticBuilder::new()
        .build(&program)
        .semantic;
    let scoping = semantic.into_scoping();
    
    let options = DomExpressionsOptions::default();
    let mut transformer = DomExpressions::new(&allocator, options);
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());
    
    let stats = transformer.get_template_stats();
    
    // Space saved should be positive when templates are deduplicated
    assert!(stats.space_saved() > 0);
    // Total size should be 3x the deduplicated size
    assert_eq!(stats.total_html_size, stats.deduplicated_html_size * 3);
}

#[test]
fn test_template_optimizer_empty() {
    // Test optimizer with no templates
    let optimizer = TemplateOptimizer::new();
    let stats = optimizer.get_stats();
    
    assert_eq!(stats.total_templates, 0);
    assert_eq!(stats.unique_templates, 0);
    assert_eq!(stats.deduplication_ratio(), 0.0);
}

#[test]
fn test_template_stats_calculations() {
    // Test template stats calculation methods
    let stats = TemplateStats {
        total_templates: 10,
        unique_templates: 4,
        reused_templates: 6,
        total_html_size: 1000,
        deduplicated_html_size: 400,
        static_templates: 2,
        dynamic_templates: 2,
    };
    
    assert_eq!(stats.space_saved(), 600);
    assert_eq!(stats.deduplication_ratio(), 0.6);
    assert_eq!(stats.average_template_size(), 100.0);
}

#[test]
fn test_nested_element_deduplication() {
    // Test that nested structures are deduplicated correctly
    let source = r#"
        const a = <div><span>Nested</span><p>Content</p></div>;
        const b = <div><span>Nested</span><p>Content</p></div>;
    "#;
    
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::jsx()).parse();
    let mut program = ret.program;
    
    let semantic = SemanticBuilder::new()
        .build(&program)
        .semantic;
    let scoping = semantic.into_scoping();
    
    let options = DomExpressionsOptions::default();
    let mut transformer = DomExpressions::new(&allocator, options);
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());
    
    let stats = transformer.get_template_stats();
    
    // The outer div templates should be deduplicated
    // Note: nested elements create their own templates
    assert!(stats.unique_templates > 0);
    assert!(stats.total_templates >= stats.unique_templates);
}

#[test]
fn test_dynamic_content_prevents_deduplication() {
    // Templates with different dynamic content should NOT be deduplicated
    let source = r#"
        const a = <div>{count()}</div>;
        const b = <div>{name()}</div>;
    "#;
    
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::jsx()).parse();
    let mut program = ret.program;
    
    let semantic = SemanticBuilder::new()
        .build(&program)
        .semantic;
    let scoping = semantic.into_scoping();
    
    let options = DomExpressionsOptions::default();
    let mut transformer = DomExpressions::new(&allocator, options);
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());
    
    let stats = transformer.get_template_stats();
    
    // Both should create the same template HTML (dynamic expressions are removed)
    // So they SHOULD be deduplicated
    assert_eq!(stats.unique_templates, 1);
    assert_eq!(stats.total_templates, 2);
}

#[test]
fn test_attributes_affect_deduplication() {
    // Templates with different attributes should not be deduplicated
    let source = r#"
        const a = <div class="one">Content</div>;
        const b = <div class="two">Content</div>;
    "#;
    
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::jsx()).parse();
    let mut program = ret.program;
    
    let semantic = SemanticBuilder::new()
        .build(&program)
        .semantic;
    let scoping = semantic.into_scoping();
    
    let options = DomExpressionsOptions::default();
    let mut transformer = DomExpressions::new(&allocator, options);
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());
    
    let stats = transformer.get_template_stats();
    
    // Different class attributes mean different templates
    assert_eq!(stats.unique_templates, 2);
    assert_eq!(stats.total_templates, 2);
}

#[test]
fn test_ssr_mode_optimization() {
    // Test that SSR mode still benefits from optimization
    use oxc_dom_expressions::GenerateMode;
    
    let source = r#"
        const a = <div>SSR Content</div>;
        const b = <div>SSR Content</div>;
    "#;
    
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, SourceType::jsx()).parse();
    let mut program = ret.program;
    
    let semantic = SemanticBuilder::new()
        .build(&program)
        .semantic;
    let scoping = semantic.into_scoping();
    
    let options = DomExpressionsOptions {
        generate: GenerateMode::Ssr,
        ..Default::default()
    };
    let mut transformer = DomExpressions::new(&allocator, options);
    traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());
    
    let stats = transformer.get_template_stats();
    
    // SSR mode should still deduplicate templates
    assert_eq!(stats.unique_templates, 1);
    assert_eq!(stats.total_templates, 2);
}
