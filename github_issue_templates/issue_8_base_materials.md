---
name: Validate Base Materials References
about: Add validation for base materials references (TODO in validator)
title: 'Validate Base Materials References'
labels: 'validation, priority:medium, good first issue'
assignees: ''
---

## Description

The validator currently only checks color group references but doesn't validate base materials references. This is marked with a TODO comment in the code.

## Current State

- ✅ Color group references are validated
- ❌ Base materials references are not validated  
- ❌ `basematerialid` attributes not checked
- ❌ `pid` attributes can reference either color groups or base materials per spec, but only color groups are currently validated

## Expected Outcome

- Add data structure for base materials
- Parse base materials from XML (`<base materials>` elements)
- Validate that `basematerialid` attributes reference valid base materials
- Ensure `pid` can reference either color groups or base materials
- Update validator to check both types of material references

## Code Location

`src/validator.rs:210` - Look for the TODO comment:

```rust
// TODO: Also validate basematerials references
```

## Implementation Notes

**Data Structure:**
```rust
pub struct BaseMaterial {
    pub id: usize,
    pub name: String,
    pub displaycolor: String, // #RRGGBB or #RRGGBBAA format
}
```

**Validation Logic:**
- When an object has a `basematerialid` attribute, verify it references a valid base material
- When a `pid` is used, it can reference either a color group ID or a base material ID
- Update `validate_material_references()` to handle both cases

## Test Files

Base materials are used in many test files, especially in:
- `suite2_core_prod_matl` 
- `suite6_core_matl`

## Acceptance Criteria

- [ ] `BaseMaterial` data structure added to `src/model.rs`
- [ ] Parser extracts base materials from XML
- [ ] `basematerialid` attributes validated
- [ ] `pid` references can be to color groups OR base materials
- [ ] Tests updated to cover base materials
- [ ] TODO comment removed from code

## References

- src/validator.rs, line 210
- [3MF Materials Extension Specification](https://github.com/3MFConsortium/spec_materials)

## Additional Context

This is a good first issue because:
- Clear TODO in the code points to exact location
- Pattern already exists for color groups that can be followed
- Limited scope - just one validation rule
- Test files available for validation
