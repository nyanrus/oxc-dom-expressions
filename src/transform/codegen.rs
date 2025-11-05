//! Code generation for modern format
//!
//! This module contains AST generation helpers for the modern output format

use oxc_allocator::Box;
use oxc_allocator::Vec as OxcVec;
use oxc_ast::ast::*;
use oxc_span::{Atom, SPAN};

use crate::template::Template;

use super::DomExpressions;

impl<'a> DomExpressions<'a> {
    /// Create a call expression: $template(`<html>...</html>`)
    pub(super) fn create_template_call(&self, html: &str) -> Expression<'a> {
        // Create template literal
        let quasi = TemplateLiteral {
            span: SPAN,
            quasis: {
                let mut quasis = OxcVec::new_in(self.allocator);
                quasis.push(TemplateElement {
                    span: SPAN,
                    tail: true,
                    value: TemplateElementValue {
                        raw: Atom::from(html),
                        cooked: Some(Atom::from(html)),
                    },
                    lone_surrogates: OxcVec::new_in(self.allocator),
                });
                quasis
            },
            expressions: OxcVec::new_in(self.allocator),
        };

        // Create $template call
        let mut arguments = OxcVec::new_in(self.allocator);
        arguments.push(Argument::TemplateLiteral(Box::new_in(quasi, self.allocator)));

