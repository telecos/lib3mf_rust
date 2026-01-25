# Migration Script for Expected Failures

This script migrates the `expected_failures.json` file from the old format to the new format that supports multiple suites per test case ID.

## Usage

```bash
python3 migrate_expected_failures.py
```

## What it does

1. Reads the existing `tests/expected_failures.json` file
2. Groups test cases by their test case ID (extracted from filename)
3. Merges entries that have the same test case ID, test type, and reason
4. Generates a new file `tests/expected_failures_new.json` with the migrated format
5. Prints a summary of the migration

## Output

The script creates:
- `tests/expected_failures_new.json` - The migrated file in new format

You can then review the new file and replace the old one:

```bash
mv tests/expected_failures_new.json tests/expected_failures.json
```

## Example

Before (old format with duplicates):
```json
{
  "expected_failures": [
    {
      "file": "N_XPM_0421_01.3mf",
      "suite": "suite2_core_prod_matl",
      "test_type": "negative",
      "reason": "Build transform bounds validation...",
      ...
    },
    {
      "file": "N_XXX_0421_01.3mf",
      "suite": "suite3_core",
      "test_type": "negative",
      "reason": "Build transform bounds validation...",
      ...
    }
  ]
}
```

After (new format with grouped suites):
```json
{
  "expected_failures": [
    {
      "test_case_id": "0421_01",
      "suites": ["suite2_core_prod_matl", "suite3_core"],
      "test_type": "negative",
      "reason": "Build transform bounds validation...",
      ...
    }
  ]
}
```

## Notes

- The script preserves backward compatibility by keeping the old format fields when they don't match the pattern
- Test case IDs are extracted from filenames following the pattern: `[P/N]_[PREFIX]_[NNNN]_[NN].3mf`
- Entries with similar reasons are normalized and grouped together
- The earliest date_added is preserved when merging entries
