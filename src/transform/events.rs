//! Event handling transformations
//!
//! This module contains all event-related transformation methods:
//! - Delegated event handlers
//! - Direct event listeners (addEventListener)
//! - Capture phase event listeners
//! - Event wrapper functions

use oxc_allocator::Box;
use oxc_allocator::Vec as OxcVec;
use oxc_ast::ast::*;
use oxc_span::SPAN;

use super::DomExpressions;

impl<'a> DomExpressions<'a> {
    /// Create a delegated event handler assignment: element.$$eventName = handler;
    pub(super) fn create_delegated_event_handler(
        &self,
        element_var: &str,
        event_name: &str,
        handler_expr: &Expression<'a>,
    ) -> Option<Statement<'a>> {
        use oxc_allocator::CloneIn;
        use oxc_ast::ast::*;

        // Normalize event name to lowercase for delegation
        let normalized_event = event_name.to_lowercase();

        // Create: element.$$eventName = handler;
        let element_ref = IdentifierReference {
            span: SPAN,
            name: Atom::from(self.allocator.alloc_str(element_var)),
            reference_id: None.into(),
        };

        let prop_name = format!("$${}", normalized_event);
        let prop_ident = IdentifierName {
            span: SPAN,
            name: Atom::from(self.allocator.alloc_str(&prop_name)),
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
            right: handler_expr.clone_in(self.allocator),
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

    /// Create an addEventListener call: element.addEventListener(eventName, handler);
    pub(super) fn create_add_event_listener(
        &self,
        element_var: &str,
        event_name: &str,
        handler_expr: &Expression<'a>,
        _is_capture: bool,
    ) -> Option<Statement<'a>> {
        use oxc_allocator::CloneIn;
        use oxc_ast::ast::*;

        // Create: element.addEventListener("eventName", handler);
        let element_ref = IdentifierReference {
            span: SPAN,
            name: Atom::from(self.allocator.alloc_str(element_var)),
            reference_id: None.into(),
        };

        let member_expr = StaticMemberExpression {
            span: SPAN,
            object: Expression::Identifier(Box::new_in(element_ref, self.allocator)),
            property: IdentifierName {
                span: SPAN,
                name: Atom::from("addEventListener"),
            },
            optional: false,
        };

        let mut args = OxcVec::new_in(self.allocator);

        // First argument: event name as lowercase string
        let lowercase_event = event_name.to_lowercase();
        args.push(Argument::StringLiteral(Box::new_in(
            StringLiteral {
                span: SPAN,
                value: Atom::from(self.allocator.alloc_str(&lowercase_event)),
                raw: None,
                lone_surrogates: false,
            },
            self.allocator,
        )));

        // Second argument: handler expression
        args.push(Argument::from(handler_expr.clone_in(self.allocator)));

        let call = CallExpression {
            span: SPAN,
            callee: Expression::from(MemberExpression::StaticMemberExpression(Box::new_in(
                member_expr,
                self.allocator,
            ))),
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

    /// Create a capture phase event listener: element.addEventListener(eventName, handler, true);
    pub(super) fn create_capture_event_listener(
        &self,
        element_var: &str,
        event_name: &str,
        handler_expr: &Expression<'a>,
    ) -> Option<Statement<'a>> {
        use oxc_allocator::CloneIn;
        use oxc_ast::ast::*;

        // Create: element.addEventListener("eventName", handler, true);
        let element_ref = IdentifierReference {
            span: SPAN,
            name: Atom::from(self.allocator.alloc_str(element_var)),
            reference_id: None.into(),
        };

        let member_expr = StaticMemberExpression {
            span: SPAN,
            object: Expression::Identifier(Box::new_in(element_ref, self.allocator)),
            property: IdentifierName {
                span: SPAN,
                name: Atom::from("addEventListener"),
            },
            optional: false,
        };

        let mut args = OxcVec::new_in(self.allocator);

        // First argument: event name as string
        args.push(Argument::StringLiteral(Box::new_in(
            StringLiteral {
                span: SPAN,
                value: Atom::from(self.allocator.alloc_str(event_name)),
                raw: None,
                lone_surrogates: false,
            },
            self.allocator,
        )));

        // Second argument: handler expression
        args.push(Argument::from(handler_expr.clone_in(self.allocator)));

        // Third argument: true for capture
        args.push(Argument::BooleanLiteral(Box::new_in(
            BooleanLiteral {
                span: SPAN,
                value: true,
            },
            self.allocator,
        )));

        let call = CallExpression {
            span: SPAN,
            callee: Expression::from(MemberExpression::StaticMemberExpression(Box::new_in(
                member_expr,
                self.allocator,
            ))),
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

    /// Create element.$$eventNameData = data;
    #[allow(dead_code)]
    pub(super) fn create_delegated_event_data(
        &self,
        element_var: &str,
        event_name: &str,
        data_expr: &Expression<'a>,
    ) -> Option<Statement<'a>> {
        use oxc_allocator::CloneIn;
        use oxc_ast::ast::*;

        // Normalize event name to lowercase
        let normalized_event = event_name.to_lowercase();

        // Create: element.$$eventNameData = data;
        let element_ref = IdentifierReference {
            span: SPAN,
            name: Atom::from(self.allocator.alloc_str(element_var)),
            reference_id: None.into(),
        };

        let prop_name = format!("$${}Data", normalized_event);
        let prop_ident = IdentifierName {
            span: SPAN,
            name: Atom::from(self.allocator.alloc_str(&prop_name)),
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
            right: data_expr.clone_in(self.allocator),
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

    /// Create a wrapper function for event handlers with data: e => handler(data, e)
    pub(super) fn create_event_wrapper(
        &self,
        handler: &Expression<'a>,
        data: &Expression<'a>,
    ) -> Expression<'a> {
        use oxc_allocator::CloneIn;
        use oxc_ast::ast::*;

        // Create parameter: e
        let event_param = FormalParameter {
            span: SPAN,
            decorators: OxcVec::new_in(self.allocator),
            pattern: BindingPattern {
                kind: BindingPatternKind::BindingIdentifier(Box::new_in(
                    BindingIdentifier {
                        span: SPAN,
                        name: Atom::from("e"),
                        symbol_id: None.into(),
                    },
                    self.allocator,
                )),
                type_annotation: None,
                optional: false,
            },
            accessibility: None,
            readonly: false,
            r#override: false,
        };

        let mut params = OxcVec::new_in(self.allocator);
        params.push(event_param);

        // Create call: handler(data, e)
        let mut call_args = OxcVec::new_in(self.allocator);
        call_args.push(Argument::from(data.clone_in(self.allocator)));
        call_args.push(Argument::from(Expression::Identifier(Box::new_in(
            IdentifierReference {
                span: SPAN,
                name: Atom::from("e"),
                reference_id: None.into(),
            },
            self.allocator,
        ))));

        let call_expr = CallExpression {
            span: SPAN,
            callee: handler.clone_in(self.allocator),
            arguments: call_args,
            optional: false,
            type_arguments: None,
            pure: false,
        };

        // Create arrow function: e => handler(data, e)
        let arrow_fn = ArrowFunctionExpression {
            span: SPAN,
            expression: true, // Expression body, not block
            r#async: false,
            params: Box::new_in(
                FormalParameters {
                    span: SPAN,
                    kind: FormalParameterKind::ArrowFormalParameters,
                    items: params,
                    rest: None,
                },
                self.allocator,
            ),
            body: Box::new_in(
                FunctionBody {
                    span: SPAN,
                    directives: OxcVec::new_in(self.allocator),
                    statements: {
                        let mut stmts = OxcVec::new_in(self.allocator);
                        stmts.push(Statement::ExpressionStatement(Box::new_in(
                            ExpressionStatement {
                                span: SPAN,
                                expression: Expression::CallExpression(Box::new_in(
                                    call_expr,
                                    self.allocator,
                                )),
                            },
                            self.allocator,
                        )));
                        stmts
                    },
                },
                self.allocator,
            ),
            type_parameters: None,
            return_type: None,
            scope_id: None.into(),
            pure: false,
            pife: false,
        };

        Expression::ArrowFunctionExpression(Box::new_in(arrow_fn, self.allocator))
    }

    /// Create _$addEventListener helper call
    pub(super) fn create_add_event_listener_helper(
        &self,
        element_var: &str,
        event_name: &str,
        handler_expr: &Expression<'a>,
        is_delegated: bool,
        lowercase_event: bool, // Whether to lowercase the event name
    ) -> Option<Statement<'a>> {
        use oxc_allocator::CloneIn;
        use oxc_ast::ast::*;

        // Create: _$addEventListener(element, "eventName", handler, true_if_delegated);
        let helper_fn = IdentifierReference {
            span: SPAN,
            name: Atom::from("_$addEventListener"),
            reference_id: None.into(),
        };

        let mut args = OxcVec::new_in(self.allocator);

        // First argument: element reference
        args.push(Argument::Identifier(Box::new_in(
            IdentifierReference {
                span: SPAN,
                name: Atom::from(self.allocator.alloc_str(element_var)),
                reference_id: None.into(),
            },
            self.allocator,
        )));

        // Second argument: event name as string (lowercase if needed)
        let event_str = if lowercase_event {
            event_name.to_lowercase()
        } else {
            event_name.to_string()
        };
        args.push(Argument::StringLiteral(Box::new_in(
            StringLiteral {
                span: SPAN,
                value: Atom::from(self.allocator.alloc_str(&event_str)),
                raw: None,
                lone_surrogates: false,
            },
            self.allocator,
        )));

        // Third argument: handler expression
        args.push(Argument::from(handler_expr.clone_in(self.allocator)));

        // Fourth argument: true if delegated (for backwards compat)
        if is_delegated {
            args.push(Argument::BooleanLiteral(Box::new_in(
                BooleanLiteral {
                    span: SPAN,
                    value: true,
                },
                self.allocator,
            )));
        }

        let call = CallExpression {
            span: SPAN,
            callee: Expression::Identifier(Box::new_in(helper_fn, self.allocator)),
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

    /// Create wrapped event handler: element.addEventListener(event, e => handler(data, e))
    #[allow(dead_code)]
    pub(super) fn create_wrapped_event_handler(
        &self,
        element_var: &str,
        event_name: &str,
        handler_expr: &Expression<'a>,
        data_expr: &Expression<'a>,
    ) -> Option<Statement<'a>> {
        use oxc_allocator::CloneIn;
        use oxc_ast::ast::*;

        // Create the wrapper function: e => handler(data, e)
        // First create the call: handler(data, e)
        let mut call_args = OxcVec::new_in(self.allocator);
        call_args.push(Argument::from(data_expr.clone_in(self.allocator)));
        call_args.push(Argument::Identifier(Box::new_in(
            IdentifierReference {
                span: SPAN,
                name: Atom::from("e"),
                reference_id: None.into(),
            },
            self.allocator,
        )));

        let handler_call = CallExpression {
            span: SPAN,
            callee: handler_expr.clone_in(self.allocator),
            arguments: call_args,
            optional: false,
            type_arguments: None,
            pure: false,
        };

        // Wrap in arrow function: e => handler(data, e)
        let mut params_items = OxcVec::new_in(self.allocator);
        params_items.push(FormalParameter {
            span: SPAN,
            decorators: OxcVec::new_in(self.allocator),
            pattern: BindingPattern {
                kind: BindingPatternKind::BindingIdentifier(Box::new_in(
                    BindingIdentifier {
                        span: SPAN,
                        name: Atom::from("e"),
                        symbol_id: None.into(),
                    },
                    self.allocator,
                )),
                type_annotation: None,
                optional: false,
            },
            accessibility: None,
            readonly: false,
            r#override: false,
        });

        let arrow_fn = ArrowFunctionExpression {
            span: SPAN,
            expression: true,
            r#async: false,
            type_parameters: None,
            params: Box::new_in(
                FormalParameters {
                    span: SPAN,
                    kind: FormalParameterKind::ArrowFormalParameters,
                    items: params_items,
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
                                expression: Expression::CallExpression(Box::new_in(
                                    handler_call,
                                    self.allocator,
                                )),
                            },
                            self.allocator,
                        ))],
                        self.allocator,
                    ),
                },
                self.allocator,
            ),
            scope_id: Default::default(),
            pure: false,
            pife: false,
        };

        // Now create the addEventListener call
        self.create_add_event_listener(
            element_var,
            event_name,
            &Expression::ArrowFunctionExpression(Box::new_in(arrow_fn, self.allocator)),
            false,
        )
    }
}
