# Expected Test Failures Infrastructure - Implementation Summary

## Overview

This document summarizes the implementation of the expected test failures infrastructure for the 3MF conformance testing system.

## Problem Statement

A test file `P_XXX_2202_01.3mf` in suite 9 was failing because it is incorrect in the official 3MF Consortium test suite:
- The file declares the production extension as a required extension
- However, the build element does not have a UUID attribute, which is mandatory when using the production extension
- Suite 9 is for testing core extensions, and this appears to be a mistake in the official test suite
- Since we cannot modify the official test suite files, we needed a way to document expected failures

## Solution

We implemented a comprehensive expected test failures infrastructure that allows documenting test files that are known to be incorrect or have issues that cannot be resolved on our side.

### Components

1. **Configuration File** (`tests/expected_failures.json`)
   - JSON format for easy editing and validation
   - Documents each expected failure with:
     - File name
     - Suite name
     - Test type (positive/negative)
     - Detailed reason
     - Optional issue URL
     - Date added

2. **Expected Failures Manager** (`tests/common/expected_failures.rs`)
   - Loads and parses the configuration file
   - Provides methods to check if a test is expected to fail
   - Includes unit tests for validation
   - Cloneable for use in parallel test execution

3. **Integration with Test Infrastructure**
   - Updated `tests/conformance_tests.rs` to use expected failures
   - Updated `tests/conformance_individual.rs` to use expected failures
   - Both test files now handle expected failures consistently

4. **Validation Tests** (`tests/expected_failures_test.rs`)
   - Tests that the configuration file is valid JSON
   - Tests that expected failures are loaded correctly
   - Tests that specific files (like P_XXX_2202_01.3mf) are properly marked
   - Tests the cloneability of the manager

5. **Documentation** (`tests/EXPECTED_FAILURES.md`)
   - Comprehensive guide on how to use the system
   - Explains the configuration file structure
   - Provides examples and maintenance guidelines

## Behavior

### For Positive Tests (files expected to parse successfully)

When a file is marked as an expected failure:
- If the file **fails** to parse → Test **passes** (expected behavior)
  - Prints: `✓ Expected failure: filename.3mf - Reason: ...`
- If the file **succeeds** in parsing → Test **fails**
  - This alerts us that the issue may have been fixed

### For Negative Tests (files expected to fail parsing)

When a file is marked as an expected failure:
- If the file **succeeds** in parsing → Test **passes** (expected behavior)
  - Prints: `✓ Expected failure: filename.3mf - Reason: ...`
- If the file **fails** to parse → Test **fails**
  - This alerts us that the issue may have been fixed

## Files Changed

1. `Cargo.toml` - Added serde and serde_json dependencies
2. `tests/expected_failures.json` - New configuration file
3. `tests/common/expected_failures.rs` - New module for expected failures management
4. `tests/common/mod.rs` - Export expected failures module
5. `tests/conformance_tests.rs` - Integrate expected failures
6. `tests/conformance_individual.rs` - Integrate expected failures
7. `tests/expected_failures_test.rs` - New test file
8. `tests/EXPECTED_FAILURES.md` - New documentation

## Testing

All tests pass successfully:
- ✅ Library tests (42 passed)
- ✅ Conformance tests (24 passed, 1 ignored)
- ✅ Expected failures tests (8 passed)
- ✅ No regressions introduced

## Usage Example

To add a new expected failure:

```json
{
  "file": "problematic_file.3mf",
  "suite": "suite9_core_ext",
  "test_type": "positive",
  "reason": "Detailed explanation of the issue",
  "issue_url": "https://github.com/example/issue/123",
  "date_added": "2026-01-23"
}
```

## Benefits

1. **Clear Documentation** - All known issues are documented in one place with reasons
2. **Automated Handling** - Tests automatically pass for expected failures
3. **Alert on Fixes** - If an expected failure starts passing, we're notified
4. **No Code Changes** - Test logic remains clean and simple
5. **Easy Maintenance** - Simple JSON file for adding/removing expected failures
6. **Comprehensive Testing** - The infrastructure itself is well-tested

## Future Considerations

- Consider adding a warning when expected failures are too old (e.g., > 1 year)
- Could integrate with issue tracking to automatically update status
- May want to add statistics about expected failures in test reports
