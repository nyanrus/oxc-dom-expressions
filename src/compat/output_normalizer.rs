//! Output normalization for babel plugin compatibility
//!
//! This module provides utilities to normalize the generated code output
//! to match the exact format produced by babel-plugin-jsx-dom-expressions.
//! This is necessary for fixture test compatibility.

/// Normalizer for babel plugin output compatibility
pub struct BabelOutputNormalizer;

impl BabelOutputNormalizer {
    /// Normalize generated code to match babel plugin output format
    ///
    /// This performs several transformations:
    /// - Converts `/* @__PURE__ */` to `/*#__PURE__*/` (babel format)
    /// - Replaces tabs with double spaces (babel formatting)
    /// - Formats template variable declarations with proper line breaks
    ///
    /// # Arguments
    ///
    /// * `code` - The generated JavaScript code to normalize
    ///
    /// # Returns
    ///
    /// Normalized code string matching babel plugin format
    pub fn normalize(code: &str) -> String {
        let mut result = code.to_string();

        // Replace /* @__PURE__ */ with /*#__PURE__*/
        result = result.replace("/* @__PURE__ */", "/*#__PURE__*/");

        // Replace tabs with double spaces to match babel output
        result = result.replace('\t', "  ");

        // Format multi-line variable declarations
        // Replace all instances of ", _tmpl$" with ",\n  _tmpl$" in the entire code
        result = result.replace(", _tmpl$", ",\n  _tmpl$");

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_comment_normalization() {
        let input = "const x = /* @__PURE__ */ fn();";
        let expected = "const x = /*#__PURE__*/ fn();";
        assert_eq!(BabelOutputNormalizer::normalize(input), expected);
    }

    #[test]
    fn test_tab_normalization() {
        let input = "function test() {\n\treturn true;\n}";
        let expected = "function test() {\n  return true;\n}";
        assert_eq!(BabelOutputNormalizer::normalize(input), expected);
    }

    #[test]
    fn test_template_variable_formatting() {
        let input = "const _tmpl$1 = x(), _tmpl$2 = y();";
        let expected = "const _tmpl$1 = x(),\n  _tmpl$2 = y();";
        assert_eq!(BabelOutputNormalizer::normalize(input), expected);
    }
}
