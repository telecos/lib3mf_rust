---
name: Validate Base Materials References
about: Fix TODO - Add validation for base materials references
title: 'Validate Base Materials References (TODO in code)'
labels: 'validation, priority:high, good first issue'
assignees: ''
---

## Description

The validator currently only checks color group references but doesn't validate base materials references. This is explicitly marked with a TODO comment in the code at `src/validator.rs:210`.

## Current State

- ✅ Color group references validated correctly
- ❌ Base materials references NOT validated
- ❌ `basematerialid` attributes not checked
- ❌ `pid` can reference either color groups OR base materials per spec, but only color groups currently validated
- ⚠️ **TODO marker** in code points to this gap

## Code Location

`src/validator.rs:210`:
```rust
// TODO: Also validate basematerials references
```

## Expected Outcome

1. **Add BaseMaterial Data Structure**:
   ```rust
   pub struct BaseMaterial {
       pub id: usize,
       pub name: String,
       pub displaycolor: String,  // #RRGGBB or #RRGGBBAA
   }
   ```

2. **Parser Updates**:
   - Parse `<basematerials>` elements from resources
   - Extract individual `<base>` material definitions
   - Store in `model.resources.base_materials`

3. **Validation Logic**:
   - When object has `basematerialid`, verify it references valid base material
   - When `pid` is used, check BOTH color groups and base materials
   - Update `validate_material_references()` in validator

## Implementation Notes

**Materials Extension Spec**:
- `<basematerials>` - Container for base material definitions
- `<base>` - Individual material with `name` and `displaycolor`
- `basematerialid` - Object attribute referencing base material ID
- `pid` - Can reference either color group ID or base material ID

**Where to Change**:
1. `src/model.rs` - Add `BaseMaterial` struct, add to `Resources`
2. `src/parser.rs` - Parse `<basematerials>` and `<base>` elements
3. `src/validator.rs` - Update `validate_material_references()` to check both types
4. `src/lib.rs` - Export `BaseMaterial` in public API

## Test Files

Base materials used extensively in:
- `suite2_core_prod_matl` conformance suite
- `suite6_core_matl` conformance suite

## Acceptance Criteria

- [ ] `BaseMaterial` struct added to `src/model.rs`
- [ ] `Resources` has `base_materials: Vec<BaseMaterial>` field
- [ ] Parser extracts `<basematerials>` from XML
- [ ] `basematerialid` attributes validated against base materials list
- [ ] `pid` can reference EITHER color groups OR base materials
- [ ] **TODO comment removed** from `src/validator.rs:210`
- [ ] Tests added for base materials validation
- [ ] Documentation updated

## Why This Is A Good First Issue

- ✅ Clear TODO in code points to exact location
- ✅ Existing pattern for color groups can be followed
- ✅ Limited scope - one validation rule
- ✅ Test files available in conformance suite
- ✅ Well-defined acceptance criteria
- ✅ Good introduction to codebase structure

## References

- `src/validator.rs:210` - TODO comment
- [3MF Materials Extension Spec](https://github.com/3MFConsortium/spec_materials)

## Related Issues

- Negative Test Conformance (#1)
- Advanced Material Properties

## Priority

**High** - Explicit TODO in code, needed for Materials extension compliance, likely contributes to negative test failures.

## Effort Estimate

**Small (1-2 days)** - Straightforward implementation following existing color group pattern.
