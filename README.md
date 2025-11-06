# oxc-dom-expressions

A drop-in replacement of [babel-plugin-jsx-dom-expressions](https://github.com/ryansolid/dom-expressions) for [Solid.js](https://www.solidjs.com/) implemented with [oxc](https://github.com/oxc-project/oxc) in Rust.

## Overview

This plugin is a JSX compiler built for DOM Expressions to provide a general JSX to DOM transformation for reactive libraries that do fine-grained change detection. The goal is to convert JSX statements to native DOM statements and wrap JSX expressions with functions that can be implemented with the library of your choice.

This is a Rust implementation using oxc, providing:
- ⚡ **Fast compilation** - Leverages oxc's high-performance Rust-based parsing and transformation
- 🔄 **Drop-in replacement** - Compatible with babel-plugin-jsx-dom-expressions configuration
- 🎯 **Solid.js optimized** - Designed specifically for Solid.js reactivity patterns
- 📦 **Zero JavaScript overhead** - Pure Rust implementation

## Features

This plugin treats all lowercase tags as HTML elements and mixed-cased tags as Custom Functions. This enables breaking up your view into components.

Key features include:
- ✅ Support for Web Component Custom Elements
- ✅ Common camelCase event handlers (like React)
- ✅ DOM-safe attributes like `class` and `for`
- ✅ Simple `ref` property
- ✅ Parsing of objects for `style` and `classList` properties
- ✅ Event delegation for performance
- ✅ Heuristic-based reactive wrapping
- ✅ Template generation with `cloneNode` optimization
- ✅ Fragment support with `<></>` notation

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
oxc-dom-expressions = "0.1"
```

### Optional Features

The `opt` feature (enabled by default) provides optimization capabilities:
- Template deduplication and statistics
- HTML minimization
- Static expression evaluation

To disable optimizations and reduce binary size:

```toml
[dependencies]
oxc-dom-expressions = { version = "0.1", default-features = false }
```

## Usage

### Modern Transform (Recommended for new projects)

```rust
use oxc_allocator::Allocator;
use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions};

let allocator = Allocator::default();
let options = DomExpressionsOptions::new("solid-js/web");
let transformer = DomExpressions::new(&allocator, options);
```

Produces clean, declarative output with automatically injected helper functions:
```javascript
// Injected helpers that wrap the original runtime API
import { template as _template, insert as _insert, effect as _effect, /* ... */ } from "solid-js/web";

function $template(html) { return _template(html); }
function $clone(tmpl) { return tmpl(); }
function $bind(element, path, bindings) { /* ... wraps original API ... */ }

// Your transformed code
const _tmpl$ = $template(`<div id="main"><h1>...</h1></div>`);
const element = (() => {
  const _root$ = $clone(_tmpl$);
  $bind(_root$, [0], { id: () => dynamicId });
  return _root$;
})();
```

**Note:** The transformer automatically injects helper functions that wrap the original dom-expressions API (like `template`, `insert`, `effect` from solid-js/web). This means you don't need a separate polyfill package - everything is self-contained in the transformed output.

### Babel-Compatible Transform

```rust
use oxc_allocator::Allocator;
use oxc_dom_expressions::{DomExpressionsCompat2, DomExpressionsOptions};

