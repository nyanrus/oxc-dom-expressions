//! Binding generation for modern format
//!
//! This module handles generating the binding objects passed to $bind()

use oxc_allocator::Box;
use oxc_allocator::Vec as OxcVec;
use oxc_ast::ast::*;
use oxc_span::{Atom, SPAN};

use super::DomExpressions;

impl<'a> DomExpressions<'a> {
    /// Create an empty bindings object
    pub(super) fn create_empty_bindings_object(&self) -> ObjectExpression<'a> {
        ObjectExpression {
            span: SPAN,
            properties: OxcVec::new_in(self.allocator),
        }
    }

    /// Add a property to a bindings object
    /// For example: { classList: { selected: () => unknown } }
    pub(super) fn add_binding_property(
        &self,
        obj: &mut ObjectExpression<'a>,
        key: &str,
        value: Expression<'a>,
    ) {
        obj.properties.push(ObjectPropertyKind::ObjectProperty(Box::new_in(
            ObjectProperty {
                span: SPAN,
                kind: PropertyKind::Init,
                key: PropertyKey::StaticIdentifier(Box::new_in(
                    IdentifierName {
                        span: SPAN,
                        name: Atom::from(key),
                    },
                    self.allocator,
                )),
                value,
                method: false,
                shorthand: false,
                computed: false,
            },
            self.allocator,
        )));
    }

    /// Wrap an expression in an arrow function: () => expr
    pub(super) fn wrap_in_arrow(&self, expr: Expression<'a>) -> Expression<'a> {
        Expression::ArrowFunctionExpression(Box::new_in(
            ArrowFunctionExpression {
                span: SPAN,
                expression: true,
                r#async: false,
                type_parameters: None,
                params: Box::new_in(
                    FormalParameters {
                        span: SPAN,
                        kind: FormalParameterKind::ArrowFormalParameters,
                        items: OxcVec::new_in(self.allocator),
                        rest: None,
                    },
                    self.allocator,
                ),
                return_type: None,
                body: Box::new_in(
                    FunctionBody {
                        span: SPAN,
                        directives: OxcVec::new_in(self.allocator),
                        statements: OxcVec::from_iter_in(
                            [Statement::ExpressionStatement(Box::new_in(
                                ExpressionStatement {
                                    span: SPAN,
                                    expression: expr,
                                },
                                self.allocator,
                            ))],
                            self.allocator,
                        ),
                    },
                    self.allocator,
                ),
                scope_id: Default::default(),
                pife: false,
            },
            self.allocator,
        ))
    }
}

