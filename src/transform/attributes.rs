//! Attribute handling transformations
//!
//! This module contains all attribute-related transformation methods:
//! - Style attributes (inline styles, style objects)
//! - Boolean attributes  
//! - Dynamic and static attributes
//! - Special attributes (ref, spread, classList, className, use directives)

use oxc_allocator::Box;
use oxc_allocator::Vec as OxcVec;
use oxc_ast::ast::*;
use oxc_span::SPAN;

use super::DomExpressions;

impl<'a> DomExpressions<'a> {
    pub(super) fn create_set_style_property_call(
        &self,
        element_var: &str,
        property_name: &str,
        value_expr: &Expression<'a>,
    ) -> Option<Statement<'a>> {
        use oxc_allocator::CloneIn;
        use oxc_ast::ast::*;

        // Check if the expression is a call expression (reactive)
        let is_reactive = matches!(value_expr, Expression::CallExpression(_));

        // Create: _$setStyleProperty(element, "property", value)
        let set_style_prop_fn = IdentifierReference {
            span: SPAN,
            name: Atom::from("_$setStyleProperty"),
            reference_id: None.into(),
        };

        let mut set_style_prop_args = OxcVec::new_in(self.allocator);

        // First argument: element reference
        set_style_prop_args.push(Argument::Identifier(Box::new_in(
            IdentifierReference {
                span: SPAN,
                name: Atom::from(self.allocator.alloc_str(element_var)),
                reference_id: None.into(),
            },
            self.allocator,
        )));

        // Second argument: property name as string literal
        set_style_prop_args.push(Argument::StringLiteral(Box::new_in(
            StringLiteral {
                span: SPAN,
                value: Atom::from(self.allocator.alloc_str(property_name)),
                raw: None,
                lone_surrogates: false,
            },
            self.allocator,
        )));

        // Third argument: value expression
        set_style_prop_args.push(Argument::from(value_expr.clone_in(self.allocator)));

        let set_style_prop_call = CallExpression {
            span: SPAN,
            callee: Expression::Identifier(Box::new_in(set_style_prop_fn, self.allocator)),
            arguments: set_style_prop_args,
            optional: false,
            type_arguments: None,
            pure: false,
        };

