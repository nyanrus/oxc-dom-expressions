//! Runtime helper that wraps the original dom-expressions API
//!
//! For the modern transform, we take a minimal approach:
//! - Use the original runtime API directly (template, insert, effect, etc.)
//! - Generate clean, direct code without complex helpers
//! - Only import what we actually use

/// Get import statement for needed runtime functions
pub fn get_runtime_imports(module_name: &str, imports: &[&str]) -> String {
    if imports.is_empty() {
        format!(r#"import {{ template as _$template }} from "{}";"#, module_name)
    } else {
        let mut all_imports = vec!["template as _$template"];
        for imp in imports {
            all_imports.push(match *imp {
                "insert" => "insert as _$insert",
                "effect" => "effect as _$effect",
                "setAttribute" => "setAttribute as _$setAttribute",
                "addEventListener" => "addEventListener as _$addEventListener",
                "delegateEvents" => "delegateEvents as _$delegateEvents",
                "classList" => "classList as _$classList",
                "style" => "style as _$style",
                "spread" => "spread as _$spread",
                "use" => "use as _$use",
                _ => imp,
            });
        }
        format!(
            r#"import {{ {} }} from "{}";"#,
            all_imports.join(", "),
            module_name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_imports_generation() {
        let imports = get_runtime_imports("solid-js/web", &[]);
        assert!(imports.contains("solid-js/web"));
        assert!(imports.contains("_$template"));
    }
    
    #[test]
    fn test_imports_with_functions() {
        let imports = get_runtime_imports("solid-js/web", &["insert", "effect"]);
        assert!(imports.contains("_$insert"));
        assert!(imports.contains("_$effect"));
    }
}
