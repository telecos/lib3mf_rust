---
name: Extract Slice Extension Stack Data
about: Fully implement Slice extension data extraction
title: 'Extract Slice Extension Stack Definitions and Data'
labels: 'extension-support, priority:medium'
assignees: ''
---

## Description

The Slice extension is currently recognized and validated, but slice-specific data structures (slice stacks, slice references, polygon segments) are not extracted. This prevents applications from accessing layer-by-layer manufacturing data.

## Current State

- ✅ Slice extension recognized for validation
- ✅ Files with `xmlns:s` namespace parse without errors
- ✅ Test file available (`test_files/box_sliced.3mf`)
- ❌ `s:slicestack` elements not extracted
- ❌ Slice references from objects not captured
- ❌ Polygon segment data not accessible

## Expected Outcome

1. **Data Structures**:
   ```rust
   pub struct SliceStack {
       pub id: usize,
       pub zbottom: f64,
       pub slices: Vec<Slice>,
   }
   
   pub struct Slice {
       pub ztop: f64,
       pub polygons: Vec<Polygon>,
   }
   
   pub struct Polygon {
       pub start_v: usize,
       pub segments: Vec<Segment>,
   }
   
   pub enum Segment {
       Line { v: usize },
       // Additional segment types as needed
   }
   ```

2. **Parser Updates**:
   - Parse `<s:slicestack>` elements from model
   - Extract individual `<s:slice>` elements with polygon data
   - Parse `slicestackid` references from objects
   - Store in Model resources

3. **API Access**:
   - Add `slice_stacks: Vec<SliceStack>` to Resources
   - Add `slice_stack_id: Option<usize>` to Object
   - Make slice data accessible via Model API

## Implementation Notes

**Slice Extension Elements**:
- `s:slicestack` - Container for slice definitions
- `s:slice` - Individual layer/slice with ztop
- `s:polygon` - 2D polygon in a slice
- `s:segment` - Line segment connecting vertices

**Parser Location**: `src/parser.rs`
- Handle Slice namespace (`xmlns:s`)
- Parse slice stack resources
- Link objects to slice stacks via `slicestackid`

**Spec Reference**: [3MF Slice Extension 1.0.2](https://github.com/3MFConsortium/spec_slice)

## Test Files

- `test_files/box_sliced.3mf` - Slice extension example
- Suites 1, 4 in conformance tests use Slice extension

## Acceptance Criteria

- [ ] `SliceStack`, `Slice`, `Polygon`, `Segment` structs added
- [ ] Parser extracts `s:slicestack` elements
- [ ] Parser links objects to slice stacks
- [ ] Slice polygon data accessible via API
- [ ] Test added to verify slice extraction
- [ ] Documentation updated
- [ ] IMPLEMENTATION_SUMMARY.md shows Slice as fully supported

## Related Issues

- Production Extension Data Extraction
- Beam Lattice Extension Data Extraction

## Priority

**Medium** - Slice extension is important for layer-based manufacturing and preview applications.
