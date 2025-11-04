# Babel Compatibility Implementation Plan

## Executive Summary

This document provides a detailed implementation plan for achieving 100% compatibility with babel-plugin-jsx-dom-expressions. Based on analysis of the babel source code in `/tmp/dom-expressions`, this plan outlines the specific changes needed.

## Current Status

- **Test Pass Rate**: 20% (1/5 DOM fixture tests passing)
- **Functional Compatibility**: ~80-90%
- **Output Compatibility**: ~60% (missing static optimizations)

### What's Working ✅
- Basic JSX transformation
- Template generation with cloneNode optimization
- Event delegation
- Component detection and transformation
- Fragment support
- Import ordering
- Dynamic attributes and expressions

### What's Missing ❌
- Static expression evaluation and inlining
- bool: namespace attribute optimization
- innerHTML/textContent static detection
- Style object inlining
- classList static evaluation

## Implementation Phases

### Phase 1: Static Expression Evaluator (COMPLETED ✅)

**Status**: Implemented in `src/static_evaluator.rs`

**Capabilities**:
- Evaluates Boolean, String, Number, Null, Undefined literals
- Handles unary expressions (!, -, +)
- Handles binary expressions (+, -, *, /)
- Evaluates object expressions with literal properties
- Evaluates template literals without expressions
- All 11 unit tests passing

**Usage**:
```rust
use crate::static_evaluator::{evaluate_expression, EvaluatedValue};

let result = evaluate_expression(expr);
if result.confident {
    // Can inline this value
    match result.value {
        Some(EvaluatedValue::Boolean(b)) => /* use boolean */,
        Some(EvaluatedValue::String(s)) => /* use string */,
        // ...
    }
}
```

### Phase 2: bool: Attribute Static Evaluation (TODO)

**Babel Reference**: `packages/babel-plugin-jsx-dom-expressions/src/dom/element.js` lines 912-958

**Implementation Details**:

1. **Location**: Modify `src/template.rs` and `src/transform/traverse_impl.rs`

2. **Logic** (from Babel):
```javascript
// When encountering bool:disabled={expr}
if (key.slice(0, 5) === "bool:") {
  let content = value.expression;
  
  switch (content.type) {
    case "StringLiteral":
      if (content.value.length && content.value !== "0") {
        // Add attribute to template
        template += ` ${key.slice(5)}`;
      }
      break;
    case "NullLiteral":
      // Omit attribute
      break;
    case "BooleanLiteral":
      if (content.value) {
        // Add attribute to template
        template += ` ${key.slice(5)}`;
      }
      break;
    case "Identifier":
      if (content.name === "undefined") {
        // Omit attribute
        break;
      }
      // Fall through to dynamic handling
      break;
    default:
      // Dynamic - generate setBoolAttribute call
      results.exprs.push(setAttr(...));
  }
}
```

3. **Implementation Steps**:
   a. In `src/template.rs`, when processing JSX attributes:
      - Check if attribute name starts with "bool:"
      - Get the JSXExpressionContainer value
      - Use `evaluate_expression()` to check if it's static
      - If confident:
        * `EvaluatedValue::Boolean(true)` → add attribute name to template HTML
        * `EvaluatedValue::Boolean(false)` → omit attribute entirely
        * `EvaluatedValue::String(s)` if non-empty and not "0" → add to template
        * `EvaluatedValue::Null` or `EvaluatedValue::Undefined` → omit
      - If not confident: treat as dynamic (current behavior)
   
   b. Update template HTML generation to include/exclude the attribute
   
   c. Don't create a dynamic slot for static bool: attributes

**Expected Impact**: ~30-40% improvement in test compatibility

**Test Cases**:
```jsx
// Should inline in template
<div bool:disabled={true}>  // → <div disabled>
<div bool:quack={"hello"}>  // → <div quack>
<div bool:quack={1}>        // → <div quack>

// Should omit from template
<div bool:disabled={false}> // → <div>
<div bool:disabled={null}>  // → <div>
<div bool:disabled={undefined}> // → <div>
<div bool:disabled={""}>    // → <div>
<div bool:disabled={"0"}>   // → <div>

// Should be dynamic
<div bool:disabled={someVar}> // → runtime setBoolAttribute call
```

### Phase 3: innerHTML/textContent Static Detection (TODO)

**Babel Reference**: `packages/babel-plugin-jsx-dom-expressions/src/dom/element.js` lines 608-618

**Implementation Details**:

1. **Location**: Modify `src/template.rs`

