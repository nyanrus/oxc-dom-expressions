# Spread Attributes Implementation Notes

## Current Status
Spread attributes are partially implemented with a placeholder `SlotType::Spread` enum variant, but the full implementation is missing.

## What Spread Attributes Do
When JSX contains spread attributes like `<div {...props} foo="bar">`, the transformation needs to:

1. **Merge Props**: Combine the spread expression with other attributes using `_$mergeProps`
2. **Apply to Element**: Use `_$spread` to apply the merged props to the element
3. **Attribute Ordering**: Attributes BEFORE the first spread go in the template HTML; attributes AFTER spreads become part of the merge object

## Example Transformation
Input:
```jsx
<div id="main" {...results} foo disabled>
```

Expected Output:
```javascript
(() => {
  var _el$ = _tmpl$(); // _tmpl$ = `<div id=main>`
  _$spread(_el$, _$mergeProps(results, {
    foo: "",
    disabled: true
  }), false, true);
  return _el$;
})()
```

## Implementation Requirements

### 1. Template Builder Changes (src/template.rs)
- Scan for spread attributes first
- Track position of first spread
- Only add attributes BEFORE first spread to HTML
- Create Spread slots for each spread expression
- Collect attributes AFTER spreads for merging

### 2. Transform Changes (src/transform.rs)
- Implement `create_spread_call` method
- Implement `create_merge_props_call` method
- Build object expressions for static attributes
- Handle dynamic attributes with getters
- Add `spread` and `mergeProps` imports

### 3. Expression Extraction
- Extract spread expressions separately from regular attributes
- Build props object with static and dynamic properties
- Support nested objects (classList, style) in merge

## Challenges
1. **Attribute Grouping**: Need to track which attributes come before/after spreads
2. **Object Building**: Creating proper AST nodes for props objects is complex
3. **Dynamic Properties**: Some attributes need getters (e.g., `get title() { return welcoming(); }`)
4. **Multiple Spreads**: Handle multiple spread attributes on same element
5. **Special Attributes**: classList, style, ref, etc. need special handling in merge objects

## Files to Modify
- `src/template.rs` - Template building logic
- `src/transform.rs` - Add create_spread_call, create_merge_props_call methods
- `src/utils.rs` - Helper functions for attribute classification

## Priority
HIGH - This is blocking 3 out of 5 DOM mode tests and likely most hydratable/SSR tests.
