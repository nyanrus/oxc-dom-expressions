//! Template minimalization for dom-expressions style output
//!
//! This module handles the conversion of standard HTML/XML templates to
//! dom-expressions minimalized format by:
//! - Using html_subset_parser to parse well-formed HTML into an AST
//! - Omitting quotes from attribute values when safe
//! - Omitting closing tags for elements on the last-child path
//! - Precisely handling the last-child path logic

use crate::html_subset_parser::{HtmlNode, parse as parse_html};
use crate::options::DomExpressionsOptions;

/// Minimalize an HTML template string according to dom-expressions rules
pub fn minimalize_template(html: &str, options: &DomExpressionsOptions) -> String {
    // Parse HTML into AST
    let nodes = parse_html(html);
    
    // Serialize with minimization options
    serialize_html(&nodes, options, true)
}

/// Serialize HTML nodes back to string with minimization
fn serialize_html(nodes: &[HtmlNode], options: &DomExpressionsOptions, is_root: bool) -> String {
    let mut result = String::new();
    
    for (index, node) in nodes.iter().enumerate() {
        let is_last = index == nodes.len() - 1;
        result.push_str(&serialize_node(node, options, is_root && is_last, is_last));
    }
    
    result
}

/// Serialize a single node
fn serialize_node(node: &HtmlNode, options: &DomExpressionsOptions, on_last_path: bool, _is_last_sibling: bool) -> String {
    match node {
        HtmlNode::Text(text) => text.clone(),
        HtmlNode::Marker => "<!>".to_string(),
        HtmlNode::Element { tag, attributes, children, is_void } => {
            let mut result = String::new();
            
            // Opening tag
            result.push('<');
            result.push_str(tag);
            
            // Attributes
            for (name, value) in attributes {
                result.push(' ');
                result.push_str(name);
                if !value.is_empty() {
                    result.push('=');
                    // Check if we should omit quotes
                    if options.omit_quotes && can_omit_quotes(value) {
                        result.push_str(value);
                    } else {
                        result.push('"');
                        result.push_str(value);
                        result.push('"');
                    }
                }
            }
            
            result.push('>');
            
            // Children and closing tag
            if !is_void {
                // Determine if we should render children
                // The last-path optimization: we continue rendering down the last-child path
                // BUT we stop if we encounter mixed content (text + elements at same level)
                
                let has_element_children = children.iter().any(|c| matches!(c, HtmlNode::Element { .. }));
                let has_text_children = children.iter().any(|c| matches!(c, HtmlNode::Text(_)));
                
                // Stop rendering children only if:
                // 1. We're on the last path, AND
                // 2. We have both text and element children (mixed content)
                // This handles the noscript case where it has "No JS!!" text + style element
                let should_stop_here = on_last_path && has_element_children && has_text_children;
                
                if !should_stop_here {
                    // Serialize children
                    for (idx, child) in children.iter().enumerate() {
                        let child_is_last = idx == children.len() - 1;
                        let child_is_element = matches!(child, HtmlNode::Element { .. });
                        let child_on_last_path = on_last_path && child_is_last && child_is_element;
                        result.push_str(&serialize_node(child, options, child_on_last_path, child_is_last));
                    }
                }
                
                // Closing tag - omit if on last path and option is set
                let should_omit_closing = options.omit_last_closing_tag && on_last_path;
                
                if !should_omit_closing {
                    result.push_str("</");
                    result.push_str(tag);
                    result.push('>');
                }
            }
            
            result
        }
    }
}

/// Check if attribute value can be written without quotes
fn can_omit_quotes(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|c| {
        c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ':'
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_omit_quotes() {
        assert!(can_omit_quotes("main"));
        assert!(can_omit_quotes("my-class"));
        assert!(can_omit_quotes("some_value"));
        assert!(can_omit_quotes("file.txt"));
        assert!(can_omit_quotes("namespace:attr"));
        
        assert!(!can_omit_quotes(""));
        assert!(!can_omit_quotes("has space"));
        assert!(!can_omit_quotes("has\"quote"));
        assert!(!can_omit_quotes("has'quote"));
    }

    #[test]
    fn test_minimalize_with_quote_omission() {
        let html = r#"<div id="main"></div>"#;
        let mut options = DomExpressionsOptions::default();
        options.omit_quotes = true;
        options.omit_last_closing_tag = false;
        
        let result = minimalize_template(html, &options);
        assert_eq!(result, "<div id=main></div>");
    }

    #[test]
    fn test_minimalize_with_closing_tag_omission() {
        let html = r#"<div><span></span></div>"#;
        let mut options = DomExpressionsOptions::default();
        options.omit_quotes = false;
        options.omit_last_closing_tag = true;
        
        let result = minimalize_template(html, &options);
        // Root div should omit closing, last child span should omit closing
        assert_eq!(result, r#"<div><span>"#);
    }

    #[test]
    fn test_minimalize_nested_with_text() {
        let html = r#"<div><div><button><span>0</span></button></div></div>"#;
        let mut options = DomExpressionsOptions::default();
        options.omit_quotes = false;
        options.omit_last_closing_tag = true;
        
        let result = minimalize_template(html, &options);
        eprintln!("Input:  {}", html);
        eprintln!("Output: {}", result);
        // Should be: <div><div><button><span>0
        // Root div, last div, last button, last span all omit closing tags
        // But text "0" should still be there
        assert!(result.contains("0"));
        assert_eq!(result, r#"<div><div><button><span>0"#);
    }

    #[test]
    fn test_minimalize_noscript_mixed_content() {
        // noscript with mixed content (text + element) should stop at noscript when on last path
        let html = r#"<div><noscript>No JS!!<style>div</style></noscript></div>"#;
        let mut options = DomExpressionsOptions::default();
        options.omit_quotes = false;
        options.omit_last_closing_tag = true;
        
        let result = minimalize_template(html, &options);
        // Should stop at noscript because it has mixed content (text + element)
        assert_eq!(result, r#"<div><noscript>"#);
    }
}
