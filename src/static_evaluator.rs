//! Static Expression Evaluator
//!
//! This module provides compile-time evaluation of JSX expressions to match
//! the behavior of babel-plugin-jsx-dom-expressions's `.evaluate().confident` feature.
//!
//! It supports evaluating:
//! - Literal values (boolean, string, number, null, undefined)
//! - Simple object expressions with literal properties
//! - Template literals with no expressions
//! - Unary expressions (!, -, +)
//! - Binary expressions (+, -, *, /, etc.) with literal operands

use oxc_ast::ast::*;
use std::collections::HashMap;

/// The result of evaluating an expression
#[derive(Debug, Clone, PartialEq)]
pub enum EvaluatedValue {
    Boolean(bool),
    String(String),
    Number(f64),
    Null,
    Undefined,
    Object(HashMap<String, EvaluatedValue>),
}

impl EvaluatedValue {
    /// Convert to a string representation
    pub fn to_string(&self) -> String {
        match self {
            EvaluatedValue::Boolean(b) => b.to_string(),
            EvaluatedValue::String(s) => s.clone(),
            EvaluatedValue::Number(n) => {
                if n.fract() == 0.0 && n.is_finite() {
                    format!("{}", *n as i64)
                } else {
                    n.to_string()
                }
            }
            EvaluatedValue::Null => "null".to_string(),
            EvaluatedValue::Undefined => "undefined".to_string(),
            EvaluatedValue::Object(_) => "[object Object]".to_string(),
        }
    }

    /// Check if this value is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            EvaluatedValue::Boolean(b) => *b,
            EvaluatedValue::String(s) => !s.is_empty(),
            EvaluatedValue::Number(n) => *n != 0.0 && !n.is_nan(),
            EvaluatedValue::Null | EvaluatedValue::Undefined => false,
            EvaluatedValue::Object(_) => true,
        }
    }
}

/// Result of attempting to evaluate an expression
pub struct EvaluationResult {
    /// Whether the evaluation was confident (successful)
    pub confident: bool,
    /// The evaluated value, if confident
    pub value: Option<EvaluatedValue>,
}

/// Evaluate a JSX expression at compile time
///
/// This mimics Babel's `path.evaluate()` behavior, returning a confident result
/// when the expression can be statically determined.
pub fn evaluate_expression(expr: &Expression) -> EvaluationResult {
    match expr {
        // Literal values
        Expression::BooleanLiteral(lit) => EvaluationResult {
            confident: true,
            value: Some(EvaluatedValue::Boolean(lit.value)),
        },
        Expression::StringLiteral(lit) => EvaluationResult {
            confident: true,
            value: Some(EvaluatedValue::String(lit.value.to_string())),
        },
        Expression::NumericLiteral(lit) => EvaluationResult {
            confident: true,
            value: Some(EvaluatedValue::Number(lit.value)),
        },
        Expression::NullLiteral(_) => EvaluationResult {
            confident: true,
            value: Some(EvaluatedValue::Null),
        },

        // Identifier "undefined"
        Expression::Identifier(ident) if ident.name == "undefined" => EvaluationResult {
            confident: true,
            value: Some(EvaluatedValue::Undefined),
        },

        // Template literals without expressions
        Expression::TemplateLiteral(tmpl) if tmpl.expressions.is_empty() => {
            let string_value = tmpl
                .quasis
                .iter()
                .map(|q| q.value.raw.as_str())
                .collect::<String>();
            EvaluationResult {
                confident: true,
                value: Some(EvaluatedValue::String(string_value)),
            }
        }

        // Unary expressions
        Expression::UnaryExpression(unary) => evaluate_unary_expression(unary),

        // Binary expressions
        Expression::BinaryExpression(binary) => evaluate_binary_expression(binary),

        // Object expressions with all literal properties
        Expression::ObjectExpression(obj) => evaluate_object_expression(obj),

        // Everything else is not confidently evaluatable
        _ => EvaluationResult {
            confident: false,
            value: None,
        },
    }
}

