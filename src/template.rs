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
    let mut template = Template {
        html: String::new(),
        dynamic_slots: Vec::new(),
    };
    
    build_element_html(element, &mut template.html, &mut template.dynamic_slots, &mut Vec::new());
    template
}

/// Build HTML string from JSX element recursively
fn build_element_html(
    element: &JSXElement,
    html: &mut String,
    slots: &mut Vec<DynamicSlot>,
    path: &mut Vec<String>,
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
                        let _ = write!(html, " {}=\"{}\"", name, static_value);
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
        
        for (i, child) in element.children.iter().enumerate() {
            // Update path for this child
            if i == 0 {
                path.push("firstChild".to_string());
            } else {
                // Replace last element with nextSibling
                if let Some(last) = path.last_mut() {
                    *last = "nextSibling".to_string();
                }
            }
            
            build_child_html(child, html, slots, path);
        }
        
        // Restore path
        path.truncate(child_path_start);
        
        // Closing tag
        let _ = write!(html, "</{}>", tag_name);
    }
}

/// Build HTML for a JSX child
fn build_child_html(
    child: &JSXChild,
    html: &mut String,
    slots: &mut Vec<DynamicSlot>,
    path: &mut Vec<String>,
) {
    match child {
        JSXChild::Text(text) => {
            // Static text
            let text_value = text.value.as_str();
            if !text_value.trim().is_empty() {
                html.push_str(text_value);
            }
        }
        JSXChild::Element(elem) => {
            build_element_html(elem, html, slots, path);
        }
        JSXChild::ExpressionContainer(_) => {
            // Dynamic content - leave empty in template
            // Record the slot for later
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
        _ => None, // Dynamic values are not included in template
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
