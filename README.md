# lib3mf_rust

A pure Rust implementation for parsing 3MF (3D Manufacturing Format) files with **no unsafe code**.

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

## Overview

This library provides a pure Rust implementation for reading and parsing 3MF files, which are ZIP-based containers following the Open Packaging Conventions (OPC) standard and containing XML-based 3D model data.

The 3MF format is the modern standard for 3D printing, supporting rich model information including:
- 3D mesh geometry (vertices and triangles)
- Materials and colors
- Metadata
- Build specifications
- And more

## Features

- ✅ **Pure Rust implementation** - No C/C++ dependencies
- ✅ **No unsafe code** - Enforced with `#![forbid(unsafe_code)]`
- ✅ Parse 3MF file structure (ZIP/OPC container)
- ✅ Read 3D model data including meshes, vertices, and triangles
- ✅ Support for materials and colors
- ✅ Metadata extraction
- ✅ Build item specifications
- ✅ Comprehensive error handling

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
lib3mf = "0.1"
```

## Usage

### Basic Example

```rust
use lib3mf::Model;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open and parse a 3MF file
    let file = File::open("model.3mf")?;
    let model = Model::from_reader(file)?;

    // Access model information
    println!("Unit: {}", model.unit);
    println!("Objects: {}", model.resources.objects.len());

    // Iterate through objects
    for obj in &model.resources.objects {
        if let Some(ref mesh) = obj.mesh {
            println!("Object {} has {} vertices and {} triangles",
                obj.id, mesh.vertices.len(), mesh.triangles.len());
        }
    }

    Ok(())
}
```

### Running Examples

The repository includes a complete example demonstrating parsing:

```bash
cargo run --example parse_3mf
```

## Architecture

The library is organized into several modules:

- **`model`** - Data structures representing 3MF models (vertices, triangles, meshes, objects, etc.)
- **`opc`** - Open Packaging Conventions (OPC) handling for ZIP-based containers
- **`parser`** - XML parsing for 3D model files
- **`error`** - Comprehensive error types with detailed messages

## 3MF Format Support

This implementation currently supports:

- **Core 3MF Specification**
  - Model structure (resources, build, metadata)
  - Mesh geometry (vertices, triangles)
  - Object definitions
  - Build items with transformations
  - Basic materials and colors

- **Materials Extension**
  - Color groups (m:colorgroup)
  - Per-triangle material references (pid attributes)
  - Base materials with display colors

### Extension Support Status

The parser can successfully read and parse files using the following 3MF extensions:

- ✅ **Core Specification** - Fully supported
- ✅ **Materials Extension** - Color groups and base materials supported
- ⚠️  **Production Extension** - Files parse successfully, UUID attributes not yet extracted
- ⚠️  **Slice Extension** - Files parse successfully, slice data not yet extracted
- ⚠️  **Beam Lattice Extension** - Files parse successfully, beam data not yet extracted

Note: Files using extensions parse correctly and basic mesh data is extracted. 
Extension-specific data structures (beams, slices, UUIDs) are not yet fully modeled.

### Future Enhancements

Potential future additions could include:
- Full production extension support (UUID extraction, path references)
- Slice extension support (slice stacks and slice data)
- Beam lattice extension support (beam definitions and properties)
- Advanced material properties (textures, composite materials)
- Components and assemblies
- Custom extensions

## Testing

The library includes comprehensive unit and integration tests:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run linter
cargo clippy -- -D warnings
```

## Safety

This library is designed with safety in mind:

- **No unsafe code** - The entire codebase forbids unsafe code
- **Type safety** - Leverages Rust's type system for correctness
- **Memory safety** - All memory management is handled by Rust's ownership system
- **Error handling** - Comprehensive error types using `thiserror`

## Dependencies

The library uses minimal, well-maintained dependencies:

- `zip` - For reading ZIP archives (3MF container format)
- `quick-xml` - For parsing XML model files
- `thiserror` - For error handling

All dependencies are regularly updated and checked for vulnerabilities.

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE) or http://opensource.org/licenses/MIT)

at your option.

## References

- [3MF Specification](https://3mf.io/specification/)
- [3MF Consortium](https://3mf.io/)
- [lib3mf (Official C++ implementation)](https://github.com/3MFConsortium/lib3mf)

## Acknowledgments

This implementation is inspired by the official lib3mf library but is a complete rewrite in Rust with a focus on safety and simplicity.

