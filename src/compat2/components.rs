//! Component and fragment transformations
//!
//! This module contains methods for transforming JSX components and fragments:
//! - Component transformation
//! - Component props and children handling
//! - Fragment transformation  
//! - JSX child conversion

use oxc_allocator::Box;
use oxc_allocator::Vec as OxcVec;
use oxc_ast::ast::*;
use oxc_span::SPAN;

use crate::utils::is_component;

use super::DomExpressionsCompat2;

impl<'a> DomExpressionsCompat2<'a> {
    pub(super) fn transform_component(
        &mut self,
        jsx_elem: Box<'a, JSXElement<'a>>,
    ) -> Expression<'a> {
        use oxc_ast::ast::*;

        // Add the createComponent import
        self.add_import("createComponent");

        // Get the component name
        let component_name = match &jsx_elem.opening_element.name {
            JSXElementName::Identifier(ident) => ident.name,
            JSXElementName::IdentifierReference(ident) => ident.name,
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
    pub(super) fn create_component_props(
        &mut self,
        jsx_elem: &JSXElement<'a>,
    ) -> ObjectExpression<'a> {
        use oxc_ast::ast::*;

        let mut properties = OxcVec::new_in(self.allocator);

        // Add attributes as properties
        for attr in &jsx_elem.opening_element.attributes {
            if let JSXAttributeItem::Attribute(jsx_attr) = attr {
                if let JSXAttributeName::Identifier(name_ident) = &jsx_attr.name {
                    let prop_name = name_ident.name;

                    // Get the value
                    let prop_value = if let Some(value) = &jsx_attr.value {
                        match value {
                            JSXAttributeValue::StringLiteral(str_lit) => {
                                // Decode HTML entities in JSX string literal attributes for components
                                let decoded =
                                    crate::utils::decode_html_entities(str_lit.value.as_str());
                                Expression::StringLiteral(Box::new_in(
                                    StringLiteral {
                                        span: SPAN,
                                        value: Atom::from(self.allocator.alloc_str(&decoded)),
                                        raw: None,
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
            // Check if we need a getter for children
            // Getter is needed when we have mixed text and expression children
            let significant_children: Vec<_> = jsx_elem
                .children
                .iter()
                .filter(|child| match child {
                    JSXChild::Text(text) => {
                        let text_value = text.value.as_str();
                        !text_value.trim().is_empty()
                            || (!text_value.contains('\n') && !text_value.is_empty())
                    }
                    _ => true,
                })
                .collect();

            let has_text = significant_children
                .iter()
                .any(|child| matches!(child, JSXChild::Text(_)));
            let has_expression = significant_children.iter().any(|child| {
                matches!(
                    child,
                    JSXChild::ExpressionContainer(_) | JSXChild::Element(_) | JSXChild::Fragment(_)
                )
            });
            let needs_getter = has_text && has_expression && significant_children.len() > 1;

            let children_value = self.create_component_children(&jsx_elem.children);

            let prop_key = PropertyKey::StaticIdentifier(Box::new_in(
                IdentifierName {
                    span: SPAN,
                    name: Atom::from("children"),
                },
                self.allocator,
            ));

            if needs_getter {
                // Create getter: get children() { return [...]; }
                // Create function body with return statement
                let return_stmt = Statement::ReturnStatement(Box::new_in(
                    ReturnStatement {
                        span: SPAN,
                        argument: Some(children_value),
                    },
                    self.allocator,
                ));

                let func_body = FunctionBody {
                    span: SPAN,
                    directives: OxcVec::new_in(self.allocator),
                    statements: OxcVec::from_iter_in([return_stmt], self.allocator),
                };

                let getter_fn = Function {
                    r#type: FunctionType::FunctionExpression,
                    span: SPAN,
                    id: None,
                    generator: false,
                    r#async: false,
                    declare: false,
                    type_parameters: None,
                    this_param: None,
                    params: Box::new_in(
                        FormalParameters {
                            span: SPAN,
                            kind: FormalParameterKind::FormalParameter,
                            items: OxcVec::new_in(self.allocator),
                            rest: None,
                        },
                        self.allocator,
                    ),
                    body: Some(Box::new_in(func_body, self.allocator)),
                    return_type: None,
                    scope_id: Default::default(),
                    pure: false,
                    pife: false,
                };

                let getter_value =
                    Expression::FunctionExpression(Box::new_in(getter_fn, self.allocator));

                properties.push(ObjectPropertyKind::ObjectProperty(Box::new_in(
                    ObjectProperty {
                        span: SPAN,
                        kind: PropertyKind::Get,
                        key: prop_key,
                        value: getter_value,
                        method: false,
                        shorthand: false,
                        computed: false,
                    },
                    self.allocator,
                )));
            } else {
                // Regular property
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
        }

        ObjectExpression {
            span: SPAN,
            properties,
        }
    }

    /// Create children value for a component (can be a single value or array)
    pub(super) fn create_component_children(
        &mut self,
        children: &OxcVec<'a, JSXChild<'a>>,
    ) -> Expression<'a> {
        use oxc_ast::ast::*;

        // Filter out whitespace-only text nodes that contain newlines (formatting whitespace)
        // Keep whitespace without newlines (content whitespace like single spaces)
        let significant_children: Vec<_> = children
            .iter()
            .filter(|child| match child {
                JSXChild::Text(text) => {
                    let text_value = text.value.as_str();
                    // Keep if not empty when trimmed OR if it's whitespace without newlines
                    !text_value.trim().is_empty()
                        || (!text_value.contains('\n') && !text_value.is_empty())
                }
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
    pub(super) fn jsx_child_to_expression(&mut self, child: &JSXChild<'a>) -> Expression<'a> {
        use oxc_allocator::CloneIn;
        use oxc_ast::ast::*;

        match child {
            JSXChild::Text(text) => {
                // Handle text value - only trim if it contains newlines
                let text_value = text.value.as_str();
                let output_text = if text_value.contains('\n') {
                    // Text with newlines should be trimmed
                    text_value.trim()
                } else {
                    // Text without newlines (like single spaces) should be kept as-is
                    text_value
                };

                // Decode HTML entities for component/fragment children
                // This converts &nbsp; to \xA0, &lt; to <, etc.
                let decoded_text = crate::utils::decode_html_entities(output_text);

                Expression::StringLiteral(Box::new_in(
                    StringLiteral {
                        span: SPAN,
                        value: Atom::from(self.allocator.alloc_str(&decoded_text)),
                        raw: None,
                        lone_surrogates: false,
                    },
                    self.allocator,
                ))
            }
            JSXChild::ExpressionContainer(expr_container) => match &expr_container.expression {
                jsx_expr if jsx_expr.is_expression() => {
                    self.clone_expression(jsx_expr.as_expression().unwrap())
                }
                _ => {
                    Expression::NullLiteral(Box::new_in(NullLiteral { span: SPAN }, self.allocator))
                }
            },
            JSXChild::Element(elem) => {
                // Transform JSX element inline
                // Check if this is a component
                let tag_name = match &elem.opening_element.name {
                    JSXElementName::Identifier(ident) => ident.name.as_str(),
                    JSXElementName::IdentifierReference(ident) => ident.name.as_str(),
                    _ => "",
                };

                if is_component(tag_name) {
                    // Transform component - clone and box the element
                    let elem_clone = elem.as_ref().clone_in(self.allocator);
                    let boxed_elem = Box::new_in(elem_clone, self.allocator);
                    self.transform_component(boxed_elem)
                } else {
                    // Build template and transform element
                    let template =
                        crate::template::build_template_with_options(elem, Some(&self.options));
                    let template_var = self.get_template_var(&template.html);

                    let has_dynamic_content = !template.dynamic_slots.is_empty();

                    if has_dynamic_content {
                        // Extract expressions from the element
                        let mut expressions = Vec::new();
                        self.extract_expressions_from_jsx(elem, &mut expressions);

                        // Generate IIFE with dynamic binding code
                        let iife = self.create_template_iife_from_expressions(
                            expressions,
                            &template,
                            &template_var,
                        );
                        Expression::CallExpression(iife)
                    } else {
                        // Simple template call for static content
                        let template_var_str = self.allocator.alloc_str(&template_var);
                        let call_expr = self.create_template_call(template_var_str);
                        Expression::CallExpression(call_expr)
                    }
                }
            }
            JSXChild::Fragment(frag) => {
                // Transform JSX fragment inline - clone and box the fragment
                let frag_clone = frag.as_ref().clone_in(self.allocator);
                let boxed_frag = Box::new_in(frag_clone, self.allocator);
                self.transform_fragment(boxed_frag)
            }
            _ => {
                // For other types (Spread), return null for now
                Expression::NullLiteral(Box::new_in(NullLiteral { span: SPAN }, self.allocator))
            }
        }
    }

    /// Transform a JSX fragment into an array or string
    pub(super) fn transform_fragment(
        &mut self,
        jsx_frag: Box<'a, JSXFragment<'a>>,
    ) -> Expression<'a> {
        use oxc_ast::ast::*;

        // Filter out whitespace-only text nodes that contain newlines
        // Keep single spaces or whitespace without newlines
        let significant_children: Vec<_> = jsx_frag
            .children
            .iter()
            .filter(|child| match child {
                JSXChild::Text(text) => {
                    let text_value = text.value.as_str();
                    // Keep if not empty when trimmed OR if it's a single space without newlines
                    !text_value.trim().is_empty()
                        || (!text_value.contains('\n') && !text_value.is_empty())
                }
                _ => true,
            })
            .collect();

        if significant_children.len() == 1 {
            // Single child - return it directly, wrapping call expressions with _$memo
            let expr = self.jsx_child_to_expression(significant_children[0]);
            self.maybe_wrap_with_memo(expr)
        } else {
            // Multiple children - return as array
            let mut elements = OxcVec::new_in(self.allocator);
            for child in significant_children {
                let expr = self.jsx_child_to_expression(child);
                // Wrap call expressions with _$memo in fragments
                let wrapped_expr = self.maybe_wrap_with_memo(expr);
                elements.push(ArrayExpressionElement::from(wrapped_expr));
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
}
