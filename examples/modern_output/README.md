# Modern Output Format Example

This directory contains an example of the modern, declarative output format that uses `$template`, `$clone`, and `$bind` from `solid-runtime/polyfill`.

## Files

- `input.jsx` - Original JSX code
- `output.js` - Modern transformed output using $bind API

## Format Comparison

### Babel-Compatible Format (compat2)

The `compat2` module generates babel-plugin-jsx-dom-expressions compatible output:

```javascript
import { template as _$template, insert as _$insert, effect as _$effect, /* ... */ } from "solid-js/web";

const _tmpl$ = _$template(`<div id="main"><h1 class="base"><a href="/">Welcome</a></h1></div>`);

const template = (() => {
  const _el$ = _tmpl$.cloneNode(true);
  const _el$2 = _el$.firstChild;
  const _el$3 = _el$2.firstChild;
  
  // Imperative DOM manipulation
  _$spread(_el$, _$mergeProps(results, {
    classList: { selected: unknown },
    style: { color }
  }));
  
  _$effect(() => _$setAttribute(_el$2, "id", id));
  _$spread(_el$2, results());
  // ... more imperative calls
  
  return _el$;
})();
```

### Modern Format (transform)

The new `transform` module generates clean, direct output:

```javascript
import { template as _$template } from "solid-js/web";

const _tmpl$ = _$template(`<div id="main"><h1 class="base"><a href="/">Welcome</a></h1></div>`);

const template = (() => {
  const _el$ = _tmpl$();
  
  // Dynamic content uses runtime functions directly:
  // _el$.firstChild.id = id;
  // _effect(() => _setAttribute(_el$.firstChild, "title", welcoming()));
  
  return _el$;
})();
```

## Advantages of Modern Format

1. **Simple and direct** - No complex helper functions
2. **Transformer-friendly** - Easy to generate clean code
3. **Runtime-friendly** - Fast execution, minimal overhead
4. **Clean output** - Just imports and direct calls
5. **Universal approach** - One consistent pattern for all cases

## Path System

Elements are accessed by their child index path from the root:

- `[0]` - First child of root (the `div`)
- `[0, 0]` - First child of first child (the `h1`)
- `[0, 0, 0]` - First child's first child's first child (the `a`)

This is more predictable than traversing with `.firstChild`, `.nextSibling`, etc.

## Binding Object

The third parameter to `$bind` is an object that can contain:

- `ref`: Element reference callback or assignment
- `spread`: Array of spread expressions
- `classList`: Object mapping class names to conditions
- `style`: Object mapping style properties to values
- `on:eventName`: Event handlers
- `attr:name`: Static attributes
- `prop:name`: Property assignments
- `bool:name`: Boolean attributes
- `use:directive`: Custom directives
- Any other key: Regular attributes (reactive if value is a function)

## Implementation Status

The modern `transform` module uses a clean, universal approach:
- Imports runtime functions directly from the original library (e.g., `solid-js/web`)
- No complex helper functions - keeps output simple and fast
- Transformer-friendly code generation
- Runtime-friendly execution with minimal overhead

This is production-ready for simple static templates. Dynamic content support is planned.

## Usage

```rust
use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions, DomExpressionsCompat2};

// For modern output with clean, direct runtime calls
let modern_transformer = DomExpressions::new(&allocator, options);

// For babel-compatible output
let compat_transformer = DomExpressionsCompat2::new(&allocator, options);
```
