//! Template string generation from JSX elements
//!
//! This module handles the conversion of JSX elements into HTML template strings
//! with markers for dynamic content positions.
//!
//! # Template Structure
//!
//! A template consists of:
//! - **HTML string**: Static HTML structure with placeholders for dynamic content
//! - **Dynamic slots**: Positions where dynamic content needs to be inserted
//!
//! # Example
//!
//! Input JSX:
//! ```jsx
//! <div class="container">
//!   <span>{message}</span>
//! </div>
//! ```
//!
//! Output Template:
//! ```rust
//! Template {
//!     html: "<div class=\"container\"><span>",
//!     dynamic_slots: [
//!         DynamicSlot {
//!             path: vec!["firstChild"],  // Path to <span>
//!             slot_type: SlotType::TextContent,
//!             marker_path: None,
//!         }
//!     ]
//! }
//! ```
//!
//! # Dynamic Slot Types
//!
//! - **TextContent**: Text insertion points
//! - **Attribute**: Dynamic attributes (id, class, etc.)
//! - **EventHandler**: Event handlers (onClick, etc.)
//! - **Ref**: Element references
//! - **ClassList**: Dynamic class bindings
//! - **StyleObject**: Dynamic style objects
//! - **OnEvent**: Custom events (on: prefix)
//! - **OnCaptureEvent**: Capture phase events (oncapture: prefix)

use oxc_ast::ast::*;
use std::fmt::Write;

use crate::utils::{
    get_event_name, get_prefix_event_name, get_prefixed_name, is_attr_attribute, is_bool_attribute,
    is_class_list_binding, is_class_name_binding, is_event_handler, is_on_capture_event,
    is_on_prefix_event, is_prop_attribute, is_ref_binding, is_style_binding, is_style_property,
    is_use_directive, is_void_element,
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
    /// Path to the marker node (for text content insertion positioning)
    /// None if this is a trailing expression (insert at end with null)
    pub marker_path: Option<Vec<String>>,
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
    /// Boolean attribute (bool: prefix)
    BoolAttribute(String),
    /// Property attribute (prop: prefix)
    PropAttribute(String),
    /// Attribute attribute (attr: prefix)
    AttrAttribute(String),
    /// Use directive (use: prefix)
    UseDirective(String),
    /// Style property (style: prefix)
    StyleProperty(String),
    /// Class name binding (class: prefix)
    ClassName(String),
}

/// Build a template from a JSX element
pub fn build_template(element: &JSXElement) -> Template {
    build_template_with_options(element, None)
}

/// Build a template from a JSX element with options
pub fn build_template_with_options(
    element: &JSXElement,
    options: Option<&crate::options::DomExpressionsOptions>,
) -> Template {
    let mut template = Template {
        html: String::new(),
        dynamic_slots: Vec::new(),
    };

    // Build standard HTML from JSX
    build_element_html(
        element,
        &mut template.html,
        &mut template.dynamic_slots,
        &mut Vec::new(),
    );

    // Apply minimalization if options are provided
    if let Some(opts) = options {
        template.html = crate::template_minimalizer::minimalize_template(&template.html, opts);
    }

    template
}

