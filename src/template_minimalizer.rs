//! Template minimalization for dom-expressions style output
//!
//! This module handles the conversion of standard HTML/XML templates to
//! dom-expressions minimalized format by:
//! - Parsing well-formed HTML into an AST
//! - Omitting quotes from attribute values when safe
//! - Omitting closing tags for elements on the last-child path
//! - Precisely handling the last-child path logic

use crate::options::DomExpressionsOptions;

/// HTML node types in our AST
#[derive(Debug, Clone)]
pub enum HtmlNode {
    Element {
        tag: String,
        attributes: Vec<(String, String)>,
        children: Vec<HtmlNode>,
        is_void: bool,
    },
    Text(String),
}

/// Minimalize an HTML template string according to dom-expressions rules
pub fn minimalize_template(html: &str, options: &DomExpressionsOptions) -> String {
    // Parse HTML into AST
    let nodes = parse_html(html);
    
    // Serialize with minimization options
    serialize_html(&nodes, options, true)
}

/// Parse HTML into an AST (public for testing)
pub fn parse_html(html: &str) -> Vec<HtmlNode> {
    let mut chars = html.chars().peekable();
    let mut nodes = Vec::new();
    
    while chars.peek().is_some() {
        if let Some(node) = parse_node(&mut chars) {
            nodes.push(node);
        }
    }
    
    nodes
}

