# oxc-dom-expressions Project Overview

## Purpose
A drop-in replacement for babel-plugin-jsx-dom-expressions for Solid.js, implemented in Rust using the oxc compiler toolchain. It transforms JSX code into optimized DOM manipulation code.

## Tech Stack
- **Language**: Rust (edition 2021)
- **Main Dependencies**:
  - oxc (v0.93): High-performance JavaScript/TypeScript compiler toolchain
    - oxc_allocator, oxc_ast, oxc_parser, oxc_semantic, oxc_traverse, oxc_codegen
  - serde/serde_json: Serialization
- **Dev Dependencies**:
  - criterion: Performance benchmarking
  - similar: Text diffing for tests

## Project Structure
```
src/
├── lib.rs                    # Library entry point
├── options.rs                # Configuration (DomExpressionsOptions, GenerateMode)
├── transform.rs              # Main transformer (DomExpressions, Traverse implementation)
├── template.rs               # Template building (HTML generation, DynamicSlot tracking)
├── codegen.rs                # Code generation utilities
├── utils.rs                  # Utility functions (component detection, event delegation)
├── optimizer.rs              # Template optimization and statistics
├── html_subset_parser.rs     # HTML parsing utilities
├── template_minimalizer.rs   # Template minimization
└── tests.rs                  # Unit tests

tests/
├── dom_fixtures.rs           # DOM mode fixture tests
├── ssr_fixtures.rs           # SSR mode fixture tests
├── hydratable_fixtures.rs    # Hydratable mode fixture tests
├── phase*.rs                 # Phase-specific tests
├── integration.rs            # Integration tests
└── fixtures/                 # Test fixtures from original babel plugin
    ├── dom/                  # DOM mode test cases
    ├── ssr/                  # SSR mode test cases
    └── hydratable/           # Hydratable mode test cases

benches/
└── transformation_bench.rs   # Performance benchmarks

examples/
└── various demo files
```

## Key Concepts

### Transformation Flow
1. JSX → Parser → AST
2. AST → Template builder → Template (HTML string + DynamicSlots)
3. Template → Code generator → Runtime calls (insert, setAttribute, etc.)
4. Output: Optimized JS with template literals and runtime library calls

### Core Data Structures
- **Template**: Contains HTML string and dynamic slot positions
- **DynamicSlot**: Represents positions for dynamic content (text, attributes, etc.)
  - path: Navigation path to element
  - slot_type: Type of content (TextContent, Attribute, etc.)
  - marker_path: Optional marker for positioning
- **DomExpressions**: Main transformer implementing oxc Traverse trait
- **DomExpressionsOptions**: Configuration matching babel plugin API

## Current State (from FIXTURE_TEST_STATUS.md)
- **Passing**: 1/32 fixture tests (test_simple_elements)
- **Main Issue**: Dynamic content generation not yet implemented
- **Working**: Template HTML generation, IIFE structure, comment handling
- **Not Working**: Runtime call generation (_$insert, _$setAttribute, etc.)
