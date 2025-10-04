# Code Style and Conventions

## Rust Conventions
- Use Rust 2021 edition
- Follow standard Rust naming conventions:
  - snake_case for functions, variables, modules
  - PascalCase for types, traits, structs
  - SCREAMING_SNAKE_CASE for constants
- Use `'a` lifetime annotation for allocator references
- Use `Box::new_in` for heap allocation with oxc allocator

## Documentation
- Use `///` for public API documentation
- Use `//!` for module-level documentation
- Include examples in doc comments where appropriate

## Code Organization
- Group related functionality in impl blocks
- Use helper methods to reduce code duplication
- Separate concerns: transform, codegen, template generation

## oxc-specific Patterns
- Always pass allocator as `&'a Allocator`
- Use `SPAN` constant for generated AST nodes
- Clone AST nodes with `clone_in(allocator)`
- Use oxc's Atom for string interning

## Error Handling
- Return `Result<String, String>` for transformation functions
- Collect parse errors and return as formatted strings
- Use descriptive error messages

## Testing
- Use fixture-based testing to match babel plugin output
- Normalize outputs for comparison (whitespace, variable names)
- Test all three modes: DOM, SSR, Hydratable
- Include test name in panic messages for debugging
