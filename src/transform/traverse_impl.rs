//! Traverse trait implementation for modern transformer

use oxc_allocator::Box;
use oxc_allocator::Vec as OxcVec;
use oxc_ast::ast::*;
use oxc_span::{Atom, SPAN};
use oxc_traverse::{Traverse, TraverseCtx};

use crate::template::build_template_with_options;

use super::DomExpressions;

impl<'a> Traverse<'a, ()> for DomExpressions<'a> {
    fn exit_expression(&mut self, expr: &mut Expression<'a>, _ctx: &mut TraverseCtx<'a, ()>) {
        // Transform JSX elements to modern format
        if let Expression::JSXElement(jsx_elem) = expr {
            if let Some(transformed) = self.transform_jsx_element_modern(jsx_elem) {
                *expr = transformed;
            }
        }
    }

    fn exit_program(&mut self, program: &mut Program<'a>, _ctx: &mut TraverseCtx<'a, ()>) {
        // Inject imports and template declarations at the top of the program if any templates were created
        if !self.templates.is_empty() {
            let mut new_stmts = Vec::new();
            
            // 1. Add import statement
            new_stmts.push(self.create_modern_import_statement());

            // 2. Add template variable declarations
            let template_decls = self.create_template_declarations();
            new_stmts.extend(template_decls);

            // 3. Prepend new statements to the program
            let existing_stmts =
                std::mem::replace(&mut program.body, OxcVec::new_in(self.allocator));

            // Create new statement list with injected statements first
            let mut all_stmts = new_stmts;
            all_stmts.extend(existing_stmts);

            // Replace program body
            program.body = OxcVec::from_iter_in(all_stmts, self.allocator);
        }
    }
}

impl<'a> DomExpressions<'a> {
    /// Transform a JSX element to modern format
    fn transform_jsx_element_modern(&mut self, jsx_elem: &JSXElement<'a>) -> Option<Expression<'a>> {
        // Build template from JSX
        let template = build_template_with_options(jsx_elem, Some(&self.options));

        // Get HTML (minimized if opt feature is enabled)
        #[cfg(feature = "opt")]
        let html = crate::opt::minimizer::minimize_template(&template.html, &self.options);
        #[cfg(not(feature = "opt"))]
        let html = template.html.clone();

        // Get or create template variable
        let template_var = self.get_template_var(&html);

        // Track this template in optimizer (if opt feature is enabled)
        #[cfg(feature = "opt")]
        self.optimizer.record_template(template.clone());
        
        self.templates.push(template);

        // For now, create a simple IIFE with $clone and return
        // Full implementation would add $bind calls for dynamic content
        let mut statements = Vec::new();

        // const _root$ = $clone(_tmpl$);
        let clone_call = self.create_clone_call(self.allocator.alloc_str(&template_var));
        let root_var = self.allocator.alloc_str("_root$");

        let declarator = VariableDeclarator {
            span: SPAN,
            kind: VariableDeclarationKind::Const,
            id: BindingPattern {
                kind: BindingPatternKind::BindingIdentifier(Box::new_in(
                    BindingIdentifier {
                        span: SPAN,
                        name: Atom::from(root_var),
                        symbol_id: Default::default(),
                    },
                    self.allocator,
                )),
                type_annotation: None,
                optional: false,
            },
            init: Some(clone_call),
            definite: false,
        };

        let mut declarators = OxcVec::new_in(self.allocator);
        declarators.push(declarator);

        statements.push(Statement::VariableDeclaration(Box::new_in(
            VariableDeclaration {
                span: SPAN,
                kind: VariableDeclarationKind::Const,
                declarations: declarators,
                declare: false,
            },
            self.allocator,
        )));

        // TODO: Add $bind calls for dynamic content based on template.dynamic_slots
        // For now, we just return the cloned template without bindings

        // Create IIFE
        let iife = self.create_iife(statements, root_var);
        
        Some(iife)
    }
}