        if is_reactive {
            // Wrap in _$effect for reactive expressions
            let arrow_body = FunctionBody {
                span: SPAN,
                directives: OxcVec::new_in(self.allocator),
                statements: OxcVec::from_iter_in(
                    [Statement::ExpressionStatement(Box::new_in(
                        ExpressionStatement {
                            span: SPAN,
                            expression: Expression::CallExpression(Box::new_in(
                                set_style_prop_call,
                                self.allocator,
                            )),
                        },
                        self.allocator,
                    ))],
                    self.allocator,
                ),
            };

            let arrow_fn = ArrowFunctionExpression {
                span: SPAN,
                expression: true,
                r#async: false,
                params: Box::new_in(
                    FormalParameters {
                        span: SPAN,
                        kind: FormalParameterKind::ArrowFormalParameters,
                        items: OxcVec::new_in(self.allocator),
                        rest: None,
                    },
                    self.allocator,
                ),
                body: Box::new_in(arrow_body, self.allocator),
                type_parameters: None,
                return_type: None,
                scope_id: None.into(),
                pure: false,
                pife: false,
            };

            let effect_fn = IdentifierReference {
                span: SPAN,
                name: Atom::from("_$effect"),
                reference_id: None.into(),
            };

            let mut effect_args = OxcVec::new_in(self.allocator);
            effect_args.push(Argument::ArrowFunctionExpression(Box::new_in(
                arrow_fn,
                self.allocator,
            )));

            let effect_call = CallExpression {
                span: SPAN,
                callee: Expression::Identifier(Box::new_in(effect_fn, self.allocator)),
                arguments: effect_args,
                optional: false,
                type_arguments: None,
                pure: false,
            };

            Some(Statement::ExpressionStatement(Box::new_in(
                ExpressionStatement {
                    span: SPAN,
                    expression: Expression::CallExpression(Box::new_in(
                        effect_call,
                        self.allocator,
                    )),
                },
                self.allocator,
            )))
        } else {
            // Direct call for non-reactive expressions
            Some(Statement::ExpressionStatement(Box::new_in(
                ExpressionStatement {
                    span: SPAN,
                    expression: Expression::CallExpression(Box::new_in(
                        set_style_prop_call,
                        self.allocator,
                    )),
                },
                self.allocator,
            )))
        }
    }

    /// Create a setBoolAttribute call, optionally wrapped in effect
    pub(super) fn create_set_bool_attribute_call(
        &self,
        element_var: &str,
        attr_name: &str,
        value_expr: &Expression<'a>,
    ) -> Option<Statement<'a>> {
        use oxc_allocator::CloneIn;
        use oxc_ast::ast::*;

        // Check if the expression is a call expression (reactive)
        let is_reactive = matches!(value_expr, Expression::CallExpression(_));

        // Create: _$setBoolAttribute(element, "attr", value)
        let set_bool_attr_fn = IdentifierReference {
            span: SPAN,
            name: Atom::from("_$setBoolAttribute"),
            reference_id: None.into(),
        };

        let mut set_bool_attr_args = OxcVec::new_in(self.allocator);

        // First argument: element reference
        set_bool_attr_args.push(Argument::Identifier(Box::new_in(
            IdentifierReference {
                span: SPAN,
                name: Atom::from(self.allocator.alloc_str(element_var)),
                reference_id: None.into(),
            },
            self.allocator,
        )));

        // Second argument: attribute name as string literal
        set_bool_attr_args.push(Argument::StringLiteral(Box::new_in(
            StringLiteral {
                span: SPAN,
                value: Atom::from(self.allocator.alloc_str(attr_name)),
                raw: None,
                lone_surrogates: false,
            },
            self.allocator,
        )));

        // Third argument: value expression
        set_bool_attr_args.push(Argument::from(value_expr.clone_in(self.allocator)));

        let set_bool_attr_call = CallExpression {
            span: SPAN,
            callee: Expression::Identifier(Box::new_in(set_bool_attr_fn, self.allocator)),
            arguments: set_bool_attr_args,
            optional: false,
            type_arguments: None,
            pure: false,
        };

        if is_reactive {
            // Wrap in _$effect for reactive expressions
            // Wrap in arrow function: () => _$setBoolAttribute(...)
            let arrow_body = FunctionBody {
                span: SPAN,
                directives: OxcVec::new_in(self.allocator),
                statements: OxcVec::from_iter_in(
                    [Statement::ExpressionStatement(Box::new_in(
                        ExpressionStatement {
                            span: SPAN,
                            expression: Expression::CallExpression(Box::new_in(
                                set_bool_attr_call,
                                self.allocator,
                            )),
                        },
                        self.allocator,
                    ))],
                    self.allocator,
                ),
            };

            let arrow_fn = ArrowFunctionExpression {
                span: SPAN,
                expression: true,
                r#async: false,
                params: Box::new_in(
                    FormalParameters {
                        span: SPAN,
                        kind: FormalParameterKind::ArrowFormalParameters,
                        items: OxcVec::new_in(self.allocator),
                        rest: None,
                    },
                    self.allocator,
                ),
                body: Box::new_in(arrow_body, self.allocator),
                type_parameters: None,
                return_type: None,
                scope_id: None.into(),
                pure: false,
                pife: false,
            };

            // Wrap in _$effect call
            let effect_fn = IdentifierReference {
                span: SPAN,
                name: Atom::from("_$effect"),
                reference_id: None.into(),
            };

            let mut effect_args = OxcVec::new_in(self.allocator);
            effect_args.push(Argument::ArrowFunctionExpression(Box::new_in(
                arrow_fn,
                self.allocator,
            )));

            let effect_call = CallExpression {
                span: SPAN,
                callee: Expression::Identifier(Box::new_in(effect_fn, self.allocator)),
                arguments: effect_args,
                optional: false,
                type_arguments: None,
                pure: false,
            };

            Some(Statement::ExpressionStatement(Box::new_in(
                ExpressionStatement {
                    span: SPAN,
                    expression: Expression::CallExpression(Box::new_in(
                        effect_call,
                        self.allocator,
                    )),
                },
                self.allocator,
            )))
        } else {
            // Direct call for non-reactive expressions
            Some(Statement::ExpressionStatement(Box::new_in(
                ExpressionStatement {
                    span: SPAN,
                    expression: Expression::CallExpression(Box::new_in(
                        set_bool_attr_call,
                        self.allocator,
                    )),
                },
                self.allocator,
            )))
        }
    }

    /// Create a setAttribute call wrapped in effect
    pub(super) fn create_set_attribute_call(
        &self,
        element_var: &str,
        attr_name: &str,
        value_expr: &Expression<'a>,
    ) -> Option<Statement<'a>> {
        use oxc_allocator::CloneIn;
        use oxc_ast::ast::*;

        // Create: _$effect(() => _$setAttribute(element, "attr", value))

        // Inner call: _$setAttribute(element, "attr", value)
        let set_attr_fn = IdentifierReference {
            span: SPAN,
            name: Atom::from("_$setAttribute"),
            reference_id: None.into(),
        };

        let mut set_attr_args = OxcVec::new_in(self.allocator);

        // First argument: element reference
        set_attr_args.push(Argument::Identifier(Box::new_in(
            IdentifierReference {
                span: SPAN,
                name: Atom::from(self.allocator.alloc_str(element_var)),
                reference_id: None.into(),
            },
            self.allocator,
        )));

        // Second argument: attribute name as string literal
        set_attr_args.push(Argument::StringLiteral(Box::new_in(
            StringLiteral {
                span: SPAN,
                value: Atom::from(self.allocator.alloc_str(attr_name)),
                raw: None,
                lone_surrogates: false,
            },
            self.allocator,
        )));

        // Third argument: value expression
        set_attr_args.push(Argument::from(value_expr.clone_in(self.allocator)));

        let set_attr_call = CallExpression {
            span: SPAN,
            callee: Expression::Identifier(Box::new_in(set_attr_fn, self.allocator)),
            arguments: set_attr_args,
            optional: false,
            type_arguments: None,
            pure: false,
        };

        // Wrap in arrow function: () => _$setAttribute(...)
        // Use expression form (not block) for concise output
        let arrow_body = FunctionBody {
            span: SPAN,
            directives: OxcVec::new_in(self.allocator),
            statements: OxcVec::from_iter_in(
                [Statement::ExpressionStatement(Box::new_in(
                    ExpressionStatement {
                        span: SPAN,
                        expression: Expression::CallExpression(Box::new_in(
                            set_attr_call,
                            self.allocator,
                        )),
                    },
                    self.allocator,
                ))],
                self.allocator,
            ),
        };

        let arrow_fn = ArrowFunctionExpression {
            span: SPAN,
            expression: true, // Use expression form for concise arrow function
            r#async: false,
            params: Box::new_in(
                FormalParameters {
                    span: SPAN,
                    kind: FormalParameterKind::ArrowFormalParameters,
                    items: OxcVec::new_in(self.allocator),
                    rest: None,
                },
                self.allocator,
            ),
            body: Box::new_in(arrow_body, self.allocator),
            type_parameters: None,
            return_type: None,
            scope_id: None.into(),
            pure: false,
            pife: false,
        };

        // Wrap in _$effect call
        let effect_fn = IdentifierReference {
            span: SPAN,
            name: Atom::from("_$effect"),
            reference_id: None.into(),
        };

        let mut effect_args = OxcVec::new_in(self.allocator);
        effect_args.push(Argument::ArrowFunctionExpression(Box::new_in(
            arrow_fn,
            self.allocator,
        )));

        let effect_call = CallExpression {
            span: SPAN,
            callee: Expression::Identifier(Box::new_in(effect_fn, self.allocator)),
            arguments: effect_args,
            optional: false,
            type_arguments: None,
            pure: false,
        };

        Some(Statement::ExpressionStatement(Box::new_in(
            ExpressionStatement {
                span: SPAN,
                expression: Expression::CallExpression(Box::new_in(effect_call, self.allocator)),
            },
            self.allocator,
        )))
    }

    /// Create a property assignment statement: element.propName = value;
    pub(super) fn create_property_assignment(
        &self,
        element_var: &str,
        prop_name: &str,
        value_expr: &Expression<'a>,
    ) -> Option<Statement<'a>> {
        use oxc_allocator::CloneIn;
        use oxc_ast::ast::*;

        // Create: element.propName = value;
        let element_ref = IdentifierReference {
            span: SPAN,
            name: Atom::from(self.allocator.alloc_str(element_var)),
            reference_id: None.into(),
        };

        let prop_ident = IdentifierName {
            span: SPAN,
            name: Atom::from(self.allocator.alloc_str(prop_name)),
        };

        let member_expr = StaticMemberExpression {
            span: SPAN,
            object: Expression::Identifier(Box::new_in(element_ref, self.allocator)),
            property: prop_ident,
            optional: false,
        };

        let assignment = AssignmentExpression {
            span: SPAN,
            operator: AssignmentOperator::Assign,
            left: AssignmentTarget::from(SimpleAssignmentTarget::from(
                MemberExpression::StaticMemberExpression(Box::new_in(member_expr, self.allocator)),
            )),
            right: value_expr.clone_in(self.allocator),
        };

        Some(Statement::ExpressionStatement(Box::new_in(
            ExpressionStatement {
                span: SPAN,
                expression: Expression::AssignmentExpression(Box::new_in(
                    assignment,
                    self.allocator,
                )),
            },
            self.allocator,
        )))
    }

    /// Create a static setAttribute call (without effect wrapper)
    pub(super) fn create_static_set_attribute_call(
        &self,
        element_var: &str,
        attr_name: &str,
        value_expr: &Expression<'a>,
    ) -> Option<Statement<'a>> {
        use oxc_allocator::CloneIn;
        use oxc_ast::ast::*;

        // Create: _$setAttribute(element, "attr", value)
        let set_attr_fn = IdentifierReference {
            span: SPAN,
            name: Atom::from("_$setAttribute"),
            reference_id: None.into(),
        };

        let mut set_attr_args = OxcVec::new_in(self.allocator);

        // First argument: element reference
        set_attr_args.push(Argument::Identifier(Box::new_in(
            IdentifierReference {
                span: SPAN,
                name: Atom::from(self.allocator.alloc_str(element_var)),
                reference_id: None.into(),
            },
            self.allocator,
        )));

        // Second argument: attribute name as string
        set_attr_args.push(Argument::StringLiteral(Box::new_in(
            StringLiteral {
                span: SPAN,
                value: Atom::from(self.allocator.alloc_str(attr_name)),
                raw: None,
                lone_surrogates: false,
            },
            self.allocator,
        )));

        // Third argument: value expression
        set_attr_args.push(Argument::from(value_expr.clone_in(self.allocator)));

        let set_attr_call = CallExpression {
            span: SPAN,
            callee: Expression::Identifier(Box::new_in(set_attr_fn, self.allocator)),
            arguments: set_attr_args,
            optional: false,
            type_arguments: None,
            pure: false,
        };

        Some(Statement::ExpressionStatement(Box::new_in(
            ExpressionStatement {
                span: SPAN,
                expression: Expression::CallExpression(Box::new_in(set_attr_call, self.allocator)),
            },
            self.allocator,
        )))
    }

    /// Create a delegated event handler: element.$$eventName = handler;
    pub(super) fn create_ref_call(
        &self,
        element_var: &str,
        ref_expr: &Expression<'a>,
    ) -> Option<Statement<'a>> {
        use oxc_allocator::CloneIn;
        use oxc_ast::ast::*;

        // Need to add _$use import
        // This will be handled by add_import call from the slot handler

        // Create: _$use(ref, element)
        let fn_name = IdentifierReference {
            span: SPAN,
            name: Atom::from("_$use"),
            reference_id: None.into(),
        };

        let element_ref = IdentifierReference {
            span: SPAN,
            name: Atom::from(self.allocator.alloc_str(element_var)),
            reference_id: None.into(),
        };

        let mut args = OxcVec::new_in(self.allocator);
        args.push(Argument::from(ref_expr.clone_in(self.allocator)));
        args.push(Argument::Identifier(Box::new_in(
            element_ref,
            self.allocator,
        )));

        let call_expr = CallExpression {
            span: SPAN,
            callee: Expression::Identifier(Box::new_in(fn_name, self.allocator)),
            arguments: args,
            optional: false,
            type_arguments: None,
            pure: false,
        };

        Some(Statement::ExpressionStatement(Box::new_in(
            ExpressionStatement {
                span: SPAN,
                expression: Expression::CallExpression(Box::new_in(call_expr, self.allocator)),
            },
            self.allocator,
        )))
    }

    /// Create a spread call: _$spread(element, props, false, true)
    pub(super) fn create_spread_call(
        &self,
        element_var: &str,
        spread_expr: &Expression<'a>,
    ) -> Option<Statement<'a>> {
        use oxc_allocator::CloneIn;
        use oxc_ast::ast::*;

        // Create: _$spread(element, props, false, true)
        let spread_id = IdentifierReference {
            span: SPAN,
            name: Atom::from("_$spread"),
            reference_id: None.into(),
        };

        let element_ref = IdentifierReference {
            span: SPAN,
            name: Atom::from(self.allocator.alloc_str(element_var)),
            reference_id: None.into(),
        };

        let mut args = OxcVec::new_in(self.allocator);

        // Arg 1: element reference
        args.push(Argument::from(Expression::Identifier(Box::new_in(
            element_ref,
            self.allocator,
        ))));

        // Arg 2: spread expression
        args.push(Argument::from(spread_expr.clone_in(self.allocator)));

        // Arg 3: false (prevProps)
        args.push(Argument::from(Expression::BooleanLiteral(Box::new_in(
            BooleanLiteral {
                span: SPAN,
                value: false,
            },
            self.allocator,
        ))));

        // Arg 4: true (merge)
        args.push(Argument::from(Expression::BooleanLiteral(Box::new_in(
            BooleanLiteral {
                span: SPAN,
                value: true,
            },
            self.allocator,
        ))));

        let call = CallExpression {
            span: SPAN,
            callee: Expression::Identifier(Box::new_in(spread_id, self.allocator)),
            arguments: args,
            optional: false,
            type_arguments: None,
            pure: false,
        };

        Some(Statement::ExpressionStatement(Box::new_in(
            ExpressionStatement {
                span: SPAN,
                expression: Expression::CallExpression(Box::new_in(call, self.allocator)),
            },
            self.allocator,
        )))
    }

    /// Create a classList call: _$classList(element, classListObject)
    pub(super) fn create_class_list_call(
        &self,
        element_var: &str,
        class_list_expr: &Expression<'a>,
    ) -> Option<Statement<'a>> {
        use oxc_allocator::CloneIn;
        use oxc_ast::ast::*;

        // Create: _$classList(element, classListObject)
        let fn_name = IdentifierReference {
            span: SPAN,
            name: Atom::from("_$classList"),
            reference_id: None.into(),
        };

        let element_ref = IdentifierReference {
            span: SPAN,
            name: Atom::from(self.allocator.alloc_str(element_var)),
            reference_id: None.into(),
        };

        let mut args = OxcVec::new_in(self.allocator);
        args.push(Argument::Identifier(Box::new_in(
            element_ref,
            self.allocator,
        )));
        args.push(Argument::from(class_list_expr.clone_in(self.allocator)));

        let call_expr = CallExpression {
            span: SPAN,
            callee: Expression::Identifier(Box::new_in(fn_name, self.allocator)),
            arguments: args,
            optional: false,
            type_arguments: None,
            pure: false,
        };

        Some(Statement::ExpressionStatement(Box::new_in(
            ExpressionStatement {
                span: SPAN,
                expression: Expression::CallExpression(Box::new_in(call_expr, self.allocator)),
            },
            self.allocator,
        )))
    }

    /// Create a style object call: _$style(element, styleObject)
    pub(super) fn create_style_object_call(
        &self,
        element_var: &str,
        style_expr: &Expression<'a>,
    ) -> Option<Statement<'a>> {
        use oxc_allocator::CloneIn;
        use oxc_ast::ast::*;

        // Create: _$style(element, styleObject)
        let fn_name = IdentifierReference {
            span: SPAN,
            name: Atom::from("_$style"),
            reference_id: None.into(),
        };

        let element_ref = IdentifierReference {
            span: SPAN,
            name: Atom::from(self.allocator.alloc_str(element_var)),
            reference_id: None.into(),
        };

        let mut args = OxcVec::new_in(self.allocator);
        args.push(Argument::Identifier(Box::new_in(
            element_ref,
            self.allocator,
        )));
        args.push(Argument::from(style_expr.clone_in(self.allocator)));

        let call_expr = CallExpression {
            span: SPAN,
            callee: Expression::Identifier(Box::new_in(fn_name, self.allocator)),
            arguments: args,
            optional: false,
            type_arguments: None,
            pure: false,
        };

        Some(Statement::ExpressionStatement(Box::new_in(
            ExpressionStatement {
                span: SPAN,
                expression: Expression::CallExpression(Box::new_in(call_expr, self.allocator)),
            },
            self.allocator,
        )))
    }

    /// Create a use directive call: _$use(directive, element, value)
    pub(super) fn create_use_directive_call(
        &self,
        element_var: &str,
        _directive_name: &str,
        directive_expr: &Expression<'a>,
    ) -> Option<Statement<'a>> {
        use oxc_allocator::CloneIn;
        use oxc_ast::ast::*;

        // Create: directive(element, value) or _$use(directive, element, value)
        // For now, just call the directive directly
        let element_ref = IdentifierReference {
            span: SPAN,
            name: Atom::from(self.allocator.alloc_str(element_var)),
            reference_id: None.into(),
        };

        let mut args = OxcVec::new_in(self.allocator);
        args.push(Argument::Identifier(Box::new_in(
            element_ref,
            self.allocator,
        )));

        let call_expr = CallExpression {
            span: SPAN,
            callee: directive_expr.clone_in(self.allocator),
            arguments: args,
            optional: false,
            type_arguments: None,
            pure: false,
        };

        Some(Statement::ExpressionStatement(Box::new_in(
            ExpressionStatement {
                span: SPAN,
                expression: Expression::CallExpression(Box::new_in(call_expr, self.allocator)),
            },
            self.allocator,
        )))
    }

    /// Create a className call: _$className(element, className, value)
    pub(super) fn create_class_name_call(
        &self,
        element_var: &str,
        class_name: &str,
        value_expr: &Expression<'a>,
    ) -> Option<Statement<'a>> {
        use oxc_allocator::CloneIn;
        use oxc_ast::ast::*;

        // Create: _$className(element, "className", value)
        let fn_name = IdentifierReference {
            span: SPAN,
            name: Atom::from("_$className"),
            reference_id: None.into(),
        };

        let element_ref = IdentifierReference {
            span: SPAN,
            name: Atom::from(self.allocator.alloc_str(element_var)),
            reference_id: None.into(),
        };

        let mut args = OxcVec::new_in(self.allocator);
        args.push(Argument::Identifier(Box::new_in(
            element_ref,
            self.allocator,
        )));
        args.push(Argument::StringLiteral(Box::new_in(
            StringLiteral {
                span: SPAN,
                value: Atom::from(self.allocator.alloc_str(class_name)),
                raw: None,
                lone_surrogates: false,
            },
            self.allocator,
        )));
        args.push(Argument::from(value_expr.clone_in(self.allocator)));

        let call_expr = CallExpression {
            span: SPAN,
            callee: Expression::Identifier(Box::new_in(fn_name, self.allocator)),
            arguments: args,
            optional: false,
            type_arguments: None,
            pure: false,
        };

        Some(Statement::ExpressionStatement(Box::new_in(
            ExpressionStatement {
                span: SPAN,
                expression: Expression::CallExpression(Box::new_in(call_expr, self.allocator)),
            },
            self.allocator,
        )))
    }
}
