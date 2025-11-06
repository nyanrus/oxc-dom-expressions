//! Runtime helper that wraps the original dom-expressions API
//!
//! This module provides the JavaScript helper code that implements the modern
//! $template, $clone, and $bind API using the original solid-js/web API functions.

/// Get the runtime helper code that provides $template, $clone, $bind
/// 
/// This returns JavaScript source code as a string that can be parsed and injected
/// into the transformed output.
pub fn get_runtime_helper(module_name: &str) -> String {
    format!(
        r#"// Helper: $template, $clone, $bind wrapping original runtime API
import {{ template as _template, insert as _insert, effect as _effect, setAttribute as _setAttribute, className as _className, classList as _classList, style as _style, spread as _spread, mergeProps as _mergeProps, use as _use, addEventListener as _addEventListener }} from "{}";

function $template(html) {{
  return _template(html);
}}

function $clone(tmpl) {{
  return tmpl();
}}

function $bind(element, path, bindings) {{
  let target = element;
  for (const index of path) {{
    if (target.childNodes && target.childNodes[index]) {{
      target = target.childNodes[index];
    }}
  }}

  if (!bindings) return;

  if (bindings.ref) {{
    const ref = bindings.ref;
    if (typeof ref === 'function') {{
      ref(target);
    }}
  }}

  if (bindings.spread) {{
    for (const spreadExpr of bindings.spread) {{
      const props = typeof spreadExpr === 'function' ? spreadExpr() : spreadExpr;
      _spread(target, props, false, false);
    }}
  }}

  if (bindings.classList) {{
    const classes = bindings.classList;
    const dynamicClasses = {{}};
    
    for (const [key, value] of Object.entries(classes)) {{
      if (typeof value === 'function') {{
        dynamicClasses[key] = value;
      }} else if (value) {{
        target.classList.add(...key.split(' ').filter(Boolean));
      }}
    }}
    
    if (Object.keys(dynamicClasses).length > 0) {{
      _effect(() => _classList(target, dynamicClasses));
    }}
  }}

  if (bindings.style) {{
    const styles = bindings.style;
    const dynamicStyles = {{}};
    const staticStyles = {{}};
    
    for (const [key, value] of Object.entries(styles)) {{
      if (typeof value === 'function') {{
        dynamicStyles[key] = value;
      }} else {{
        staticStyles[key] = value;
      }}
    }}
    
    if (Object.keys(staticStyles).length > 0) {{
      _style(target, staticStyles);
    }}
    
    if (Object.keys(dynamicStyles).length > 0) {{
      _effect(_p => _style(target, Object.fromEntries(
        Object.entries(dynamicStyles).map(([k, v]) => [k, v()])
      ), _p));
    }}
  }}

  for (const [key, value] of Object.entries(bindings)) {{
    if (key.startsWith('on:')) {{
      const eventName = key.slice(3);
      _addEventListener(target, eventName, value, true);
    }} else if (key.startsWith('use:')) {{
      const directive = value;
      if (typeof directive === 'function') {{
        _use(directive, target);
      }}
    }}
  }}

  const skipKeys = new Set(['ref', 'spread', 'classList', 'style']);
  for (const [key, value] of Object.entries(bindings)) {{
    if (skipKeys.has(key) || key.startsWith('on:') || key.startsWith('use:') || 
        key.startsWith('attr:') || key.startsWith('prop:') || key.startsWith('bool:')) {{
      continue;
    }}
    
    if (typeof value === 'function') {{
      _effect(() => _setAttribute(target, key, value()));
    }} else {{
      _setAttribute(target, key, value);
    }}
  }}
}}
"#,
        module_name
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper_generation() {
        let helper = get_runtime_helper("solid-js/web");
        assert!(helper.contains("$template"));
        assert!(helper.contains("$clone"));
        assert!(helper.contains("$bind"));
        assert!(helper.contains("solid-js/web"));
    }

    #[test]
    fn test_helper_with_custom_module() {
        let helper = get_runtime_helper("my-custom-runtime");
        assert!(helper.contains("my-custom-runtime"));
    }
}
