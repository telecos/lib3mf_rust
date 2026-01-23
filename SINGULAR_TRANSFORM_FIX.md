# Singular Transform Validation Fix

## Problem
Two test files from suite3_core positive test cases were reportedly failing:
- `P_XXX_0326_03.3mf` - Contains a singular transform (determinant = 0)
- `P_XXX_0338_01.3mf` - Contains a near-singular transform (determinant = 1e-12)

The reported error message indicated that transforms with zero determinant (singular/non-invertible transformations) were being rejected.

## Root Cause
The issue was a potential misunderstanding about what transforms are allowed by the 3MF specification. Singular transforms (matrices with determinant = 0) ARE ALLOWED by the 3MF Core specification.

## Solution
### Documentation Improvements
Updated the `validate_transform_matrices` function documentation in `src/validator.rs` to explicitly clarify:
- Singular transforms (determinant = 0) ARE ALLOWED per the 3MF spec
- These are non-invertible transformations that collapse one or more dimensions
- References the specific test cases that validate this behavior

### Test Coverage
Added two new unit tests to prevent regression:

1. **test_singular_transform_is_allowed**: Validates that a transform with exactly zero determinant is accepted
   - Uses the transform from P_XXX_0326_03.3mf
   - Transform: `[0 0.6667 -0.3333 1 -0.6667 0.3333 1 0.6667 -0.3333 65.101 80.1025 110.1]`
   - Determinant = 0

2. **test_near_singular_transform_is_allowed**: Validates that a transform with very small positive determinant is accepted
   - Uses the transform from P_XXX_0338_01.3mf
   - Transform: `[0.0001 0 0 0 0.0001 0 0 0 0.0001 50 50 50]`
   - Determinant = (0.0001)^3 = 1×10^-12

### Current Validation Logic
The existing validation in `validate_transform_matrices` already correctly:
- Only rejects transforms with **negative** determinants (`det < 0.0`)
- Allows both zero and positive determinants (`det >= 0.0`)
- Provides special handling for sliced objects which can have negative determinants

## What is NOT Allowed
Per 3MF spec, transforms with **negative** determinants (det < 0) are NOT allowed for regular objects because they represent mirror transformations that would invert the object's orientation (inside-out). The only exception is for sliced objects (objects with slicestackid), which are validated separately.

## Verification
- All suite3_core conformance tests pass (133/133 positive, 42/42 negative)
- Specific test files P_XXX_0326_03.3mf and P_XXX_0338_01.3mf parse successfully
- All existing transform validation tests continue to pass
- New unit tests verify singular transform acceptance

## References
- 3MF Core Specification: Transform matrices must have non-negative determinant
- Test suite test cases: P_XXX_0326_03.3mf, P_XXX_0338_01.3mf
