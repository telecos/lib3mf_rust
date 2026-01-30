# Test Organization

This document describes the organization of tests in the lib3mf_rust project.

## Directory Structure

Tests are organized into logical categories that mirror the library's architecture:

```
tests/
├── common/                      # Shared test utilities and helpers
│   ├── mod.rs                  # Test utility functions
│   └── expected_failures.rs    # Conformance test failure tracking
│
├── core/                       # Core 3MF specification tests
│   ├── parsing.rs             # Core parsing functionality
│   ├── writing.rs             # Core writing/serialization
│   ├── components.rs          # Component hierarchies and validation
│   ├── metadata.rs            # Metadata handling
│   └── real_files.rs          # Integration tests with real 3MF files
│
├── extensions/                 # Extension-specific tests
│   ├── material/              # Material extension (Texture2D, composites)
│   │   ├── data_structures.rs
│   │   └── integration.rs
│   ├── production/            # Production extension
│   │   ├── coordinates.rs
│   │   └── regression.rs
│   ├── slice/                 # Slice extension
│   │   ├── integration.rs
│   │   └── mesh_operations.rs
│   ├── beam_lattice/          # Beam Lattice extension
│   │   └── integration.rs
│   ├── boolean_ops/           # Boolean Operations extension
│   │   └── integration.rs
│   ├── displacement/          # Displacement extension
│   │   ├── namespaces.rs
│   │   └── integration.rs
│   └── secure_content/        # Secure Content extension
│       ├── integration.rs
│       ├── handler.rs
│       └── key_provider.rs
│
├── validation/                 # Validation logic tests
│   ├── extensions.rs          # Extension validation
│   ├── errors.rs              # Error message quality
│   └── circular_references.rs # Circular dependency detection
│
├── infrastructure/             # Infrastructure and supporting systems
│   ├── thumbnails.rs          # Thumbnail handling
│   ├── custom_extensions.rs  # Custom extension API
│   ├── extension_registry.rs # Extension registry system
│   ├── extension_support.rs  # Extension support infrastructure
│   ├── post_parse_hooks.rs   # Parser lifecycle hooks
│   ├── writer_registry.rs    # Writer extension system
│   ├── mesh_operations.rs    # Mesh processing utilities
│   └── texture_paths.rs       # Texture path validation
│
├── regression/                 # Regression tests for specific issues
│   ├── issue_1605.rs          # Issue #1605 fix verification
│   ├── jpeg_cmyk.rs           # CMYK JPEG handling
│   ├── suite2_fixes.rs        # Suite 2 conformance fixes
│   ├── suite1_debug.rs        # Suite 1 debugging tests
│   └── error_type_validation.rs # Error type validation
│
├── conformance/                # Official 3MF conformance tests
│   ├── suites.rs              # All conformance test suites
│   ├── individual.rs          # Individual test file runner
│   ├── config.rs              # Suite configuration tests
│   └── expected_failures.rs   # Expected failure tracking
│
├── proptest_tests.rs          # Property-based testing
└── expected_failures.json     # Expected failures data

```

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test Suites
```bash
# Core functionality tests
cargo test --test core

# Extension tests
cargo test --test extensions

# Validation tests
cargo test --test validation

# Infrastructure tests
cargo test --test infrastructure

# Regression tests
cargo test --test regression

# Conformance tests (requires CI feature and test_suites/)
cargo test --test conformance

# Property-based tests
cargo test --test proptest_tests
```

### Run Specific Test Categories
```bash
# Material extension tests only
cargo test --test extensions material::

# Component validation tests
cargo test --test core components::

# Error message tests
cargo test --test validation errors::
```

## Test Organization Principles

1. **Separation by Concern**: Tests are grouped by the aspect of the library they test (core, extensions, validation, etc.)

2. **Consistency with Source**: The test structure mirrors the source code organization in `src/`

3. **Discoverability**: Test names and locations make it easy to find relevant tests

4. **Maintainability**: Related tests are grouped together, reducing duplication

5. **Extension Pattern**: Each extension has its own subdirectory with consistent naming:
   - `integration.rs` - End-to-end tests
   - `data_structures.rs` - Unit tests for data structures (if applicable)
   - Additional specific test files as needed

## Adding New Tests

### For Core Functionality
Add tests to the appropriate file in `tests/core/`:
- Parsing tests → `core/parsing.rs`
- Writing tests → `core/writing.rs`
- Component tests → `core/components.rs`
- Metadata tests → `core/metadata.rs`

### For Extensions
Create tests in `tests/extensions/<extension_name>/`:
- Use `integration.rs` for end-to-end tests
- Create additional files for specific test categories

### For New Extensions
1. Create a new directory: `tests/extensions/<new_extension>/`
2. Add `integration.rs` for main tests
3. Update `tests/extensions.rs` to include the new module

### For Bug Fixes
Add regression tests to `tests/regression/`:
- Name the file after the issue: `issue_NNNN.rs`
- Include a clear comment explaining what bug is being prevented

## Common Utilities

The `tests/common/` module provides shared utilities:
- Test file creation helpers
- Parser configuration builders
- Validation helpers
- Expected failures management for conformance tests

Use these utilities to avoid duplication and maintain consistency across tests.

## Migration Notes

This test organization was created to consolidate tests that were previously scattered across the repository. The new structure:
- Groups 43 previously scattered test files into 6 logical categories
- Provides clear module organization with main test files
- Maintains 100% test coverage from the previous structure
- Makes it easier to find and maintain tests

All tests from the previous structure have been preserved and continue to pass.
