# Implementation Status for dom-expressions Fixture Tests

## Current State

### Working ✅
- Template HTML generation with static text preservation
- `<!>` marker nodes properly parsed and preserved
- Template minimalization (closing tag omission)
- Basic IIFE structure generation
- Insert call generation (without marker navigation)
- Template deduplication
- Import statement generation

### Partially Working ⚠️
- **Text interpolation**: Templates correct, but insert calls lack marker positioning
  - Example: `<span><!>John` template is correct
  - Missing: `_el$2 = _el$.firstChild` navigation to marker
  - Missing: Using marker as insertion point `_$insert(_el$, expr, _el$2)`

### Not Yet Implemented ❌
- **Component transformation**: JSX components not converted to `_$createComponent`
- **Fragment transformation**: JSX fragments not converted to arrays
- **Attribute expressions**: Dynamic attributes not generating `_$setAttribute` calls
- **Event handlers**: Events not generating delegation or addEventListener
- **Ref bindings**: ref= attributes not handled
- **ClassList/Style bindings**: Special bindings not implemented

## Technical Debt

### Marker Navigation Issue
The core blocker is that marker nodes (`<!>`) create DOM elements in the template, and we need to:
1. Track the DOM path to each marker during template building
2. Generate element reference declarations to navigate to markers
3. Use marker references as insertion points in `_$insert` calls

**Current approach**: Dynamic slots track `path` but this doesn't account for markers as DOM nodes

**Needed approach**: 
- When adding `<!>`, update path tracking to account for the marker as a DOM node
- Generate navigation code: `_el$2 = _el$.firstChild.nextSibling` etc.
- Map each dynamic slot to its marker reference

### Path Tracking Complexity
The template building uses a `path` vector to track DOM navigation (firstChild, nextSibling).
However, markers add complexity:
- A `<!>` marker becomes a real DOM node when template is cloned
- Navigation must account for: text nodes, marker nodes, and element nodes
- Multiple markers in sequence create a chain of nextSibling navigations

## Proposed Solution Path

1. **Enhanced template building**:
   ```rust
   pub struct DynamicSlot {
       pub path: Vec<String>,
       pub slot_type: SlotType,
       pub marker_path: Option<Vec<String>>, // Path to the marker node itself
   }
   ```

2. **Marker path calculation**:
   - When adding `<!>`, calculate and store the path to reach that marker
   - Update path tracking after adding each child (text, element, or marker)

3. **Element reference generation**:
   - For each unique marker path, generate element reference
   - Create map from dynamic slot to marker variable

4. **Insert call generation**:
   - Use marker variable as third parameter to `_$insert`
   - For trailing expressions, use `null`
   - For leading/middle expressions, use marker reference

## Test Results

### Passing (1/11 DOM tests)
- ✅ test_simple_elements - Static templates only, no dynamic content

### Failing Due to Insert Logic (estimated 4-6 tests fixable)
- ❌ test_text_interpolation - Needs marker navigation
- ❌ test_insert_children - Needs marker navigation
- ❌ test_namespace_elements - Likely needs marker navigation
- ❌ test_svg - Likely needs marker navigation

### Failing Due to Missing Features (5-7 tests)
- ❌ test_components - Needs component transformation
- ❌ test_fragments - Needs fragment transformation
- ❌ test_attribute_expressions - Needs attribute handling
- ❌ test_event_expressions - Needs event handling
- ❌ test_conditional_expressions - Complex, needs conditionals
- ❌ test_custom_elements - May overlap with components

## Effort Estimate

### High Priority (to get 5+ tests passing)
- **Marker navigation**: 4-6 hours
  - Redesign path tracking in template building
  - Generate correct element references
  - Update insert call generation

### Medium Priority (to get 8+ tests passing)
- **Component transformation**: 2-3 hours
- **Fragment transformation**: 1-2 hours
- **Attribute handling**: 2-3 hours

### Lower Priority
- **Event handling**: 2-3 hours
- **Conditional expressions**: 3-4 hours (complex)
- **SSR/Hydratable modes**: 4-6 hours

## Recommendations

1. **Short-term**: Focus on marker navigation to unlock multiple tests
2. **Medium-term**: Implement components and fragments
3. **Long-term**: Add remaining runtime call types (attributes, events, etc.)
4. **Testing**: Use individual fixture tests to validate each feature incrementally

## References

- Original babel plugin: https://github.com/ryansolid/dom-expressions
- Expected output in: `tests/fixtures/dom/*/output.js`
- Template building: `src/template.rs`
- Code generation: `src/transform.rs` and `src/codegen.rs`
