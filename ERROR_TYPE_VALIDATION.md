# Specific Error Type Validation for Expected Failures

## Problem Statement

Previously, when conformance test files were marked as expected failures, we only documented *that* they failed, not *why* they failed. This created a risk: if a file started failing for a different reason (e.g., due to a new bug), we wouldn't detect it because we were accepting any failure.

Example scenario:
- File `P_BXX_2015_03.3mf` is marked as expected failure because it has geometry outside the positive octant
- Later, a bug is introduced that causes the file to fail with an XML parsing error
- The test still "passes" because we expected it to fail, but we've missed a regression

## Solution

We now support specifying the **expected error type** for each expected failure. This allows tests to validate that files fail for the **correct reason**.

### Implementation Details

1. **New Error Variant**: Added `OutsidePositiveOctant(usize, f64, f64, f64)` error variant
   - Error code: E3003
   - Contains: object ID and min coordinates
   - Distinct from generic `InvalidModel` error

2. **Error Type Identification**: Added `error_type()` method to `Error` enum
   - Returns a stable string identifier for each error variant
   - Used for matching against expected error types

3. **Expected Failure Configuration**: Extended `ExpectedFailure` struct
   - New optional field: `expected_error_type: Option<String>`
   - When specified, tests verify the actual error type matches

4. **Test Validation**: Updated conformance test logic
   - If `expected_error_type` is specified, validate it matches the actual error
   - If mismatch detected, test fails with clear error message
   - If no error type specified, behavior remains backward compatible

### Example Usage

In `tests/expected_failures.json`:

```json
{
  "file": "P_BXX_2015_03.3mf",
  "suite": "suite7_beam",
  "test_type": "positive",
  "reason": "File violates 3MF specification coordinate requirements...",
  "expected_error_type": "OutsidePositiveOctant"
}
```

When the test runs:
- ✅ If the file fails with `OutsidePositiveOctant`: Test passes (expected behavior)
- ❌ If the file fails with `InvalidFormat`: Test fails (caught a different issue!)
- ❌ If the file succeeds: Test fails (spec violation was fixed)

### Benefits

1. **Catch Regressions**: If a file starts failing for a different reason, we detect it
2. **Clear Intent**: Error type documents *why* the file is expected to fail
3. **Backward Compatible**: Files without `expected_error_type` work as before
4. **Debugging Aid**: Clear error messages when validation fails

### Testing

See `tests/test_error_type_validation_demo.rs` for demonstration tests showing:
- How the system catches files failing for wrong reasons
- How it validates correct error types
- Example scenarios comparing old vs new behavior

## Files Modified

- `src/error.rs`: Added `OutsidePositiveOctant` error variant and `error_type()` method
- `src/validator.rs`: Updated `validate_build_transform_bounds()` to use new error
- `tests/common/expected_failures.rs`: Added `expected_error_type` field
- `tests/conformance_tests.rs`: Added error type validation logic
- `tests/expected_failures.json`: Marked positive octant violations with error type
- `tests/test_positive_octant_error.rs`: Unit tests for error detection
- `tests/test_error_type_validation_demo.rs`: Demonstration tests
