# Test Fixtures

This directory contains test fixtures copied from the [babel-plugin-jsx-dom-expressions](https://github.com/ryansolid/dom-expressions) repository to ensure compatibility and correctness of the oxc-dom-expressions implementation.

## Structure

The fixtures are organized by transformation mode:

### `dom/` - Standard DOM Mode Tests
Contains 11 test categories covering the default DOM transformation mode:
- **simpleElements**: Basic HTML element transformation
- **eventExpressions**: Event handler bindings (onClick, on:custom, etc.)
- **attributeExpressions**: Attribute and property bindings (class, style, ref, etc.)
- **fragments**: JSX fragment (`<></>`) handling
- **textInterpolation**: Text content and interpolation
- **components**: Component detection and handling
- **conditionalExpressions**: Conditional rendering patterns
- **insertChildren**: Dynamic child insertion
- **customElements**: Web Components support
- **SVG**: SVG element handling
- **namespaceElements**: Namespaced elements

### `ssr/` - Server-Side Rendering Mode Tests
Contains 9 test categories for SSR (Server-Side Rendering) mode:
- **simpleElements**: SSR for basic elements
- **attributeExpressions**: SSR for attributes
- **fragments**: SSR for fragments
- **textInterpolation**: SSR for text content
- **components**: SSR for components
- **conditionalExpressions**: SSR for conditionals
- **insertChildren**: SSR for dynamic children
- **customElements**: SSR for custom elements
- **SVG**: SSR for SVG

### `hydratable/` - Hydratable Mode Tests
Contains 12 test categories for hydratable DOM mode (used with SSR):
- All categories from DOM mode, plus:
- **flags**: Hydration flags and markers
- **document**: Document-level hydration

## Test Format

Each test category contains:
- `code.js`: Input JSX code to transform
- `output.js`: Expected JavaScript output from the babel plugin

## Current Implementation Status

The oxc-dom-expressions implementation currently:
- ✅ **Parses all fixture test inputs** without errors
- ✅ **Transforms all JSX structures** through the transformer pipeline
- ⚠️ **Does not yet generate complete output** matching the expected output files
  - Full AST transformation and code generation is still in progress
  - The current tests verify that the transformer can handle all input patterns

## Testing Approach

The test suites (`dom_fixtures.rs`, `ssr_fixtures.rs`, `hydratable_fixtures.rs`) currently:
1. Load each fixture's input code
2. Parse the JSX using oxc_parser
3. Run the DomExpressions transformer
4. Verify that transformation completes without errors

Future enhancements will include:
- Full output comparison with expected results
- AST structure validation
- Generated code verification

## Running the Tests

```bash
# Run all fixture tests
cargo test --test dom_fixtures --test ssr_fixtures --test hydratable_fixtures

# Run specific fixture test suite
cargo test --test dom_fixtures
cargo test --test ssr_fixtures
cargo test --test hydratable_fixtures
```

## Compatibility Notes

These fixtures ensure that oxc-dom-expressions maintains compatibility with the babel plugin's behavior. The test files are exact copies from the babel plugin repository, ensuring we test against the same cases.

### Key Differences

While we use the same input fixtures, there are some expected differences in implementation:
- oxc-dom-expressions uses Rust and the oxc parser/AST
- babel-plugin-jsx-dom-expressions uses JavaScript and Babel's parser/AST
- Both target the same transformation semantics for Solid.js

## References

- Original babel plugin: https://github.com/ryansolid/dom-expressions
- Test fixtures source: `packages/babel-plugin-jsx-dom-expressions/test/`
