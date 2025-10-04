//! Babel-specific constants for compatibility
//!
//! This module defines constants used for matching babel-plugin-jsx-dom-expressions
//! behavior. These are primarily for documentation and clarity.

/// Pure comment format used by babel (gets normalized from oxc's format)
pub const BABEL_PURE_COMMENT: &str = "/*#__PURE__*/";

/// Pure comment format produced by oxc (needs normalization)
pub const OXC_PURE_COMMENT: &str = "/* @__PURE__ */";

/// Indentation used by babel output (double spaces, not tabs)
pub const BABEL_INDENT: &str = "  ";

/// Default module name for Solid.js
pub const DEFAULT_MODULE_NAME: &str = "solid-js/web";

/// Alternative module name for r-dom (used in tests)
pub const R_DOM_MODULE_NAME: &str = "r-dom";

/// Template variable prefix (without counter)
pub const TEMPLATE_VAR_PREFIX: &str = "_tmpl$";

/// Element variable prefix (without counter)
pub const ELEMENT_VAR_PREFIX: &str = "_el$";

/// Runtime function prefix
pub const RUNTIME_FN_PREFIX: &str = "_$";

/// Default effect wrapper function name
pub const DEFAULT_EFFECT_WRAPPER: &str = "effect";

/// Default memo wrapper function name
pub const DEFAULT_MEMO_WRAPPER: &str = "memo";

/// Static marker comment decorator
pub const DEFAULT_STATIC_MARKER: &str = "@once";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_comments() {
        assert_eq!(BABEL_PURE_COMMENT, "/*#__PURE__*/");
        assert_eq!(OXC_PURE_COMMENT, "/* @__PURE__ */");
    }

    #[test]
    fn test_var_prefixes() {
        assert_eq!(TEMPLATE_VAR_PREFIX, "_tmpl$");
        assert_eq!(ELEMENT_VAR_PREFIX, "_el$");
        assert_eq!(RUNTIME_FN_PREFIX, "_$");
    }

    #[test]
    fn test_module_names() {
        assert_eq!(DEFAULT_MODULE_NAME, "solid-js/web");
        assert_eq!(R_DOM_MODULE_NAME, "r-dom");
    }
}
