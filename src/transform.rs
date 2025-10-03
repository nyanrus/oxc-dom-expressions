//! Main transformer for DOM expressions

use oxc_allocator::Vec as OxcVec;
use oxc_allocator::{Allocator, Box};
use oxc_ast::ast::*;
use oxc_span::{Atom, SPAN};
use oxc_traverse::{Traverse, TraverseCtx};
use std::collections::{HashMap, HashSet};

use crate::optimizer::{TemplateOptimizer, TemplateStats};
use crate::options::DomExpressionsOptions;
use crate::template::{SlotType, Template};
use crate::utils::{is_component, should_delegate_event};

/// The main DOM expressions transformer
pub struct DomExpressions<'a> {
    #[allow(dead_code)]
    allocator: &'a Allocator,
    options: DomExpressionsOptions,
    /// Collection of templates generated during transformation
    templates: Vec<Template>,
    /// Map of template HTML to variable name for deduplication
    template_map: HashMap<String, String>,
    /// Counter for generating unique template variable names
    template_counter: usize,
    /// Counter for generating unique element variable names
    element_counter: usize,
    /// List of required imports (preserves insertion order)
    required_imports: Vec<String>,
    /// Set of events that need delegation
    delegated_events: HashSet<String>,
    /// Optimizer for template analysis
    optimizer: TemplateOptimizer,
}

impl<'a> DomExpressions<'a> {
    /// Create a new DOM expressions transformer
    pub fn new(allocator: &'a Allocator, options: DomExpressionsOptions) -> Self {
        Self {
            allocator,
            options,
            templates: Vec::new(),
            template_map: HashMap::new(),
            template_counter: 0,
            element_counter: 0,
            required_imports: Vec::new(),
            delegated_events: HashSet::new(),
            optimizer: TemplateOptimizer::new(),
        }
    }

    /// Get the current options
    pub fn options(&self) -> &DomExpressionsOptions {
        &self.options
    }

    /// Get template statistics for optimization analysis
    pub fn get_template_stats(&self) -> TemplateStats {
        self.optimizer.get_stats()
    }

    /// Get list of templates that were reused (deduplicated)
    pub fn get_reused_templates(&self) -> Vec<(String, usize)> {
        self.optimizer.get_reused_templates()
    }

    /// Generate a unique template variable name
    fn generate_template_var(&mut self) -> String {
        self.template_counter += 1;
        if self.template_counter == 1 {
            "_tmpl$".to_string()
        } else {
            format!("_tmpl${}", self.template_counter)
        }
    }

    /// Get or create a template variable for given HTML
    fn get_template_var(&mut self, html: &str) -> String {
        if let Some(var) = self.template_map.get(html) {
            var.clone()
        } else {
            let var = self.generate_template_var();
            self.template_map.insert(html.to_string(), var.clone());
            var
        }
    }

    /// Add a required import (preserves insertion order)
    fn add_import(&mut self, name: &str) {
        if !self.required_imports.contains(&name.to_string()) {
            self.required_imports.push(name.to_string());
        }
    }

    /// Add an event for delegation
    fn add_delegated_event(&mut self, event: &str) {
        self.delegated_events.insert(event.to_lowercase());
    }

    /// Create a call expression for cloning a template
    fn create_template_call(&self, template_var: &'a str) -> Box<'a, CallExpression<'a>> {
        // Create identifier for the template variable (e.g., "_tmpl$")
        let callee_ident = IdentifierReference {
            span: SPAN,
            name: Atom::from(template_var),
            reference_id: None.into(),
        };
        let callee = Expression::Identifier(Box::new_in(callee_ident, self.allocator));

        let call_expr = CallExpression {
            span: SPAN,
            arguments: OxcVec::new_in(self.allocator),
            callee,
            optional: false,
            type_arguments: None,
            pure: false,
        };

        Box::new_in(call_expr, self.allocator)
    }

    /// Create an IIFE that clones template and applies dynamic bindings
    fn create_template_iife_from_expressions(
        &mut self,
        expressions: Vec<Expression<'a>>,
        template: &Template,
        template_var: &str,
    ) -> Box<'a, CallExpression<'a>> {
        use oxc_ast::ast::*;

        // Generate IIFE: (() => { ... })()
        // 1. Create statements for the function body
        let mut body_stmts = OxcVec::new_in(self.allocator);

        // 2. Create template cloning statement and element references
        // var _el$ = _tmpl$(), _el$2 = _el$.firstChild, ...
        let (root_var, elem_decls, path_to_var) =
            self.create_element_declarations(template, template_var);
        body_stmts.push(elem_decls);

        // 3. Create runtime calls for dynamic content
        let runtime_stmts = self.create_runtime_calls_from_expressions(
            &expressions,
            template,
            &root_var,
            &path_to_var,
        );
        body_stmts.extend(runtime_stmts);

        // 4. Create return statement
        let return_stmt = self.create_return_statement(&root_var);
        body_stmts.push(return_stmt);

        // Create function body
        let func_body = FunctionBody {
            span: SPAN,
            directives: OxcVec::new_in(self.allocator),
            statements: body_stmts,
        };

        // Create arrow function
        let arrow_fn = ArrowFunctionExpression {
            span: SPAN,
            expression: false,
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
            body: Box::new_in(func_body, self.allocator),
            type_parameters: None,
            return_type: None,
            scope_id: None.into(),
            pure: false,
            pife: false,
        };

        // Create call expression: (() => { ... })()
        let call_expr = CallExpression {
            span: SPAN,
            callee: Expression::ArrowFunctionExpression(Box::new_in(arrow_fn, self.allocator)),
            arguments: OxcVec::new_in(self.allocator),
            optional: false,
            type_arguments: None,
            pure: false,
        };

        Box::new_in(call_expr, self.allocator)
    }

