# Contributing to lib3mf_rust

Thank you for your interest in contributing to lib3mf_rust! This document provides guidelines for contributing to the project.

## Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Please be respectful and constructive in all interactions.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/lib3mf_rust.git`
3. Create a new branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Test your changes
6. Submit a pull request

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Git

### Building the Project

```bash
cargo build
```

### Running Tests

```bash
# Run all tests
cargo test

# Run conformance tests
cargo test --test conformance_tests summary -- --ignored --nocapture

# Run specific test suite
cargo test --test conformance_tests suite3_core -- --nocapture
```

### Running the Linter

```bash
cargo clippy -- -D warnings
```

### Running Benchmarks

```bash
cargo bench
```

## Code Style and Conventions

### General Rust Style

- Follow standard Rust conventions (use `rustfmt`)
- Use meaningful variable names that reflect 3MF terminology
- Prefer explicit types over type inference when it improves clarity
- **NO unsafe code allowed** - forbidden at crate level with `#![forbid(unsafe_code)]`

### Error Handling

- Use `Result<T, Error>` for all fallible operations
- Return descriptive errors using the `Error` enum defined in `error.rs`
- Use `thiserror` for custom error types
- Include context in error messages (e.g., file paths, element names)

### Testing Practices

1. **Unit tests**: Test individual functions and modules in the same file using `#[cfg(test)]`
2. **Integration tests**: Test complete workflows in `tests/` directory
3. **Conformance tests**: Official 3MF Consortium test suites
4. **Examples**: Working code in `examples/` that demonstrates features
5. **Always run clippy** before committing: `cargo clippy -- -D warnings`

### Documentation

- Add doc comments (`///`) for all public APIs
- Include usage examples in doc comments where helpful
- Keep README.md updated with new features
- Document complex algorithms with inline comments

### Naming Conventions

- Use 3MF specification terminology (e.g., `objectid`, `pid`, `requiredextensions`)
- Follow Rust naming: `snake_case` for functions/variables, `PascalCase` for types
- Struct field names match 3MF XML attribute names when possible
- Constants use `UPPER_SNAKE_CASE`

## Pull Request Guidelines

### Before Submitting

1. **Run all tests**: Ensure `cargo test` passes
2. **Run clippy**: Fix all warnings with `cargo clippy -- -D warnings`
3. **Format code**: Run `cargo fmt`
4. **Update documentation**: If you add features, update README.md and relevant docs
5. **Add examples**: For new features, add examples in `examples/`
6. **Update CHANGELOG.md**: Add your changes under "Unreleased"

### Pull Request Process

1. Ensure your PR has a clear title and description
2. Reference any related issues
3. Include test results if applicable
4. For user-facing changes, update documentation
5. Add benchmark results for performance-sensitive changes
6. Wait for CI checks to pass
7. Address any review feedback

### Commit Messages

- Write clear, concise commit messages
- Use the imperative mood ("Add feature" not "Added feature")
- First line should be 50 characters or less
- Include more details in the body if needed

Example:
```
Add support for displacement extension

- Implement displacement map parsing
- Add displacement coordinate groups
- Update conformance tests
```

## Adding New Features

### Extension Support

When adding support for a new 3MF extension:

1. Add types to `src/model.rs`
2. Implement parsing in `src/parser.rs`
3. Add validation in `src/validator.rs`
4. Create integration tests
5. Update documentation in README.md
6. Add conformance tests if available
7. Add examples demonstrating the feature

### Performance Considerations

- Parsing should scale linearly with file size
- Use pre-allocated buffers where possible
- Profile with `cargo bench` for performance-critical code
- Avoid unnecessary allocations in hot paths

## Testing

### Running Specific Tests

```bash
# Run a specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration_test_name
```

### Conformance Testing

This library is validated against the official [3MF Consortium test suites](https://github.com/3MFConsortium/test_suites). To run conformance tests:

```bash
# Run all conformance tests
cargo test --test conformance_tests summary -- --ignored --nocapture

# Run specific suite
cargo test --test conformance_tests suite3_core -- --nocapture
```

## Security

### Reporting Security Issues

If you discover a security vulnerability, please **do not** open a public issue. Instead, please email the maintainers directly or use GitHub's private security advisory feature.

### Security Guidelines

- Never add unsafe code (enforced by `#![forbid(unsafe_code)]`)
- Validate all XML input thoroughly
- Check array bounds and prevent integer overflows
- Validate resource ID references to prevent infinite loops
- Handle ZIP bomb scenarios
- Never commit secrets or credentials

## Getting Help

- Check existing [issues](https://github.com/telecos/lib3mf_rust/issues)
- Read the [README.md](README.md) and code documentation
- Look at [examples](examples/) for usage patterns
- Review the [3MF specification](https://3mf.io/specification/)

## License

By contributing to lib3mf_rust, you agree that your contributions will be licensed under the MIT License, the same as the project.
