// Modern output format using $template, $clone, and $bind
// This is transformer-friendly and runtime-friendly with a declarative API
// The helpers are automatically injected and wrap the original runtime API

// Injected helper code (automatically added by the transformer)
import { template as _template, insert as _insert, effect as _effect, setAttribute as _setAttribute, className as _className, classList as _classList, style as _style, spread as _spread, mergeProps as _mergeProps, use as _use, addEventListener as _addEventListener } from "solid-js/web";

function $template(html) {
  return _template(html);
}

function $clone(tmpl) {
  return tmpl();
}

function $bind(element, path, bindings) {
  let target = element;
  for (const index of path) {
    if (target.childNodes && target.childNodes[index]) {
      target = target.childNodes[index];
    }
  }
  
  if (!bindings) return;
  
  // Handle ref, spread, classList, style, events, directives, and attributes
  // ... (full implementation wraps the original runtime functions)
}

// Hoisted at module scope - parsed once
const _tmpl$ = $template(`<div id="main"><h1 class="base"><a href="/">Welcome</a></h1></div>`);

const selected = true;
let id = "my-h1";
let link;

const template = (() => {
  // Clone the template for this instance
  const _root$ = $clone(_tmpl$);
  
  // Bind to div (path: [0])
  $bind(_root$, [0], {
    spread: [() => results],
    classList: { selected: () => unknown },
    style: { color: () => color }
  });
  
  // Bind to h1 (path: [0, 0])
  $bind(_root$, [0, 0], {
    id: () => id,
    spread: [() => results()],
    title: () => welcoming(),
    style: { "background-color": () => color(), "margin-right": "40px" },
    classList: { dynamic: () => dynamic(), selected: () => selected }
  });
  
  // Bind to a (path: [0, 0, 0])
  $bind(_root$, [0, 0, 0], {
    ref: (el) => link = el,
    classList: { "ccc ddd": true }
  });
  
  return _root$;
})();

// Advantages of this format:
// 1. More readable - bindings are declarative
// 2. Easier to transform - less imperative code generation
// 3. Runtime-friendly - binding logic is centralized
// 4. Modern ESNext syntax
// 5. Path-based element access is simple and predictable
// 6. Easy to optimize at runtime with caching
// 7. No separate polyfill package needed - helpers are injected inline