        Expression::CallExpression(Box::new_in(
            CallExpression {
                span: SPAN,
                callee: Expression::Identifier(Box::new_in(
                    IdentifierReference {
                        span: SPAN,
                        name: Atom::from("$template"),
                        reference_id: Default::default(),
                    },
                    self.allocator,
                )),
                arguments,
                optional: false,
                type_arguments: None,
            },
            self.allocator,
        ))
    }

    /// Create a call expression: $clone(tmpl_var)
    pub(super) fn create_clone_call(&self, template_var: &str) -> Expression<'a> {
        let mut arguments = OxcVec::new_in(self.allocator);
        arguments.push(Argument::Identifier(Box::new_in(
            IdentifierReference {
                span: SPAN,
                name: Atom::from(template_var),
                reference_id: Default::default(),
            },
            self.allocator,
        )));

        Expression::CallExpression(Box::new_in(
            CallExpression {
                span: SPAN,
                callee: Expression::Identifier(Box::new_in(
                    IdentifierReference {
                        span: SPAN,
                        name: Atom::from("$clone"),
                        reference_id: Default::default(),
                    },
                    self.allocator,
                )),
                arguments,
                optional: false,
                type_arguments: None,
            },
            self.allocator,
        ))
    }

    /// Create a call expression: $bind(root, [path], { bindings })
    pub(super) fn create_bind_call(
        &self,
        root_var: &str,
        path: Vec<usize>,
        bindings_obj: Expression<'a>,
    ) -> Statement<'a> {
        let mut arguments = OxcVec::new_in(self.allocator);

        // First argument: root variable
        arguments.push(Argument::Identifier(Box::new_in(
            IdentifierReference {
                span: SPAN,
                name: Atom::from(root_var),
                reference_id: Default::default(),
            },
            self.allocator,
        )));

        // Second argument: path array
        let mut path_elements = OxcVec::new_in(self.allocator);
        for index in path {
            path_elements.push(ArrayExpressionElement::NumericLiteral(Box::new_in(
                NumericLiteral {
                    span: SPAN,
                    value: index as f64,
                    raw: &"",
                    base: oxc_syntax::number::NumberBase::Decimal,
                },
                self.allocator,
            )));
        }

        arguments.push(Argument::ArrayExpression(Box::new_in(
            ArrayExpression {
                span: SPAN,
                elements: path_elements,
            },
            self.allocator,
        )));

        // Third argument: bindings object
        arguments.push(Argument::from(bindings_obj));

        // Create the call expression
        let call_expr = Expression::CallExpression(Box::new_in(
            CallExpression {
                span: SPAN,
                callee: Expression::Identifier(Box::new_in(
                    IdentifierReference {
                        span: SPAN,
                        name: Atom::from("$bind"),
                        reference_id: Default::default(),
                    },
                    self.allocator,
                )),
                arguments,
                optional: false,
                type_arguments: None,
            },
            self.allocator,
        ));

        // Wrap in an expression statement
        Statement::ExpressionStatement(Box::new_in(
            ExpressionStatement {
                span: SPAN,
                expression: call_expr,
            },
            self.allocator,
        ))
    }

    /// Create an IIFE: (() => { ...statements... return _root$; })()
    pub(super) fn create_iife(&self, mut statements: OxcVec<'a, Statement<'a>>, root_var: &str) -> Expression<'a> {
        // Add return statement
        statements.push(Statement::ReturnStatement(Box::new_in(
            ReturnStatement {
                span: SPAN,
                argument: Some(Expression::Identifier(Box::new_in(
                    IdentifierReference {
                        span: SPAN,
                        name: Atom::from(root_var),
                        reference_id: Default::default(),
                    },
                    self.allocator,
                ))),
            },
            self.allocator,
        )));

        // Create function body
        let body = Box::new_in(
            FunctionBody {
                span: SPAN,
                directives: OxcVec::new_in(self.allocator),
                statements,
            },
            self.allocator,
        );

        // Create arrow function
        let arrow_fn = ArrowFunctionExpression {
            span: SPAN,
            expression: false,
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
                    directives: body.directives,
                    statements: body.statements,
                },
                self.allocator,
            ),
            scope_id: Default::default(),
            pife: false,
        };

        // Wrap arrow function in parens and call it
        Expression::CallExpression(Box::new_in(
            CallExpression {
                span: SPAN,
                callee: Expression::ParenthesizedExpression(Box::new_in(
                    ParenthesizedExpression {
                        span: SPAN,
                        expression: Expression::ArrowFunctionExpression(Box::new_in(
                            arrow_fn,
                            self.allocator,
                        )),
                    },
                    self.allocator,
                )),
                arguments: OxcVec::new_in(self.allocator),
                optional: false,
                type_arguments: None,
            },
            self.allocator,
        ))
    }

    /// Create import statement: import { $template, $clone, $bind } from "solid-runtime/polyfill"
    pub(super) fn create_modern_import_statement(&self) -> Statement<'a> {
        let mut specifiers = OxcVec::new_in(self.allocator);

        // Import $template
        specifiers.push(ImportDeclarationSpecifier::ImportSpecifier(Box::new_in(
            ImportSpecifier {
                span: SPAN,
                imported: ModuleExportName::Identifier(IdentifierName {
                    span: SPAN,
                    name: Atom::from("$template"),
                }),
                local: BindingIdentifier {
                    span: SPAN,
                    name: Atom::from("$template"),
                    symbol_id: Default::default(),
                },
                import_kind: ImportOrExportKind::Value,
            },
            self.allocator,
        )));

        // Import $clone
        specifiers.push(ImportDeclarationSpecifier::ImportSpecifier(Box::new_in(
            ImportSpecifier {
                span: SPAN,
                imported: ModuleExportName::Identifier(IdentifierName {
                    span: SPAN,
                    name: Atom::from("$clone"),
                }),
                local: BindingIdentifier {
                    span: SPAN,
                    name: Atom::from("$clone"),
                    symbol_id: Default::default(),
                },
                import_kind: ImportOrExportKind::Value,
            },
            self.allocator,
        )));

        // Import $bind
        specifiers.push(ImportDeclarationSpecifier::ImportSpecifier(Box::new_in(
            ImportSpecifier {
                span: SPAN,
                imported: ModuleExportName::Identifier(IdentifierName {
                    span: SPAN,
                    name: Atom::from("$bind"),
                }),
                local: BindingIdentifier {
                    span: SPAN,
                    name: Atom::from("$bind"),
                    symbol_id: Default::default(),
                },
                import_kind: ImportOrExportKind::Value,
            },
            self.allocator,
        )));

        // Determine module name - default to "solid-runtime/polyfill"
        let module_name = if self.options.module_name.contains("/web") {
            "solid-runtime/polyfill"
        } else {
            &self.options.module_name
        };

        Statement::ImportDeclaration(Box::new_in(
            ImportDeclaration {
                span: SPAN,
                specifiers: Some(specifiers),
                source: StringLiteral {
                    span: SPAN,
                    value: Atom::from(module_name),
                },
                with_clause: None,
                import_kind: ImportOrExportKind::Value,
            },
            self.allocator,
        ))
    }
}

