# GitHub Copilot Instructions for lib3mf_rust

## Project Overview

This is a **pure Rust implementation** for parsing and writing 3MF (3D Manufacturing Format) files with **no unsafe code**. The library provides complete support for the 3MF core specification and multiple extensions including Materials, Production, Slice, Beam Lattice, Secure Content, Boolean Operations, and Displacement.

**Key Technologies:**
- Language: Rust (edition 2021)
- Core dependencies: `zip`, `quick-xml`, `thiserror`, `parry3d`, `nalgebra`, `clipper2`
- Testing: Standard Rust testing with `cargo test`, official 3MF conformance tests
- Safety: `#![forbid(unsafe_code)]` enforced throughout the codebase

## Essential Commands

### Building and Testing
```bash
# Build the library
cargo build

# Run all tests (standard unit and integration tests)
cargo test

# Run specific test suite
cargo test <test_name>

# Run conformance tests (official 3MF test suites)
cargo test --test conformance_tests summary -- --ignored --nocapture

# Run specific conformance suite
cargo test --test conformance_tests suite3_core -- --nocapture

# Run linter
cargo clippy -- -D warnings

# Run benchmarks
cargo bench
```

### Examples
```bash
# Run basic parsing example
cargo run --example parse_3mf

# Test extension support
cargo run --example extension_support test_files/material/kinect_scan.3mf all

# Export to other formats
cargo run --example export_to_stl test_files/core/box.3mf output.stl
cargo run --example export_to_obj test_files/core/box.3mf output.obj
```

## Project Structure

```
src/
├── lib.rs              # Main library entry point
├── model.rs            # Data structures (Model, Object, Mesh, Vertex, Triangle, etc.)
├── parser.rs           # XML parsing for 3MF model files
├── opc.rs              # Open Packaging Conventions (ZIP container handling)
├── writer.rs           # 3MF file serialization
├── validator.rs        # Model validation logic
├── error.rs            # Error types and handling
├── mesh_ops.rs         # Mesh operations (volume, AABB, transformations)
├── polygon_clipping.rs # Polygon operations for slice data
├── streaming.rs        # Streaming parser for large files
└── decryption.rs       # Secure Content extension decryption

tests/                  # Integration and conformance tests
examples/               # Usage examples
benches/                # Performance benchmarks
test_files/             # Test data organized by extension
```

## Code Style and Conventions

### General Rust Style
- Follow standard Rust conventions (rustfmt formatting)
- Use meaningful variable names that reflect 3MF terminology
- Prefer explicit types over type inference when it improves clarity
- **NO unsafe code allowed** - forbidden at crate level

### Error Handling
- Use `Result<T, Error>` for all fallible operations
- Return descriptive errors using the `Error` enum defined in `error.rs`
- Use `thiserror` for custom error types
- Include context in error messages (e.g., file paths, element names, line numbers when available)

### Testing Practices
1. **Unit tests**: Test individual functions and modules in the same file using `#[cfg(test)]`
2. **Integration tests**: Test complete workflows in `tests/` directory
3. **Conformance tests**: Official 3MF Consortium test suites in `tests/conformance_tests.rs`
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
- Struct field names match 3MF XML attribute names when possible (lowercase, no underscores)
- Constants use `UPPER_SNAKE_CASE`

### Module Organization
- Keep related functionality together (e.g., all Material extension types in `model.rs`)
- Separate concerns: parsing logic in `parser.rs`, data structures in `model.rs`
- Use `pub(crate)` for internal APIs that shouldn't be exposed

## Safety and Security

### Absolutely Never
- **Never add unsafe code** - The crate has `#![forbid(unsafe_code)]`
- Never use panics in library code - always return `Result` with proper errors
- Never expose raw pointers in public APIs
- Never skip input validation for untrusted data (3MF files are untrusted)
- Never commit sensitive data or credentials
- Never modify test files in `test_files/` - these are reference data

### Security Considerations
- Validate all XML input thoroughly
- Check array bounds and prevent integer overflows
- Validate resource ID references to prevent infinite loops
- Handle ZIP bomb scenarios (extremely large compressed files)
- For Secure Content: Use test keys only for conformance testing, document production requirements

### Input Validation
- Validate all XML attributes against spec constraints
- Check numeric ranges (e.g., triangle indices must reference valid vertices)
- Verify resource ID uniqueness within appropriate namespaces
- Validate required vs optional attributes per 3MF specification
- Reject malformed color formats, invalid UUIDs, etc.

## 3MF-Specific Guidelines

### Extension Support
- Core specification is always supported
- Extensions are opt-in via `ParserConfig`
- When adding extension support:
  1. Add types to `model.rs`
  2. Implement parsing in `parser.rs`
  3. Add validation in `validator.rs`
  4. Create integration tests
  5. Update documentation
  6. Add conformance tests if available

### XML Parsing Patterns
- Use `quick-xml` for event-based parsing
- Match on event types: `Event::Start`, `Event::Text`, `Event::End`
- Extract attributes using `attributes()` iterator
- Handle namespaces correctly (check prefix and local name)
- Buffer text content across multiple `Event::Text` events

### Resource Management
- Objects and property groups have **separate ID namespaces**
- Validate that references (e.g., `pid`, `objectid`) point to valid resources
- Handle circular references in component hierarchies

### Conformance Testing
- Target 100% pass rate on positive tests (valid 3MF files)
- Properly reject invalid files in negative tests
- Document any intentional deviations from spec with rationale

## Git Workflow

### Branches
- Main branch is protected
- Create feature branches for new work
- Use descriptive branch names: `feature/beam-lattice-support`, `fix/color-validation`

### Commits
- Write clear, concise commit messages
- Group related changes together
- Run tests before committing: `cargo test && cargo clippy -- -D warnings`

### Pull Requests
- Ensure all tests pass in CI
- Update documentation for user-facing changes
- Add examples for new features
- Include benchmark results for performance-sensitive changes

## Performance Considerations

- Parsing should scale linearly with file size
- Use pre-allocated buffers where possible
- Profile with `cargo bench` for performance-critical code
- Avoid unnecessary allocations in hot paths

## Additional Resources

- [3MF Specification](https://3mf.io/specification/) - Official format documentation