let allocator = Allocator::default();
let options = DomExpressionsOptions::new("solid-js/web");
let transformer = DomExpressionsCompat2::new(&allocator, options);
```

Produces babel-plugin-jsx-dom-expressions compatible output.

### Configuration Options

#### `module_name` (required)
- Type: `String`
- The name of the runtime module to import methods from
- Example: `"solid-js/web"`

#### `generate`
- Type: `GenerateMode` (`Dom` | `Ssr`)
- Default: `Dom`
- The output mode of the compiler

#### `hydratable`
- Type: `bool`
- Default: `false`
- Whether the output should contain hydratable markers

#### `delegate_events`
- Type: `bool`
- Default: `true`
- Whether to enable automatic event delegation on camelCase

#### `wrap_conditionals`
- Type: `bool`
- Default: `true`
- Whether smart conditional detection should be used

#### `context_to_custom_elements`
- Type: `bool`
- Default: `false`
- Whether to set current render context on Custom Elements and slots

#### `built_ins`
- Type: `Vec<String>`
- Default: `[]`
- Array of Component exports from module that aren't included by default

#### `effect_wrapper`
- Type: `String`
- Default: `"effect"`
- The reactive wrapper function name

#### `static_marker`
- Type: `String`
- Default: `"@once"`
- Comment decorator string that indicates a static expression

#### `memo_wrapper`
- Type: `String`
- Default: `"memo"`
- The memo function name

#### `validate`
- Type: `bool`
- Default: `true`
- Whether to validate HTML nesting

#### `omit_nested_closing_tags`
- Type: `bool`
- Default: `false`
- Whether to remove unnecessary closing tags from template output

#### `omit_last_closing_tag`
- Type: `bool`
- Default: `true`
- Whether to remove tags if they are the last element

#### `omit_quotes`
- Type: `bool`
- Default: `true`
- Whether to remove quotes for HTML attributes when possible

#### `require_import_source`
- Type: `Option<String>`
- Default: `None`
- When set, restricts JSX transformation to files with specific import source pragma

## Example

### Input JSX:

```jsx
const view = ({ item }) => {
  const itemId = item.id;
  return <tr class={itemId === selected() ? "danger" : ""}>
    <td class="col-md-1">{itemId}</td>
    <td class="col-md-4">
      <a onclick={e => select(item, e)}>{item.label}</a>
    </td>
    <td class="col-md-1">
      <a onclick={e => del(item, e)}>
        <span class="glyphicon glyphicon-remove" aria-hidden="true"></span>
      </a>
    </td>
    <td class="col-md-6"></td>
  </tr>;
};
```

### Output (conceptual):

```jsx
import { template, delegateEvents, className, effect, insert } from "solid-js/web";

const _tmpl$ = template(`<tr><td class="col-md-1"></td><td class="col-md-4"><a></a></td><td class="col-md-1"><a><span class="glyphicon glyphicon-remove" aria-hidden="true"></span></a></td><td class="col-md-6"></td></tr>`);

const view = ({ item }) => {
  const itemId = item.id;
  return (() => {
    const _el$ = _tmpl$.cloneNode(true),
      _el$2 = _el$.firstChild,
      _el$3 = _el$2.nextSibling,
      _el$4 = _el$3.firstChild,
      _el$5 = _el$3.nextSibling,
      _el$6 = _el$5.firstChild;
    insert(_el$2, itemId);
    _el$4.$$click = e => select(item, e);
    insert(_el$4, () => item.label);
    _el$6.$$click = e => del(item, e);
    effect(() => className(_el$, itemId === selected() ? "danger" : ""));
    return _el$;
  })();
};

delegateEvents(["click"]);
```

## Special Bindings

### `ref`
Assigns the variable or calls a function with the DOM element:

```jsx
const Parent = () => {
  let ref;
  return <div ref={ref} />;
};
```

### `on(eventName)`
Event handlers expecting a function. The compiler delegates events where possible:

```jsx
<div onClick={handler} />
```

Bound events with arrays:
```jsx
<li onClick={[handler, item.id]} />
```

### `on:` / `oncapture:`
Bypass event delegation and use Level 3 addEventListener:

```jsx
<div on:CustomEvent={e => alert(e.detail)} />
<div oncapture:Click={e => console.log(e)} />
```

### `classList`
Object-based class assignment:

```jsx
<div classList={{ selected: isSelected(), editing: isEditing() }} />
```

### Spreads
Pass multiple props at once:

```jsx
<div {...props} />
```

## Components

Components are Capital Cased tags with getter accessors for dynamic props:

```jsx
const MyComp = props => {
  return <div>{props.param}</div>;
};

<MyComp param={dynamic()} />;
```

## Fragments

JSX Fragments with `<></>` notation are compiled to arrays:

```jsx
<>
  <div>First</div>
  <div>Second</div>
