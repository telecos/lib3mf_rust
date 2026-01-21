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
- ✅ **Extension support** - Configurable support for 3MF extensions with validation
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

### Accessing Displacement Data

The library parses displacement extension data into accessible structures:

```rust
use lib3mf::Model;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("displaced_model.3mf")?;
    let model = Model::from_reader(file)?;

    // Access displacement maps
    for disp_map in &model.resources.displacement_maps {
        println!("Displacement map {} at path: {}", disp_map.id, disp_map.path);
        println!("  Channel: {:?}, Filter: {:?}", disp_map.channel, disp_map.filter);
        println!("  Tile U: {:?}, Tile V: {:?}", disp_map.tilestyleu, disp_map.tilestylev);
    }

    // Access normalized vector groups
    for nvgroup in &model.resources.norm_vector_groups {
        println!("NormVectorGroup {} with {} vectors", nvgroup.id, nvgroup.vectors.len());
        for (i, vec) in nvgroup.vectors.iter().enumerate() {
            println!("  Vector {}: ({}, {}, {})", i, vec.x, vec.y, vec.z);
        }
    }

    // Access displacement coordinate groups
    for d2dgroup in &model.resources.disp2d_groups {
        println!("Disp2DGroup {} using displacement map {} and vectors {}",
            d2dgroup.id, d2dgroup.dispid, d2dgroup.nid);
        println!("  Height: {}, Offset: {}", d2dgroup.height, d2dgroup.offset);
        println!("  Coordinates: {} entries", d2dgroup.coords.len());
    }

    Ok(())
}
```

### Extension Support

3MF files can require specific extensions beyond the core specification. You can control which extensions your application supports:

```rust
use lib3mf::{Model, ParserConfig, Extension};
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("model.3mf")?;
    
    // Configure which extensions you support
    let config = ParserConfig::new()
        .with_extension(Extension::Material)
        .with_extension(Extension::Production);
    
    // Parse with validation - will reject files requiring unsupported extensions
    let model = Model::from_reader_with_config(file, config)?;
    
    // Check what extensions are required by the file
    for ext in &model.required_extensions {
        println!("File requires extension: {}", ext.name());
    }
    
    Ok(())
}
```

The parser supports the following extensions:
- `Extension::Core` - Core 3MF specification (always supported)
- `Extension::Material` - Materials & Properties
- `Extension::Production` - Production information (UUIDs, paths)
- `Extension::Slice` - Slice data for layer-by-layer manufacturing
- `Extension::BeamLattice` - Beam and lattice structures
- `Extension::SecureContent` - Digital signatures and encryption
- `Extension::BooleanOperations` - Volumetric design
- `Extension::Displacement` - Surface displacement maps

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

- **Displacement Extension**
  - Displacement2D resources (displacement maps with PNG textures)
  - NormVectorGroup (normalized displacement vectors)
  - Disp2DGroup (displacement coordinate groups)
  - Displacement coordinates (u, v, n, f values)
  - Texture filtering and tiling modes
  - Surface displacement data structures

### Extension Support and Validation

The parser supports **conditional extension validation**, allowing consumers to specify which 3MF extensions they support. When a 3MF file declares required extensions via the `requiredextensions` attribute, the parser validates that all required extensions are supported before processing the file.

**Supported Extensions:**

- ✅ **Core Specification** - Fully supported (always enabled)
- ✅ **Materials Extension** - Color groups and base materials
- ✅ **Production Extension** - Files parse successfully
- ✅ **Slice Extension** - Files parse successfully  
- ✅ **Beam Lattice Extension** - Files parse successfully
- ✅ **Secure Content Extension** - Recognized and validated
- ✅ **Boolean Operations Extension** - Recognized and validated
- ✅ **Displacement Extension** - Recognized and validated

**Validation Behavior:**

By default, `Model::from_reader()` accepts files with any known extension for backward compatibility. Use `Model::from_reader_with_config()` to enforce specific extension requirements:

```rust
// Only accept core files (no extensions)
let config = ParserConfig::new();

// Accept core + materials
let config = ParserConfig::new().with_extension(Extension::Material);

// Accept all known extensions
let config = ParserConfig::with_all_extensions();
```

**Note:** While all extensions are recognized for validation purposes, extension-specific data structures (beams, slices, production UUIDs) are not yet fully extracted. Basic mesh geometry and materials are fully supported.

### Future Enhancements

Potential future additions could include:
- Slice extension support (slice stacks and slice data)
- Beam lattice extension support (beam definitions and properties)
- Advanced material properties (textures, composite materials)
- Components and assemblies
- Custom extensions

## Testing

The library includes comprehensive unit, integration, and conformance tests:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run linter
cargo clippy -- -D warnings

# Run official 3MF conformance tests
cargo test --test conformance_tests summary -- --ignored --nocapture
```

### Conformance Testing

This library has been validated against the official [3MF Consortium test suites](https://github.com/3MFConsortium/test_suites), which include over 2,200 test cases covering all 3MF specifications and extensions.

**Current Conformance Results:**
- ✅ **100% Positive Test Compliance**: All 1,698 valid 3MF files parse successfully
- ✅ **33.8% Negative Test Compliance**: 160 out of 473 invalid files are correctly rejected
- 📊 **77.4% Overall Conformance**: 1,858 out of 2,400 total tests pass

**Negative Test Improvements:**
- ✅ Duplicate metadata names - ensures metadata uniqueness
- ✅ Duplicate resource IDs - validates color group ID uniqueness
- ✅ Invalid XML structure - rejects malformed models
- ⚠️ Component validation - requires component support implementation
- ⚠️ Extension-specific validation - requires extension resource parsing

The parser successfully handles files using all 3MF extensions including:
- Core Specification (1.4.0)
- Materials & Properties Extension (1.2.1)
- Production Extension (1.2.0)
- Slice Extension (1.0.2)
- Beam Lattice Extension (1.2.0)
- Secure Content Extension (1.0.2) - ⚠️ **Read-only validation** (no cryptographic operations)
- Boolean Operations Extension (1.1.1)
- Displacement Extension (1.0.0)

**Important Security Note**: The Secure Content extension is recognized for validation purposes only. This library does NOT implement cryptographic operations (encryption, decryption, or signature verification). See [SECURE_CONTENT_SUPPORT.md](SECURE_CONTENT_SUPPORT.md) for detailed security considerations and integration guidance.

See [CONFORMANCE_REPORT.md](CONFORMANCE_REPORT.md) for detailed test results and analysis.

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

