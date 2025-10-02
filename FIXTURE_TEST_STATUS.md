# Fixture Test Status

This document tracks the status of fixture tests from the original babel-plugin-jsx-dom-expressions.

## Test Results Summary

**Last Updated**: Current PR

### Overall Status
- **Total Tests**: 32 fixture tests (11 DOM + 9 SSR + 12 Hydratable)
- **Passing**: 1 test (3%)
- **Failing**: 31 tests (97%)

### By Test Suite

#### DOM Fixtures (11 tests)
- ✅ **test_simple_elements** - PASSING
- ❌ test_attribute_expressions
- ❌ test_components
- ❌ test_conditional_expressions
- ❌ test_custom_elements
- ❌ test_event_expressions
- ❌ test_fragments
- ❌ test_insert_children
- ❌ test_namespace_elements
- ❌ test_svg
- ❌ test_text_interpolation

#### SSR Fixtures (9 tests)
- ❌ test_ssr_simple_elements
- ❌ test_ssr_attribute_expressions
- ❌ test_ssr_components
- ❌ test_ssr_conditional_expressions
- ❌ test_ssr_custom_elements
- ❌ test_ssr_fragments
- ❌ test_ssr_insert_children
- ❌ test_ssr_svg
- ❌ test_ssr_text_interpolation

#### Hydratable Fixtures (12 tests)
- ❌ test_hydratable_simple_elements
- ❌ test_hydratable_components
- ❌ test_hydratable_conditional_expressions
- ❌ test_hydratable_custom_elements
- ❌ test_hydratable_document
- ❌ test_hydratable_event_expressions
- ❌ test_hydratable_flags
- ❌ test_hydratable_fragments
- ❌ test_hydratable_insert_children
- ❌ test_hydratable_svg
- ❌ test_hydratable_text_interpolation
- (1 more test)

## Recent Changes

### What Was Fixed ✅

1. **JSX Comment Handling** (src/template.rs)
   - JSXEmptyExpression nodes (JSX comments) are now properly skipped
   - Before: Created unnecessary dynamic slots
   - After: Comments are ignored as expected

2. **Template String Escaping** (src/template.rs)
   - Only escape `{` not `}` in template literals
   - Matches babel plugin behavior
   - Before: `\{ ... \}` (both escaped)
   - After: `\{ ... }` (only opening brace escaped)

3. **IIFE Structure** (src/transform.rs)
   - Implemented basic IIFE generation: `(() => { ... })()`
   - Generate element reference declarations: `var _el$ = _tmpl$()`
   - Generate return statements: `return _el$`
   - Structure is correct but bodies are empty (no runtime calls yet)

4. **Test Infrastructure** (tests/*.rs)
   - Added output normalization to ignore formatting differences
   - Applied to all test suites (DOM, SSR, Hydratable)
   - Better diff reporting for debugging
   - Pure comment format: `/*#__PURE__*/` vs `/* @__PURE__ */`

### What Still Needs Work ❌

#### 1. Dynamic Content Generation (Critical)
The main blocker is generating runtime calls inside IIFEs:
```javascript
// Expected:
(() => {
  var _el$ = _tmpl$();
  _$insert(_el$, name, null);  // ← Missing
  return _el$;
})()

// Currently generated:
(() => {
  var _el$ = _tmpl$();
  // Empty - no runtime calls
  return _el$;
})()
```

**Why it's hard**: Template struct doesn't store JSX expressions, only slot types and paths. Need expressions to generate calls like `_$insert(_el$, name, null)`.

#### 2. SSR Mode (High Priority)
SSR requires completely different code generation:
- String literals `"..."` instead of template literals `` `...` ``
- Import: `import { ssr as _$ssr }` instead of `{ template as _$template }`
- Full HTML (not minimalized): `<div></div>` not `<div>`
- Direct calls: `_$ssr(_tmpl$)` not `_tmpl$()`

#### 3. Component Transformation
Components need to be transformed to `_$createComponent` calls:
```javascript
// Input: <Component prop={value} />
// Output: _$createComponent(Component, { prop: value })
```

#### 4. Fragment Transformation
Fragments need to be converted to arrays:
```javascript
// Input: <>{a}{b}</>
// Output: [a, b]
```

#### 5. Template HTML Issues
- Multiline text handling
- Whitespace preservation
- Special character escaping
- HTML entities

## Architecture Notes

### Current Design
```
JSX → Parser → Template (HTML + slots) → Code Generator → Output
                  ↑
                  └─ Loses expression information
```

### Issue
The Template struct stores:
- ✅ HTML string
- ✅ Dynamic slot positions and types
- ❌ Actual JSX expressions

Without expressions, can't generate:
- `_$insert(_el$, name, null)` - need `name` expression
- `_$setAttribute(_el$, "id", state.id)` - need `state.id` expression
- `_$effect(() => ...)` - need effect body

### Potential Solutions

#### Option 1: Store Expressions in Template
```rust
pub struct DynamicSlot {
    pub path: Vec<String>,
    pub slot_type: SlotType,
    pub expression: Option<Box<Expression<'a>>>,  // ← Add this
}
```

#### Option 2: Two-Pass Generation
1. First pass: Build template and collect expressions
2. Second pass: Generate code with expressions

#### Option 3: Generate During Traversal
Generate runtime calls immediately during JSX traversal instead of collecting templates first.

## Next Steps

To get more tests passing:

1. **Immediate** (1-2 tests):
   - Extract and store JSX expressions during template building
   - Implement basic `_$insert` call generation for text content
   - Should make text_interpolation tests start passing

2. **Short-term** (5-10 tests):
   - Implement attribute handling (_$setAttribute, _$effect)
   - Implement event handlers
   - Implement ref binding
   - Will make attribute_expressions, event_expressions tests pass

3. **Medium-term** (15-20 tests):
   - Implement SSR code generation path
   - Implement component transformation
   - Implement fragment transformation
   - Will make SSR and component tests pass

4. **Long-term** (all tests):
   - Implement hydratable markers
   - Handle all edge cases (conditionals, loops, spread, etc.)
   - Fix remaining template HTML issues

## Testing Strategy

### Running Tests
```bash
# All fixture tests
cargo test --test dom_fixtures --test ssr_fixtures --test hydratable_fixtures

# Specific suite
cargo test --test dom_fixtures

# Specific test
cargo test --test dom_fixtures test_simple_elements -- --nocapture

# All unit tests (should always pass)
cargo test --lib
```

### Adding New Tests
When adding new fixture tests:
1. Add code.js and output.js to `tests/fixtures/{mode}/{category}/`
2. Add test function to appropriate test file
3. Use existing test pattern with compare_outputs
4. Output normalization handles formatting differences automatically

## References

- Original babel plugin: https://github.com/ryansolid/dom-expressions
- Fixture tests location: `tests/fixtures/{dom,ssr,hydratable}/*/`
- Test runner: `tests/{dom,ssr,hydratable}_fixtures.rs`
- Core transformation: `src/transform.rs`
- Template building: `src/template.rs`