fn evaluate_unary_expression(unary: &UnaryExpression) -> EvaluationResult {
    let argument_result = evaluate_expression(&unary.argument);
    if !argument_result.confident {
        return EvaluationResult {
            confident: false,
            value: None,
        };
    }

    let arg_value = argument_result.value.unwrap();

    match unary.operator {
        UnaryOperator::LogicalNot => EvaluationResult {
            confident: true,
            value: Some(EvaluatedValue::Boolean(!arg_value.is_truthy())),
        },
        UnaryOperator::UnaryNegation => {
            if let EvaluatedValue::Number(n) = arg_value {
                EvaluationResult {
                    confident: true,
                    value: Some(EvaluatedValue::Number(-n)),
                }
            } else {
                EvaluationResult {
                    confident: false,
                    value: None,
                }
            }
        }
        UnaryOperator::UnaryPlus => {
            if let EvaluatedValue::Number(n) = arg_value {
                EvaluationResult {
                    confident: true,
                    value: Some(EvaluatedValue::Number(n)),
                }
            } else {
                EvaluationResult {
                    confident: false,
                    value: None,
                }
            }
        }
        _ => EvaluationResult {
            confident: false,
            value: None,
        },
    }
}

fn evaluate_binary_expression(binary: &BinaryExpression) -> EvaluationResult {
    let left_result = evaluate_expression(&binary.left);
    let right_result = evaluate_expression(&binary.right);

    if !left_result.confident || !right_result.confident {
        return EvaluationResult {
            confident: false,
            value: None,
        };
    }

    let left_value = left_result.value.unwrap();
    let right_value = right_result.value.unwrap();

    match binary.operator {
        BinaryOperator::Addition => {
            // String concatenation or numeric addition
            match (&left_value, &right_value) {
                (EvaluatedValue::String(l), _) => EvaluationResult {
                    confident: true,
                    value: Some(EvaluatedValue::String(format!("{}{}", l, right_value.to_string()))),
                },
                (_, EvaluatedValue::String(r)) => EvaluationResult {
                    confident: true,
                    value: Some(EvaluatedValue::String(format!("{}{}", left_value.to_string(), r))),
                },
                (EvaluatedValue::Number(l), EvaluatedValue::Number(r)) => EvaluationResult {
                    confident: true,
                    value: Some(EvaluatedValue::Number(l + r)),
                },
                _ => EvaluationResult {
                    confident: false,
                    value: None,
                },
            }
        }
        BinaryOperator::Subtraction => {
            if let (EvaluatedValue::Number(l), EvaluatedValue::Number(r)) = (&left_value, &right_value) {
                EvaluationResult {
                    confident: true,
                    value: Some(EvaluatedValue::Number(l - r)),
                }
            } else {
                EvaluationResult {
                    confident: false,
                    value: None,
                }
            }
        }
        BinaryOperator::Multiplication => {
            if let (EvaluatedValue::Number(l), EvaluatedValue::Number(r)) = (&left_value, &right_value) {
                EvaluationResult {
                    confident: true,
                    value: Some(EvaluatedValue::Number(l * r)),
                }
            } else {
                EvaluationResult {
                    confident: false,
                    value: None,
                }
            }
        }
        BinaryOperator::Division => {
            if let (EvaluatedValue::Number(l), EvaluatedValue::Number(r)) = (&left_value, &right_value) {
                EvaluationResult {
                    confident: true,
                    value: Some(EvaluatedValue::Number(l / r)),
                }
            } else {
                EvaluationResult {
                    confident: false,
                    value: None,
                }
            }
        }
        _ => EvaluationResult {
            confident: false,
            value: None,
        },
    }
}

