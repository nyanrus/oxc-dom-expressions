//! Utility functions for the DOM expressions transformer

/// Check if a tag name is a lowercase HTML element
#[allow(dead_code)] // Used by full implementation
pub fn is_html_element(tag_name: &str) -> bool {
    // All lowercase tags are treated as HTML elements
    tag_name.chars().all(|c| c.is_lowercase() || c == '-')
}

/// Check if a tag name is a component (mixed case or capital case)
#[allow(dead_code)] // Used by full implementation
pub fn is_component(tag_name: &str) -> bool {
    // Components start with uppercase or contain mixed case
    tag_name.chars().next().is_some_and(|c| c.is_uppercase())
}

/// Check if an attribute is an event handler
#[allow(dead_code)] // Used by full implementation
pub fn is_event_handler(attr_name: &str) -> bool {
    attr_name.starts_with("on") && attr_name.len() > 2
}

/// Get the event name from an event handler attribute
pub fn get_event_name(attr_name: &str) -> Option<&str> {
    if is_event_handler(attr_name) {
        Some(&attr_name[2..])
    } else {
        None
    }
}

/// Check if an attribute is a ref binding
pub fn is_ref_binding(attr_name: &str) -> bool {
    attr_name == "ref"
}

/// Check if an attribute is a classList binding
pub fn is_class_list_binding(attr_name: &str) -> bool {
    attr_name == "classList"
}

/// Check if an attribute is a style binding
pub fn is_style_binding(attr_name: &str) -> bool {
    attr_name == "style" && !attr_name.starts_with("style:")
}

/// Check if an attribute uses on: prefix (non-delegated event)
pub fn is_on_prefix_event(attr_name: &str) -> bool {
    attr_name.starts_with("on:") && attr_name.len() > 3
}

/// Check if an attribute uses oncapture: prefix
pub fn is_on_capture_event(attr_name: &str) -> bool {
    attr_name.starts_with("oncapture:") && attr_name.len() > 10
}

/// Get event name from on: or oncapture: prefix
pub fn get_prefix_event_name(attr_name: &str) -> Option<&str> {
    if let Some(rest) = attr_name.strip_prefix("on:") {
        Some(rest)
    } else if let Some(rest) = attr_name.strip_prefix("oncapture:") {
        Some(rest)
    } else {
        None
    }
}

/// Check if an attribute is a special binding that needs special handling
pub fn is_special_binding(attr_name: &str) -> bool {
    is_ref_binding(attr_name)
        || is_class_list_binding(attr_name)
        || is_style_binding(attr_name)
        || is_on_prefix_event(attr_name)
        || is_on_capture_event(attr_name)
        || is_bool_attribute(attr_name)
        || is_prop_attribute(attr_name)
        || is_attr_attribute(attr_name)
        || is_use_directive(attr_name)
        || is_style_property(attr_name)
        || is_class_name_binding(attr_name)
}

/// Check if an attribute uses bool: prefix
pub fn is_bool_attribute(attr_name: &str) -> bool {
    attr_name.starts_with("bool:") && attr_name.len() > 5
}

/// Check if an attribute uses prop: prefix
pub fn is_prop_attribute(attr_name: &str) -> bool {
    attr_name.starts_with("prop:") && attr_name.len() > 5
}

/// Check if an attribute uses attr: prefix
pub fn is_attr_attribute(attr_name: &str) -> bool {
    attr_name.starts_with("attr:") && attr_name.len() > 5
}

/// Check if an attribute uses use: prefix
pub fn is_use_directive(attr_name: &str) -> bool {
    attr_name.starts_with("use:") && attr_name.len() > 4
}

/// Check if an attribute uses style: prefix (for individual style properties)
pub fn is_style_property(attr_name: &str) -> bool {
    attr_name.starts_with("style:") && attr_name.len() > 6
}

/// Check if an attribute uses class: prefix (for individual class bindings)
pub fn is_class_name_binding(attr_name: &str) -> bool {
    attr_name.starts_with("class:") && attr_name.len() > 6
}

