# Expected Test Failures

This document explains how to use the expected test failures infrastructure in the conformance testing system.

## Overview

The expected failures mechanism allows us to document and handle test files in the official 3MF Consortium test suites that are known to be incorrect or have issues that cannot be resolved on our side.

This ensures that:
1. We can still run the full conformance test suite without false failures
2. Known issues are properly documented with reasons
3. If an expected failure suddenly passes, we'll be notified (indicating the issue was fixed)

## Configuration File

Expected failures are configured in `tests/expected_failures.json`. The file uses JSON format for easy editing and validation.

### Structure

The configuration supports two formats for backward compatibility:

#### New Format (Recommended)

Use this format when the same test case appears in multiple suites:

```json
{
  "expected_failures": [
    {
      "test_case_id": "0421_01",
      "suites": ["suite2_core_prod_matl", "suite3_core"],
      "test_type": "negative",
      "reason": "Detailed explanation of why this test is expected to fail",
      "issue_url": "https://github.com/example/issue/123",
      "date_added": "2026-01-23"
    }
  ]
}
```

#### Old Format (Still Supported)

Legacy format for backward compatibility:

```json
{
  "expected_failures": [
    {
      "file": "P_XXX_2202_01.3mf",
      "suite": "suite9_core_ext",
      "test_type": "positive",
      "reason": "Detailed explanation of why this test is expected to fail",
      "issue_url": "https://github.com/example/issue/123",
      "date_added": "2026-01-23"
    }
  ]
}
```

### Fields

#### New Format Fields

- **test_case_id** (required): The test case identifier extracted from the filename (e.g., `"0421_01"`, `"2202_01"`)
  - Extracted from filenames like `P_XXX_0421_01.3mf` → `0421_01`
  - The same test case ID appears across different suites with different file prefixes
- **suites** (required): Array of suite directory names where this test case appears (e.g., `["suite2_core_prod_matl", "suite3_core"]`)
- **test_type** (required): Either `"positive"` or `"negative"`
  - `"positive"`: The file is in the "Positive Tests" folder but is expected to fail parsing
  - `"negative"`: The file is in the "Negative Tests" folder but is expected to succeed parsing
- **reason** (required): A detailed explanation of why this test is expected to fail. Should include:
  - What is wrong with the file
  - Why it cannot be fixed on our side
  - Any relevant context about the issue
- **issue_url** (optional): URL to an issue tracker or documentation about this problem
- **date_added** (required): ISO date (YYYY-MM-DD) when this expected failure was added
- **expected_error_type** (optional): Expected error type string (e.g., `"InvalidModel"`, `"OutsidePositiveOctant"`)

#### Old Format Fields (Deprecated)

- **file** (required): The exact filename of the test file (e.g., `P_XXX_2202_01.3mf`)
- **suite** (required): The suite directory name (e.g., `suite9_core_ext`, `suite3_core`)
- Other fields same as new format

### Test Case Naming Convention

Test files follow this naming pattern: `[P/N]_[PREFIX]_[test_case_id].3mf`

- `P` = Positive test, `N` = Negative test
- PREFIX indicates enabled extensions:
  - `XXX` = Core only (suite3)
  - `XPM` = Core + Production + Materials (suite2)
  - `SPX` = Slice + Production (suite1)
  - `SXX` = Slice only (suite4)
  - `XPX` = Production only (suite5)
  - `XXM` = Materials only (suite6)
- test_case_id is the numeric identifier (e.g., `0421_01`)

The same test case ID may appear in multiple suites with different prefixes to test the same scenario with different extension combinations.

## How It Works

### For Positive Tests

A positive test (files in "Positive Tests" directories) is normally expected to parse successfully.

When marked as an expected failure:
- If the file **fails** to parse → Test **passes** (expected behavior)
- If the file **succeeds** in parsing → Test **fails** with a message indicating the file was expected to fail

### For Negative Tests

A negative test (files in "Negative Tests" directories) is normally expected to fail parsing.

