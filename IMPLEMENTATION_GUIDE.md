# Implementation Guide for Fixture Test Compatibility

This document provides detailed guidance for implementing the remaining features needed to pass all fixture tests from the original babel-plugin-jsx-dom-expressions.

## Current Status

✅ **Working Features:**
- Basic JSX element transformation
- Template generation with cloneNode optimization
- Event delegation (for click, mouse, key, touch, pointer events)
- Direct event listeners (for change, input, blur, focus, etc.)
- Dynamic text content insertion
- Fragments
- Components
- Variable naming matches babel output

❌ **Missing Features:**
- Spread attributes (`{...props}`)
- Event handler array syntax (`onClick={[handler, data]}`)
- Bool namespace attributes (`bool:attr`)
- Ref type checks (`typeof ref === "function" ? use(ref) : assign`)
- Style object helpers
- Static expression evaluation

## Implementation Priority

### 1. Spread Attributes (CRITICAL)

**Impact:** Blocks ~60% of attributeExpressions test

**Files to modify:**
- `src/template.rs` - ✅ Already detects JSXSpreadAttribute
- `src/transform.rs` - Need to implement code generation

**Implementation steps:**

#### Step 1.1: Create spread_call method in transform.rs

```rust
/// Create a spread call: _$spread(element, props, prevProps, merge)
fn create_spread_call(
    &self,
    element_var: &str,
    spread_expr: &Expression<'a>,
) -> Option<Statement<'a>> {
    use oxc_allocator::CloneIn;
    use oxc_ast::ast::*;
    
    // Import spread helper
    self.add_import("spread");
    
    // Create: _$spread(element, props, false, true)
    let spread_id = IdentifierReference {
        span: SPAN,
        name: Atom::from("_$spread"),
        reference_id: None.into(),
    };
    
    let mut args = OxcVec::new_in(self.allocator);
    
    // Arg 1: element reference
    args.push(Argument::from(Expression::Identifier(Box::new_in(
        IdentifierReference {
            span: SPAN,
            name: Atom::from(self.allocator.alloc_str(element_var)),
            reference_id: None.into(),
        },
        self.allocator,
    ))));
    
    // Arg 2: spread expression
    args.push(Argument::from(spread_expr.clone_in(self.allocator)));
    
    // Arg 3: false (prevProps)
    args.push(Argument::from(Expression::BooleanLiteral(Box::new_in(
        BooleanLiteral { span: SPAN, value: false },
        self.allocator,
    ))));
    
    // Arg 4: true (merge)
    args.push(Argument::from(Expression::BooleanLiteral(Box::new_in(
        BooleanLiteral { span: SPAN, value: true },
        self.allocator,
    ))));
    
    let call = CallExpression {
        span: SPAN,
        callee: Expression::Identifier(Box::new_in(spread_id, self.allocator)),
        arguments: args,
        optional: false,
        type_arguments: None,
        pure: false,
    };
    
    Some(Statement::ExpressionStatement(Box::new_in(
        ExpressionStatement {
            span: SPAN,
            expression: Expression::CallExpression(Box::new_in(call, self.allocator)),
        },
        self.allocator,
    )))
}
```

#### Step 1.2: Handle SlotType::Spread in create_runtime_calls_from_expressions

In the match statement around line 824, replace the TODO with:

```rust
SlotType::Spread => {
    // Spread attributes - generate _$spread call
    if expr_index < expressions.len() {
        let element_var = if slot.path.is_empty() {
            root_var
        } else {
            path_to_var
                .get(&slot.path)
                .map(|s| s.as_str())
                .unwrap_or(root_var)
        };
        
        if let Some(stmt) = self.create_spread_call(
            element_var,
            &expressions[expr_index],
        ) {
            stmts.push(stmt);
        }
        expr_index += 1;
    }
}
```

#### Step 1.3: Advanced - Handle mergeProps

For complex cases where there are static attributes mixed with spreads:

```javascript
<div id="static" {...spread} foo="bar" />
// Should become:
_$spread(_el$, _$mergeProps(spread, { foo: "bar" }), false, true)
```

This requires tracking which attributes come before/after spreads and building object expressions.

**Complexity note:** Full spread implementation requires:
1. Grouping attributes by position relative to spreads
2. Building ObjectExpression AST nodes
3. Creating getters for dynamic values
4. Handling special props (classList, style, ref)

**Testing:**
```bash
cargo test test_attribute_expressions -- --nocapture
# Look for _$spread calls in output
```

### 2. Event Handler Arrays (HIGH)

**Impact:** Blocks ~40% of eventExpressions test

**Implementation:**

#### Step 2.1: Detect array expressions in event handlers

In `create_runtime_calls_from_expressions`, before calling event handler methods:

```rust
// Check if handler is an array expression
let is_array = matches!(handler_expr, Expression::ArrayExpression(_));

if is_array {
    // Extract handler and data from array
    if let Expression::ArrayExpression(arr) = handler_expr {
        let handler = arr.elements.get(0).and_then(|e| e.as_expression());
        let data = arr.elements.get(1).and_then(|e| e.as_expression());
        
        if should_delegate {
            // Generate: el.$$event = handler; el.$$eventData = data;
            // ...
        } else {
            // Generate: el.addEventListener("event", e => handler(data, e))
            // ...
        }
    }
} else {
    // Regular handler
    // ... existing code
}
```

