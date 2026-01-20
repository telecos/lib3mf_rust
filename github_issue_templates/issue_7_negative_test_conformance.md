---
name: Improve Negative Test Conformance
about: Improve validation to correctly reject invalid 3MF files
title: 'Improve Negative Test Conformance (1.7% → 100%)'
labels: 'validation, priority:high, help wanted'
assignees: ''
---

## Description

Currently only 1.7% (9 out of 543) of invalid 3MF files are correctly rejected. The parser is too permissive and accepts many files that violate the 3MF specification. This is a critical gap in spec compliance.

## Current State

- ✅ 100% positive test compliance (1,698/1,698 valid files parse)
- ⚠️ 1.7% negative test compliance (9/543 invalid files rejected)
- 📊 76.2% overall conformance (1,707/2,241 total tests)

The parser correctly handles all valid files but fails to reject most invalid ones.

## Expected Outcome

- Analyze negative test failures systematically
- Categorize failures by error type (using test codes from filenames)
- Implement validation rules for each category
- Achieve >90% negative test compliance
- Document any intentionally permissive behavior

## Implementation Approach

1. **Analyze failures:** Run `cargo run --example categorize_failures` to group failures by test code
2. **Understand requirements:** For each code category, review the 3MF specification to understand what should be validated
3. **Implement validators:** Add validation rules to `src/validator.rs`
4. **Track progress:** Run `cargo run --example analyze_negative_tests` to measure improvement
5. **Prioritize:** Focus on high-frequency failure categories first

## Test Files

- 543 negative test cases available in `test_suites/*/negative_test_cases/`
- Test file naming: `N_XXX_NNNN_VV.3mf` where NNNN is the test code category
- Examples already exist: `examples/categorize_failures.rs`, `examples/analyze_negative_tests.rs`

## Acceptance Criteria

- [ ] Negative test compliance improved to >90% (485+ out of 543 invalid files rejected)
- [ ] Failures categorized by test code
- [ ] Validation rules documented
- [ ] Error messages are clear and reference spec violations
- [ ] No regression in positive tests (maintain 100%)
- [ ] README updated with new conformance statistics

## References

- README.md, lines 204-206
- TESTING_QUICK_REFERENCE.md, lines 131-133
- [3MF Core Specification](https://3mf.io/specification/)
- [3MF Consortium Test Suites](https://github.com/3MFConsortium/test_suites)

## Related Issues

- #[issue for base materials validation]
- #[issue for component validation]
- #[issue for metadata validation]

## Additional Context

This is the highest priority issue as it represents the largest gap in spec compliance. Breaking it down into smaller validation tasks (one per test code category) may make implementation more manageable.

Consider creating sub-issues for specific validation categories once the failure analysis is complete.