When marked as an expected failure:
- If the file **succeeds** in parsing → Test **passes** (expected behavior)
- If the file **fails** to parse → Test **fails** with a message indicating the file was expected to succeed

## Adding a New Expected Failure

### For a Single Suite

1. Identify the failing test file and understand why it's failing
2. Determine if it's a problem with the test file itself (not our implementation)
3. Add an entry to `tests/expected_failures.json` using the new format:

```json
{
  "test_case_id": "0420_01",
  "suites": ["suite3_core"],
  "test_type": "negative",
  "reason": "Your detailed explanation here",
  "issue_url": "",
  "date_added": "YYYY-MM-DD"
}
```

### For Multiple Suites

If the same test case appears in multiple suites (identified by the test case ID like `0421_01`), add a single entry with all suites:

```json
{
  "test_case_id": "0421_01",
  "suites": ["suite2_core_prod_matl", "suite3_core"],
  "test_type": "negative",
  "reason": "Build transform bounds validation. The file tests for negative coordinates...",
  "issue_url": "",
  "date_added": "2026-01-25"
}
```

This will automatically match:
- `N_XPM_0421_01.3mf` in suite2_core_prod_matl
- `N_XXX_0421_01.3mf` in suite3_core

### Migration from Old Format

If you have old format entries that need to be combined, use the migration script:

```bash
python3 migrate_expected_failures.py
```

This will automatically:
- Group test cases by their test case ID
- Merge entries with the same test case ID and reason
- Generate the new format with multiple suites

4. Run the tests to verify the expected failure is handled correctly
5. Commit the changes with a descriptive message

## Examples

### Example 1: Multiple Suites

Test case `0421_01` appears in multiple suites testing the same issue - negative coordinates below build plate. Instead of duplicating the configuration, we use a single entry:

```json
{
  "test_case_id": "0421_01",
  "suites": ["suite2_core_prod_matl", "suite3_core", "suite5_core_prod", "suite6_core_matl"],
  "test_type": "negative",
  "reason": "Build transform bounds validation. The file tests for negative coordinates (below build plate). However, the 3MF specification allows negative coordinates for centering objects. The test case appears to be testing a constraint that is not part of the core 3MF specification.",
  "issue_url": "",
  "date_added": "2026-01-25"
}
```

This automatically handles:
- `N_XPM_0421_01.3mf` in suite2_core_prod_matl
- `N_XXX_0421_01.3mf` in suite3_core
- `N_XPX_0421_01.3mf` in suite5_core_prod
- `N_XXM_0421_01.3mf` in suite6_core_matl

### Example 2: Single Suite (Old Format Compatible)

For a test case that only appears in one suite, you can still use the new format:

```json
{
  "test_case_id": "2202_01",
  "suites": ["suite9_core_ext"],
  "test_type": "positive",
  "reason": "File is incorrect per official test suite. It declares production extension as required but the build element does not have a UUID attribute, which is mandatory when using the production extension.",
  "issue_url": "",
  "date_added": "2026-01-23"
}
```

This handles `P_XXX_2202_01.3mf` in suite9_core_ext.

## Running Tests

The expected failures infrastructure is automatically integrated into all conformance tests:

```bash
# Run all conformance tests (includes expected failures handling)
cargo test --test conformance_tests

# Run individual file tests (includes expected failures handling)
cargo test --test conformance_individual

# Run a specific suite
cargo test --test conformance_tests suite9_core_ext
```

When a test is handled as an expected failure, you'll see output like:
```
✓ Expected failure: P_XXX_2202_01.3mf - Reason: File is incorrect per official test suite...
```

## Maintenance

Periodically review the expected failures list to check if:
1. The official test suite has been updated and the issues fixed
2. Our implementation has changed in a way that affects the expected failures
3. Any expected failures are no longer needed

If an expected failure starts passing unexpectedly, the test will fail with a message explaining that the file was expected to fail but succeeded. This prompts you to investigate and potentially remove it from the expected failures list.
