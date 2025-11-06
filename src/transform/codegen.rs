//! Code generation for modern format
//!
//! This module contains AST generation helpers for the modern $template/$clone/$bind format

use oxc_allocator::Box;
use oxc_allocator::Vec as OxcVec;
use oxc_ast::ast::*;
use oxc_span::{Atom, SPAN};

use super::DomExpressions;

impl<'a> DomExpressions<'a> {
    /// Helper: Create an identifier reference
    fn ident(&self, name: &'a str) -> IdentifierReference<'a> {
        IdentifierReference {
            span: SPAN,
            name: Atom::from(name),
            reference_id: Default::default(),
        }
    }

    /// Helper: Create a binding identifier
    fn binding_ident(&self, name: &'a str) -> BindingIdentifier<'a> {
        BindingIdentifier {
            span: SPAN,
            name: Atom::from(name),
            symbol_id: Default::default(),
        }
    }

    /// Helper: Create a call expression
    fn call_expr(&self, callee_name: &'a str, args: OxcVec<'a, Argument<'a>>) -> Expression<'a> {
        Expression::CallExpression(Box::new_in(
            CallExpression {
                span: SPAN,
                callee: Expression::Identifier(Box::new_in(self.ident(callee_name), self.allocator)),
                arguments: args,
                optional: false,
                type_arguments: None,
                pure: false,
            },
            self.allocator,
        ))
    }

    /// Helper: Create a const declaration
    fn const_decl(&self, name: &'a str, init: Expression<'a>) -> Statement<'a> {
        let declarator = VariableDeclarator {
            span: SPAN,
            kind: VariableDeclarationKind::Const,
            id: BindingPattern {
                kind: BindingPatternKind::BindingIdentifier(Box::new_in(
                    self.binding_ident(name),
                    self.allocator,
                )),
                type_annotation: None,
                optional: false,
            },
            init: Some(init),
            definite: false,
        };

        let mut declarators = OxcVec::new_in(self.allocator);
        declarators.push(declarator);

        Statement::VariableDeclaration(Box::new_in(
            VariableDeclaration {
                span: SPAN,
                kind: VariableDeclarationKind::Const,
                declarations: declarators,
                declare: false,
            },
            self.allocator,
        ))
    }

    /// Create template declarations for all collected templates
    pub(super) fn create_template_declarations(&self) -> Vec<Statement<'a>> {
        self.template_map
            .iter()
            .map(|(html, var_name)| {
                // Create template literal
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

                // Create $template(html) call
                let mut args = OxcVec::new_in(self.allocator);
                args.push(Argument::TemplateLiteral(Box::new_in(
                    TemplateLiteral {
                        span: SPAN,
                        quasis,
                        expressions: OxcVec::new_in(self.allocator),
                    },
                    self.allocator,
                )));

                let template_call = self.call_expr(self.allocator.alloc_str("$template"), args);
                self.const_decl(self.allocator.alloc_str(var_name.as_str()), template_call)
            })
            .collect()
    }

    /// Create a $clone() call expression
    pub(super) fn create_clone_call(&self, template_var: &'a str) -> Expression<'a> {
        let mut args = OxcVec::new_in(self.allocator);
        args.push(Argument::Identifier(Box::new_in(self.ident(template_var), self.allocator)));
        self.call_expr(self.allocator.alloc_str("$clone"), args)
    }

    /// Create an IIFE: (() => { ...statements... return _root$; })()
    pub(super) fn create_iife(&self, statements: Vec<Statement<'a>>, root_var: &'a str) -> Expression<'a> {
        let mut all_stmts = OxcVec::from_iter_in(statements, self.allocator);
        
        // Add return statement
        all_stmts.push(Statement::ReturnStatement(Box::new_in(
            ReturnStatement {
                span: SPAN,
                argument: Some(Expression::Identifier(Box::new_in(
                    self.ident(root_var),
                    self.allocator,
                ))),
            },
            self.allocator,
        )));

        // Create arrow function with statements
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

        // Wrap arrow function in parens and call it: (() => {...})()
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

    /// Create helper function statements by parsing the JavaScript helper code
    /// Returns the parsed statements that define $template, $clone, and $bind
    pub(super) fn create_helper_statements(&self) -> Vec<Statement<'a>> {
        use super::helper::get_runtime_helper;
        use oxc_parser::Parser;
        use oxc_span::SourceType;
        
        // Get the helper code
        let helper_code_owned = get_runtime_helper(&self.options.module_name);
        
        // Allocate the helper code in the allocator so it lives as long as 'a
        let helper_code = self.allocator.alloc_str(&helper_code_owned);
        
        // Parse the helper code
        let source_type = SourceType::default().with_module(true);
        let ret = Parser::new(self.allocator, helper_code, source_type).parse();
        
        if ret.errors.is_empty() {
            // Extract the statements from the parsed program
            ret.program.body.into_iter().collect()
        } else {
            // If parsing fails, return empty vec (shouldn't happen with our known-good helper code)
            Vec::new()
        }
    }
}