    /// Create element reference declarations
    /// Returns (root_var_name, statement)
    fn create_element_declarations(
        &mut self,
        template: &Template,
        template_var: &str,
    ) -> (
        String,
        Statement<'a>,
        std::collections::HashMap<Vec<String>, String>,
    ) {
        use oxc_ast::ast::*;

        // Generate unique variable names
        let root_var = self.generate_element_var();
        let mut path_to_var = std::collections::HashMap::new();

        // Create declarators
        let mut declarators = OxcVec::new_in(self.allocator);

        // First declarator: var _el$ = _tmpl$()
        let root_id = BindingPattern {
            kind: BindingPatternKind::BindingIdentifier(Box::new_in(
                BindingIdentifier {
                    span: SPAN,
                    name: Atom::from(self.allocator.alloc_str(&root_var)),
                    symbol_id: None.into(),
                },
                self.allocator,
            )),
            type_annotation: None,
            optional: false,
        };

        // Create call to template function
        let template_call = self.create_template_call(self.allocator.alloc_str(template_var));

        declarators.push(VariableDeclarator {
            span: SPAN,
            kind: VariableDeclarationKind::Var,
            id: root_id,
            init: Some(Expression::CallExpression(template_call)),
            definite: false,
        });

        // Generate element references for each dynamic slot
        // Track which paths we've already created references for
        let mut created_refs = std::collections::HashSet::new();

        for slot in &template.dynamic_slots {
            // Generate reference for slot path if needed
            if !slot.path.is_empty() && !created_refs.contains(&slot.path) {
                created_refs.insert(slot.path.clone());

                let elem_var = self.generate_element_var();
                path_to_var.insert(slot.path.clone(), elem_var.clone());
                let elem_id = BindingPattern {
                    kind: BindingPatternKind::BindingIdentifier(Box::new_in(
                        BindingIdentifier {
                            span: SPAN,
                            name: Atom::from(self.allocator.alloc_str(&elem_var)),
                            symbol_id: None.into(),
                        },
                        self.allocator,
                    )),
                    type_annotation: None,
                    optional: false,
                };

                // Build path expression: _el$.firstChild.nextSibling...
                let mut expr = Expression::Identifier(Box::new_in(
                    IdentifierReference {
                        span: SPAN,
                        name: Atom::from(self.allocator.alloc_str(&root_var)),
                        reference_id: None.into(),
                    },
                    self.allocator,
                ));

                for segment in &slot.path {
                    expr = Expression::StaticMemberExpression(Box::new_in(
                        StaticMemberExpression {
                            span: SPAN,
                            object: expr,
                            property: IdentifierName {
                                span: SPAN,
                                name: Atom::from(self.allocator.alloc_str(segment)),
                            },
                            optional: false,
                        },
                        self.allocator,
                    ));
                }

                declarators.push(VariableDeclarator {
                    span: SPAN,
                    kind: VariableDeclarationKind::Var,
                    id: elem_id,
                    init: Some(expr),
                    definite: false,
                });
            }

            // Generate reference for marker path if needed
            if let Some(marker_path) = &slot.marker_path {
                if !marker_path.is_empty() && !created_refs.contains(marker_path) {
                    created_refs.insert(marker_path.clone());

                    let elem_var = self.generate_element_var();
                    path_to_var.insert(marker_path.clone(), elem_var.clone());
                    let elem_id = BindingPattern {
                        kind: BindingPatternKind::BindingIdentifier(Box::new_in(
                            BindingIdentifier {
                                span: SPAN,
                                name: Atom::from(self.allocator.alloc_str(&elem_var)),
                                symbol_id: None.into(),
                            },
                            self.allocator,
                        )),
                        type_annotation: None,
                        optional: false,
                    };

                    // Build path expression: _el$.firstChild.nextSibling...
                    let mut expr = Expression::Identifier(Box::new_in(
                        IdentifierReference {
                            span: SPAN,
                            name: Atom::from(self.allocator.alloc_str(&root_var)),
                            reference_id: None.into(),
                        },
                        self.allocator,
                    ));

                    for segment in marker_path {
                        expr = Expression::StaticMemberExpression(Box::new_in(
                            StaticMemberExpression {
                                span: SPAN,
                                object: expr,
                                property: IdentifierName {
                                    span: SPAN,
                                    name: Atom::from(self.allocator.alloc_str(segment)),
                                },
                                optional: false,
                            },
                            self.allocator,
                        ));
                    }

                    declarators.push(VariableDeclarator {
                        span: SPAN,
                        kind: VariableDeclarationKind::Var,
                        id: elem_id,
                        init: Some(expr),
                        definite: false,
                    });
                }
            }
        }

        let var_decl = VariableDeclaration {
            span: SPAN,
            kind: VariableDeclarationKind::Var,
            declarations: declarators,
            declare: false,
        };

        (
            root_var,
            Statement::VariableDeclaration(Box::new_in(var_decl, self.allocator)),
            path_to_var,
        )
    }