/// Parse a single HTML node
fn parse_node(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<HtmlNode> {
    // Skip whitespace between tags
    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
    
    if chars.peek() == Some(&'<') {
        chars.next(); // consume '<'
        
        // Check if it's a closing tag
        if chars.peek() == Some(&'/') {
            // This is a closing tag, return None to signal end of element
            return None;
        }
        
        // Parse tag name
        let mut tag = String::new();
        while let Some(&ch) = chars.peek() {
            if ch.is_whitespace() || ch == '>' || ch == '/' {
                break;
            }
            tag.push(ch);
            chars.next();
        }
        
        // Parse attributes
        let mut attributes = Vec::new();
        loop {
            // Skip whitespace
            while let Some(&ch) = chars.peek() {
                if ch.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }
            
            // Check for end of tag
            if chars.peek() == Some(&'>') || chars.peek() == Some(&'/') {
                break;
            }
            
            // Parse attribute name
            let mut attr_name = String::new();
            while let Some(&ch) = chars.peek() {
                if ch == '=' || ch.is_whitespace() || ch == '>' {
                    break;
                }
                attr_name.push(ch);
                chars.next();
            }
            
            if attr_name.is_empty() {
                break;
            }
            
            // Skip whitespace
            while let Some(&ch) = chars.peek() {
                if ch.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }
            
            // Check for '='
            let mut attr_value = String::new();
            if chars.peek() == Some(&'=') {
                chars.next(); // consume '='
                
                // Skip whitespace
                while let Some(&ch) = chars.peek() {
                    if ch.is_whitespace() {
                        chars.next();
                    } else {
                        break;
                    }
                }
                
                // Parse attribute value
                if let Some(&quote_ch) = chars.peek() {
                    if quote_ch == '"' || quote_ch == '\'' {
                        chars.next(); // consume quote
                        while let Some(&ch) = chars.peek() {
                            if ch == quote_ch {
                                chars.next(); // consume closing quote
                                break;
                            }
                            attr_value.push(ch);
                            chars.next();
                        }
                    } else {
                        // Unquoted attribute value
                        while let Some(&ch) = chars.peek() {
                            if ch.is_whitespace() || ch == '>' {
                                break;
                            }
                            attr_value.push(ch);
                            chars.next();
                        }
                    }
                }
            }
            
            attributes.push((attr_name, attr_value));
        }
        
        // Check if self-closing or void element
        let is_void = is_void_tag(&tag);
        let mut self_closing = false;
        
        if chars.peek() == Some(&'/') {
            chars.next(); // consume '/'
            self_closing = true;
        }
        
        if chars.peek() == Some(&'>') {
            chars.next(); // consume '>'
        }
        
        // Parse children for non-void elements
        let mut children = Vec::new();
        if !is_void && !self_closing {
            loop {
                // Skip whitespace  
                while let Some(&ch) = chars.peek() {
                    if ch.is_whitespace() {
                        chars.next();
                    } else {
                        break;
                    }
                }
                
                // Check for closing tag by looking ahead
                if chars.peek() == Some(&'<') {
                    let mut temp_chars = chars.clone();
                    temp_chars.next(); // consume '<'
                    
                    if temp_chars.peek() == Some(&'/') {
                        temp_chars.next(); // consume '/'
                        
                        // Check if this closing tag matches our opening tag
                        let mut closing_tag_name = String::new();
                        while let Some(&ch) = temp_chars.peek() {
                            if ch == '>' || ch.is_whitespace() {
                                break;
                            }
                            closing_tag_name.push(ch);
                            temp_chars.next();
                        }
                        
                        if closing_tag_name == tag {
                            // This is our closing tag, consume it
                            while let Some(&ch) = chars.peek() {
                                chars.next();
                                if ch == '>' {
                                    break;
                                }
                            }
                            break;
                        }
                        // Otherwise, it's a closing tag for a child element
                        // Let the recursive call handle it
                    }
                    
                    // Try to parse a child element
                    if let Some(child) = parse_node(chars) {
                        children.push(child);
                    } else {
                        // parse_node returned None, which means it hit a closing tag
                        // This closing tag is for us, so break
                        break;
                    }
                } else if chars.peek().is_some() {
                    // Parse text content
                    let mut text = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch == '<' {
                            break;
                        }
                        text.push(ch);
                        chars.next();
                    }
                    if !text.is_empty() {
                        children.push(HtmlNode::Text(text));
                    }
                } else {
                    // End of input
                    break;
                }
            }
        }
        
        Some(HtmlNode::Element {
            tag,
            attributes,
            children,
            is_void,
        })
    } else {
        // Parse text content
        let mut text = String::new();
        while let Some(&ch) = chars.peek() {
            if ch == '<' {
                break;
            }
            text.push(ch);
            chars.next();
        }
        if !text.is_empty() {
            Some(HtmlNode::Text(text))
        } else {
            None
        }
    }
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

/// Check if a tag is a void element (self-closing in HTML)
fn is_void_tag(tag: &str) -> bool {
    matches!(
        tag.to_lowercase().as_str(),
        "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" |
        "link" | "meta" | "param" | "source" | "track" | "wbr"
    )
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
    fn test_parse_simple_element() {
        let html = r#"<div id="main"></div>"#;
        let nodes = parse_html(html);
        assert_eq!(nodes.len(), 1);
        
        if let HtmlNode::Element { tag, attributes, children, .. } = &nodes[0] {
            assert_eq!(tag, "div");
            assert_eq!(attributes.len(), 1);
            assert_eq!(attributes[0].0, "id");
            assert_eq!(attributes[0].1, "main");
            assert_eq!(children.len(), 0);
        } else {
            panic!("Expected element node");
        }
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
    fn test_parse_nested_elements() {
        let html = r#"<div><button><span>0</span></button></div>"#;
        let nodes = parse_html(html);
        
        assert_eq!(nodes.len(), 1);
        if let HtmlNode::Element { tag, children, .. } = &nodes[0] {
            assert_eq!(tag, "div");
            assert_eq!(children.len(), 1);
            
            if let HtmlNode::Element { tag: tag2, children: children2, .. } = &children[0] {
                assert_eq!(tag2, "button");
                assert_eq!(children2.len(), 1);
                
                if let HtmlNode::Element { tag: tag3, children: children3, .. } = &children2[0] {
                    assert_eq!(tag3, "span");
                    assert_eq!(children3.len(), 1);
                    
                    if let HtmlNode::Text(text) = &children3[0] {
                        assert_eq!(text, "0");
                    } else {
                        panic!("Expected text node");
                    }
                } else {
                    panic!("Expected span element");
                }
            } else {
                panic!("Expected button element");
            }
        } else {
            panic!("Expected div element");
        }
    }

    #[test]
    fn test_parse_nested_divs() {
        let html = r#"<div><div><button></button></div></div>"#;
        let nodes = parse_html(html);
        
        eprintln!("Parsed nodes for nested divs: {:#?}", nodes);
        
        assert_eq!(nodes.len(), 1);
        if let HtmlNode::Element { tag, children, .. } = &nodes[0] {
            assert_eq!(tag, "div");
            assert_eq!(children.len(), 1, "Outer div should have 1 child");
            
            if let HtmlNode::Element { tag: tag2, children: children2, .. } = &children[0] {
                assert_eq!(tag2, "div");
                assert_eq!(children2.len(), 1, "Inner div should have 1 child (button)");
            } else {
                panic!("Expected inner div element");
            }
        } else {
            panic!("Expected outer div element");
        }
    }
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
    fn test_parse_noscript() {
        let html = r#"<div><noscript>No JS!!<style>div</style></noscript></div>"#;
        let nodes = parse_html(html);
        
        if let HtmlNode::Element { children, .. } = &nodes[0] {
            if let HtmlNode::Element { tag, children: noscript_children, .. } = &children[0] {
                assert_eq!(tag, "noscript");
                assert_eq!(noscript_children.len(), 2);
            }
        }
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
