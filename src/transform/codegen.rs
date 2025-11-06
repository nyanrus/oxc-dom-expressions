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
    /// Uses _$template directly from the runtime
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

                // Create _$template(html) call - use runtime function directly
                let mut args = OxcVec::new_in(self.allocator);
                args.push(Argument::TemplateLiteral(Box::new_in(
                    TemplateLiteral {
                        span: SPAN,
                        quasis,
                        expressions: OxcVec::new_in(self.allocator),
                    },
                    self.allocator,
                )));

                let template_call = self.call_expr(self.allocator.alloc_str("_$template"), args);
                self.const_decl(self.allocator.alloc_str(var_name.as_str()), template_call)
            })
            .collect()
    }

    /// Create a template clone call: tmpl()
    /// The template function returns a cloneable element
    pub(super) fn create_clone_call(&self, template_var: &'a str) -> Expression<'a> {
        // Just call the template: _tmpl$()
        let args = OxcVec::new_in(self.allocator);
        self.call_expr(template_var, args)
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
    /// Returns just the import statement - we use runtime functions directly
    pub(super) fn create_helper_statements(&self) -> Vec<Statement<'a>> {
        use super::helper::get_runtime_imports;
        use oxc_parser::Parser;
        use oxc_span::SourceType;
        
        // Get the import statement with needed functions
        let imports: Vec<&str> = self.imports_needed.iter().map(|s| s.as_str()).collect();
        let imports_code_owned = get_runtime_imports(&self.options.module_name, &imports);
        
        // Allocate the code in the allocator so it lives as long as 'a
        let imports_code = self.allocator.alloc_str(&imports_code_owned);
        
        // Parse the imports
        let source_type = SourceType::default().with_module(true);
        let ret = Parser::new(self.allocator, imports_code, source_type).parse();
        
        if ret.errors.is_empty() {
            // Extract the statements from the parsed program
            ret.program.body.into_iter().collect()
        } else {
            // This should never happen since we control the import code
            eprintln!("ERROR: Failed to parse imports. This is a bug in oxc-dom-expressions.");
            for error in &ret.errors {
                eprintln!("  Parse error: {}", error);
            }
            Vec::new()
        }
    }

    /// Create a member expression like `_el$.firstChild` or `_el$.nextSibling`
    pub(super) fn create_member_expr(&self, object: Expression<'a>, property: &'a str) -> Expression<'a> {
        Expression::StaticMemberExpression(Box::new_in(
            StaticMemberExpression {
                span: SPAN,
                object,
                property: IdentifierName {
                    span: SPAN,
                    name: Atom::from(property),
                },
                optional: false,
            },
            self.allocator,
        ))
    }

    /// Navigate to element using path like ["firstChild", "nextSibling"]
    pub(super) fn navigate_to_element(&self, base_var: &'a str, path: &[String]) -> Expression<'a> {
        let mut expr = Expression::Identifier(Box::new_in(self.ident(base_var), self.allocator));
        
        for step in path {
            expr = self.create_member_expr(expr, self.allocator.alloc_str(step));
        }
        
        expr
    }

    /// Create an expression statement: _$insert(_el$, value, marker)
    pub(super) fn create_insert_call(&self, element_expr: Expression<'a>, value_expr: Expression<'a>, marker_expr: Option<Expression<'a>>) -> Statement<'a> {
        let mut args = OxcVec::new_in(self.allocator);
        args.push(Argument::from(element_expr));
        args.push(Argument::from(value_expr));
        
        // Add marker argument (usually null for simple cases)
        if let Some(marker) = marker_expr {
            args.push(Argument::from(marker));
        } else {
            args.push(Argument::NullLiteral(Box::new_in(
                NullLiteral { span: SPAN },
                self.allocator,
            )));
        }
        
        let call = self.call_expr(self.allocator.alloc_str("_$insert"), args);
        
        Statement::ExpressionStatement(Box::new_in(
            ExpressionStatement {
                span: SPAN,
                expression: call,
            },
            self.allocator,
        ))
    }

    /// Create: _$effect(() => _$setAttribute(el, "id", value))
    pub(super) fn create_set_attribute_effect(&self, element_expr: Expression<'a>, attr_name: &'a str, value_expr: Expression<'a>) -> Statement<'a> {
        // Inner call: _$setAttribute(el, "attr", value)
        let mut set_attr_args = OxcVec::new_in(self.allocator);
        set_attr_args.push(Argument::from(element_expr));
        set_attr_args.push(Argument::StringLiteral(Box::new_in(
            StringLiteral {
                span: SPAN,
                value: Atom::from(attr_name),
                raw: None,
                lone_surrogates: false,
            },
            self.allocator,
        )));
        set_attr_args.push(Argument::from(value_expr));
        
        let set_attr_call = self.call_expr(self.allocator.alloc_str("_$setAttribute"), set_attr_args);
        
        // Wrap in arrow function: () => _$setAttribute(...)
        let mut arrow_body_stmts = OxcVec::new_in(self.allocator);
        arrow_body_stmts.push(Statement::ExpressionStatement(Box::new_in(
            ExpressionStatement {
                span: SPAN,
                expression: set_attr_call,
            },
            self.allocator,
        )));
        
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
                    statements: arrow_body_stmts,
                },
                self.allocator,
            ),
            scope_id: None.into(),
            pife: false,
            pure: false,
        };
        
        // Wrap in _$effect
        let mut effect_args = OxcVec::new_in(self.allocator);
        effect_args.push(Argument::ArrowFunctionExpression(Box::new_in(arrow_fn, self.allocator)));
        
        let effect_call = self.call_expr(self.allocator.alloc_str("_$effect"), effect_args);
        
        Statement::ExpressionStatement(Box::new_in(
            ExpressionStatement {
                span: SPAN,
                expression: effect_call,
            },
            self.allocator,
        ))
    }

    /// Create: _$addEventListener(el, "click", handler, true)
    pub(super) fn create_event_listener(&self, element_expr: Expression<'a>, event_name: &'a str, handler_expr: Expression<'a>) -> Statement<'a> {
        let mut args = OxcVec::new_in(self.allocator);
        args.push(Argument::from(element_expr));
        args.push(Argument::StringLiteral(Box::new_in(
            StringLiteral {
                span: SPAN,
                value: Atom::from(event_name),
                raw: None,
                lone_surrogates: false,
            },
            self.allocator,
        )));
        args.push(Argument::from(handler_expr));
        args.push(Argument::BooleanLiteral(Box::new_in(
            BooleanLiteral {
                span: SPAN,
                value: true,
            },
            self.allocator,
        )));
        
        let call = self.call_expr(self.allocator.alloc_str("_$addEventListener"), args);
        
        Statement::ExpressionStatement(Box::new_in(
            ExpressionStatement {
                span: SPAN,
                expression: call,
            },
            self.allocator,
        ))
    }
}