    /// Generate unique element variable name
    fn generate_element_var(&mut self) -> String {
        self.element_counter += 1;
        if self.element_counter == 1 {
            "_el$".to_string()
        } else {
            format!("_el${}", self.element_counter)
        }
    }

    /// Create runtime calls for dynamic content from extracted expressions
    fn create_runtime_calls_from_expressions(
        &mut self,
        expressions: &[Expression<'a>],
        template: &Template,
        root_var: &str,
        path_to_var: &std::collections::HashMap<Vec<String>, String>,
    ) -> OxcVec<'a, Statement<'a>> {
        let mut stmts = OxcVec::new_in(self.allocator);

        // Track which expression we're at
        let mut expr_index = 0;

        // For each dynamic slot, generate the appropriate runtime call
        for slot in &template.dynamic_slots {
            match &slot.slot_type {
                SlotType::TextContent => {
                    // Generate insert call
                    self.add_import("insert");

                    if expr_index < expressions.len() {
                        // Determine marker variable (3rd argument to insert)
                        let marker_var = if let Some(marker_path) = &slot.marker_path {
                            path_to_var.get(marker_path).map(|s| s.as_str())
                        } else {
                            None
                        };

                        if let Some(insert_stmt) = self.create_insert_call_with_marker(
                            root_var,
                            &expressions[expr_index],
                            marker_var,
                        ) {
                            stmts.push(insert_stmt);
                        }
                        expr_index += 1;
                    }
                }
                _ => {
                    // Other slot types not yet implemented
                    // TODO: Implement attribute, event, ref, classList, style bindings
                }
            }
        }

        stmts
    }

    /// Extract all dynamic expressions from JSX element in order (cloning them)
    fn extract_expressions_from_jsx(
        &self,
        jsx_elem: &JSXElement<'a>,
        expressions: &mut Vec<Expression<'a>>,
    ) {
        // Walk through children and extract expressions
        for child in &jsx_elem.children {
            self.extract_expressions_from_child(child, expressions);
        }
    }

    /// Extract expressions from a JSX child (cloning them)
    fn extract_expressions_from_child(
        &self,
        child: &JSXChild<'a>,
        expressions: &mut Vec<Expression<'a>>,
    ) {
        use oxc_allocator::CloneIn;

        match child {
            JSXChild::Element(elem) => {
                // Recursively extract from nested elements
                self.extract_expressions_from_jsx(elem, expressions);
            }
            JSXChild::ExpressionContainer(container) => {
                match &container.expression {
                    JSXExpression::StringLiteral(_)
                    | JSXExpression::NumericLiteral(_)
                    | JSXExpression::EmptyExpression(_) => {
                        // Static or empty - skip (already in template)
                    }
                    // All other JSXExpression variants are dynamic expressions
                    // JSXExpression inherits from Expression via macro
                    expr => {
                        // Convert JSXExpression to Expression and clone it
                        if let Some(expr_ref) = expr.as_expression() {
                            expressions.push(expr_ref.clone_in(self.allocator));
                        }
                    }
                }
            }
            JSXChild::Text(_) => {
                // Static text - skip
            }
            JSXChild::Fragment(_) | JSXChild::Spread(_) => {
                // Not implemented yet
            }
        }
    }

    /// Create an insert call statement with optional marker
    fn create_insert_call_with_marker(
        &self,
        element_var: &str,
        expr: &Expression<'a>,
        marker_var: Option<&str>,
    ) -> Option<Statement<'a>> {
        use oxc_allocator::CloneIn;
        use oxc_ast::ast::*;

        // Create call to _$insert(element, expression, marker)
        let insert_fn = IdentifierReference {
            span: SPAN,
            name: Atom::from("_$insert"),
            reference_id: None.into(),
        };

        // First argument: element reference
        let elem_arg = Argument::Identifier(Box::new_in(
            IdentifierReference {
                span: SPAN,
                name: Atom::from(self.allocator.alloc_str(element_var)),
                reference_id: None.into(),
            },
            self.allocator,
        ));

        // Second argument: the expression (clone it)
        let expr_arg = Argument::from(expr.clone_in(self.allocator));

        // Third argument: marker position (either a variable reference or null)
        let marker_arg = if let Some(marker) = marker_var {
            Argument::Identifier(Box::new_in(
                IdentifierReference {
                    span: SPAN,
                    name: Atom::from(self.allocator.alloc_str(marker)),
                    reference_id: None.into(),
                },
                self.allocator,
            ))
        } else {
            Argument::NullLiteral(Box::new_in(NullLiteral { span: SPAN }, self.allocator))
        };

        let mut args = OxcVec::new_in(self.allocator);
        args.push(elem_arg);
        args.push(expr_arg);
        args.push(marker_arg);

        let call_expr = CallExpression {
            span: SPAN,
            callee: Expression::Identifier(Box::new_in(insert_fn, self.allocator)),
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

    /// Create return statement
    fn create_return_statement(&self, root_var: &str) -> Statement<'a> {
        use oxc_ast::ast::*;

        let return_expr = Expression::Identifier(Box::new_in(
            IdentifierReference {
                span: SPAN,
                name: Atom::from(self.allocator.alloc_str(root_var)),
                reference_id: None.into(),
            },
            self.allocator,
        ));

        Statement::ReturnStatement(Box::new_in(
            ReturnStatement {
                span: SPAN,
                argument: Some(return_expr),
            },
            self.allocator,
        ))
    }

    /// Create multiple import statements (one per import)
    fn create_import_statements(&self) -> Vec<Statement<'a>> {
        use oxc_ast::ast::*;

        let mut statements = Vec::new();

        // Define import priority order (lower number = higher priority)
        let get_priority = |name: &str| -> usize {
            match name {
                "template" => 0,
                "delegateEvents" => 1,
                "createComponent" => 2,
                "memo" => 3,
                "For" => 4,
                "Show" => 5,
                "Suspense" => 6,
                "SuspenseList" => 7,
                "Switch" => 8,
                "Match" => 9,
                "Index" => 10,
                "ErrorBoundary" => 11,
                "mergeProps" => 12,
                "spread" => 13,
                "use" => 14,
                "insert" => 15,
                "setAttribute" => 16,
                "setAttributeNS" => 17,
                "setBoolAttribute" => 18,
                "className" => 19,
                "style" => 20,
                "setStyleProperty" => 21,
                "addEventListener" => 22,
                "effect" => 23,
                "getOwner" => 24,
                _ => 100, // Unknown imports go last
            }
        };

        // Sort imports by priority
        let mut sorted_imports: Vec<_> = self.required_imports.iter().collect();
        sorted_imports.sort_by_key(|name| get_priority(name));

        for import_name in sorted_imports {
            // Create local binding name (e.g., _$template for template)
            let local_name = format!("_${}", import_name);
            let local = BindingIdentifier {
                span: SPAN,
                name: Atom::from(self.allocator.alloc_str(&local_name)),
                symbol_id: None.into(),
            };

            // Create imported name
            let imported = ModuleExportName::IdentifierName(IdentifierName {
                span: SPAN,
                name: Atom::from(self.allocator.alloc_str(import_name)),
            });

            // Create import specifier
            let specifier = ImportDeclarationSpecifier::ImportSpecifier(Box::new_in(
                ImportSpecifier {
                    span: SPAN,
                    imported,
                    local,
                    import_kind: ImportOrExportKind::Value,
                },
                self.allocator,
            ));

            let mut specifiers = OxcVec::new_in(self.allocator);
            specifiers.push(specifier);

            // Create source string
            let source = StringLiteral {
                span: SPAN,
                value: Atom::from(self.allocator.alloc_str(&self.options.module_name)),
                raw: None,
                lone_surrogates: false,
            };

            // Create import declaration
            let import_decl = ImportDeclaration {
                span: SPAN,
                specifiers: Some(specifiers),
                source,
                with_clause: None,
                import_kind: ImportOrExportKind::Value,
                phase: None,
            };

            // Wrap in ModuleDeclaration and Statement
            let module_decl =
                ModuleDeclaration::ImportDeclaration(Box::new_in(import_decl, self.allocator));

            statements.push(Statement::from(module_decl));
        }

        statements
    }

