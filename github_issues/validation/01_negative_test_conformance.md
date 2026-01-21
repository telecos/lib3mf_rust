---
name: Improve Negative Test Conformance to 90%+
about: CRITICAL - Implement validation to reject 534 invalid 3MF files
title: 'CRITICAL: Improve Negative Test Conformance from 1.7% to 90%+'
labels: 'validation, priority:critical, help wanted'
assignees: ''
---

## Description

**CRITICAL COMPLIANCE ISSUE**: Currently only 1.7% (9 out of 543) of invalid 3MF files are correctly rejected. The parser is far too permissive and accepts 534 files that violate the 3MF specification. This is the #1 blocker to spec compliance.

## Current State

- ✅ **100% Positive Test Compliance**: All 1,698 valid files parse correctly
- ⚠️ **1.7% Negative Test Compliance**: Only 9/543 invalid files rejected
- 📊 **76.2% Overall Conformance**: 1,707/2,241 tests pass
- ❌ **534 invalid files incorrectly accepted**

The parser correctly handles ALL valid files but fails to detect violations in invalid ones.

## Impact

- Spec non-compliance
- Applications may process corrupt/invalid 3MF files  
- Potential data loss or incorrect manufacturing
- Cannot trust validation results

## Expected Outcome

- Analyze the 534 failing negative tests systematically
- Categorize failures by spec violation type (use test codes)
- Implement validation rules for each category
- **Target: >90% negative test compliance** (485+ out of 543 rejected)
- Document any intentionally permissive behavior

## Implementation Approach

1. **Analyze Failures**:
   ```bash
   cargo run --example categorize_failures
   ```
   Groups failures by test code (e.g., N_XXX_0205 = specific violation type)

2. **Understand Requirements**:
   - For each code category, review 3MF spec
   - Understand what validation rule is missing
   - Identify which spec section is violated

3. **Implement Validators**:
   - Add validation rules to `src/validator.rs`
   - Ensure clear error messages citing spec sections
   - Test each validation rule

4. **Track Progress**:
   ```bash
   cargo run --example analyze_negative_tests
   ```
   Shows current pass/fail count

5. **Prioritize**:
   - Focus on high-frequency failure categories first
   - Quick wins: validation rules that catch many failures
   - Complex cases: May need deeper structural changes

## Test Infrastructure

- **543 negative test files** in `test_suites/*/negative_test_cases/`
- **Test naming**: `N_XXX_NNNN_VV.3mf` where NNNN is violation category
- **Analysis tools**:
  - `examples/categorize_failures.rs` - Groups by category
  - `examples/analyze_negative_tests.rs` - Counts pass/fail

## Validation Areas to Check

Based on common 3MF spec violations:
- [ ] Duplicate object IDs (may already work)
- [ ] Invalid vertex indices (may already work)
- [ ] Missing required attributes
- [ ] Invalid attribute values (negative, out of range)
- [ ] Circular component references
- [ ] Invalid material references
- [ ] Malformed XML structure
- [ ] Missing required elements
- [ ] Invalid relationships
- [ ] Namespace violations

## Acceptance Criteria

- [ ] Negative test compliance >90% (485+ / 543)
- [ ] Failures categorized and documented by test code
- [ ] Each validation rule has clear error message
- [ ] Error messages reference spec sections
- [ ] No regression in positive tests (maintain 100%)
- [ ] README.md updated with new conformance stats
- [ ] Conformance report generated

## References

- README.md, lines 204-206
- [3MF Core Specification](https://3mf.io/specification/)
- [3MF Consortium Test Suites](https://github.com/3MFConsortium/test_suites)

## Related Issues

- Base Materials Validation
- Component Reference Validation
- Metadata Validation
- Conformance Report Generation

## Priority

**CRITICAL** - This is the #1 gap in spec compliance. Without proper validation, the parser cannot be trusted to reject malformed files, potentially leading to data corruption or manufacturing errors.

## Effort Estimate

**Large (1-2 weeks)** - Requires systematic analysis of 534 test failures, implementation of multiple validation rules, and thorough testing. Can be broken into smaller sub-issues per violation category.

## Notes

Consider creating sub-issues for specific validation categories once failure analysis is complete. This will make implementation more manageable and allow parallel work.
