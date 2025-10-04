# Marker Placement Fix Summary

## What Was Fixed

### 1. Adjacent Expression Marker Sharing
**Problem**: When two expressions were adjacent (like `{greeting}{name}`), each was getting its own marker instead of sharing one.

**Solution**: Implemented logic to detect when an expression immediately follows another expression and reuse the same marker path.

**Code**: `src/template.rs` - Added `prev_is_expression` tracking and `last_marker_path` to share markers between adjacent expressions.

**Test Cases Fixed**:
- multiExprTogether: `<span> {greeting}{name} </span>` now correctly generates template `<span> <!> </span>` with both expressions using the same marker.

### 2. Optimal Marker Placement Strategy
**Problem**: Markers were being added unnecessarily, not matching the babel plugin's optimization strategy.

**Solution**: Implemented the babel plugin's marker minimization rules:
1. If expression is the FIRST node (no preceding content), use the next node as insertion point (no marker needed)
2. If expression is the LAST child, use null for insertion at end (no marker needed)
3. Adjacent expressions share one marker
4. Otherwise, add a marker after the expression

**Code**: `src/template.rs` - `build_child_html_with_context` now checks `num_nodes_so_far == 0` to determine if expression is first node.

**Test Cases Fixed**:
- multiExpr: `<span>{greeting} {name}</span>` → template `<span> </span>` (no markers, uses middle space and null)
- multiExprSpaced: `<span> {greeting} {name} </span>` → template `<span> <!> <!> </span>` (two markers for middle expressions)
- leadingExpr: `<span>{greeting} John</span>` → template `<span> John` (uses text as insertion point)
- trailingExpr: `<span>Hello {name}</span>` → template `<span>Hello ` (uses null for end insertion)

## Test Status

### Before Fix
- 1/5 DOM tests passing
- Marker generation was fundamentally broken

### After Fix
- 2/5 DOM tests passing (test_simple_elements, test_fragments)
- Marker placement logic now matches babel plugin
- Template HTML generation is mostly correct for markers

## Remaining Issues

### High Priority

#### 1. Whitespace Normalization in Templates
**Problem**: Templates with multiple spaces preserve all spaces instead of normalizing.

**Example**:
- Input: `<span>Hello   John</span>` (3 spaces from prettier-ignore)
- Expected template: `<span>Hello John` (normalized to 1 space)
- Actual template: `<span>Hello   John` (all 3 spaces preserved)

**Files**: `src/template.rs` - Need to normalize whitespace in static text content.

**Impact**: Affects test_text_interpolation

#### 2. Static Expression Handling
**Problem**: Expressions that can be evaluated at compile time should be inlined into the template.

**Example**:
- Input: `<span>Hello {value + "!"}</span>` where `value = "World"`
- Expected: Evaluate to `<span>Hello World!` at compile time
- Actual: Treated as dynamic expression

**Files**: `src/template.rs` - Need to detect and evaluate static expressions.

**Impact**: Affects test_text_interpolation (evaluated, evaluatedNonString cases)

#### 3. Component Children with HTML Entities
**Problem**: HTML entities in component children are being decoded instead of staying encoded.

**Example**:
- Input: `<Comp>&nbsp;&lt;Hi&gt;&nbsp;</Comp>`
- Expected: `children: "&nbsp;&lt;Hi&gt;&nbsp;"`
- Actual: `children: "\xA0<Hi>\xA0"`

**Files**: `src/transform.rs` - Component transformation needs to preserve HTML entity encoding in string literals.

**Impact**: Affects test_text_interpolation (escape2, escapeCompAttribute cases)

#### 4. Fragment Transformation
**Problem**: Fragments with expressions need special handling.

**Example**:
- Input: `<>&nbsp;&lt;Hi&gt;&nbsp;</>`
- Expected: `"&nbsp;&lt;Hi&gt;&nbsp;"`
- Actual: Component-like transformation

**Files**: `src/transform.rs` - Fragment transformation logic.

**Impact**: Affects test_text_interpolation (escape3 case)

### Medium Priority

#### 5. Attribute Expression Handling
**Problem**: Dynamic attributes need proper runtime call generation (setAttribute, classList, style, etc.)

**Files**: `src/transform.rs` - Attribute slot handling in `create_runtime_calls_from_expressions`.

**Impact**: Affects test_attribute_expressions entirely

#### 6. Event Expression Handling
**Problem**: Event handlers need proper delegation setup and runtime calls.

**Files**: `src/transform.rs` - Event slot handling.

**Impact**: Affects test_event_expressions entirely

## Architecture Notes

### Current Flow
1. JSX → Template Builder → Template (HTML + DynamicSlots)
2. Template → Extract Expressions → List of Expression AST nodes
3. Template + Expressions → Generate Runtime Calls

### Key Data Structures
- `DynamicSlot`: Tracks position and type of dynamic content
  - `path`: Navigation to element
  - `marker_path`: Navigation to insertion marker
  - `slot_type`: Type of slot (TextContent, Attribute, etc.)
- `last_marker_path`: Shared state to track marker reuse between adjacent expressions

### Design Decisions
1. **Marker Minimization**: Follows babel plugin's strategy of using existing nodes as insertion points when possible
2. **Adjacent Expression Detection**: Uses `prev_is_expression` flag to detect when expressions are immediately consecutive
3. **First Node Optimization**: Expressions that are the first real node can use `firstChild` as insertion point without a marker

## Recommendations

### For Full Compatibility
1. **Whitespace Normalization** (4-6 hours): Implement proper whitespace collapsing in static text
2. **Static Expression Evaluation** (3-4 hours): Add compile-time evaluation for constant expressions
3. **HTML Entity Preservation** (2-3 hours): Fix component/fragment child encoding
4. **Attribute Handlers** (4-6 hours): Complete attribute expression runtime call generation
5. **Event Handlers** (3-4 hours): Complete event expression runtime call generation

### Quick Wins
- Whitespace normalization would immediately fix several test cases
- HTML entity preservation is isolated to component/fragment transformation

## Testing Strategy

The fixture tests in `tests/dom_fixtures.rs` provide excellent coverage:
- Each test compares generated output against babel plugin output
- Normalization handles formatting differences (variable names, spacing)
- Use `cargo test --test dom_fixtures test_name -- --nocapture` to see detailed diffs

## Conclusion

The marker placement logic is now correct and matches the babel plugin's optimization strategy. This was the most complex part of the template generation. The remaining issues are more straightforward to fix and mostly involve:
1. Text processing (whitespace normalization)
2. Expression evaluation (static expressions)
3. String encoding (HTML entities)
4. Completing the runtime call generation for different slot types

The foundation is solid and the architecture is clean. With focused work on the remaining issues, full test compatibility is achievable.
