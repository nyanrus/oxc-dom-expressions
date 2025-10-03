# Code Style and Conventions

## General Rust Conventions
- Follow standard Rust naming conventions
- Use snake_case for functions and variables
- Use CamelCase for types and traits
- Use SCREAMING_SNAKE_CASE for constants

## Code Formatting
- **Always** run `cargo fmt` before committing
- Use 4-space indentation (handled by rustfmt)
- Maximum line length: 100 characters (rustfmt default)

## Documentation
- Add doc comments (`///`) for all public APIs
- Use `//!` for module-level documentation
- Include examples in doc comments where helpful
- Document panic conditions with `# Panics` section
- Document error conditions with `# Errors` section

## Code Organization
- Keep functions focused and single-purpose
- Group related functionality together
- Use meaningful variable names
- Avoid deep nesting (prefer early returns)

## Error Handling
- Prefer `Result<T, E>` over panics for recoverable errors
- Use `Option<T>` for optional values
- Use `expect()` with descriptive messages when panicking is acceptable
- Use `unwrap()` sparingly and only when certain

## Testing
- Write unit tests for new functions
- Add integration tests for new features
- Use descriptive test names: `test_<functionality>_<scenario>`
- Aim for good test coverage
- Keep tests readable and maintainable

## Specific to This Project

### Allocator Usage
- Use `oxc_allocator::Box::new_in(value, self.allocator)` for heap allocations
- Use `OxcVec::new_in(self.allocator)` for vectors
- Store `&'a Allocator` reference in structs

### AST Manipulation
- Use `oxc_span::SPAN` for placeholder spans
- Use `Atom::from()` for creating interned strings
- Clone AST nodes with `clone_in(allocator)` when needed

### Naming Patterns
- Template variables: `_tmpl$`, `_tmpl$2`, etc.
- Element variables: `_el$`, `_el$2`, etc.
- Runtime function names match babel plugin: `_$insert`, `_$setAttribute`, etc.

### Comments
- Use `//` for single-line comments
- Use `/* */` for multi-line comments
- Mark pure functions with `/*#__PURE__*/` for tree-shaking
