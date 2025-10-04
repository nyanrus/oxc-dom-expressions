//! Import ordering for babel-plugin-jsx-dom-expressions compatibility
//!
//! This module defines the import priority order to match the exact
//! output format of babel-plugin-jsx-dom-expressions. This is purely
//! for compatibility with fixture tests and doesn't affect functionality.

/// Get the priority order for an import name to match babel plugin output
///
/// Lower numbers = higher priority (imported first).
/// This ordering is based on fixture test expectations from the original
/// babel-plugin-jsx-dom-expressions.
///
/// # Arguments
///
/// * `name` - The import name to get priority for
///
/// # Returns
///
/// Priority value (0 = highest priority)
pub fn get_import_priority(name: &str) -> usize {
    match name {
        "template" => 0,
        "ssr" => 0, // SSR import has same priority as template
        "delegateEvents" => 1,
        "createComponent" => 2,
        "memo" => 3,
        "addEventListener" => 4,
        "insert" => 5,
        "setAttribute" => 6,
        "setBoolAttribute" => 7,
        "className" => 8,
        "style" => 9,
        "setStyleProperty" => 10,
        "effect" => 11,
        "classList" => 12,
        "use" => 13,
        "spread" => 14,
        "mergeProps" => 15,
        "For" => 20,
        "Show" => 21,
        "Suspense" => 22,
        "SuspenseList" => 23,
        "Switch" => 24,
        "Match" => 25,
        "Index" => 26,
        "ErrorBoundary" => 27,
        "setAttributeNS" => 28,
        "getOwner" => 29,
        _ => 1000, // Unknown imports go last
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_priority_order() {
        // Template should have highest priority (lowest number)
        assert_eq!(get_import_priority("template"), 0);
        assert_eq!(get_import_priority("ssr"), 0);
        
        // Common runtime functions should be in the middle
        assert_eq!(get_import_priority("delegateEvents"), 1);
        assert_eq!(get_import_priority("createComponent"), 2);
        
        // Unknown imports should have lowest priority
        assert!(get_import_priority("unknown") > 100);
    }

    #[test]
    fn test_template_vs_insert_priority() {
        // Template should come before insert
        assert!(get_import_priority("template") < get_import_priority("insert"));
    }
}