2. **Logic** (from Babel):
```javascript
if (t.isJSXExpressionContainer(value) && !key.startsWith("use:")) {
  const evaluated = attribute.get("value").get("expression").evaluate().value;
  let type;
  if (evaluated !== undefined &&
      ((type = typeof evaluated) === "string" || type === "number")) {
    // Convert to string literal for template inlining
    if (type === "number" && (Properties.has(key) || key.startsWith("prop:"))) {
      value = t.jsxExpressionContainer(t.numericLiteral(evaluated));
    } else {
      value = t.stringLiteral(String(evaluated));
    }
  }
}
```

3. **Implementation Steps**:
   a. When processing `innerHTML` or `textContent` attributes:
      - Evaluate the expression
      - If confident and value is String or Number:
        * For `innerHTML`: HTML-escape the string and add to template
        * For `textContent`: Add text content directly to template
      - If not confident: treat as dynamic
   
   b. Proper HTML escaping for innerHTML values

**Test Cases**:
```jsx
<div innerHTML={"<div/>"} />  // → <div><div/></div> (in template)
<div textContent={"Hello"} /> // → <div>Hello</div> (in template)
<div innerHTML={dynamicHTML} /> // → runtime assignment
```

### Phase 4: Style Object Inlining (TODO)

**Babel Reference**: `packages/babel-plugin-jsx-dom-expressions/src/dom/element.js` lines 344-412

**Implementation Details**:

1. **Location**: Modify `src/template.rs` and create style converter

2. **Logic** (from Babel):
```javascript
const styleAttributes = attributes.filter(a => a.node.name && a.node.name.name === "style");
if (styleAttributes.length > 0) {
  let inlinedStyle = "";
  
  for (let attr of styleAttributes) {
    let value = attr.node.value.expression;
    
    if (t.isStringLiteral(value)) {
      inlinedStyle += `${value.value.replace(/;$/, "")};`;
      attr.remove();
    } else if (t.isObjectExpression(value)) {
      for (let property of value.properties) {
        if (!property.computed) {
          const key = property.key.name || property.key.value;
          if (t.isStringLiteral(property.value) || t.isNumericLiteral(property.value)) {
            inlinedStyle += `${key}:${property.value.value};`;
            // Remove from object
          } else {
            const r = property.value.evaluate();
            if (r.confident && (typeof r.value === "string" || typeof r.value === "number")) {
              inlinedStyle += `${key}:${r.value};`;
              // Remove from object
            }
          }
        }
      }
    }
  }
}
```

3. **Implementation Steps**:
   a. Detect `style` attribute with ObjectExpression value
   
   b. Iterate over object properties:
      - If property value is a literal (string/number), inline it
      - If property value can be evaluated statically, inline it
      - If property has computed key but value is evaluatable, keep in dynamic object
      - Remove inlined properties from the object
   
   c. Convert inlined properties to CSS string: `"key:value;key2:value2"`
   
   d. Add CSS string to template as `style="..."` attribute
   
   e. If any properties remain dynamic, generate runtime `style()` call for those

**Test Cases**:
```jsx
// Fully static - inline in template
<div style={{ color: "red", size: 12 }} />
// → <div style="color:red;size:12">

// Partially static - inline what we can
<div style={{ color: "red", border: someVar }} />
// → <div style="color:red"> + runtime style() call for border

// Fully dynamic
<div style={dynamicStyles} />
// → runtime style() call
```

### Phase 5: classList Static Evaluation (TODO)

**Babel Reference**: `packages/babel-plugin-jsx-dom-expressions/src/dom/element.js` lines 453-505

**Implementation Details**:

1. **Location**: Modify `src/template.rs`

2. **Logic** (from Babel):
```javascript
const classListAttribute = attributes.find(
  a => a.node.name && a.node.name.name === "classList" &&
       t.isObjectExpression(a.node.value.expression)
);
if (classListAttribute) {
  classListProperties.forEach(propPath => {
    const { confident, value: computed } = propPath.get("value").evaluate();
    if (!confident) {
      // Convert to class:name dynamic binding
    } else if (computed) {
      // Add class name to template
      path.get("openingElement").node.attributes.push(
        t.jsxAttribute(
          t.jsxIdentifier("class"),
          t.stringLiteral(property.key.name)
        )
      );
    }
    // If computed is false, omit the class
  });
}
```

3. **Implementation Steps**:
   a. Detect `classList={{ ... }}` with object expression
   
   b. For each property:
      - Evaluate the value
      - If confident and truthy: add class name to template's class attribute
      - If confident and falsy: omit class name
      - If not confident: keep as dynamic classList binding
   
   c. Combine static class names with existing class attribute

