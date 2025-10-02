//! Template string generation from JSX elements

use oxc_ast::ast::*;
use std::fmt::Write;

use crate::utils::{
    get_event_name, get_prefix_event_name, is_class_list_binding, is_event_handler,
    is_on_capture_event, is_on_prefix_event, is_ref_binding, is_style_binding, is_void_element,
};

/// Represents a template with its HTML string and dynamic expression positions
#[derive(Debug, Clone)]
pub struct Template {
    /// The HTML template string
    pub html: String,
    /// Positions where dynamic content should be inserted
    pub dynamic_slots: Vec<DynamicSlot>,
}

/// Represents a position where dynamic content needs to be inserted
#[derive(Debug, Clone)]
pub struct DynamicSlot {
    /// Path to the element (e.g., ["firstChild", "nextSibling"])
    pub path: Vec<String>,
    /// Type of dynamic content (text, attribute, etc.)
    pub slot_type: SlotType,
}

/// Type of dynamic slot
#[derive(Debug, Clone)]
pub enum SlotType {
    /// Text content insertion
    TextContent,
    /// Attribute or property
    Attribute(String),
    /// Event handler
    EventHandler(String),
    /// Ref binding
    Ref,
    /// ClassList binding
    ClassList,
    /// Style binding (object)
    StyleObject,
    /// Non-delegated event (on: prefix)
    OnEvent(String),
    /// Capture event (oncapture: prefix)
    OnCaptureEvent(String),
}

/// Build a template from a JSX element
pub fn build_template(element: &JSXElement) -> Template {
    build_template_with_options(element, None)
}

/// Build a template from a JSX element with options
pub fn build_template_with_options(element: &JSXElement, options: Option<&crate::options::DomExpressionsOptions>) -> Template {
    let mut template = Template {
        html: String::new(),
        dynamic_slots: Vec::new(),
    };
    
    // Build with depth 0 (root level), not on last path yet, but is root
    build_element_html(element, &mut template.html, &mut template.dynamic_slots, &mut Vec::new(), options, 0, false, true);
    template
}