/// Build HTML string from JSX element recursively
/// This produces standard, well-formed HTML without minimalization
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
                        marker_path: None,
                    });
                } else if is_class_list_binding(&name) {
                    // ClassList binding
                    slots.push(DynamicSlot {
                        path: path.clone(),
                        slot_type: SlotType::ClassList,
                        marker_path: None,
                    });
                } else if is_style_binding(&name) && attr.value.is_some() {
                    // Style object binding
                    if !matches!(attr.value, Some(JSXAttributeValue::StringLiteral(_))) {
                        slots.push(DynamicSlot {
                            path: path.clone(),
                            slot_type: SlotType::StyleObject,
                            marker_path: None,
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
                            marker_path: None,
                        });
                    }
                } else if is_on_capture_event(&name) {
                    // oncapture: prefix event
                    if let Some(event_name) = get_prefix_event_name(&name) {
                        slots.push(DynamicSlot {
                            path: path.clone(),
                            slot_type: SlotType::OnCaptureEvent(event_name.to_string()),
                            marker_path: None,
                        });
                    }
                } else if is_bool_attribute(&name) {
                    // bool: prefix attribute
                    if let Some(attr_name) = get_prefixed_name(&name) {
                        slots.push(DynamicSlot {
                            path: path.clone(),
                            slot_type: SlotType::BoolAttribute(attr_name.to_string()),
                            marker_path: None,
                        });
                    }
                } else if is_prop_attribute(&name) {
                    // prop: prefix attribute
                    if let Some(attr_name) = get_prefixed_name(&name) {
                        slots.push(DynamicSlot {
                            path: path.clone(),
                            slot_type: SlotType::PropAttribute(attr_name.to_string()),
                            marker_path: None,
                        });
                    }
                } else if is_attr_attribute(&name) {
                    // attr: prefix attribute
                    if let Some(attr_name) = get_prefixed_name(&name) {
                        slots.push(DynamicSlot {
                            path: path.clone(),
                            slot_type: SlotType::AttrAttribute(attr_name.to_string()),
                            marker_path: None,
                        });
                    }
                } else if is_use_directive(&name) {
                    // use: prefix directive
                    if let Some(directive_name) = get_prefixed_name(&name) {
                        slots.push(DynamicSlot {
                            path: path.clone(),
                            slot_type: SlotType::UseDirective(directive_name.to_string()),
                            marker_path: None,
                        });
                    }
                } else if is_style_property(&name) {
                    // style: prefix property
                    if let Some(prop_name) = get_prefixed_name(&name) {
                        slots.push(DynamicSlot {
                            path: path.clone(),
                            slot_type: SlotType::StyleProperty(prop_name.to_string()),
                            marker_path: None,
                        });
                    }
                } else if is_class_name_binding(&name) {
                    // class: prefix binding
                    if let Some(class_name) = get_prefixed_name(&name) {
                        slots.push(DynamicSlot {
                            path: path.clone(),
                            slot_type: SlotType::ClassName(class_name.to_string()),
                            marker_path: None,
                        });
                    }
                } else if is_event_handler(&name) {
                    // Regular event handler
                    if let Some(event_name) = get_event_name(&name) {
                        slots.push(DynamicSlot {
                            path: path.clone(),
                            slot_type: SlotType::EventHandler(event_name.to_string()),
                            marker_path: None,
                        });
                    }
                } else if let Some(value) = &attr.value {
                    // Regular attribute
                    if let Some(static_value) = get_static_attribute_value(value) {
                        // Static attribute - add to template with quotes
                        let _ = write!(html, " {}=\"{}\"", name, static_value);
                    } else {
                        // Dynamic attribute - track for later
                        slots.push(DynamicSlot {
                            path: path.clone(),
                            slot_type: SlotType::Attribute(name.clone()),
                            marker_path: None,
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

        // First pass: determine which children will create nodes
        let mut will_create_node = Vec::new();
        for child in &element.children {
            let creates_node = match child {
                JSXChild::Text(text) => {
                    let text_value = text.value.as_str();
                    !(text_value.trim().is_empty() && text_value.contains('\n'))
                }
                JSXChild::Element(_) => true,
                JSXChild::ExpressionContainer(container) => !matches!(
                    &container.expression,
                    JSXExpression::StringLiteral(_)
                        | JSXExpression::NumericLiteral(_)
                        | JSXExpression::EmptyExpression(_)
                ),
                JSXChild::Fragment(_) | JSXChild::Spread(_) => false,
            };
            will_create_node.push(creates_node);
        }

        // Second pass: process children and track paths
        let mut num_nodes_added = 0;
        let mut last_marker_path: Option<Vec<String>> = None;

        for (i, child) in element.children.iter().enumerate() {
            let is_last_child = i == element.children.len() - 1;
            let is_expression = matches!(child, JSXChild::ExpressionContainer(_));
            
            // Check if previous child was an expression (to detect adjacent expressions)
            let prev_is_expression = if i > 0 {
                matches!(element.children[i - 1], JSXChild::ExpressionContainer(_))
            } else {
                false
            };

            // Calculate the path for this child based on nodes added so far
            let mut child_path = Vec::new();
            if num_nodes_added == 0 {
                child_path.push("firstChild".to_string());
            } else {
                child_path.push("firstChild".to_string());
                for _ in 0..num_nodes_added {
                    child_path.push("nextSibling".to_string());
                }
            }

            // Set the path for processing this child
            *path = child_path.clone();

            // Process the child
            build_child_html_with_context(
                child,
                html,
                slots,
                path,
                is_last_child,
                prev_is_expression,
                num_nodes_added,
                &mut last_marker_path,
                &element.children,
                i,
            );

            // Update count if this child will create a node (marker or actual content)
            if will_create_node[i] {
                num_nodes_added += 1;
            }
            
            // Clear last_marker_path if this wasn't an expression
            if !is_expression {
                last_marker_path = None;
            }
        }

        // Restore path
        path.truncate(child_path_start);

        // Always add closing tag for standard HTML
        let _ = write!(html, "</{}>", tag_name);
    }
}

/// Build HTML for a JSX child with context about its position
fn build_child_html_with_context(
    child: &JSXChild,
    html: &mut String,
    slots: &mut Vec<DynamicSlot>,
    path: &mut Vec<String>,
    is_last_child: bool,
    prev_is_expression: bool,
    num_nodes_so_far: usize,
    last_marker_path: &mut Option<Vec<String>>,
    _all_children: &[JSXChild],
    _i: usize,
) {
    match child {
        JSXChild::Text(text) => {
            // Static text - escape for template literals
            // Only escape opening braces to match babel plugin behavior
            let text_value = text.value.as_str();

            // Skip pure formatting whitespace (newlines + indentation)
            // BUT preserve inline spaces (e.g., between expressions)
            if text_value.trim().is_empty() && text_value.contains('\n') {
                // This is formatting whitespace with newlines - skip it
                return;
            }

            // Preserve all other text, including spaces
            let escaped = text_value.replace('\\', "\\\\").replace('{', "\\{");
            html.push_str(&escaped);
        }
        JSXChild::Element(elem) => {
            build_element_html(elem, html, slots, path);
        }
        JSXChild::ExpressionContainer(container) => {
            // Check if this is a static literal that can be inlined
            match &container.expression {
                JSXExpression::StringLiteral(string_lit) => {
                    // Static string - include in template with escaping
                    // Only escape opening braces to match babel plugin behavior
                    let escaped = string_lit
                        .value
                        .as_str()
                        .replace('\\', "\\\\")
                        .replace('{', "\\{");
                    html.push_str(&escaped);
                    return;
                }
                JSXExpression::NumericLiteral(num_lit) => {
                    // Static number - include in template
                    html.push_str(&num_lit.value.to_string());
                    return;
                }
                JSXExpression::EmptyExpression(_) => {
                    // Empty expression (comment) - skip it
                    return;
                }
                _ => {}
            }

            // Dynamic content - determine marker strategy:
            // The babel plugin minimizes template size by avoiding markers when possible.
            // Rules:
            // 1. Adjacent expressions share one marker
            // 2. If this is the first NODE (num_nodes_so_far == 0), use next node as insertion point
            // 3. If expression is LAST child, insert at end with null (no marker)
            // 4. Otherwise, add a marker after the expression
            
            // Check if this is the first real node (not counting skipped formatting whitespace)
            let is_first_node = num_nodes_so_far == 0;
            
            let marker_path = if prev_is_expression && last_marker_path.is_some() {
                // Adjacent to previous expression - reuse marker
                last_marker_path.clone()
            } else if is_first_node && !is_last_child {
                // First node but not last child - use next node as insertion point
                let mut next_path = Vec::new();
                next_path.push("firstChild".to_string());
                Some(next_path)
            } else if is_last_child {
                // Last child - insert at end
                None
            } else {
                // Middle child - add marker
                html.push_str("<!>");
                let marker = Some(path.clone());
                *last_marker_path = marker.clone();
                marker
            };

            slots.push(DynamicSlot {
                path: Vec::new(), // Insert into parent element (empty path)
                slot_type: SlotType::TextContent,
                marker_path,
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
        JSXElementName::ThisExpression(_) => "div".to_string(),
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
        JSXAttributeName::NamespacedName(namespaced) => Some(format!(
            "{}:{}",
            namespaced.namespace.name, namespaced.name.name
        )),
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
#[cfg(test)]
mod template_debug {
    use super::*;

    #[test]
    fn debug_template_structure() {
        let test_cases = vec![
            r#"<span>Hello {name}</span>"#,
            r#"<span>{greeting} {name}</span>"#,
            r#"<span> {greeting} {name} </span>"#,
        ];

        for code in test_cases {
            println!("\n=== Testing: {} ===", code);
            let allocator = oxc_allocator::Allocator::default();
            let ret =
                oxc_parser::Parser::new(&allocator, code, oxc_span::SourceType::jsx()).parse();

            if let Some(expr) = ret.program.body.first() {
                if let oxc_ast::ast::Statement::ExpressionStatement(stmt) = expr {
                    if let oxc_ast::ast::Expression::JSXElement(elem) = &stmt.expression {
                        let template = crate::template::build_template(elem);
                        println!("HTML: {:?}", template.html);
                        for (i, slot) in template.dynamic_slots.iter().enumerate() {
                            println!(
                                "Slot {}: path={:?}, marker_path={:?}, type={:?}",
                                i, slot.path, slot.marker_path, slot.slot_type
                            );
                        }
                    }
                }
            }
        }
    }
}