#### Step 2.2: Create wrapper function for non-delegated array handlers

```rust
fn create_event_wrapper(
    &self,
    handler: &Expression<'a>,
    data: &Expression<'a>,
) -> Expression<'a> {
    // Create: e => handler(data, e)
    // This requires building an ArrowFunctionExpression with params and body
}
```

#### Step 2.3: Create data property assignment for delegated arrays

```rust
// After: el.$$click = handler
// Add: el.$$clickData = data
```

**Testing:**
```bash
cargo test test_event_expressions -- --nocapture
# Check for wrapper functions and $$eventData assignments
```

### 3. Bool Namespace Attributes (MEDIUM)

**Impact:** Blocks ~30% of attributeExpressions test

**Implementation:**

#### Step 3.1: Evaluate static bool expressions

In `template.rs`, when processing `bool:` attributes:

```rust
if is_bool_attribute(&name) {
    if let Some(value) = &attr.value {
        // Try to evaluate as static
        if let Some(bool_value) = evaluate_static_bool(value) {
            if bool_value {
                // Truthy: add attribute to template
                let attr_name = get_prefixed_name(&name).unwrap();
                let _ = write!(html, " {}", attr_name);
            }
            // Falsy: omit attribute completely
        } else {
            // Dynamic: add slot
            slots.push(DynamicSlot { /*...*/ });
        }
    }
}
```

#### Step 3.2: Create evaluate_static_bool helper

```rust
fn evaluate_static_bool(value: &JSXAttributeValue) -> Option<bool> {
    match value {
        JSXAttributeValue::StringLiteral(s) => {
            // Empty string is false, non-empty is true
            Some(!s.value.is_empty())
        }
        JSXAttributeValue::ExpressionContainer(container) => {
            match &container.expression {
                JSXExpression::BooleanLiteral(b) => Some(b.value),
                JSXExpression::NumericLiteral(n) => Some(n.value != 0.0),
                JSXExpression::StringLiteral(s) => Some(!s.value.is_empty()),
                JSXExpression::NullLiteral(_) => Some(false),
                // Can't evaluate other expressions statically
                _ => None,
            }
        }
        _ => None,
    }
}
```

**Testing:**
```bash
# Check templates have `quack` attribute for truthy static values
# Check no attribute for falsy static values  
# Check _$setBoolAttribute calls for dynamic values
```

### 4. Ref Type Checks (MEDIUM)

**Impact:** Blocks ~10% of attributeExpressions test

**Current behavior:**
```javascript
_$use(ref, _el$)
```

**Expected behavior:**
```javascript
var _ref$ = refExpr;
typeof _ref$ === "function" ? _$use(_ref$, _el$) : (refExpr = _el$);
```

**Implementation:**

In `create_ref_call`:

```rust
// 1. Create temp variable: var _ref$ = refExpr
// 2. Create typeof check
// 3. Create conditional: check ? use(temp, el) : (refExpr = el)
```

This requires building ConditionalExpression AST nodes.

**Simplification:** For simple identifiers, can detect if it's a function or variable and generate appropriate code without the typeof check.

### 5. Style and ClassName Helpers (LOW)

**Impact:** Minor improvements

#### Style object:
```rust
// Current: _$setStyleProperty(_el$, "color", a())
// Expected: _$style(_el$, { color: a() })
```

#### ClassName:
```rust
// When class attribute has single dynamic expression:
// Use: _$className(_el$, expr)
// Instead of: _$setAttribute(_el$, "class", expr)
```

### 6. Static Expression Evaluation (LOW)

**Impact:** Mainly textInterpolation test

Implement constant folding for:
- String concatenation
- Arithmetic operations
- Variable lookups (when const/let values are known)

This is complex and has lower priority than functionality features.

## Testing Strategy

1. **Unit tests** - Add tests for each helper method
2. **Fixture tests** - Run specific fixture tests as you implement features:
   ```bash
   cargo test test_attribute_expressions
   cargo test test_event_expressions
   ```
3. **Incremental** - Implement and test one feature at a time
4. **Regression** - Ensure simpleElements and fragments keep passing

## Code Organization

### Where to add code:

**New helper methods** → `src/transform.rs` in `impl DomExpressions<'a>` block

**Static evaluation** → `src/utils.rs` as public functions

**Template logic** → `src/template.rs` in `build_element_html` function

**Tests** → `tests/` directory, add unit tests for complex helpers

## Common Pitfalls

1. **Allocator usage** - Always use `self.allocator` for AST nodes
2. **SPAN** - Use `SPAN` constant for all span fields
3. **Clone expressions** - Use `.clone_in(self.allocator)` for cloning
4. **Box::new_in** - Wrap nodes with `Box::new_in(node, self.allocator)`
5. **OxcVec** - Use `OxcVec::new_in(self.allocator)` for vectors
6. **Atom** - Convert strings with `Atom::from(self.allocator.alloc_str(s))`

## Resources

- Original babel plugin: `babel-plugin-jsx-dom-expressions`
- OXC AST documentation: https://oxc.rs/docs
- Test fixtures: `tests/fixtures/dom/*`
- Expected outputs: `tests/fixtures/dom/*/output.js`

## Success Metrics

✅ All 5 DOM fixture tests passing
✅ All SSR fixture tests passing  
✅ All Hydratable fixture tests passing
✅ No clippy warnings
✅ Performance benchmarks maintained or improved
