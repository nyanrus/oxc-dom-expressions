// Modern output format - clean and direct
// Uses the runtime API directly without complex helpers

// Just import what we need from the runtime
import { template as _$template } from "solid-js/web";

// Hoisted at module scope - parsed once
const _tmpl$ = _$template(`<div id="main"><h1 class="base"><a href="/">Welcome</a></h1></div>`);

const template = (() => {
  // Clone the template by calling it
  const _el$ = _tmpl$();
  
  // For dynamic content, use runtime functions directly:
  // _el$.firstChild.id = id;
  // _effect(() => _setAttribute(_el$.firstChild, "title", welcoming()));
  // etc.
  
  return _el$;
})();

// Advantages of this format:
// 1. Simple and direct - no complex helpers
// 2. Transformer-friendly - easy to generate  
// 3. Runtime-friendly - fast execution, no wrapper overhead
// 4. Clean output - minimal code
// 5. Uses runtime API directly - no abstraction layer