fn evaluate_object_expression(obj: &ObjectExpression) -> EvaluationResult {
    let mut map = HashMap::new();

    for prop in &obj.properties {
        match prop {
            ObjectPropertyKind::ObjectProperty(object_prop) => {
                // Only handle non-computed, non-spread properties
                if object_prop.computed || object_prop.shorthand {
                    return EvaluationResult {
                        confident: false,
                        value: None,
                    };
                }

                // Get the property key
                let key = match &object_prop.key {
                    PropertyKey::StaticIdentifier(ident) => ident.name.to_string(),
                    PropertyKey::StringLiteral(lit) => lit.value.to_string(),
                    PropertyKey::NumericLiteral(lit) => lit.value.to_string(),
                    _ => {
                        return EvaluationResult {
                            confident: false,
                            value: None,
                        }
                    }
                };

                // Evaluate the value
                let value_result = evaluate_expression(&object_prop.value);
                if !value_result.confident {
                    return EvaluationResult {
                        confident: false,
                        value: None,
                    };
                }

                map.insert(key, value_result.value.unwrap());
            }
            _ => {
                // Spread elements make object non-evaluatable
                return EvaluationResult {
                    confident: false,
                    value: None,
                };
            }
        }
    }

    EvaluationResult {
        confident: true,
        value: Some(EvaluatedValue::Object(map)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;

    fn test_evaluate(code: &str, expected_value: Option<EvaluatedValue>) {
        let allocator = Allocator::default();
        let source_type = SourceType::jsx();
        let ret = Parser::new(&allocator, code, source_type).parse_expression();
        let expr = ret.expect("Failed to parse expression");
        let result = evaluate_expression(&expr);
        assert_eq!(result.value, expected_value);
        if expected_value.is_some() {
            assert!(result.confident);
        }
    }

    #[test]
    fn test_evaluate_boolean() {
        test_evaluate("true", Some(EvaluatedValue::Boolean(true)));
        test_evaluate("false", Some(EvaluatedValue::Boolean(false)));
    }

    #[test]
    fn test_evaluate_string() {
        test_evaluate("\"hello\"", Some(EvaluatedValue::String("hello".to_string())));
    }

    #[test]
    fn test_evaluate_number() {
        test_evaluate("42", Some(EvaluatedValue::Number(42.0)));
    }

    #[test]
    fn test_evaluate_null() {
        test_evaluate("null", Some(EvaluatedValue::Null));
    }

    #[test]
    fn test_evaluate_undefined() {
        test_evaluate("undefined", Some(EvaluatedValue::Undefined));
    }

    #[test]
    fn test_evaluate_not_operator() {
        test_evaluate("!true", Some(EvaluatedValue::Boolean(false)));
        test_evaluate("!false", Some(EvaluatedValue::Boolean(true)));
    }

    #[test]
    fn test_evaluate_addition() {
        test_evaluate("1 + 1", Some(EvaluatedValue::Number(2.0)));
    }

    #[test]
    fn test_evaluate_string_concat() {
        test_evaluate(
            "\"hello\" + \" world\"",
            Some(EvaluatedValue::String("hello world".to_string())),
        );
    }

    #[test]
    fn test_evaluate_object() {
        let allocator = Allocator::default();
        let source_type = SourceType::jsx();
        let code = "{ color: \"red\", size: 12 }";
        let ret = Parser::new(&allocator, code, source_type).parse_expression();
        let expr = ret.expect("Failed to parse expression");
        let result = evaluate_expression(&expr);
        assert!(result.confident);
        if let Some(EvaluatedValue::Object(map)) = result.value {
            assert_eq!(
                map.get("color"),
                Some(&EvaluatedValue::String("red".to_string()))
            );
            assert_eq!(map.get("size"), Some(&EvaluatedValue::Number(12.0)));
        } else {
            panic!("Expected object value");
        }
    }

    #[test]
    fn test_non_evaluatable_identifier() {
        let allocator = Allocator::default();
        let source_type = SourceType::jsx();
        let code = "someVariable";
        let ret = Parser::new(&allocator, code, source_type).parse_expression();
        let expr = ret.expect("Failed to parse expression");
        let result = evaluate_expression(&expr);
        assert!(!result.confident);
    }

    #[test]
    fn test_non_evaluatable_call() {
        let allocator = Allocator::default();
        let source_type = SourceType::jsx();
        let code = "someFunction()";
        let ret = Parser::new(&allocator, code, source_type).parse_expression();
        let expr = ret.expect("Failed to parse expression");
        let result = evaluate_expression(&expr);
        assert!(!result.confident);
    }
}
