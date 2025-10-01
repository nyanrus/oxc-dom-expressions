# Architecture

This document describes the architecture of oxc-dom-expressions and how it relates to the original babel-plugin-jsx-dom-expressions.

## Overview

oxc-dom-expressions is a Rust implementation of babel-plugin-jsx-dom-expressions using the oxc compiler infrastructure. It provides the same JSX transformation capabilities but with the performance benefits of Rust and oxc's optimized AST handling.

## Component Structure

### 1. Configuration (`src/options.rs`)

The `DomExpressionsOptions` struct provides all configuration options that match the babel plugin:

```rust
pub struct DomExpressionsOptions {
    pub module_name: String,           // Runtime module name
    pub generate: GenerateMode,        // DOM or SSR mode
    pub hydratable: bool,              // Hydration markers
    pub delegate_events: bool,         // Event delegation
    pub wrap_conditionals: bool,       // Smart conditionals
    // ... and more
}
```

**Key features:**
- Builder pattern for easy configuration
- Serialization support for config files
- Default values matching babel plugin

### 2. Main Transformer (`src/transform.rs`)

The `DomExpressions` struct implements the `Traverse` trait from oxc_traverse, providing hooks for AST transformation:

```rust
pub struct DomExpressions<'a> {
    allocator: &'a Allocator,
    options: DomExpressionsOptions,
}

impl<'a> Traverse<'a> for DomExpressions<'a> {
    fn enter_program(&mut self, ...) { }
    fn exit_program(&mut self, ...) { }
    fn enter_jsx_element(&mut self, ...) { }
    fn enter_jsx_fragment(&mut self, ...) { }
    // ... more hooks
}
```

**Transformation hooks:**
- `enter_program`: Initialize transformation state
- `exit_program`: Add imports and final code generation
- `enter_jsx_element`: Transform JSX elements to DOM calls
- `enter_jsx_fragment`: Transform fragments to arrays
- `enter_jsx_attribute`: Handle special attributes
- `enter_jsx_expression_container`: Wrap dynamic expressions

### 3. Utility Functions (`src/utils.rs`)

Helper functions for element and attribute detection:

```rust
pub fn is_html_element(tag_name: &str) -> bool
pub fn is_component(tag_name: &str) -> bool
pub fn is_event_handler(attr_name: &str) -> bool
pub fn should_delegate_event(event_name: &str) -> bool
pub fn is_void_element(tag_name: &str) -> bool
```

## Transformation Pipeline

The transformation follows this flow:

```
JSX Source Code
    ↓
oxc_parser::Parser (parse JSX)
    ↓
oxc_semantic::SemanticBuilder (build semantic info)
    ↓
DomExpressions::Traverse (transform AST)
    ↓
oxc_codegen::Codegen (generate output)
    ↓
Transformed JavaScript
```

## Transformation Examples

### Example 1: Simple Element

**Input:**
```jsx
<div class="container">Hello</div>
```

**Output:**
```javascript
import { template } from "solid-js/web";

const _tmpl$ = template(`<div class="container">Hello</div>`);

_tmpl$.cloneNode(true);
```

### Example 2: Dynamic Content

**Input:**
```jsx
<div>{count()}</div>
```

**Output:**
```javascript
import { template, insert } from "solid-js/web";

const _tmpl$ = template(`<div></div>`);

(() => {
  const _el$ = _tmpl$.cloneNode(true);
  insert(_el$, count);
  return _el$;
})();
```

### Example 3: Event Handler

**Input:**
```jsx
<button onClick={handleClick}>Click</button>
```

**Output:**
```javascript
import { template, delegateEvents } from "solid-js/web";

const _tmpl$ = template(`<button>Click</button>`);

(() => {
  const _el$ = _tmpl$.cloneNode(true);
  _el$.$$click = handleClick;
  return _el$;
})();

delegateEvents(["click"]);
```

## Comparison with Babel Plugin

| Feature | babel-plugin-jsx-dom-expressions | oxc-dom-expressions |
|---------|----------------------------------|---------------------|
| Language | JavaScript | Rust |
| Parser | Babel | oxc |
| Performance | Good | Excellent |
| Configuration | ✅ Full support | ✅ Full support |
| JSX Elements | ✅ | ⚠️ Hooks ready |
| Event Delegation | ✅ | ⚠️ Hooks ready |
| Special Attributes | ✅ | ⚠️ Hooks ready |
| Components | ✅ | ⚠️ Hooks ready |
| Fragments | ✅ | ⚠️ Hooks ready |
| Template Generation | ✅ | 🚧 Planned |
| Import Injection | ✅ | 🚧 Planned |
| SSR Mode | ✅ | ⚠️ Config ready |

Legend:
- ✅ Fully implemented
- ⚠️ API ready, implementation pending
- 🚧 Planned

## Implementation Phases

### Phase 1: Foundation ✅
- [x] Project structure
- [x] Configuration options
- [x] Utility functions
- [x] Traverse hooks
- [x] Basic tests

### Phase 2: Core Transformation (Planned)
- [ ] Template string generation
- [ ] Element cloning code
- [ ] Property/attribute setters
- [ ] Dynamic expression wrapping

### Phase 3: Advanced Features (Planned)
- [ ] Event delegation
- [ ] Special bindings (ref, classList, style)
- [ ] Component handling
- [ ] Fragment support
- [ ] Import injection

### Phase 4: Optimization (Planned)
- [ ] Template deduplication
- [ ] Static analysis
- [ ] Performance benchmarks
- [ ] SSR mode implementation

## Integration with oxc

oxc-dom-expressions integrates with the oxc ecosystem:

```rust
use oxc_allocator::Allocator;
use oxc_parser::{Parser, ParserReturn};
use oxc_semantic::SemanticBuilder;
use oxc_traverse::traverse_mut;
use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions};

// Parse JSX
let allocator = Allocator::default();
let source_text = "const view = <div>Hello</div>";
let ret = Parser::new(&allocator, source_text, SourceType::jsx()).parse();
let mut program = ret.program;

// Build semantic model
let semantic = SemanticBuilder::new(&source_text)
    .build(&program)
    .semantic;

// Apply transformation
let options = DomExpressionsOptions::default();
let mut transformer = DomExpressions::new(&allocator, options);
traverse_mut(&mut transformer, &allocator, &mut program, semantic);

// Generate code
// (use oxc_codegen::Codegen)
```

## Development Guidelines

### Adding New Features

1. Add configuration option to `DomExpressionsOptions` if needed
2. Implement the transformation logic in `DomExpressions::Traverse`
3. Add utility functions to `utils.rs` if needed
4. Write tests in `tests/integration.rs`
5. Update documentation

### Testing Strategy

- Unit tests for utility functions
- Integration tests for configuration
- Future: AST transformation tests
- Future: End-to-end tests comparing output with babel plugin

## Performance Considerations

oxc-dom-expressions benefits from:

1. **Rust's performance**: Native compiled code vs interpreted JavaScript
2. **oxc's optimized AST**: Efficient memory layout and traversal
3. **Single-pass transformation**: All changes in one AST traversal
4. **Zero-copy operations**: Where possible, reuse existing AST nodes

## Future Enhancements

Potential improvements for future versions:

1. **Parallel processing**: Process multiple files concurrently
2. **Incremental compilation**: Only retransform changed files
3. **Custom optimizations**: Rust-specific performance optimizations
4. **WASM support**: Compile to WebAssembly for browser use
5. **Language Server Protocol**: IDE integration

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## License

MIT - See [LICENSE](LICENSE)
