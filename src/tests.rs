//! Integration tests for the DOM expressions transformer

#[cfg(test)]
mod integration_tests {
    use crate::{DomExpressions, DomExpressionsOptions};
    use oxc_allocator::Allocator;

    #[test]
    fn test_basic_options() {
        let options = DomExpressionsOptions::default();
        assert_eq!(options.module_name, "solid-js/web");
        assert_eq!(options.effect_wrapper, "effect");
        assert_eq!(options.memo_wrapper, "memo");
        assert!(options.delegate_events);
        assert!(options.wrap_conditionals);
    }

    #[test]
    fn test_custom_options() {
        let options = DomExpressionsOptions::new("custom-runtime").with_delegate_events(false);

        assert_eq!(options.module_name, "custom-runtime");
        assert!(!options.delegate_events);
    }

    #[test]
    fn test_transformer_creation() {
        let allocator = Allocator::default();
        let options = DomExpressionsOptions::default();
        let transformer = DomExpressions::new(&allocator, options);

        assert_eq!(transformer.options().module_name, "solid-js/web");
    }
}
