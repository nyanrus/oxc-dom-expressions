//! Traverse trait implementation
//!
//! This module implements the oxc Traverse trait for DomExpressions,
//! providing hooks for AST traversal and transformation.

use oxc_allocator::{Box, Vec as OxcVec};
use oxc_ast::ast::*;
use oxc_span::SPAN;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::template::SlotType;
use crate::utils::{is_component, should_delegate_event};

use super::DomExpressions;

impl<'a> Traverse<'a, ()> for DomExpressions<'a> {
    fn enter_program(&mut self, _program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a, ()>) {
        use crate::options::GenerateMode;

        // Entry point for the transformation
        // Initialize state for collecting templates and imports
        self.templates.clear();
        self.template_map.clear();
        self.template_counter = 0;
        self.element_counter = 0; // Reset to 0
        self.first_root_generated = false;
        self.required_imports.clear();
        self.delegated_events.clear();

        // Add the template import (will be needed for any JSX)
        // Use "ssr" for SSR mode, "template" for DOM mode
        let import_name = if self.options.generate == GenerateMode::Ssr {
            "ssr"
        } else {
            "template"
        };
        self.add_import(import_name);
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
            program.body = OxcVec::from_iter_in(all_stmts, self.allocator);
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
                SlotType::BoolAttribute(_)
                | SlotType::PropAttribute(_)
                | SlotType::AttrAttribute(_)
                | SlotType::UseDirective(_)
                | SlotType::StyleProperty(_)
                | SlotType::ClassName(_)
                | SlotType::Spread => {
                    // These slot types don't need special import handling here
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