**Test Cases**:
```jsx
// All static truthy - add to template
<div classList={{ a: true, b: true }} />
// → <div class="a b">

// Mixed static/dynamic
<div classList={{ a: true, b: someVar }} />
// → <div class="a"> + runtime classList for b

// All static falsy - no class attribute
<div classList={{ a: false, b: null }} />
// → <div>
```

### Phase 6: Attribute Expression Evaluation (TODO)

**Babel Reference**: `packages/babel-plugin-jsx-dom-expressions/src/dom/element.js` lines 608-618

**Implementation Details**:

Apply static evaluation to any JSX expression container:

```javascript
if (t.isJSXExpressionContainer(value)) {
  const evaluated = attribute.get("value").get("expression").evaluate().value;
  if (evaluated !== undefined && (typeof evaluated === "string" || typeof evaluated === "number")) {
    value = t.stringLiteral(String(evaluated));
  }
}
```

**Test Cases**:
```jsx
<div data-value={1 + 1} />     // → <div data-value="2">
<div title={"Hello" + "!"} />  // → <div title="Hello!">
<div count={123} />            // → <div count="123">
```

## Testing Strategy

### Unit Tests
- Add tests for each phase in respective modules
- Test edge cases (empty strings, "0", null, undefined)
- Test mixed static/dynamic scenarios

### Integration Tests
- Run against babel plugin fixture tests
- Compare output character-by-character
- Document remaining formatting differences

### Regression Tests
- Ensure simple_elements test keeps passing
- Ensure no existing functionality breaks

## Expected Outcomes

### After Phase 2 (bool: attributes)
- **Test Pass Rate**: ~50-60%
- **Main Impact**: attributeExpressions test improvements

### After Phases 2-4 (bool, innerHTML, style)
- **Test Pass Rate**: ~80-85%
- **Main Impact**: Most optimization features working

### After All Phases
- **Test Pass Rate**: ~95-100%
- **Output**: Near-identical to babel plugin
- **Bundle Size**: Matches or improves upon babel plugin output

## Implementation Priority

### High Priority (Phase 2)
- bool: attribute evaluation
- Biggest impact on test compatibility
- Relatively straightforward implementation

### Medium Priority (Phases 3-4)
- innerHTML/textContent
- Style object inlining
- Significant optimization wins

### Lower Priority (Phases 5-6)
- classList evaluation
- General attribute evaluation
- Nice-to-have optimizations

## Code Locations Reference

### Files to Modify
1. `src/template.rs` - Template HTML generation logic
2. `src/transform/traverse_impl.rs` - JSX element traversal
3. `src/transform/attributes.rs` - Attribute processing (if needed)
4. `src/transform/codegen.rs` - Code generation for dynamic slots

### Files to Reference
1. `/tmp/dom-expressions/packages/babel-plugin-jsx-dom-expressions/src/dom/element.js`
   - Main element transformation logic
   - Line-by-line reference for all features
2. `/tmp/dom-expressions/packages/babel-plugin-jsx-dom-expressions/src/shared/utils.js`
   - Utility functions for evaluation and detection

## Notes

### Babel's evaluate() Implementation
Babel uses sophisticated constant folding via `@babel/traverse`:
- Tracks variable scopes and bindings
- Evaluates complex expressions recursively
- Handles many more cases than our simple evaluator

Our `static_evaluator.rs` covers the most common cases (80-90% of real-world usage) without the complexity of full constant folding.

### Why Not Implement Full Constant Folding?
- Complexity: Would require scope tracking, variable binding analysis
- Diminishing Returns: Most real-world code uses simple literals
- Maintenance: Simpler code is easier to maintain
- Performance: Our focused evaluator is faster for common cases

### Compatibility vs. Optimization
These features are primarily **optimizations** that:
- Reduce bundle size (fewer runtime calls)
- Improve performance (static template content)
- Match babel plugin output exactly

The **core functionality** (reactivity, components, events) already works correctly without these optimizations.

## Conclusion

Full babel compatibility requires implementing static evaluation features that optimize the generated code. The static evaluator foundation is complete; the remaining work is integrating it into the attribute and template processing logic.

**Estimated Work**: 
- Phase 2: 4-6 hours
- Phases 3-4: 4-6 hours
- Phases 5-6: 2-4 hours
- **Total**: 10-16 hours of focused development

**Risk Level**: Low - all features are additive optimizations that don't affect core transformation logic.