    /// Create template variable declarations
    fn create_template_declarations(&self) -> Option<Statement<'a>> {
        use oxc_ast::ast::*;

        if self.template_map.is_empty() {
            return None;
        }

        // Create variable declarators for all templates
        let mut declarators = OxcVec::new_in(self.allocator);

        // Sort template map by variable name to get consistent order (numerically)
        let mut sorted_templates: Vec<_> = self.template_map.iter().collect();
        sorted_templates.sort_by(|a, b| {
            // Extract the numeric part from variable names like "_tmpl$" or "_tmpl$2"
            let get_num = |name: &str| -> usize {
                if name == "_tmpl$" {
                    1
                } else {
                    name.strip_prefix("_tmpl$")
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(0)
                }
            };
            get_num(a.1).cmp(&get_num(b.1))
        });

        for (html, var_name) in sorted_templates {
            // Create the binding pattern for the variable
            let id = BindingPattern {
                kind: BindingPatternKind::BindingIdentifier(Box::new_in(
                    BindingIdentifier {
                        span: SPAN,
                        name: Atom::from(self.allocator.alloc_str(var_name)),
                        symbol_id: None.into(),
                    },
                    self.allocator,
                )),
                type_annotation: None,
                optional: false,
            };

            // Create template literal argument (using backticks)
            let template_element = TemplateElement {
                span: SPAN,
                tail: true,
                value: TemplateElementValue {
                    raw: Atom::from(self.allocator.alloc_str(html)),
                    cooked: Some(Atom::from(self.allocator.alloc_str(html))),
                },
                lone_surrogates: false,
            };

            let mut elements = OxcVec::new_in(self.allocator);
            elements.push(template_element);

            let template_literal = TemplateLiteral {
                span: SPAN,
                quasis: elements,
                expressions: OxcVec::new_in(self.allocator),
            };

            // Create call to _$template(...)
            let template_fn = IdentifierReference {
                span: SPAN,
                name: Atom::from("_$template"),
                reference_id: None.into(),
            };

            let mut args = OxcVec::new_in(self.allocator);
            args.push(Argument::TemplateLiteral(Box::new_in(
                template_literal,
                self.allocator,
            )));

            let init_call = CallExpression {
                span: SPAN,
                callee: Expression::Identifier(Box::new_in(template_fn, self.allocator)),
                arguments: args,
                optional: false,
                type_arguments: None,
                pure: true, // Mark as /*#__PURE__*/
            };

            // Create variable declarator
            let declarator = VariableDeclarator {
                span: SPAN,
                kind: VariableDeclarationKind::Var,
                id,
                init: Some(Expression::CallExpression(Box::new_in(
                    init_call,
                    self.allocator,
                ))),
                definite: false,
            };

            declarators.push(declarator);
        }

