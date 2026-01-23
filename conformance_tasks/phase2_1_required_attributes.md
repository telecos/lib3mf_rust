---
name: "Implement Required Attribute Validation"
about: Validate presence of all required attributes per 3MF spec
title: "Phase 2.1: Required Attribute Validation (Quick Win)"
labels: "conformance, priority:high, validation"
assignees: ""
---

## Description

Implement validation to ensure all required attributes are present per the 3MF specification.

## Implementation

Add checks in `src/parser.rs` for required attributes:
- Object: `id` (required)
- Vertex: `x`, `y`, `z` (all required)
- Triangle: `v1`, `v2`, `v3` (all required)
- Item: `objectid` (required)

## Expected Impact

10-20% improvement in negative test pass rate (50-100 tests).

## Timeline

**Effort**: 2-3 days  
**Priority**: HIGH
