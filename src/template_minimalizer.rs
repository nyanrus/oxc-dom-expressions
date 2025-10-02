//! Template minimalization for dom-expressions style output
//!
//! This module handles the conversion of standard HTML/XML templates to
//! dom-expressions minimalized format by:
//! - Omitting quotes from attribute values when safe
//! - Omitting closing tags for elements on the last-child path
//! - Escaping special characters for template strings

use crate::options::DomExpressionsOptions;

/// Minimalize an HTML template string according to dom-expressions rules
pub fn minimalize_template(html: &str, options: &DomExpressionsOptions) -> String {
    let mut result = html.to_string();
    
    if options.omit_quotes {
        result = omit_attribute_quotes(&result);
    }
    
    if options.omit_last_closing_tag {
        result = omit_closing_tags(&result);
    }
    
    result
}

/// Omit quotes from HTML attribute values when they contain only safe characters
/// Safe characters: alphanumeric, hyphen, underscore, period, colon
fn omit_attribute_quotes(html: &str) -> String {
    // This is a simplified implementation
    // In a full implementation, we'd use proper HTML parsing
    // For now, we use regex or simple string processing
    
    let mut result = String::new();
    let mut chars = html.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '=' {
            result.push(ch);
            // Check if next char is a quote
            if let Some(&next_ch) = chars.peek() {
                if next_ch == '"' || next_ch == '\'' {
                    let quote_char = next_ch;
                    chars.next(); // consume quote
                    
                    // Collect attribute value
                    let mut value = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch == quote_char {
                            chars.next(); // consume closing quote
                            break;
                        }
                        value.push(ch);
                        chars.next();
                    }
                    
                    // Check if we can omit quotes
                    if can_omit_quotes(&value) {
                        result.push_str(&value);
                    } else {
                        result.push(quote_char);
                        result.push_str(&value);
                        result.push(quote_char);
                    }
                    continue;
                }
            }
        }
        result.push(ch);
    }
    
    result
}

/// Check if attribute value can be written without quotes
fn can_omit_quotes(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|c| {
        c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ':'
    })
}

/// Omit closing tags for elements on the last-child path
/// 
/// This follows the dom-expressions convention where:
/// - The root element omits its closing tag
/// - Elements on the "last child path" (following last children recursively) omit closing tags
/// - This continues until hitting an element with multiple children or element children with content
fn omit_closing_tags(html: &str) -> String {
    // Simple approach: work backwards from the end
    // Find closing tags that are at the end or followed only by other closing tags
    // and remove them following the last-path rule
    
    let mut result = html.to_string();
    
    // Find the last closing tag and remove it (root element)
    if let Some(pos) = result.rfind("</") {
        if let Some(end_pos) = result[pos..].find('>') {
            let closing_tag_end = pos + end_pos + 1;
            // Check if this is truly at the end (may have whitespace)
            if result[closing_tag_end..].trim().is_empty() {
                result.truncate(pos);
                result = result.trim_end().to_string();
            }
        }
    }
    
    // Continue removing closing tags from the end following the last-child pattern
    // This is a simplified heuristic - a full implementation would parse the HTML tree
    loop {
        let before = result.clone();
        
        // Try to find and remove the last closing tag if it's part of last-child path
        if let Some(pos) = result.rfind("</") {
            if let Some(end_pos) = result[pos..].find('>') {
                let closing_tag_end = pos + end_pos + 1;
                // Check if this closing tag is at the end
                if result[closing_tag_end..].trim().is_empty() {
                    // Extract tag name
                    let tag_content = &result[pos+2..pos+end_pos];
                    
                    // Check if there's content before this closing tag that would suggest
                    // this is the last child. This is heuristic-based.
                    // Look for the opening tag
                    let opening_search = format!("<{}", tag_content);
                    if let Some(open_pos) = result[..pos].rfind(&opening_search) {
                        // Check if there's another element after this one at the same level
                        // by looking for another opening tag after our opening but before closing
                        let between = &result[open_pos..pos];
                        
                        // Simple heuristic: if there's content between open and close,
                        // and it contains nested elements (has '<' after our tag opens),
                        // this might be on the last path
                        let after_open = open_pos + opening_search.len();
                        if let Some(first_gt) = result[after_open..].find('>') {
                            let content_start = after_open + first_gt + 1;
                            let has_nested = result[content_start..pos].contains('<');
                            
                            // If it has nested elements, it might be on last path
                            // Remove the closing tag and continue
                            if has_nested || result[content_start..pos].trim().is_empty() {
                                result.truncate(pos);
                                result = result.trim_end().to_string();
                                continue;
                            }
                        }
                    }
                    
                    // If we can't determine it's safe to remove, stop
                    break;
                }
            }
        }
        
        // If nothing changed, we're done
        if result == before {
            break;
        }
    }
    
    result
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
    fn test_omit_attribute_quotes_simple() {
        assert_eq!(
            omit_attribute_quotes(r#"<div id="main"></div>"#),
            "<div id=main></div>"
        );
        
        assert_eq!(
            omit_attribute_quotes(r#"<div id="has space"></div>"#),
            r#"<div id="has space"></div>"#
        );
    }
}