/// Build HTML string from JSX element recursively
fn build_element_html(
    element: &JSXElement,
    html: &mut String,
    slots: &mut Vec<DynamicSlot>,
    path: &mut Vec<String>,
    options: Option<&crate::options::DomExpressionsOptions>,
    depth: usize,
    on_last_path: bool,
    is_root: bool,
) {
    let tag_name = get_element_name(&element.opening_element);
    
    // Opening tag
    let _ = write!(html, "<{}", tag_name);
    
    // Process attributes
    for attr in &element.opening_element.attributes {
        if let JSXAttributeItem::Attribute(attr) = attr {
            if let Some(name) = get_attribute_name(&attr.name) {
                // Check for special bindings
                if is_ref_binding(&name) {
                    // Ref binding - track for later code generation
                    slots.push(DynamicSlot {
                        path: path.clone(),
                        slot_type: SlotType::Ref,
                    });
                } else if is_class_list_binding(&name) {
                    // ClassList binding
                    slots.push(DynamicSlot {
                        path: path.clone(),
                        slot_type: SlotType::ClassList,
                    });
                } else if is_style_binding(&name) && attr.value.is_some() {
                    // Style object binding
                    if !matches!(attr.value, Some(JSXAttributeValue::StringLiteral(_))) {
                        slots.push(DynamicSlot {
                            path: path.clone(),
                            slot_type: SlotType::StyleObject,
                        });
                    } else if let Some(value) = &attr.value {
                        // Static style string
                        if let Some(static_value) = get_static_attribute_value(value) {
                            let _ = write!(html, " style=\"{}\"", static_value);
                        }
                    }
                } else if is_on_prefix_event(&name) {
                    // on: prefix event
                    if let Some(event_name) = get_prefix_event_name(&name) {
                        slots.push(DynamicSlot {
                            path: path.clone(),
                            slot_type: SlotType::OnEvent(event_name.to_string()),
                        });
                    }
                } else if is_on_capture_event(&name) {
                    // oncapture: prefix event
                    if let Some(event_name) = get_prefix_event_name(&name) {
                        slots.push(DynamicSlot {
                            path: path.clone(),
                            slot_type: SlotType::OnCaptureEvent(event_name.to_string()),
                        });
                    }
                } else if is_event_handler(&name) {
                    // Regular event handler
                    if let Some(event_name) = get_event_name(&name) {
                        slots.push(DynamicSlot {
                            path: path.clone(),
                            slot_type: SlotType::EventHandler(event_name.to_string()),
                        });
                    }
                } else if let Some(value) = &attr.value {
                    // Regular attribute
                    if let Some(static_value) = get_static_attribute_value(value) {
                        // Static attribute - add to template
                        // Check if we should omit quotes
                        let omit_quotes = if let Some(opts) = options {
                            opts.omit_quotes && can_omit_quotes(&static_value)
                        } else {
                            false
                        };
                        
                        if omit_quotes {
                            let _ = write!(html, " {}={}", name, static_value);
                        } else {
                            let _ = write!(html, " {}=\"{}\"", name, static_value);
                        }
                    } else {
                        // Dynamic attribute - track for later
                        slots.push(DynamicSlot {
                            path: path.clone(),
                            slot_type: SlotType::Attribute(name.clone()),
                        });
                    }
                } else {
                    // Boolean attribute
                    let _ = write!(html, " {}", name);
                }
            }
        }
    }
    
    let _ = write!(html, ">");
    
    // Children
    if !is_void_element(&tag_name) {
        let child_path_start = path.len();
        
        // Check if we should stop rendering children
        // If on_last_path, check if the last element child has element OR expression children (not just text)
        let should_stop_here = if on_last_path {
            if let Some(last_elem_child) = element.children.iter()
                .filter_map(|child| if let JSXChild::Element(elem) = child { Some(elem) } else { None })
                .last()
            {
                // Stop if the last element child has element children OR expression containers
                last_elem_child.children.iter().any(|child| {
                    matches!(child, JSXChild::Element(_) | JSXChild::ExpressionContainer(_))
                })
            } else {
                // No element children at all, so this is a leaf - don't stop
                false
            }
        } else {
            false
        };
        
        let should_render_children = !should_stop_here;
        
        if should_render_children {
            // Find the index of the last element child (ignoring whitespace text)
            let last_element_index = element.children.iter().enumerate()
                .filter(|(_, child)| matches!(child, JSXChild::Element(_)))
                .last()
                .map(|(i, _)| i);
            
            for (i, child) in element.children.iter().enumerate() {
                // Update path for element children
                if matches!(child, JSXChild::Element(_)) {
                    if i == 0 || !element.children[..i].iter().any(|c| matches!(c, JSXChild::Element(_))) {
                        path.push("firstChild".to_string());
                    } else {
                        // Replace last element with nextSibling
                        if let Some(last) = path.last_mut() {
                            *last = "nextSibling".to_string();
                        }
                    }
                }
                
                // Check if this element child is the last one
                let is_last_element_child = Some(i) == last_element_index;
                // Only the last element child (or root's last child) is on the last path
                let child_on_last_path = (on_last_path || is_root) && is_last_element_child;
                
                build_child_html(child, html, slots, path, options, depth, child_on_last_path, false);
            }
        }
        
        // Restore path
        path.truncate(child_path_start);
        
        // Closing tag - check if we should omit it
        // Omit if this is the root or on the last path, and option is set
        let omit_closing = if let Some(opts) = options {
            opts.omit_last_closing_tag && (is_root || on_last_path)
        } else {
            false
        };
        
        if !omit_closing {
            let _ = write!(html, "</{}>", tag_name);
        }
    }
}

/// Check if attribute value can be written without quotes
fn can_omit_quotes(value: &str) -> bool {
    // Can omit quotes if value contains only safe characters
    // Safe characters: alphanumeric, hyphen, underscore, period, colon
    !value.is_empty() && value.chars().all(|c| {
        c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ':'
    })
}

