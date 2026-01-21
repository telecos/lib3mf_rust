---
name: Validate Component References and Assemblies
about: Add support for component parsing and validation
title: 'Validate Component References and Detect Circular Dependencies'
labels: 'validation, priority:high'
assignees: ''
---

## Description

3MF supports components (objects that reference other objects to create assemblies and hierarchical structures). The parser currently doesn't parse component elements or validate these references, missing a core 3MF feature.

## Current State

- ❌ `<component>` elements not parsed
- ❌ Component `objectid` references not validated
- ❌ Circular component references not detected
- ❌ Component transformations not captured
- ❌ Assembly hierarchies not accessible

## Impact

- Cannot parse files with assemblies/hierarchies
- Invalid component references accepted
- Circular references could cause infinite loops
- Missing core 3MF functionality

## Expected Outcome

1. **Data Structure**:
   ```rust
   pub struct Component {
       pub objectid: usize,
       pub transform: Option<[f64; 12]>,  // 3x4 transformation matrix
   }
   
   // Add to Object:
   pub struct Object {
       // ... existing fields
       pub components: Vec<Component>,
   }
   ```

2. **Parser Updates**:
   - Parse `<component>` elements from `<object>` 
   - Extract `objectid` and `transform` attributes
   - Store in Object's component list

3. **Validation**:
   - Validate `objectid` references exist
   - Detect circular component references (A→B→C→A)
   - Ensure components don't reference themselves
   - Validate transformation matrices if present

## Implementation Notes

**Component Elements** (3MF Core Spec):
- `<component>` - References another object
- `objectid` - Required, references object in resources
- `transform` - Optional 3x4 matrix
- Multiple components allowed per object

**Circular Reference Detection**:
Use depth-first search with visited tracking:
```rust
fn detect_circular_components(
    model: &Model,
    object_id: usize,
    visited: &mut HashSet<usize>,
    stack: &mut Vec<usize>
) -> Result<()> {
    if stack.contains(&object_id) {
        return Err(Error::InvalidModel(
            format!("Circular component reference: {}", 
                stack.iter().chain(std::iter::once(&object_id))
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(" → "))
        ));
    }
    // ... continue DFS
}
```

**Spec Reference**: 3MF Core Specification, Chapter 6 (Components)

## Test Files

Components used in conformance test suites - check for files with `<component>` elements.

## Acceptance Criteria

- [ ] `Component` struct added to `src/model.rs`
- [ ] `Object` has `components: Vec<Component>` field
- [ ] Parser extracts `<component>` elements
- [ ] `objectid` references validated
- [ ] Circular references detected and rejected
- [ ] Self-references detected and rejected
- [ ] Transformation matrices parsed if present
- [ ] Tests added for component validation
- [ ] Error messages clearly describe circular references
- [ ] Documentation updated

## Validation Cases

- [ ] Valid component reference
- [ ] Component references non-existent object → Error
- [ ] Component references itself → Error
- [ ] Circular reference (A→B→A) → Error
- [ ] Circular reference (A→B→C→A) → Error
- [ ] Component with transformation matrix → OK
- [ ] Deep but non-circular hierarchy → OK

## References

- [3MF Core Spec - Components](https://3mf.io/specification/)
- Future Enhancement: Component assemblies in README

## Related Issues

- Negative Test Conformance (#1)
- Advanced Features

## Priority

**High** - Core 3MF feature, likely contributes to negative test failures, needed for assembly support.

## Effort Estimate

**Medium (3-5 days)** - Requires parsing, validation logic, and circular reference detection algorithm.