/// Get the name after a prefix
pub fn get_prefixed_name(attr_name: &str) -> Option<&str> {
    if let Some(rest) = attr_name.strip_prefix("bool:") {
        Some(rest)
    } else if let Some(rest) = attr_name.strip_prefix("prop:") {
        Some(rest)
    } else if let Some(rest) = attr_name.strip_prefix("attr:") {
        Some(rest)
    } else if let Some(rest) = attr_name.strip_prefix("use:") {
        Some(rest)
    } else if let Some(rest) = attr_name.strip_prefix("style:") {
        Some(rest)
    } else if let Some(rest) = attr_name.strip_prefix("class:") {
        Some(rest)
    } else {
        None
    }
}

/// Check if an event should be delegated
#[allow(dead_code)] // Used by full implementation
pub fn should_delegate_event(event_name: &str) -> bool {
    // List of events that can be delegated (bubble or can be composed)
    matches!(
        event_name.to_lowercase().as_str(),
        "click"
            | "dblclick"
            | "input"
            | "change"
            | "submit"
            | "reset"
            | "mousedown"
            | "mouseup"
            | "mouseover"
            | "mouseout"
            | "mousemove"
            | "keydown"
            | "keyup"
            | "keypress"
            | "focus"
            | "blur"
            | "touchstart"
            | "touchend"
            | "touchmove"
            | "touchcancel"
    )
}

/// List of void elements that don't have closing tags
#[allow(dead_code)] // Used by full implementation
pub fn is_void_element(tag_name: &str) -> bool {
    matches!(
        tag_name.to_lowercase().as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_html_element() {
        assert!(is_html_element("div"));
        assert!(is_html_element("span"));
        assert!(is_html_element("custom-element"));
        assert!(!is_html_element("MyComponent"));
        assert!(!is_html_element("Component"));
    }

    #[test]
    fn test_is_component() {
        assert!(is_component("MyComponent"));
        assert!(is_component("Component"));
        assert!(!is_component("div"));
        assert!(!is_component("span"));
    }

    #[test]
    fn test_is_event_handler() {
        assert!(is_event_handler("onClick"));
        assert!(is_event_handler("onChange"));
        assert!(!is_event_handler("class"));
        assert!(!is_event_handler("ref"));
    }

    #[test]
    fn test_get_event_name() {
        assert_eq!(get_event_name("onClick"), Some("Click"));
        assert_eq!(get_event_name("onChange"), Some("Change"));
        assert_eq!(get_event_name("class"), None);
    }

    #[test]
    fn test_should_delegate_event() {
        assert!(should_delegate_event("click"));
        assert!(should_delegate_event("Click"));
        assert!(should_delegate_event("input"));
        assert!(!should_delegate_event("customEvent"));
    }

    #[test]
    fn test_is_void_element() {
        assert!(is_void_element("br"));
        assert!(is_void_element("img"));
        assert!(is_void_element("input"));
        assert!(!is_void_element("div"));
        assert!(!is_void_element("span"));
    }

    #[test]
    fn test_special_bindings() {
        assert!(is_ref_binding("ref"));
        assert!(!is_ref_binding("class"));

        assert!(is_class_list_binding("classList"));
        assert!(!is_class_list_binding("class"));

        assert!(is_style_binding("style"));
        assert!(!is_style_binding("style:color"));

        assert!(is_on_prefix_event("on:CustomEvent"));
        assert!(!is_on_prefix_event("onClick"));

        assert!(is_on_capture_event("oncapture:Click"));
        assert!(!is_on_capture_event("onClick"));

        assert_eq!(get_prefix_event_name("on:CustomEvent"), Some("CustomEvent"));
        assert_eq!(get_prefix_event_name("oncapture:Click"), Some("Click"));
        assert_eq!(get_prefix_event_name("onClick"), None);

        assert!(is_special_binding("ref"));
        assert!(is_special_binding("classList"));
        assert!(is_special_binding("style"));
        assert!(is_special_binding("on:CustomEvent"));
        assert!(!is_special_binding("class"));
    }
}