/// Build HTML for a JSX child
fn build_child_html(
    child: &JSXChild,
    html: &mut String,
    slots: &mut Vec<DynamicSlot>,
    path: &mut Vec<String>,
    options: Option<&crate::options::DomExpressionsOptions>,
    depth: usize,
    on_last_path: bool,
    is_root: bool,
) {
    match child {
        JSXChild::Text(text) => {
            // Static text - only include if non-whitespace
            let text_value = text.value.as_str();
            // Skip whitespace-only text nodes (common in formatted JSX)
            if text_value.trim().is_empty() {
                return;
            }
            // Trim and escape special characters for template strings
            let trimmed = text_value.trim();
            let escaped = trimmed.replace('{', "\\{").replace('}', "\\}");
            html.push_str(&escaped);
        }
        JSXChild::Element(elem) => {
            build_element_html(elem, html, slots, path, options, depth + 1, on_last_path, is_root);
        }
        JSXChild::ExpressionContainer(container) => {
            // Check if this is a static literal that can be inlined
            match &container.expression {
                JSXExpression::StringLiteral(string_lit) => {
                    // Static string - include in template with escaping
                    let escaped = string_lit.value.as_str().replace('{', "\\{").replace('}', "\\}");
                    html.push_str(&escaped);
                    return;
                }
                JSXExpression::NumericLiteral(num_lit) => {
                    // Static number - include in template
                    html.push_str(&num_lit.value.to_string());
                    return;
                }
                _ => {}
            }
            // Dynamic content - leave empty in template and record the slot
            slots.push(DynamicSlot {
                path: path.clone(),
                slot_type: SlotType::TextContent,
            });
        }
        JSXChild::Fragment(_) | JSXChild::Spread(_) => {
            // Not implemented yet
        }
    }
}

/// Get element name from JSX opening element
fn get_element_name(opening: &JSXOpeningElement) -> String {
    match &opening.name {
        JSXElementName::Identifier(ident) => ident.name.to_string(),
        JSXElementName::IdentifierReference(ident) => ident.name.to_string(),
        JSXElementName::NamespacedName(namespaced) => {
            format!("{}:{}", namespaced.namespace.name, namespaced.name.name)
        }
        JSXElementName::MemberExpression(_) => {
            // Component member expression - not supported in templates
            "div".to_string()
        }
        JSXElementName::ThisExpression(_) => {
            "div".to_string()
        }
    }
}

/// Get attribute name from JSX attribute name
fn get_attribute_name(name: &JSXAttributeName) -> Option<String> {
    match name {
        JSXAttributeName::Identifier(ident) => {
            // Convert JSX attribute names to HTML
            let attr_name = ident.name.as_str();
            Some(match attr_name {
                "className" => "class".to_string(),
                "htmlFor" => "for".to_string(),
                _ => attr_name.to_string(),
            })
        }
        JSXAttributeName::NamespacedName(namespaced) => {
            Some(format!("{}:{}", namespaced.namespace.name, namespaced.name.name))
        }
    }
}

/// Get static value from JSX attribute value
fn get_static_attribute_value(value: &JSXAttributeValue) -> Option<String> {
    match value {
        JSXAttributeValue::StringLiteral(lit) => Some(lit.value.to_string()),
        JSXAttributeValue::ExpressionContainer(container) => {
            // Check if the expression is a static literal
            match &container.expression {
                JSXExpression::StringLiteral(string_lit) => Some(string_lit.value.to_string()),
                JSXExpression::NumericLiteral(num_lit) => Some(num_lit.value.to_string()),
                JSXExpression::BooleanLiteral(bool_lit) => Some(bool_lit.value.to_string()),
                _ => None, // Dynamic values are not included in template
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These are placeholder tests
    // In a full implementation, we would parse JSX and test template generation
    
    #[test]
    fn test_template_struct() {
        let template = Template {
            html: String::from("<div></div>"),
            dynamic_slots: Vec::new(),
        };
        assert_eq!(template.html, "<div></div>");
        assert_eq!(template.dynamic_slots.len(), 0);
    }
}
