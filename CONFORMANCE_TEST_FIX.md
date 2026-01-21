# Conformance Test Extension Support Fix

## Problem Statement

The conformance tests were not running with appropriate extension support enabled for each test suite. Previously, all tests used `Model::from_reader()` which defaults to `ParserConfig::with_all_extensions()`. This meant:

- All extensions were enabled regardless of what the suite was testing
- Files requiring unsupported extensions would incorrectly succeed
- Extension validation was not being properly tested

For example, when running Suite 8 (SecureContent), the parser should have SecureContent extension enabled. Similarly for other extension-specific suites.

## Solution

Created a centralized extension configuration system for conformance tests that maps each test suite to its required extensions.

### Changes Made

1. **Shared Test Utility Module** (`tests/common/mod.rs`)
   - Created `get_suite_config()` function to map suite names to required extensions
   - Eliminates code duplication across test files
   - Ensures consistent configuration

2. **Updated Conformance Tests** (`tests/conformance_tests.rs`)
   - Modified to use `common::get_suite_config()` for each suite
   - Updated test functions to accept and use `ParserConfig` parameter
   - Applied to both positive and negative tests

3. **Updated Individual Tests** (`tests/conformance_individual.rs`)
   - Modified to use `common::get_suite_config()` for each suite
   - Updated test functions to accept and use `ParserConfig` parameter

4. **Verification Tests**
   - `conformance_config_test.rs`: Tests that each suite has correct extension configuration
   - `extension_validation_test.rs`: Integration tests with real 3MF files proving extension validation works

### Extension Mappings

| Suite | Extensions Enabled |
|-------|-------------------|
| Suite 1 | Core + Production + Slice |
| Suite 2 | Core + Production + Materials |
| Suite 3 | Core only |
| Suite 4 | Core + Slice |
| Suite 5 | Core + Production |
| Suite 6 | Core + Materials |
| Suite 7 | Core + Beam Lattice |
| Suite 8 | Core + Secure Content |
| Suite 9 | All extensions |
| Suite 10 | Core + Boolean Operations |
| Suite 11 | Core + Displacement |

## Testing

All tests pass successfully:

```
Running tests/conformance_tests.rs
  22 passed; 0 failed; 1 ignored

Running tests/conformance_individual.rs
  (passes when test_suites directory exists)

Running tests/conformance_config_test.rs
  11 passed; 0 failed

Running tests/extension_validation_test.rs
  4 passed; 0 failed
```

## Impact

- **Better Test Coverage**: Now properly validates that files requiring specific extensions fail when those extensions aren't enabled
- **Correct Validation**: Each suite tests only the extensions it's designed for
- **Maintainable**: Centralized configuration makes it easy to update or add new suites
- **Documented**: Clear mapping of which extensions each suite requires

## Example Behavior

Before this change:
```rust
// Suite 8 (SecureContent) tests ran with ALL extensions enabled
Model::from_reader(file)  // Uses with_all_extensions()
```

After this change:
```rust
// Suite 8 runs with only Core + SecureContent enabled
let config = ParserConfig::new().with_extension(Extension::SecureContent);
Model::from_reader_with_config(file, config)
```

This ensures that if a file in Suite 8 requires an extension other than SecureContent (like BeamLattice), it will correctly fail with an `UnsupportedExtension` error.
