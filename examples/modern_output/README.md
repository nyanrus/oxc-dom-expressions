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

The new `transform` module generates modern, declarative output:

```javascript
import { $template, $clone, $bind } from "solid-runtime/polyfill";

const _tmpl$ = $template(`<div id="main"><h1 class="base"><a href="/">Welcome</a></h1></div>`);

const template = (() => {
  const _root$ = $clone(_tmpl$);
  
  // Declarative binding API
  $bind(_root$, [0], {
    spread: [() => results],
    classList: { selected: () => unknown },
    style: { color: () => color }
  });
  
  $bind(_root$, [0, 0], {
    id: () => id,
    spread: [() => results()],
    title: () => welcoming(),
    // ...
  });
  
  return _root$;
})();
```

## Advantages of Modern Format

1. **More Readable**: Bindings are declarative and grouped by element
2. **Transformer-Friendly**: Less complex code generation required
3. **Runtime-Friendly**: Centralized binding logic, easier to optimize
4. **Modern Syntax**: Uses ESNext features
5. **Predictable**: Path-based element access is simple and consistent
6. **Performance**: Runtime can optimize binding application with caching

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

The modern `transform` module is currently a stub/placeholder with documentation.
For a working implementation, use the `compat2` module which provides babel-compatible output.

## Usage

```rust
use oxc_dom_expressions::{DomExpressions, DomExpressionsOptions, DomExpressionsCompat2};

// For modern output (stub - not yet implemented)
let modern_transformer = DomExpressions::new(&allocator, options);

// For babel-compatible output (fully implemented)
let compat_transformer = DomExpressionsCompat2::new(&allocator, options);
```
