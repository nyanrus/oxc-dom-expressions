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
#[allow(dead_code)] // Used by full implementation
pub fn get_event_name(attr_name: &str) -> Option<&str> {
    if is_event_handler(attr_name) {
        Some(&attr_name[2..])
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
}
