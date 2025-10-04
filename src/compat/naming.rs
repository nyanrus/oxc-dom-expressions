//! Variable naming conventions for babel-plugin-jsx-dom-expressions compatibility
//!
//! This module centralizes the variable naming logic used by the original
//! babel-plugin-jsx-dom-expressions. These naming conventions are purely for
//! compatibility and don't affect functionality.
//!
//! # Naming Patterns
//!
//! - `_tmpl$` / `_tmpl$2` - Template variable names
//! - `_el$` / `_el$2` - Element variable names
//! - `_$functionName` - Runtime function prefixes

use super::constants::{ELEMENT_VAR_PREFIX, RUNTIME_FN_PREFIX, TEMPLATE_VAR_PREFIX};

/// Generate a template variable name following babel conventions
///
/// # Arguments
///
/// * `counter` - Template counter value (0 for first template)
///
/// # Returns
///
/// Variable name like `_tmpl$` or `_tmpl$2`
///
/// # Examples
///
/// ```
/// use oxc_dom_expressions::compat::naming::template_var_name;
///
/// assert_eq!(template_var_name(1), "_tmpl$");
/// assert_eq!(template_var_name(2), "_tmpl$2");
/// assert_eq!(template_var_name(10), "_tmpl$10");
/// ```
pub fn template_var_name(counter: usize) -> String {
    if counter == 1 {
        TEMPLATE_VAR_PREFIX.to_string()
    } else {
        format!("{}{}", TEMPLATE_VAR_PREFIX, counter)
    }
}

/// Generate an element variable name following babel conventions
///
/// # Arguments
///
/// * `counter` - Element counter value (1 for first element)
///
/// # Returns
///
/// Variable name like `_el$1`, `_el$2`, etc.
///
/// # Examples
///
/// ```
/// use oxc_dom_expressions::compat::naming::element_var_name;
///
/// assert_eq!(element_var_name(1), "_el$1");
/// assert_eq!(element_var_name(2), "_el$2");
/// assert_eq!(element_var_name(10), "_el$10");
/// ```
pub fn element_var_name(counter: usize) -> String {
    format!("{}{}", ELEMENT_VAR_PREFIX, counter)
}

/// Get the runtime function name with babel prefix
///
/// # Arguments
///
/// * `name` - Base function name (e.g., "insert", "template")
///
/// # Returns
///
/// Prefixed function name like `_$insert`, `_$template`
///
/// # Examples
///
/// ```
/// use oxc_dom_expressions::compat::naming::runtime_function_name;
///
/// assert_eq!(runtime_function_name("insert"), "_$insert");
/// assert_eq!(runtime_function_name("template"), "_$template");
/// assert_eq!(runtime_function_name("delegateEvents"), "_$delegateEvents");
/// ```
pub fn runtime_function_name(name: &str) -> String {
    format!("{}{}", RUNTIME_FN_PREFIX, name)
}

/// Check if a variable name is a template variable
///
/// # Arguments
///
/// * `name` - Variable name to check
///
/// # Returns
///
/// `true` if the name matches the template variable pattern
pub fn is_template_var(name: &str) -> bool {
    name.starts_with(TEMPLATE_VAR_PREFIX)
}

/// Check if a variable name is an element variable
///
/// # Arguments
///
/// * `name` - Variable name to check
///
/// # Returns
///
/// `true` if the name matches the element variable pattern
pub fn is_element_var(name: &str) -> bool {
    name.starts_with(ELEMENT_VAR_PREFIX)
}

/// Extract the counter from a template variable name
///
/// # Arguments
///
/// * `name` - Template variable name
///
/// # Returns
///
/// The counter value, or None if not a valid template variable
///
/// # Examples
///
/// ```
/// use oxc_dom_expressions::compat::naming::extract_template_counter;
///
/// assert_eq!(extract_template_counter("_tmpl$"), Some(1));
/// assert_eq!(extract_template_counter("_tmpl$5"), Some(5));
/// assert_eq!(extract_template_counter("other"), None);
/// ```
pub fn extract_template_counter(name: &str) -> Option<usize> {
    if name == TEMPLATE_VAR_PREFIX {
        Some(1)
    } else if let Some(num_str) = name.strip_prefix(TEMPLATE_VAR_PREFIX) {
        num_str.parse::<usize>().ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_var_name() {
        assert_eq!(template_var_name(1), "_tmpl$");
        assert_eq!(template_var_name(2), "_tmpl$2");
        assert_eq!(template_var_name(10), "_tmpl$10");
    }

    #[test]
    fn test_element_var_name() {
        assert_eq!(element_var_name(1), "_el$1");
        assert_eq!(element_var_name(2), "_el$2");
        assert_eq!(element_var_name(10), "_el$10");
    }

    #[test]
    fn test_runtime_function_name() {
        assert_eq!(runtime_function_name("insert"), "_$insert");
        assert_eq!(runtime_function_name("template"), "_$template");
        assert_eq!(runtime_function_name("delegateEvents"), "_$delegateEvents");
    }

    #[test]
    fn test_is_template_var() {
        assert!(is_template_var("_tmpl$"));
        assert!(is_template_var("_tmpl$2"));
        assert!(is_template_var("_tmpl$100"));
        assert!(!is_template_var("_el$"));
        assert!(!is_template_var("other"));
    }

    #[test]
    fn test_is_element_var() {
        assert!(is_element_var("_el$1"));
        assert!(is_element_var("_el$2"));
        assert!(is_element_var("_el$100"));
        assert!(!is_element_var("_tmpl$"));
        assert!(!is_element_var("other"));
    }

    #[test]
    fn test_extract_template_counter() {
        assert_eq!(extract_template_counter("_tmpl$"), Some(1));
        assert_eq!(extract_template_counter("_tmpl$5"), Some(5));
        assert_eq!(extract_template_counter("_tmpl$100"), Some(100));
        assert_eq!(extract_template_counter("other"), None);
        assert_eq!(extract_template_counter("_el$"), None);
    }
}
