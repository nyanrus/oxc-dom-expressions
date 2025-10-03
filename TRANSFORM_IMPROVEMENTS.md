# Transform Improvements Summary

## Changes Made

### 1. Import Ordering (✅ Fixed)
**File**: `src/transform.rs`

**Problem**: Imports were being sorted alphabetically, but babel plugin uses a priority-based order.

**Solution**: 
- Changed `required_imports` from `HashSet<String>` to `Vec<String>` to preserve insertion order
- Implemented priority-based sorting in `create_import_statements()` with hardcoded priorities
- `template` always comes first, followed by other imports in priority order

**Impact**: Import statements now match babel plugin output order.

### 2. Template Variable Numbering (✅ Fixed)
**File**: `src/transform.rs`

**Problem**: Template variables were being sorted lexicographically (`_tmpl$10` before `_tmpl$2`)

**Solution**: Implemented numeric sorting in `create_template_declarations()` by extracting and comparing the numeric part of variable names.

**Impact**: Template declarations now appear in correct numeric order.

### 3. Attribute Expression Extraction (✅ Implemented)
**File**: `src/transform.rs`

**Problem**: Expressions from attributes were not being extracted, only child expressions.

**Solution**: Modified `extract_expressions_from_jsx()` to first extract attribute expressions, then child expressions, maintaining the correct order.

**Impact**: Dynamic attribute values are now properly extracted and can be processed.

### 4. Basic Attribute Handling (✅ Implemented)
**File**: `src/transform.rs`

**Problem**: `SlotType::Attribute` cases were not being handled.

**Solution**: 
- Implemented `create_set_attribute_call()` method
- Generates `_$effect(() => _$setAttribute(element, "attr", value))` calls
- Handles attribute slots in `create_runtime_calls_from_expressions()`

**Impact**: Basic dynamic attributes now generate runtime calls (though template generation still needs work).

### 5. Code Cleanup (✅ Done)
**File**: `src/transform.rs`

**Problem**: Unused methods causing compiler warnings.

**Solution**: Removed `create_insert_call()` and `create_import_statement()` methods.

**Impact**: Clean build with no warnings.

## Test Results

**Before changes**: 1/5 tests passing (test_simple_elements)
**After changes**: 1/5 tests passing (test_simple_elements)

The same test continues to pass because it contains only static content. The failing tests require template generation fixes.

## Remaining Work

### Critical: Template Generation (template.rs)

The main blocker for passing tests is that template HTML generation doesn't match babel plugin behavior:

**Whitespace Handling Pattern**:
```javascript
// Input: <span>{greeting} {name}</span>
// Expected template: `<span> ` (just the space)
// Our template: `<span><!><!>` (markers, no space)

// Input: <span> {greeting} {name} </span>
// Expected template: `<span> <!> <!> ` (spaces + markers)
// Our template: Similar but whitespace handling differs
```

**Analysis**: The babel plugin uses sophisticated logic:
1. When expressions have NO surrounding whitespace, preserve whitespace between them (don't add markers)
2. When expressions HAVE surrounding whitespace, add `<!>` markers for expressions
3. Whitespace text nodes serve dual purpose as both content and insertion markers

**Required Changes**:
- Rewrite `build_child_html_with_context()` in `src/template.rs`
- Implement lookahead to determine if expressions have surrounding whitespace
- Add logic to decide when to use `<!>` markers vs. preserving whitespace
- Update marker_path logic to use text nodes when available

### Missing Features (transform.rs)

1. **Event Handling**
   - Slot types: `EventHandler`, `OnEvent`, `OnCaptureEvent`
   - Need to implement event extraction and runtime call generation
   - Should use `addEventListener` or delegated events based on event type

2. **Special Attribute Types**
   - `ClassList`: `_$classList()`
   - `StyleObject`: `_$style()`
   - `Ref`: Ref binding
   - Boolean attributes: `_$setBoolAttribute()`

3. **Component Transformation**
   - Currently basic implementation exists
   - May need refinement for prop handling, children, etc.

4. **Fragment Transformation**
   - Basic implementation exists
   - May need improvements for various fragment patterns

## Recommendations

### Short-term (Minimal Changes)
If the goal is minimal changes to demonstrate progress:
1. ✅ Already done: Import ordering, template numbering, basic attribute handling
2. Could add: Event handler extraction and basic runtime call generation
3. Could add: Additional attribute type handling (classList, style, etc.)

### Long-term (Full Compatibility)
To fully pass babel plugin tests:
1. **Must do**: Rewrite template generation in `template.rs` (significant work)
2. **Should do**: Complete all slot type handlers
3. **Should do**: Refine component and fragment transformation
4. **Nice to have**: Add more unit tests for new functionality

## Conclusion

The changes made improve the infrastructure and fix several issues in transform.rs:
- Import ordering now matches babel plugin
- Template numbering is correct
- Basic attribute handling is implemented
- Code is cleaner with no warnings

However, to pass the remaining 4 failing fixture tests, the core template generation logic in `template.rs` must be rewritten to match babel plugin's whitespace handling strategy. This is a significant undertaking that goes beyond "minimal changes" to transform.rs.

The question for the project maintainers is: 
- Accept the current infrastructure improvements and plan template.rs rewrite separately?
- Or invest in the template.rs rewrite now to achieve full test compatibility?
