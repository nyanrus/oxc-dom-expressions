//! Code generation utilities for DOM manipulation

use std::fmt::Write;

use crate::options::DomExpressionsOptions;
use crate::template::{SlotType, Template};

/// Generate the cloning code for a template
pub fn generate_clone_code(
    template_var: &str,
    element_var: &str,
) -> String {
    format!("const {} = {}$.cloneNode(true)", element_var, template_var)
}

/// Generate element reference code based on path
pub fn generate_element_ref(
    base_var: &str,
    path: &[String],
) -> String {
    if path.is_empty() {
        return base_var.to_string();
    }
    
    let mut result = base_var.to_string();
    for segment in path {
        result.push('.');
        result.push_str(segment);
    }
    result
}

/// Generate code for dynamic content insertion
pub fn generate_insert_code(
    _options: &DomExpressionsOptions,
    element_ref: &str,
    expression: &str,
) -> String {
    format!("insert({}, {})", element_ref, expression)
}

/// Generate code for setting an attribute/property
pub fn generate_set_attribute_code(
    element_ref: &str,
    attr_name: &str,
    value: &str,
    is_dynamic: bool,
    options: &DomExpressionsOptions,
) -> String {
    if is_dynamic {
        // Dynamic attribute - wrap with effect
        if attr_name == "class" || attr_name == "className" {
            format!(
                "{}(() => className({}, {}))",
                options.effect_wrapper,
                element_ref,
                value
            )
        } else {
            format!(
                "{}(() => setAttribute({}, \"{}\", {}))",
                options.effect_wrapper,
                element_ref,
                attr_name,
                value
            )
        }
    } else {
        // Static attribute
        format!("{}.setAttribute(\"{}\", {})", element_ref, attr_name, value)
    }
}

/// Generate code for event handler
pub fn generate_event_handler_code(
    element_ref: &str,
    event_name: &str,
    handler: &str,
    delegate: bool,
) -> String {
    if delegate {
        // Use delegation
        let delegated_name = event_name.to_lowercase();
        format!("{}.$$click = {}", element_ref, handler)
            .replace("click", &delegated_name)
    } else {
        // Direct addEventListener
        format!(
            "{}.addEventListener(\"{}\", {})",
            element_ref,
            event_name.to_lowercase(),
            handler
        )
    }
}

/// Generate the complete transformation for a template
pub fn generate_template_transformation(
    template: &Template,
    template_var: &str,
    options: &DomExpressionsOptions,
) -> String {
    let mut code = String::new();
    
    // Create IIFE
    code.push_str("(() => {\n");
    
    // Clone the template
    let _ = writeln!(code, "  const _el$ = {}$.cloneNode(true);", template_var);
    
    // Generate element references
    let mut element_refs = vec![("_el$".to_string(), Vec::<String>::new())];
    for (i, slot) in template.dynamic_slots.iter().enumerate() {
        if !slot.path.is_empty() {
            let ref_var = format!("_el${}", i + 2);
            let ref_expr = generate_element_ref("_el$", &slot.path);
            let _ = writeln!(code, "  const {} = {};", ref_var, ref_expr);
            element_refs.push((ref_var, slot.path.clone()));
        }
    }
    
    // Generate dynamic operations
    for (i, slot) in template.dynamic_slots.iter().enumerate() {
        let element_ref = if slot.path.is_empty() {
            "_el$"
        } else {
            &format!("_el${}", i + 2)
        };
        
        match &slot.slot_type {
            SlotType::TextContent => {
                let _ = writeln!(
                    code,
                    "  insert({}, {{/* expression */}});",
                    element_ref
                );
            }
            SlotType::Attribute(name) => {
                let _ = writeln!(
                    code,
                    "  {}(() => setAttribute({}, \"{}\", {{/* value */}}));",
                    options.effect_wrapper,
                    element_ref,
                    name
                );
            }
            SlotType::EventHandler(_name) => {
                let _ = writeln!(
                    code,
                    "  {}.$$click = {{/* handler */}};",
                    element_ref
                );
            }
        }
    }
    
    // Return the element
    code.push_str("  return _el$;\n");
    code.push_str("})()");
    
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_clone_code() {
        let code = generate_clone_code("_tmpl", "_el");
        assert_eq!(code, "const _el = _tmpl$.cloneNode(true)");
    }

    #[test]
    fn test_generate_element_ref() {
        assert_eq!(generate_element_ref("_el$", &[]), "_el$");
        assert_eq!(
            generate_element_ref("_el$", &["firstChild".to_string()]),
            "_el$.firstChild"
        );
        assert_eq!(
            generate_element_ref("_el$", &["firstChild".to_string(), "nextSibling".to_string()]),
            "_el$.firstChild.nextSibling"
        );
    }

    #[test]
    fn test_generate_insert_code() {
        let options = DomExpressionsOptions::default();
        let code = generate_insert_code(&options, "_el$", "count");
        assert_eq!(code, "insert(_el$, count)");
    }
}
