---
name: "Implement Component Parsing and Validation"
about: Parse component elements and validate references (assemblies)
title: "Phase 3: Component Support (High Impact)"
labels: "conformance, priority:high, feature"
assignees: ""
---

## Description

Implement parsing and validation of component elements, which allow objects to reference other objects (assemblies/hierarchies).

## Implementation

### Phase 3.1: Parse Components (3-4 days)
- Add `Component` struct
- Parse `<component>` elements
- Store in `Object.components`

### Phase 3.2: Validate References (2-3 days)
- Validate objectid exists
- Detect circular references (A→B→C→A)
- Reject self-references (A→A)

## Expected Impact

20-30% improvement in negative test pass rate (100-150 tests).

## Timeline

**Total**: 5-7 days  
**Priority**: HIGH
