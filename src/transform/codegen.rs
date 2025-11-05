//! Code generation for modern format
//!
//! This module contains AST generation helpers for the modern $template/$clone/$bind format

use oxc_allocator::Box;
use oxc_allocator::Vec as OxcVec;
use oxc_ast::ast::*;
use oxc_span::{Atom, SPAN};

use super::DomExpressions;

impl<'a> DomExpressions<'a> {
    /// Create import statement: import { $template, $clone, $bind } from "solid-runtime/polyfill"
    pub(super) fn create_modern_import_statement(&self) -> Statement<'a> {
        let mut specifiers = OxcVec::new_in(self.allocator);

        // Import $template
        specifiers.push(ImportDeclarationSpecifier::ImportSpecifier(Box::new_in(
            ImportSpecifier {
                span: SPAN,
                imported: ModuleExportName::IdentifierName(IdentifierName {
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
                imported: ModuleExportName::IdentifierName(IdentifierName {
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
                imported: ModuleExportName::IdentifierName(IdentifierName {
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
            self.allocator.alloc_str("solid-runtime/polyfill")
        } else {
            self.allocator.alloc_str(&self.options.module_name)
        };

        Statement::ImportDeclaration(Box::new_in(
            ImportDeclaration {
                span: SPAN,
                specifiers: Some(specifiers),
                source: StringLiteral {
                    span: SPAN,
                    value: Atom::from(module_name),
                    raw: None,
                    lone_surrogates: false,
                },
                with_clause: None,
                import_kind: ImportOrExportKind::Value,
                phase: None,
            },
            self.allocator,
        ))
    }

    /// Create template declarations for all collected templates
    pub(super) fn create_template_declarations(&self) -> Vec<Statement<'a>> {
        let mut declarations = Vec::new();
        
        // Clone to avoid borrow issues
        let template_entries: Vec<_> = self.template_map.iter().collect();
        
        for (html, var_name) in template_entries {
            // Create template literal for $template call
            let mut quasis = OxcVec::new_in(self.allocator);
            quasis.push(TemplateElement {
                span: SPAN,
                tail: true,
                value: TemplateElementValue {
                    raw: Atom::from(self.allocator.alloc_str(html)),
                    cooked: Some(Atom::from(self.allocator.alloc_str(html))),
                },
                lone_surrogates: false,
            });

            let template_literal = TemplateLiteral {
                span: SPAN,
                quasis,
                expressions: OxcVec::new_in(self.allocator),
            };

            // Create $template() call
            let mut arguments = OxcVec::new_in(self.allocator);
            arguments.push(Argument::TemplateLiteral(Box::new_in(template_literal, self.allocator)));

            let template_call = Expression::CallExpression(Box::new_in(
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
                    pure: false,
                },
                self.allocator,
            ));

            // Create const declaration
            let var_name_atom = Atom::from(self.allocator.alloc_str(var_name.as_str()));
            
            let declarator = VariableDeclarator {
                span: SPAN,
                kind: VariableDeclarationKind::Const,
                id: BindingPattern {
                    kind: BindingPatternKind::BindingIdentifier(Box::new_in(
                        BindingIdentifier {
                            span: SPAN,
                            name: var_name_atom,
                            symbol_id: Default::default(),
                        },
                        self.allocator,
                    )),
                    type_annotation: None,
                    optional: false,
                },
                init: Some(template_call),
                definite: false,
            };

            let mut declarators = OxcVec::new_in(self.allocator);
            declarators.push(declarator);

            declarations.push(Statement::VariableDeclaration(Box::new_in(
                VariableDeclaration {
                    span: SPAN,
                    kind: VariableDeclarationKind::Const,
                    declarations: declarators,
                    declare: false,
                },
                self.allocator,
            )));
        }
        
        declarations
    }

    /// Create a $clone() call expression
    pub(super) fn create_clone_call(&self, template_var: &'a str) -> Expression<'a> {
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
                pure: false,
            },
            self.allocator,
        ))
    }

    /// Create an IIFE: (() => { ...statements... return _root$; })()
    pub(super) fn create_iife(&self, statements: Vec<Statement<'a>>, root_var: &'a str) -> Expression<'a> {
        // Add return statement
        let mut all_stmts = OxcVec::from_iter_in(statements, self.allocator);
        
        all_stmts.push(Statement::ReturnStatement(Box::new_in(
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
                    directives: OxcVec::new_in(self.allocator),
                    statements: all_stmts,
                },
                self.allocator,
            ),
            scope_id: None.into(),
            pife: false,
            pure: false,
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
                pure: false,
            },
            self.allocator,
        ))
    }
}