</>
```

## Development Status

This implementation provides core transformation infrastructure for babel-plugin-jsx-dom-expressions compatibility:

- ✅ Configuration options matching the babel plugin
- ✅ JSX element and fragment traversal hooks
- ✅ Event handler detection and delegation logic
- ✅ Utility functions for element type detection
- ✅ Template string generation from JSX
- ✅ Code generation for DOM manipulation
- ✅ State management for templates and imports
- ✅ Event delegation support
- ✅ Special bindings (ref, classList, style)
- ✅ on: and oncapture: event prefixes
- ✅ Component detection and handling
- ✅ Fragment support
- ✅ Template deduplication optimization
- ✅ Static analysis and performance metrics
- ✅ Benchmark suite
- ✅ SSR mode optimization
- ✅ Compatibility layer for babel plugin output format
- ⚠️ Full AST replacement (in progress)
- ⚠️ Import injection (planned)
- ⚠️ Complete code generation (planned)

## Architecture

The codebase is organized into focused modules for maintainability and clarity:

### Core Transformation
- **transform**: Modern JSX transformation using declarative $bind API
  - Produces clean, intuitive output with `$template`, `$clone`, and `$bind`
  - Path-based element access for predictability
  - Concise code generation with helper functions
- **compat2**: Babel-compatible transformation (legacy format)
  - Maintains full compatibility with babel-plugin-jsx-dom-expressions
  - Used for existing Solid.js projects
- **template**: Template string generation and dynamic slot tracking
  - Converts JSX to HTML templates with dynamic markers
- **utils**: Shared utilities for element/event detection

### Optimization (src/opt)
All optimization and minification code is separated into the `opt` module:
- **optimizer**: Template deduplication and static analysis
- **minimizer**: HTML template minimization and whitespace handling  
- **evaluator**: Static expression evaluation for compile-time optimization

This separation keeps the core transformation logic focused on correctness and maintainability,
while optimization features can be independently developed and tested.

### Compatibility Layer
- **compat**: Ensures compatibility with babel-plugin-jsx-dom-expressions
  - Output normalization for exact format matching
  - Import ordering to match babel plugin expectations
  - Variable naming conventions (template/element variable names)
  - Babel-specific transformation behaviors

## Choosing a Transformer

- Use **`DomExpressions`** (from `transform`) for new projects wanting modern, readable output
- Use **`DomExpressionsCompat2`** (from `compat2`) for babel-plugin-jsx-dom-expressions compatibility

## Performance

Phase 4 introduces comprehensive optimization features:

### Template Deduplication
Identical templates are automatically deduplicated, reducing memory usage and bundle size:
```rust
let stats = transformer.get_template_stats();
println!("Space saved: {} bytes", stats.space_saved());
println!("Deduplication ratio: {:.1}%", stats.deduplication_ratio() * 100.0);
```

### Static Analysis
Track static vs dynamic templates for optimization opportunities:
```rust
let stats = transformer.get_template_stats();
println!("Static templates: {}", stats.static_templates);
println!("Dynamic templates: {}", stats.dynamic_templates);
```

### Benchmarks
Run performance benchmarks to measure transformation speed:
```bash
cargo bench
```

See `examples/phase4_demo.rs` for a complete demonstration of optimization features.

## Contributing

Contributions are welcome! This project aims to provide a high-performance, drop-in replacement for the babel plugin.

## License

MIT

## Acknowledgements

- [Solid.js](https://www.solidjs.com/) - The reactive JavaScript library this plugin is designed for
- [babel-plugin-jsx-dom-expressions](https://github.com/ryansolid/dom-expressions) - The original babel plugin this is based on
- [oxc](https://github.com/oxc-project/oxc) - The fast JavaScript/TypeScript compiler toolchain in Rust
- [Surplus](https://github.com/adamhaile/surplus) - Inspiration for JSX to DOM compilation

## Related Projects

- [Solid](https://github.com/ryansolid/solid) - A declarative JavaScript library for building user interfaces
- [ko-jsx](https://github.com/ryansolid/ko-jsx) - Knockout JS with JSX rendering
- [mobx-jsx](https://github.com/ryansolid/mobx-jsx) - MobX with JSX rendering
