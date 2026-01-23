# 3MF Conformance Test Report

**Generated:** 2026-01-22

## Overall Summary

This library has been validated against the official [3MF Consortium test suites](https://github.com/3MFConsortium/test_suites), which include over 2,200 test cases covering all 3MF specifications and extensions.

### Key Metrics

- ✅ **100% Positive Test Compliance**: All 1,719 valid 3MF files parse successfully
- ✅ **90.6% Negative Test Compliance**: 500 out of 552 invalid files are correctly rejected  
- 📊 **97.7% Overall Conformance**: 2,219 out of 2,271 total tests pass

### Test Suite Coverage

The test suites cover the following 3MF specifications:

- **Core Specification** (v1.4.0)
- **Materials & Properties Extension** (v1.2.1)
- **Production Extension** (v1.2.0)
- **Slice Extension** (v1.0.2)
- **Beam Lattice Extension** (v1.2.0)
- **Secure Content Extension** (v1.0.2) - Read-only validation
- **Boolean Operations Extension** (v1.1.1)
- **Displacement Extension** (v1.0.0)

## Recent Improvements

### Per-Vertex Property Validation (Fixed 4 tests)
**Implementation:** Validates that when using per-vertex material properties (p1/p2/p3), ALL THREE attributes must be specified.

**Spec Reference:** 3MF Materials Extension - Per-vertex properties must be complete.

**Test Codes Fixed:** 
- N_XPM_0601_01 - Partial p1 specification
- N_XPM_0604_03 - Partial p1 specification  
- Similar pattern across multiple test files

**Impact:** Prevents incomplete material property application across triangle vertices.

### Strict Color Format Validation (Fixed 2 tests)
**Implementation:** Invalid hexadecimal color values now cause parse errors instead of being silently skipped.

**Spec Reference:** 3MF Materials Extension - Color format requirements.

**Test Codes Fixed:**
- N_XPM_0608_01 - Invalid hex character in color value (#FFHFFF contains 'H')
- Similar invalid color formats

**Impact:** Ensures all color values are valid hexadecimal, preventing silent corruption of color data.

## Remaining Validation Gaps

### Slice Extension Validations (~23 tests)
**Test Codes:** SPX/SXX series (0415, 0417, 0419, 0421, 1605-1612)

**Status:** Requires implementation of slice-specific validation rules.

### Production Extension Validations (~3 tests)  
**Test Codes:** XPX series (0418, 0420, 0421, 0803)

**Status:** Some validations intentionally disabled (see validator.rs comments).

### Materials Extension Validations (~22 tests)
**Test Codes:** XPM/XXM series (0605, 0606, 0607, 0610, etc.)

**Status:** Additional material property validations needed.

### Displacement Extension (~1 test)
**Test Code:** DPX_3314

**Status:** Requires displacement-specific validation.

## Test Execution

To run the conformance tests locally:

```bash
# Clone the test suites
git clone --depth 1 https://github.com/3MFConsortium/test_suites.git

# Run all conformance tests
cargo test --test conformance_tests

# Run specific suite
cargo test --test conformance_tests suite3_core

# Generate summary
cargo test --test conformance_tests summary -- --ignored --nocapture

# Analyze negative test compliance
cargo run --example analyze_negative_tests

# Categorize failures by test code
cargo run --example categorize_failures
```

## Validation Architecture

The validation system is implemented in `src/validator.rs` with the following key components:

1. **Structural Validation**: Object IDs, mesh geometry, build references
2. **Material Validation**: Property group references, pindex bounds, per-vertex properties
3. **Extension Validation**: Extension-specific rules for each 3MF extension
4. **Production Validation**: UUID requirements, production paths
5. **Security Validation**: DTD rejection, thumbnail format validation

Each validation rule includes:
- Clear error messages with spec references
- Specific test code documentation
- Examples of valid vs. invalid usage

## Compliance Trends

| Date | Positive | Negative | Overall | Notes |
|------|----------|----------|---------|-------|
| 2026-01-22 | 100% (1719/1719) | 90.6% (500/552) | 97.7% (2219/2271) | Per-vertex props + color validation |
| Previous | 100% (1698/1698) | 33.8% (160/473) | 77.4% (1858/2400) | Baseline before improvements |

**Improvement:** +56.8 percentage points in negative test compliance, +20.3 points overall.

## Known Limitations

1. **Vertex Order Validation (0418)**: Intentionally disabled due to complexity and reliability issues
2. **DTD Declaration (0420)**: Validated in parser, not post-parse validator  
3. **Build Transform Bounds (0421)**: Intentionally disabled to avoid false positives
4. **Slice Extension**: Requires additional validation implementation
5. **Some Materials Tests**: Edge cases in complex material property combinations

See `src/validator.rs` for detailed comments on each intentionally disabled validation.

## References

- [3MF Core Specification v1.4.0](https://github.com/3MFConsortium/spec_core/blob/1.4.0/3MF%20Core%20Specification.md)
- [3MF Test Suites](https://github.com/3MFConsortium/test_suites)
- [3MF Consortium](https://3mf.io/)
