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

- **file** (required): The exact filename of the test file (e.g., `P_XXX_2202_01.3mf`)
- **suite** (required): The suite directory name (e.g., `suite9_core_ext`, `suite3_core`)
- **test_type** (required): Either `"positive"` or `"negative"`
  - `"positive"`: The file is in the "Positive Tests" folder but is expected to fail parsing
  - `"negative"`: The file is in the "Negative Tests" folder but is expected to succeed parsing
- **reason** (required): A detailed explanation of why this file is expected to fail. Should include:
  - What is wrong with the file
  - Why it cannot be fixed on our side
  - Any relevant context about the issue
- **issue_url** (optional): URL to an issue tracker or documentation about this problem
- **date_added** (required): ISO date (YYYY-MM-DD) when this expected failure was added

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

1. Identify the failing test file and understand why it's failing
2. Determine if it's a problem with the test file itself (not our implementation)
3. Add an entry to `tests/expected_failures.json`:

```json
{
  "file": "your_file_name.3mf",
  "suite": "suite_directory_name",
  "test_type": "positive",
  "reason": "Your detailed explanation here",
  "issue_url": "",
  "date_added": "YYYY-MM-DD"
}
```

4. Run the tests to verify the expected failure is handled correctly
5. Commit the changes with a descriptive message

## Example: P_XXX_2202_01.3mf

This file in suite 9 (Core Extensions) is marked as an expected failure because:

- The file declares the production extension as a required extension
- However, the build element does not have a UUID attribute
- UUID is mandatory when using the production extension
- Suite 9 is meant to test core extensions, and this appears to be a mistake in the official test suite
- Since we cannot modify the official test suite files, we document this as an expected failure

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
