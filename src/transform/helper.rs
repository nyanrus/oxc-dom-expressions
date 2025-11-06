//! Runtime helper that wraps the original dom-expressions API
//!
//! For the modern transform, we take a minimal approach:
//! - Use the original runtime API directly (template, insert, effect, etc.)
//! - Generate clean, direct code without complex helpers
//! - Only add small wrappers when they genuinely simplify the output

/// Get minimal helper code for the modern transform
/// This is just imports - we use the runtime functions directly
pub fn get_runtime_imports(module_name: &str) -> String {
    format!(
        r#"import {{ template as _$template }} from "{}";"#,
        module_name
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_imports_generation() {
        let imports = get_runtime_imports("solid-js/web");
        assert!(imports.contains("solid-js/web"));
        assert!(imports.contains("_$template"));
    }
}