        // Create variable declaration
        let var_decl = VariableDeclaration {
            span: SPAN,
            kind: VariableDeclarationKind::Var,
            declarations: declarators,
            declare: false,
        };

        Some(Statement::VariableDeclaration(Box::new_in(
            var_decl,
            self.allocator,
        )))
    }

    /// Create delegateEvents call
    fn create_delegate_events_call(&self) -> Option<Statement<'a>> {
        use oxc_ast::ast::*;

        if self.delegated_events.is_empty() {
            return None;
        }

        // Create array of event names
        let mut elements = OxcVec::new_in(self.allocator);
        let mut sorted_events: Vec<_> = self.delegated_events.iter().collect();
        sorted_events.sort();

        for event in sorted_events {
            let string_lit = StringLiteral {
                span: SPAN,
                value: Atom::from(self.allocator.alloc_str(event)),
                raw: None,
                lone_surrogates: false,
            };
            elements.push(ArrayExpressionElement::StringLiteral(Box::new_in(
                string_lit,
                self.allocator,
            )));
        }

        let array_expr = ArrayExpression {
            span: SPAN,
            elements,
        };

        // Create call to _$delegateEvents([...])
        let fn_name = IdentifierReference {
            span: SPAN,
            name: Atom::from("_$delegateEvents"),
            reference_id: None.into(),
        };

        let mut args = OxcVec::new_in(self.allocator);
        args.push(Argument::ArrayExpression(Box::new_in(
            array_expr,
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

        // Wrap in expression statement
        Some(Statement::ExpressionStatement(Box::new_in(
            ExpressionStatement {
                span: SPAN,
                expression: Expression::CallExpression(Box::new_in(call_expr, self.allocator)),
            },
            self.allocator,
        )))
    }

    /// Transform a component JSX element into a createComponent call
    fn transform_component(&mut self, jsx_elem: Box<'a, JSXElement<'a>>) -> Expression<'a> {
        use oxc_ast::ast::*;

        // Add the createComponent import
        self.add_import("createComponent");

        // Get the component name
        let component_name = match &jsx_elem.opening_element.name {
            JSXElementName::Identifier(ident) => ident.name.clone(),
            JSXElementName::IdentifierReference(ident) => ident.name.clone(),
            _ => Atom::from("Unknown"),
        };

        // Create the component identifier for the first argument
        let component_ident = IdentifierReference {
            span: SPAN,
            name: component_name,
            reference_id: None.into(),
        };

        // Create arguments array
        let mut arguments = OxcVec::new_in(self.allocator);

        // First argument: component identifier
        arguments.push(Argument::from(Expression::Identifier(Box::new_in(
            component_ident,
            self.allocator,
        ))));

        // Second argument: props object
        let props_obj = self.create_component_props(&jsx_elem);
        arguments.push(Argument::from(Expression::ObjectExpression(Box::new_in(
            props_obj,
            self.allocator,
        ))));

        // Create the call expression: _$createComponent(Component, {...})
        let callee_ident = IdentifierReference {
            span: SPAN,
            name: Atom::from("_$createComponent"),
            reference_id: None.into(),
        };

        let call_expr = CallExpression {
            span: SPAN,
            callee: Expression::Identifier(Box::new_in(callee_ident, self.allocator)),
            arguments,
            optional: false,
            type_arguments: None,
            pure: false,
        };

        Expression::CallExpression(Box::new_in(call_expr, self.allocator))
    }

    /// Create props object for a component
    fn create_component_props(&self, jsx_elem: &JSXElement<'a>) -> ObjectExpression<'a> {
        use oxc_ast::ast::*;

        let mut properties = OxcVec::new_in(self.allocator);

        // Add attributes as properties
        for attr in &jsx_elem.opening_element.attributes {
            if let JSXAttributeItem::Attribute(jsx_attr) = attr {
                if let JSXAttributeName::Identifier(name_ident) = &jsx_attr.name {
                    let prop_name = name_ident.name.clone();

                    // Get the value
                    let prop_value = if let Some(value) = &jsx_attr.value {
                        match value {
                            JSXAttributeValue::StringLiteral(str_lit) => {
                                Expression::StringLiteral(Box::new_in(
                                    StringLiteral {
                                        span: SPAN,
                                        value: str_lit.value.clone(),
                                        raw: None.into(),
                                        lone_surrogates: false,
                                    },
                                    self.allocator,
                                ))
                            }
                            JSXAttributeValue::ExpressionContainer(expr_container) => {
                                match &expr_container.expression {
                                    jsx_expr if jsx_expr.is_expression() => {
                                        // Clone the expression
                                        self.clone_expression(jsx_expr.as_expression().unwrap())
                                    }
                                    _ => {
                                        // For other cases, use true
                                        Expression::BooleanLiteral(Box::new_in(
                                            BooleanLiteral {
                                                span: SPAN,
                                                value: true,
                                            },
                                            self.allocator,
                                        ))
                                    }
                                }
                            }
                            _ => Expression::BooleanLiteral(Box::new_in(
                                BooleanLiteral {
                                    span: SPAN,
                                    value: true,
                                },
                                self.allocator,
                            )),
                        }
                    } else {
                        Expression::BooleanLiteral(Box::new_in(
                            BooleanLiteral {
                                span: SPAN,
                                value: true,
                            },
                            self.allocator,
                        ))
                    };

                    // Create property
                    let prop_key = PropertyKey::StaticIdentifier(Box::new_in(
                        IdentifierName {
                            span: SPAN,
                            name: prop_name,
                        },
                        self.allocator,
                    ));

                    properties.push(ObjectPropertyKind::ObjectProperty(Box::new_in(
                        ObjectProperty {
                            span: SPAN,
                            kind: PropertyKind::Init,
                            key: prop_key,
                            value: prop_value,
                            method: false,
                            shorthand: false,
                            computed: false,
                        },
                        self.allocator,
                    )));
                }
            }
        }

        // Add children if present
        if !jsx_elem.children.is_empty() {
            let children_value = self.create_component_children(&jsx_elem.children);

            let prop_key = PropertyKey::StaticIdentifier(Box::new_in(
                IdentifierName {
                    span: SPAN,
                    name: Atom::from("children"),
                },
                self.allocator,
            ));

            properties.push(ObjectPropertyKind::ObjectProperty(Box::new_in(
                ObjectProperty {
                    span: SPAN,
                    kind: PropertyKind::Init,
                    key: prop_key,
                    value: children_value,
                    method: false,
                    shorthand: false,
                    computed: false,
                },
                self.allocator,
            )));
        }

        ObjectExpression {
            span: SPAN,
            properties,
        }
    }

    /// Create children value for a component (can be a single value or array)
    fn create_component_children(&self, children: &OxcVec<'a, JSXChild<'a>>) -> Expression<'a> {
        use oxc_ast::ast::*;

        // Filter out whitespace-only text nodes
        let significant_children: Vec<_> = children
            .iter()
            .filter(|child| match child {
                JSXChild::Text(text) => !text.value.trim().is_empty(),
                _ => true,
            })
            .collect();

        if significant_children.len() == 1 {
            // Single child - return it directly
            self.jsx_child_to_expression(significant_children[0])
        } else {
            // Multiple children - return as array
            let mut elements = OxcVec::new_in(self.allocator);
            for child in significant_children {
                let expr = self.jsx_child_to_expression(child);
                elements.push(ArrayExpressionElement::from(expr));
            }
            Expression::ArrayExpression(Box::new_in(
                ArrayExpression {
                    span: SPAN,
                    elements,
                },
                self.allocator,
            ))
        }
    }

    /// Convert a JSX child to an expression
    fn jsx_child_to_expression(&self, child: &JSXChild<'a>) -> Expression<'a> {
        use oxc_ast::ast::*;

        match child {
            JSXChild::Text(text) => Expression::StringLiteral(Box::new_in(
                StringLiteral {
                    span: SPAN,
                    value: text.value.clone(),
                    raw: None.into(),
                    lone_surrogates: false,
                },
                self.allocator,
            )),
            JSXChild::ExpressionContainer(expr_container) => match &expr_container.expression {
                jsx_expr if jsx_expr.is_expression() => {
                    self.clone_expression(jsx_expr.as_expression().unwrap())
                }
                _ => {
                    Expression::NullLiteral(Box::new_in(NullLiteral { span: SPAN }, self.allocator))
                }
            },
            JSXChild::Element(_elem) => {
                // Transform JSX element - need to handle this recursively
                // For now, create a call expression (elements should already be transformed in exit_expression)
                // Return a null literal as a placeholder - the actual element should be transformed elsewhere
                Expression::NullLiteral(Box::new_in(NullLiteral { span: SPAN }, self.allocator))
            }
            JSXChild::Fragment(_frag) => {
                // Transform JSX fragment
                // Return a null literal as a placeholder - the actual fragment should be transformed elsewhere
                Expression::NullLiteral(Box::new_in(NullLiteral { span: SPAN }, self.allocator))
            }
            _ => {
                // For other types, return null for now
                Expression::NullLiteral(Box::new_in(NullLiteral { span: SPAN }, self.allocator))
            }
        }
    }

    /// Transform a JSX fragment into an array or string
    fn transform_fragment(&self, jsx_frag: Box<'a, JSXFragment<'a>>) -> Expression<'a> {
        use oxc_ast::ast::*;

        // Filter out whitespace-only text nodes
        let significant_children: Vec<_> = jsx_frag
            .children
            .iter()
            .filter(|child| match child {
                JSXChild::Text(text) => !text.value.trim().is_empty(),
                _ => true,
            })
            .collect();

        if significant_children.len() == 1 {
            // Single child - return it directly
            self.jsx_child_to_expression(significant_children[0])
        } else {
            // Multiple children - return as array
            let mut elements = OxcVec::new_in(self.allocator);
            for child in significant_children {
                let expr = self.jsx_child_to_expression(child);
                elements.push(ArrayExpressionElement::from(expr));
            }
            Expression::ArrayExpression(Box::new_in(
                ArrayExpression {
                    span: SPAN,
                    elements,
                },
                self.allocator,
            ))
        }
    }

    /// Clone an expression (deep copy)
    fn clone_expression(&self, expr: &Expression<'a>) -> Expression<'a> {
        use oxc_ast::ast::*;

        match expr {
            Expression::Identifier(ident) => Expression::Identifier(Box::new_in(
                IdentifierReference {
                    span: SPAN,
                    name: ident.name.clone(),
                    reference_id: None.into(),
                },
                self.allocator,
            )),
            Expression::StringLiteral(str_lit) => Expression::StringLiteral(Box::new_in(
                StringLiteral {
                    span: SPAN,
                    value: str_lit.value.clone(),
                    raw: None.into(),
                    lone_surrogates: false,
                },
                self.allocator,
            )),
            Expression::BooleanLiteral(bool_lit) => Expression::BooleanLiteral(Box::new_in(
                BooleanLiteral {
                    span: SPAN,
                    value: bool_lit.value,
                },
                self.allocator,
            )),
            Expression::NumericLiteral(num_lit) => Expression::NumericLiteral(Box::new_in(
                NumericLiteral {
                    span: SPAN,
                    value: num_lit.value,
                    raw: num_lit.raw.clone(),
                    base: num_lit.base,
                },
                self.allocator,
            )),
            Expression::StaticMemberExpression(static_member) => {
                let object = self.clone_expression(&static_member.object);
                Expression::StaticMemberExpression(Box::new_in(
                    StaticMemberExpression {
                        span: SPAN,
                        object,
                        property: IdentifierName {
                            span: SPAN,
                            name: static_member.property.name.clone(),
                        },
                        optional: static_member.optional,
                    },
                    self.allocator,
                ))
            }
            Expression::ComputedMemberExpression(computed_member) => {
                let object = self.clone_expression(&computed_member.object);
                let expression = self.clone_expression(&computed_member.expression);
                Expression::ComputedMemberExpression(Box::new_in(
                    ComputedMemberExpression {
                        span: SPAN,
                        object,
                        expression,
                        optional: computed_member.optional,
                    },
                    self.allocator,
                ))
            }
            Expression::BinaryExpression(bin_expr) => {
                let left = self.clone_expression(&bin_expr.left);
                let right = self.clone_expression(&bin_expr.right);
                Expression::BinaryExpression(Box::new_in(
                    BinaryExpression {
                        span: SPAN,
                        left,
                        operator: bin_expr.operator,
                        right,
                    },
                    self.allocator,
                ))
            }
            _ => {
                // For other expression types, return a null literal for now
                Expression::NullLiteral(Box::new_in(NullLiteral { span: SPAN }, self.allocator))
            }
        }
    }
}

impl<'a> Traverse<'a, ()> for DomExpressions<'a> {
    fn enter_program(&mut self, _program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a, ()>) {
        // Entry point for the transformation
        // Initialize state for collecting templates and imports
        self.templates.clear();
        self.template_map.clear();
        self.template_counter = 0;
        self.element_counter = 0;
        self.required_imports.clear();
        self.delegated_events.clear();

        // Add the template import (will be needed for any JSX)
        self.add_import("template");
    }

    fn exit_program(&mut self, program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a, ()>) {
        // Exit point for the transformation
        // Add delegate events import if needed
        if self.options.delegate_events && !self.delegated_events.is_empty() {
            self.add_import("delegateEvents");
        }

        // Build the list of statements to inject at the beginning
        let mut new_stmts = Vec::new();

        // 1. Add import statements (one per import)
        if !self.required_imports.is_empty() {
            let import_stmts = self.create_import_statements();
            new_stmts.extend(import_stmts);
        }

        // 2. Add template declarations
        if !self.template_map.is_empty() {
            if let Some(template_decl) = self.create_template_declarations() {
                new_stmts.push(template_decl);
            }
        }

        // 3. Prepend new statements to the program
        if !new_stmts.is_empty() {
            // Get existing statements
            let existing_stmts =
                std::mem::replace(&mut program.body, OxcVec::new_in(self.allocator));

            // Create new statement list with injected statements first
            let mut all_stmts = new_stmts;
            all_stmts.extend(existing_stmts);

            // Replace program body
            program.body = OxcVec::from_iter_in(all_stmts.into_iter(), self.allocator);
        }

        // 4. Add delegateEvents call if needed
        if self.options.delegate_events && !self.delegated_events.is_empty() {
            if let Some(delegate_call) = self.create_delegate_events_call() {
                program.body.push(delegate_call);
            }
        }
    }

    fn enter_jsx_element(&mut self, elem: &mut JSXElement<'a>, _ctx: &mut TraverseCtx<'a, ()>) {
        // Check if this is a component or HTML element
        let tag_name = match &elem.opening_element.name {
            JSXElementName::Identifier(ident) => ident.name.as_str(),
            JSXElementName::IdentifierReference(ident) => ident.name.as_str(),
            _ => return, // Skip complex element names for now
        };

        // Components are handled differently
        if is_component(tag_name) {
            // Component handling - track that we need component imports
            // For now, just skip transformation
            return;
        }

        // Handle JSX elements
        // Build a template from the JSX element
        let template = crate::template::build_template_with_options(elem, Some(&self.options));

        // Record template for optimization analysis
        self.optimizer.record_template(template.clone());

        // Get effect wrapper name before borrowing self mutably
        let _effect_wrapper = self.options.effect_wrapper.clone(); // TODO: Use when implementing full dynamic binding
        let delegate_events = self.options.delegate_events;

        // Track which imports are needed based on dynamic slots
        // NOTE: Currently we only generate simple template calls without dynamic binding code,
        // so we don't need to import these yet. When full IIFE generation is implemented,
        // uncomment this code.
        #[allow(clippy::never_loop)]
        for slot in &template.dynamic_slots {
            match &slot.slot_type {
                SlotType::TextContent => {
                    // self.add_import("insert");
                }
                SlotType::Attribute(_) => {
                    // self.add_import("setAttribute");
                    // self.add_import(&effect_wrapper);
                }
                SlotType::EventHandler(event_name) => {
                    if delegate_events && should_delegate_event(event_name) {
                        self.add_delegated_event(event_name);
                    }
                }
                SlotType::Ref => {
                    // Ref doesn't need imports
                }
                SlotType::ClassList => {
                    // self.add_import("classList");
                    // self.add_import(&effect_wrapper);
                }
                SlotType::StyleObject => {
                    // self.add_import("style");
                    // self.add_import(&effect_wrapper);
                }
                SlotType::OnEvent(_) | SlotType::OnCaptureEvent(_) => {
                    // These use direct addEventListener, no imports needed
                }
            }
        }

        // Store the template for later code generation
        self.templates.push(template);

        // Note: In a full implementation, we would:
        // 1. Replace the JSX element with generated code
        // 2. Create an IIFE that clones the template
        // 3. Add code to set up dynamic bindings
        // 4. Handle event handlers
    }

    fn enter_jsx_fragment(&mut self, _frag: &mut JSXFragment<'a>, _ctx: &mut TraverseCtx<'a, ()>) {
        // Handle JSX fragments
        // Fragments are converted to arrays in Solid
        // Track that we encountered one and may need special handling

        // In a full implementation, we would:
        // 1. Process each child of the fragment
        // 2. Wrap them in an array
        // 3. Handle dynamic children appropriately

        // For now, just note that fragments are being tracked
    }

    fn enter_jsx_opening_element(
        &mut self,
        _elem: &mut JSXOpeningElement<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        // Handle JSX opening elements
        // This is where we would process attributes
    }

    fn enter_jsx_attribute(
        &mut self,
        _attr: &mut JSXAttribute<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        // Handle JSX attributes
        // Process special attributes and event handlers
    }

    fn enter_jsx_spread_attribute(
        &mut self,
        _attr: &mut JSXSpreadAttribute<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        // Handle JSX spread attributes
        // In a full implementation, we would handle spread props
    }

    fn enter_jsx_expression_container(
        &mut self,
        _expr: &mut JSXExpressionContainer<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        // Handle JSX expression containers
        // Wrap dynamic expressions with effect() or insert() as appropriate
    }

    fn exit_expression(&mut self, expr: &mut Expression<'a>, _ctx: &mut TraverseCtx<'a, ()>) {
        // Replace JSX elements and fragments with appropriate calls
        use oxc_ast::ast::*;
        use std::mem;

        // Handle fragments first
        if matches!(expr, Expression::JSXFragment(_)) {
            let placeholder =
                Expression::NullLiteral(Box::new_in(NullLiteral { span: SPAN }, self.allocator));
            let jsx_expr = mem::replace(expr, placeholder);

            if let Expression::JSXFragment(jsx_frag) = jsx_expr {
                let transformed = self.transform_fragment(jsx_frag);
                *expr = transformed;
            }
            return;
        }

        // Check if this is a JSX element
        if !matches!(expr, Expression::JSXElement(_)) {
            return;
        }

        // Temporarily replace with null to get ownership
        let placeholder =
            Expression::NullLiteral(Box::new_in(NullLiteral { span: SPAN }, self.allocator));
        let jsx_expr = mem::replace(expr, placeholder);

        // Now we have ownership of the JSX element
        if let Expression::JSXElement(jsx_elem) = jsx_expr {
            // Check if this is a component
            let tag_name = match &jsx_elem.opening_element.name {
                JSXElementName::Identifier(ident) => ident.name.as_str(),
                JSXElementName::IdentifierReference(ident) => ident.name.as_str(),
                _ => {
                    // Restore the expression and return
                    *expr = Expression::JSXElement(jsx_elem);
                    return;
                }
            };

            if is_component(tag_name) {
                // Transform component
                let component_call = self.transform_component(jsx_elem);
                *expr = component_call;
                return;
            }

            // Build template and get the template variable
            let template =
                crate::template::build_template_with_options(&jsx_elem, Some(&self.options));
            let template_var = self.get_template_var(&template.html);

            // Check if this template has dynamic content
            let has_dynamic_content = !template.dynamic_slots.is_empty();

            if has_dynamic_content {
                // Extract expressions before we lose the JSX element
                let mut expressions = Vec::new();
                self.extract_expressions_from_jsx(&jsx_elem, &mut expressions);

                // Generate an IIFE with dynamic binding code
                let iife = self.create_template_iife_from_expressions(
                    expressions,
                    &template,
                    &template_var,
                );
                *expr = Expression::CallExpression(iife);
            } else {
                // Simple template call for static content
                let template_var_str = self.allocator.alloc_str(&template_var);
                let call_expr = self.create_template_call(template_var_str);
                *expr = Expression::CallExpression(call_expr);
            }
        }
    }
}
