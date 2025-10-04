# Project Overview

## Purpose
oxc-dom-expressions is a drop-in replacement of babel-plugin-jsx-dom-expressions for Solid.js, implemented with oxc in Rust. It's a JSX compiler that transforms JSX to DOM expressions for reactive libraries with fine-grained change detection.

## Tech Stack
- **Language**: Rust (edition 2021)
- **Key Dependencies**:
  - oxc (v0.93) - High-performance Rust-based parsing and transformation toolchain
  - serde/serde_json - Serialization/deserialization
- **Dev Dependencies**:
  - criterion - Benchmarking
  - similar - Text diffing for test comparisons

## Codebase Structure
```
├── src/
│   ├── lib.rs - Main library entry point
│   ├── transform.rs - Core JSX transformation logic
│   ├── template.rs - Template generation
│   ├── codegen.rs - Code generation
│   ├── options.rs - Configuration options
│   ├── utils.rs - Utility functions
│   ├── optimizer.rs - Optimization logic
│   ├── html_subset_parser.rs - HTML parsing
│   ├── template_minimalizer.rs - Template minimization
│   └── tests.rs - Unit tests
├── tests/
│   ├── dom_fixtures.rs - DOM mode fixture tests
│   ├── ssr_fixtures.rs - SSR mode fixture tests
│   ├── hydratable_fixtures.rs - Hydratable mode fixture tests
│   └── fixtures/ - Test fixture files from original babel plugin
├── examples/ - Example code demonstrating features
└── benches/ - Performance benchmarks
```

## Key Features
- JSX to DOM transformation
- Event delegation for performance
- Template generation with cloneNode optimization
- SSR, DOM, and Hydratable modes
- Template deduplication
- Special bindings (ref, classList, style, etc.)
