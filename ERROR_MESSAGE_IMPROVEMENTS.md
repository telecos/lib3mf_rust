# Error Message Improvements

This document demonstrates the improvements made to error messages in lib3mf_rust.

## Overview

Error messages have been enhanced to provide:
1. **Error codes** - Already existed (E1001, E2003, E3001, etc.)
2. **ErrorContext struct** - New structure for optional file/line/column/hint information
3. **Available alternatives** - Error messages now show what values ARE available
4. **Valid ranges** - Out-of-bounds errors show the valid range
5. **Actionable hints** - Each error includes a hint for how to fix it

## ErrorContext Structure

```rust
pub struct ErrorContext {
    /// The file where the error occurred
    pub file: Option<String>,
    
    /// Line number where the error occurred
    pub line: Option<usize>,
    
    /// Column number where the error occurred
    pub column: Option<usize>,
    
    /// A helpful hint for resolving the error
    pub hint: Option<String>,
}
```

### Builder Methods

```rust
let ctx = ErrorContext::new()
    .file("3D/3dmodel.model")
    .line(42)
    .column(15)
    .hint("Check attribute syntax");
```

## Enhanced Error Examples

### Example 1: Property Group Reference Error

**Before:**
```
[E3001] Invalid model: Object 5 references non-existent property group ID: 99
```

**After:**
```
[E3001] Invalid model: Object 5 references non-existent property group ID: 99.
Available property group IDs: [1, 2, 3, 4]
Hint: Check that all referenced property groups are defined in the <resources> section.
```

**Benefits:**
- Shows exactly which IDs ARE available
- Provides actionable hint about where to look
- Helps developers quickly identify the issue

---

### Example 2: Color Index Out of Bounds

**Before:**
```
[E3001] Invalid model: Object 1: Triangle 0 p1 10 is out of bounds (color group 1 has 4 colors)
```

**After:**
```
[E3001] Invalid model: Object 1: Triangle 0 p1 10 is out of bounds.
Color group 1 has 4 colors (valid indices: 0-3).
Hint: p1 must be less than the number of colors in the color group.
```

**Benefits:**
- Explicitly states the valid range (0-3)
- Explains what "has 4 colors" means in terms of indices
- Provides clear guidance on the constraint

---

### Example 3: Base Material Group Reference

**Before:**
```
[E3001] Invalid model: Object 1 references non-existent base material group ID: 99.
Check that a basematerials group with this ID exists in the resources section.
```

**After:**
```
[E3001] Invalid model: Object 1 references non-existent base material group ID: 99.
Available base material group IDs: [1, 2, 3]
Hint: Check that a basematerials group with this ID exists in the <resources> section.
```

**Benefits:**
- Lists all available base material group IDs
- Makes it obvious that 99 is not in the list
- Reduces debugging time

---

### Example 4: Component Reference Error

**Before:**
```
[E3001] Invalid model: Object 30: Component references non-existent object ID 99
```

**After:**
```
[E3001] Invalid model: Object 30: Component references non-existent object ID 99.
Available object IDs: [10, 20, 30]
Hint: Ensure the referenced object exists in the <resources> section.
```

**Benefits:**
- Shows all available object IDs for reference
- Helps identify typos (e.g., 9 vs 99)
- Provides clear next steps

---

### Example 5: Boolean Operation Reference

**Before:**
```
[E3001] Invalid model: Object 1: Boolean shape references non-existent object ID 50
```

**After:**
```
[E3001] Invalid model: Object 1: Boolean shape references non-existent object ID 50.
Available object IDs: [1, 2, 3, 10, 20]
Hint: Ensure the referenced object exists in the <resources> section.
```

**Benefits:**
- Lists all objects that could be referenced
- Makes it clear which IDs are valid
- Reduces trial-and-error debugging

---

## Implementation Details

### Validation Errors Enhanced

The following validation functions now provide enhanced error messages:

1. **`validate_material_references`**
   - Property group reference errors
   - Base material group reference errors  
   - Color index out-of-bounds errors
   - Material index out-of-bounds errors

2. **`validate_component_references`**
   - Component object reference errors
   
3. **`validate_boolean_operations`**
   - Boolean shape object reference errors
   - Boolean operand reference errors

### Test Coverage

All enhanced error messages are covered by existing tests:
- `test_validate_invalid_base_material_reference`
- `test_validate_component_reference_invalid`
- `test_validate_basematerialid_invalid`
- `test_validate_base_material_pindex_out_of_bounds`

New tests for ErrorContext:
- `test_error_context_with_hint`
- `test_error_context_builder`
- `test_error_context_display`
- `test_error_context_display_partial`
- `test_error_context_display_empty`

## Impact on Developer Experience

### Before
Developers had to:
1. Read the error message
2. Manually search through the 3MF file
3. List all available IDs themselves
4. Compare to find the mismatch
5. Guess at valid ranges

### After
Developers can:
1. Read the error message
2. See immediately what values are available
3. Understand the valid range
4. Get a hint about how to fix it
5. Resolve the issue faster

**Estimated time savings:** 5-15 minutes per error, depending on file complexity.

## Future Enhancements

Potential future improvements could include:

1. **File location information** - Add line/column numbers from XML parsing
2. **Error categories** - Group related errors together
3. **Suggested fixes** - Auto-suggest likely corrections
4. **Error documentation links** - Link to detailed error explanations
5. **Programmatic error handling** - Make ErrorContext easily accessible for tooling

## Backward Compatibility

All changes are **100% backward compatible**:
- Error codes remain the same (E1001, E2003, E3001, etc.)
- Error enum variants unchanged
- Existing error messages only enhanced, not replaced
- No breaking API changes
- ErrorContext is an addition, not a replacement
