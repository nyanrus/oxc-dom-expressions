//! AST-based code generation helpers
//!
//! This module contains helper methods for generating AST nodes:
//! - Runtime function calls using AstBuilder
//! - Import statement generation
//! - Expression extraction from JSX
//! - Insert call generation
//! - Expression utilities (clone, memo wrapping)
//!
//! All code generation in this module follows Oxc's best practices:
//! - Manual AST construction using `AstBuilder` (accessed via `self.allocator`)
//! - Type-safe node creation with `Box::new_in` and `OxcVec::new_in`
//! - No string-based code generation
//! - Comments in the code document what the generated AST will look like

use oxc_allocator::Box;
use oxc_allocator::Vec as OxcVec;
use oxc_ast::ast::*;
use oxc_span::SPAN;

use crate::compat::get_import_priority;
use crate::template::{SlotType, Template};

use super::DomExpressions;

impl<'a> DomExpressions<'a> {
    pub(super) fn create_runtime_calls_from_expressions(
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
                SlotType::Attribute(attr_name) => {
                    // Generate setAttribute call wrapped in effect
                    self.add_import("setAttribute");
                    self.add_import("effect");

                    if expr_index < expressions.len() {
                        // Get element variable for this slot's path
                        let element_var = if slot.path.is_empty() {
                            root_var
                        } else {
                            path_to_var
                                .get(&slot.path)
                                .map(|s| s.as_str())
                                .unwrap_or(root_var)
                        };

                        if let Some(attr_stmt) = self.create_set_attribute_call(
                            element_var,
                            attr_name,
                            &expressions[expr_index],
                        ) {
                            stmts.push(attr_stmt);
                        }
                        expr_index += 1;
                    }
                }
                SlotType::BoolAttribute(attr_name) => {
                    // Generate setBoolAttribute call
                    self.add_import("setBoolAttribute");
                    // Also need effect for reactive expressions
                    self.add_import("effect");

                    if expr_index < expressions.len() {
                        let element_var = if slot.path.is_empty() {
                            root_var
                        } else {
                            path_to_var
                                .get(&slot.path)
                                .map(|s| s.as_str())
                                .unwrap_or(root_var)
                        };

                        if let Some(stmt) = self.create_set_bool_attribute_call(
                            element_var,
                            attr_name,
                            &expressions[expr_index],
                        ) {
                            stmts.push(stmt);
                        }
                        expr_index += 1;
                    }
                }
                SlotType::PropAttribute(attr_name) => {
                    // Generate direct property assignment: element.propName = value;
                    if expr_index < expressions.len() {
                        let element_var = if slot.path.is_empty() {
                            root_var
                        } else {
                            path_to_var
                                .get(&slot.path)
                                .map(|s| s.as_str())
                                .unwrap_or(root_var)
                        };

                        if let Some(stmt) = self.create_property_assignment(
                            element_var,
                            attr_name,
                            &expressions[expr_index],
                        ) {
                            stmts.push(stmt);
                        }
                        expr_index += 1;
                    }
                }
                SlotType::AttrAttribute(attr_name) => {
                    // Generate setAttribute call (without effect wrapper for static values)
                    self.add_import("setAttribute");

                    if expr_index < expressions.len() {
                        let element_var = if slot.path.is_empty() {
                            root_var
                        } else {
                            path_to_var
                                .get(&slot.path)
                                .map(|s| s.as_str())
                                .unwrap_or(root_var)
                        };

                        if let Some(stmt) = self.create_static_set_attribute_call(
                            element_var,
                            attr_name,
                            &expressions[expr_index],
                        ) {
                            stmts.push(stmt);
                        }
                        expr_index += 1;
                    }
                }
                SlotType::EventHandler(event_name) => {
                    // Generate event handler - either delegated or direct addEventListener
                    if expr_index < expressions.len() {
                        let element_var = if slot.path.is_empty() {
                            root_var
                        } else {
                            path_to_var
                                .get(&slot.path)
                                .map(|s| s.as_str())
                                .unwrap_or(root_var)
                        };

                        // Check if event should be delegated
                        use crate::utils::should_delegate_event;
                        let should_delegate =
                            self.options.delegate_events && should_delegate_event(event_name);

                        let handler_expr = &expressions[expr_index];

                        // Check if handler is an array expression
                        let is_array = matches!(handler_expr, Expression::ArrayExpression(_));

                        if is_array {
                            // Handle array form [handler] or [handler, data]
                            if let Expression::ArrayExpression(arr) = handler_expr {
                                let handler = arr.elements.first().and_then(|e| match e {
                                    oxc_ast::ast::ArrayExpressionElement::SpreadElement(_) => None,
                                    oxc_ast::ast::ArrayExpressionElement::Elision(_) => None,
                                    _ => e.as_expression(),
                                });
                                let data = arr.elements.get(1).and_then(|e| match e {
                                    oxc_ast::ast::ArrayExpressionElement::SpreadElement(_) => None,
                                    oxc_ast::ast::ArrayExpressionElement::Elision(_) => None,
                                    _ => e.as_expression(),
                                });

                                if let Some(handler) = handler {
                                    if should_delegate {
                                        // Delegated event with optional data
                                        if let Some(data) = data {
                                            // Generate: el.$$event = handler; el.$$eventData = data;
                                            if let Some(stmt) = self.create_delegated_event_handler(
                                                element_var,
                                                event_name,
                                                handler,
                                            ) {
                                                stmts.push(stmt);
                                            }
                                            if let Some(stmt) = self.create_delegated_event_data(
                                                element_var,
                                                event_name,
                                                data,
                                            ) {
                                                stmts.push(stmt);
                                            }
                                        } else {
                                            // Just [handler] with no data - for arrays, use direct delegation even if resolvable
                                            if let Some(stmt) = self.create_delegated_event_handler(
                                                element_var,
                                                event_name,
                                                handler,
                                            ) {
                                                stmts.push(stmt);
                                            }
                                        }
                                    } else {
                                        // Non-delegated event with optional data
                                        if let Some(data) = data {
                                            // Generate: el.addEventListener("event", e => handler(data, e))
                                            let wrapper = self.create_event_wrapper(handler, data);
                                            if let Some(stmt) = self.create_add_event_listener(
                                                element_var,
                                                event_name,
                                                &wrapper,
                                                false,
                                            ) {
                                                stmts.push(stmt);
                                            }
                                        } else {
                                            // Just [handler] with no data - use direct addEventListener for arrays
                                            if let Some(stmt) = self.create_add_event_listener(
                                                element_var,
                                                event_name,
                                                handler,
                                                false,
                                            ) {
                                                stmts.push(stmt);
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            // Regular (non-array) handler
                            // Conservative approach: only inline functions are considered resolvable
                            // Identifiers need scope analysis to determine if they're functions,
                            // so we treat them as non-resolvable and use the helper
                            let is_inline_function = matches!(
                                handler_expr,
                                Expression::ArrowFunctionExpression(_)
                                    | Expression::FunctionExpression(_)
                            );

                            if should_delegate {
                                if is_inline_function {
                                    // Use direct delegation for inline functions
                                    if let Some(stmt) = self.create_delegated_event_handler(
                                        element_var,
                                        event_name,
                                        handler_expr,
                                    ) {
                                        stmts.push(stmt);
                                    }
                                } else {
                                    // Use _$addEventListener helper for identifiers (can't determine if resolvable)
                                    self.add_import("addEventListener");
                                    if let Some(stmt) = self.create_add_event_listener_helper(
                                        element_var,
                                        event_name,
                                        handler_expr,
                                        true, // delegated
                                        true, // lowercase event name
                                    ) {
                                        stmts.push(stmt);
                                    }
                                }
                            } else {
                                // Non-delegated handler
                                if is_inline_function {
                                    // Use direct addEventListener for inline functions
                                    if let Some(stmt) = self.create_add_event_listener(
                                        element_var,
                                        event_name,
                                        handler_expr,
                                        false,
                                    ) {
                                        stmts.push(stmt);
                                    }
                                } else {
                                    // Use helper for identifiers
                                    self.add_import("addEventListener");
                                    if let Some(stmt) = self.create_add_event_listener_helper(
                                        element_var,
                                        event_name,
                                        handler_expr,
                                        false, // not delegated
                                        true,  // lowercase event name
                                    ) {
                                        stmts.push(stmt);
                                    }
                                }
                            }
                        }
                        expr_index += 1;
                    }
                }
                SlotType::OnEvent(event_name) => {
                    // Generate on: prefix event - always use _$addEventListener helper
                    self.add_import("addEventListener");

                    if expr_index < expressions.len() {
                        let element_var = if slot.path.is_empty() {
                            root_var
                        } else {
                            path_to_var
                                .get(&slot.path)
                                .map(|s| s.as_str())
                                .unwrap_or(root_var)
                        };

                        // Use the helper function for on: prefix events
                        if let Some(stmt) = self.create_add_event_listener_helper(
                            element_var,
                            event_name,
                            &expressions[expr_index],
                            false, // not delegated
                            false, // don't lowercase - preserve case for custom events
                        ) {
                            stmts.push(stmt);
                        }
                        expr_index += 1;
                    }
                }
                SlotType::OnCaptureEvent(event_name) => {
                    // Generate oncapture: prefix event - addEventListener with capture
                    if expr_index < expressions.len() {
                        let element_var = if slot.path.is_empty() {
                            root_var
                        } else {
                            path_to_var
                                .get(&slot.path)
                                .map(|s| s.as_str())
                                .unwrap_or(root_var)
                        };

                        if let Some(stmt) = self.create_capture_event_listener(
                            element_var,
                            event_name,
                            &expressions[expr_index],
                        ) {
                            stmts.push(stmt);
                        }
                        expr_index += 1;
                    }
                }
                SlotType::UseDirective(directive_name) => {
                    // Generate use directive call
                    if expr_index < expressions.len() {
                        let element_var = if slot.path.is_empty() {
                            root_var
                        } else {
                            path_to_var
                                .get(&slot.path)
                                .map(|s| s.as_str())
                                .unwrap_or(root_var)
                        };

                        if let Some(stmt) = self.create_use_directive_call(
                            element_var,
                            directive_name,
                            &expressions[expr_index],
                        ) {
                            stmts.push(stmt);
                        }
                        expr_index += 1;
                    }
                }
                SlotType::ClassName(class_name) => {
                    // Generate className call
                    self.add_import("className");

                    if expr_index < expressions.len() {
                        let element_var = if slot.path.is_empty() {
                            root_var
                        } else {
                            path_to_var
                                .get(&slot.path)
                                .map(|s| s.as_str())
                                .unwrap_or(root_var)
                        };

                        if let Some(stmt) = self.create_class_name_call(
                            element_var,
                            class_name,
                            &expressions[expr_index],
                        ) {
                            stmts.push(stmt);
                        }
                        expr_index += 1;
                    }
                }
                SlotType::StyleProperty(property_name) => {
                    // Generate setStyleProperty call
                    self.add_import("setStyleProperty");
                    self.add_import("effect");

                    if expr_index < expressions.len() {
                        let element_var = if slot.path.is_empty() {
                            root_var
                        } else {
                            path_to_var
                                .get(&slot.path)
                                .map(|s| s.as_str())
                                .unwrap_or(root_var)
                        };

                        if let Some(stmt) = self.create_set_style_property_call(
                            element_var,
                            property_name,
                            &expressions[expr_index],
                        ) {
                            stmts.push(stmt);
                        }
                        expr_index += 1;
                    }
                }
                SlotType::Ref => {
                    // Generate ref binding: _$use(ref, element) or conditional assignment
                    self.add_import("use");

                    if expr_index < expressions.len() {
                        let element_var = if slot.path.is_empty() {
                            root_var
                        } else {
                            path_to_var
                                .get(&slot.path)
                                .map(|s| s.as_str())
                                .unwrap_or(root_var)
                        };

                        if let Some(stmt) =
                            self.create_ref_call(element_var, &expressions[expr_index])
                        {
                            stmts.push(stmt);
                        }
                        expr_index += 1;
                    }
                }
                SlotType::ClassList => {
                    // Generate classList call: _$classList(element, {...})
                    self.add_import("classList");

                    if expr_index < expressions.len() {
                        let element_var = if slot.path.is_empty() {
                            root_var
                        } else {
                            path_to_var
                                .get(&slot.path)
                                .map(|s| s.as_str())
                                .unwrap_or(root_var)
                        };

                        if let Some(stmt) =
                            self.create_class_list_call(element_var, &expressions[expr_index])
                        {
                            stmts.push(stmt);
                        }
                        expr_index += 1;
                    }
                }
                SlotType::StyleObject => {
                    // Generate style object call: _$style(element, {...})
                    self.add_import("style");

                    if expr_index < expressions.len() {
                        let element_var = if slot.path.is_empty() {
                            root_var
                        } else {
                            path_to_var
                                .get(&slot.path)
                                .map(|s| s.as_str())
                                .unwrap_or(root_var)
                        };

                        if let Some(stmt) =
                            self.create_style_object_call(element_var, &expressions[expr_index])
                        {
                            stmts.push(stmt);
                        }
                        expr_index += 1;
                    }
                }
                SlotType::Spread => {
                    // Spread attributes - generate _$spread call
                    self.add_import("spread");

                    if expr_index < expressions.len() {
                        let element_var = if slot.path.is_empty() {
                            root_var
                        } else {
                            path_to_var
                                .get(&slot.path)
                                .map(|s| s.as_str())
                                .unwrap_or(root_var)
                        };

                        if let Some(stmt) =
                            self.create_spread_call(element_var, &expressions[expr_index])
                        {
                            stmts.push(stmt);
                        }
                        expr_index += 1;
                    }
                }
            }
        }

        stmts
    }

    /// Extract all dynamic expressions from JSX element in order (cloning them)
    pub(super) fn extract_expressions_from_jsx(
        &self,
        jsx_elem: &JSXElement<'a>,
        expressions: &mut Vec<Expression<'a>>,
    ) {
        use oxc_allocator::CloneIn;

        // First extract expressions from attributes
        for attr in &jsx_elem.opening_element.attributes {
            if let JSXAttributeItem::Attribute(attr) = attr {
                if let Some(JSXAttributeValue::ExpressionContainer(container)) = &attr.value {
                    match &container.expression {
                        JSXExpression::StringLiteral(_)
                        | JSXExpression::NumericLiteral(_)
                        | JSXExpression::EmptyExpression(_) => {
                            // Static or empty - skip
                        }
                        expr => {
                            // Dynamic attribute expression
                            if let Some(expr_ref) = expr.as_expression() {
                                expressions.push(expr_ref.clone_in(self.allocator));
                            }
                        }
                    }
                }
            }
        }

        // Then walk through children and extract expressions
        for child in &jsx_elem.children {
            self.extract_expressions_from_child(child, expressions);
        }
    }

    /// Extract expressions from a JSX child (cloning them)
    pub(super) fn extract_expressions_from_child(
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
    pub(super) fn create_return_statement(&self, root_var: &str) -> Statement<'a> {
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
    pub(super) fn create_import_statements(&self) -> Vec<Statement<'a>> {
        use oxc_ast::ast::*;

        let mut statements = Vec::new();

        // Sort imports by priority using the compat module
        let mut sorted_imports: Vec<_> = self.required_imports.iter().collect();
        sorted_imports.sort_by_key(|name| get_import_priority(name));

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
    pub(super) fn create_delegate_events_call(&self) -> Option<Statement<'a>> {
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
    pub(super) fn clone_expression(&self, expr: &Expression<'a>) -> Expression<'a> {
        use oxc_allocator::CloneIn;
        // Use CloneIn trait for deep cloning
        expr.clone_in(self.allocator)
    }

    /// Wrap expressions with _$memo() for reactivity in fragments
    /// - Call expressions with no args -> _$memo(callee) (unwrap the call)
    /// - Call expressions (except IIFEs, templates, components) -> _$memo(expr)
    /// - Other complex expressions (member access, etc.) -> _$memo(() => expr)
    /// - Simple expressions (identifiers, literals) -> as-is
    pub(super) fn maybe_wrap_with_memo(&mut self, expr: Expression<'a>) -> Expression<'a> {
        use oxc_allocator::CloneIn;
        use oxc_ast::ast::*;

        match &expr {
            Expression::CallExpression(call_expr) => {
                // Check if this is an IIFE (immediately invoked function expression)
                let is_iife = matches!(
                    call_expr.callee,
                    Expression::FunctionExpression(_)
                        | Expression::ArrowFunctionExpression(_)
                        | Expression::ParenthesizedExpression(_)
                );

                if is_iife {
                    // Don't wrap IIFEs
                    return expr;
                }

                // Check if this is a template or component call - those shouldn't be wrapped
                use crate::compat::naming::is_template_var;
                if let Expression::Identifier(ident) = &call_expr.callee {
                    if is_template_var(&ident.name) || ident.name.starts_with("_$createComponent") {
                        return expr;
                    }
                }

                // For zero-argument calls, unwrap and pass the callee to memo
                // This transforms {foo()} to _$memo(foo) so memo can call it reactively
                let expr_to_wrap = if call_expr.arguments.is_empty() {
                    // Check if callee is a simple reference (Identifier or MemberExpression)
                    match &call_expr.callee {
                        Expression::Identifier(_)
                        | Expression::StaticMemberExpression(_)
                        | Expression::ComputedMemberExpression(_)
                        | Expression::PrivateFieldExpression(_) => {
                            // Clone the callee and use it as the argument
                            call_expr.callee.clone_in(self.allocator)
                        }
                        _ => {
                            // For other callees (nested calls, etc.), keep the full call expression
                            expr
                        }
                    }
                } else {
                    // Call has arguments, wrap the whole call expression
                    expr
                };

                // Wrap with _$memo
                self.add_import("memo");
                let memo_fn = IdentifierReference {
                    span: SPAN,
                    name: Atom::from("_$memo"),
                    reference_id: None.into(),
                };

                let mut memo_args = OxcVec::new_in(self.allocator);
                memo_args.push(Argument::from(expr_to_wrap));

                let memo_call = CallExpression {
                    span: SPAN,
                    callee: Expression::Identifier(Box::new_in(memo_fn, self.allocator)),
                    arguments: memo_args,
                    optional: false,
                    type_arguments: None,
                    pure: false,
                };

                Expression::CallExpression(Box::new_in(memo_call, self.allocator))
            }
            // Simple expressions that don't need wrapping
            Expression::Identifier(_)
            | Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_) => {
                // Return as-is
                expr
            }
            // Complex expressions (member access, etc.) -> wrap with _$memo(() => expr)
            _ => {
                // Wrap with _$memo(() => expr)
                self.add_import("memo");

                // Create arrow function: () => expr (expression form)
                let arrow_fn = ArrowFunctionExpression {
                    span: SPAN,
                    expression: true, // Expression form, not block
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
                    pure: false,
                    pife: false,
                };

                // Create: _$memo(() => expr)
                let memo_fn = IdentifierReference {
                    span: SPAN,
                    name: Atom::from("_$memo"),
                    reference_id: None.into(),
                };

                let mut memo_args = OxcVec::new_in(self.allocator);
                memo_args.push(Argument::from(Expression::ArrowFunctionExpression(
                    Box::new_in(arrow_fn, self.allocator),
                )));

                let memo_call = CallExpression {
                    span: SPAN,
                    callee: Expression::Identifier(Box::new_in(memo_fn, self.allocator)),
                    arguments: memo_args,
                    optional: false,
                    type_arguments: None,
                    pure: false,
                };

                Expression::CallExpression(Box::new_in(memo_call, self.allocator))
            }
        }
    }
    pub(super) fn create_insert_call_with_marker(
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
}
