# lib3mf_rust

A pure Rust implementation for reading and writing 3MF (3D Manufacturing Format) files with **no unsafe code**.

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

## Overview

This library provides a pure Rust implementation for reading and writing 3MF files, which are ZIP-based containers following the Open Packaging Conventions (OPC) standard and containing XML-based 3D model data.

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
- ✅ **Read and write** 3MF files with full round-trip support
- ✅ Parse 3MF file structure (ZIP/OPC container)
- ✅ Read and write 3D model data including meshes, vertices, and triangles
- ✅ Support for materials and colors
- ✅ Metadata extraction and creation
- ✅ Build item specifications
- ✅ Comprehensive error handling

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
lib3mf = "0.1"
```

## Usage

### Reading 3MF Files

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

### Writing 3MF Files

```rust
use lib3mf::{Model, Object, Mesh, Vertex, Triangle, BuildItem};
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new model
    let mut model = Model::new();
    model.metadata.insert("Title".to_string(), "My 3D Model".to_string());
    model.metadata.insert("Designer".to_string(), "Rust App".to_string());

    // Create a simple triangle mesh (a pyramid)
    let mut mesh = Mesh::new();
    
    // Add vertices
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));    // Base vertices
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(10.0, 10.0, 0.0));
    mesh.vertices.push(Vertex::new(0.0, 10.0, 0.0));
    mesh.vertices.push(Vertex::new(5.0, 5.0, 10.0));   // Apex
    
    // Add triangles (faces)
    mesh.triangles.push(Triangle::new(0, 1, 4));  // Side faces
    mesh.triangles.push(Triangle::new(1, 2, 4));
    mesh.triangles.push(Triangle::new(2, 3, 4));
    mesh.triangles.push(Triangle::new(3, 0, 4));
    mesh.triangles.push(Triangle::new(0, 2, 1));  // Base
    mesh.triangles.push(Triangle::new(0, 3, 2));

    // Create an object with the mesh
    let mut obj = Object::new(1);
    obj.name = Some("Pyramid".to_string());
    obj.mesh = Some(mesh);
    model.resources.objects.push(obj);

    // Add to build (specify what to manufacture)
    model.build.items.push(BuildItem::new(1));

    // Write to file
    let file = File::create("pyramid.3mf")?;
    model.to_writer(file)?;
    
    println!("3MF file created successfully!");
    Ok(())
}
```

### Working with Materials and Colors

```rust
use lib3mf::{Model, Object, Mesh, Vertex, Triangle, BuildItem, Material, Extension};
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut model = Model::new();
    
    // Add material extension support
    model.required_extensions.push(Extension::Material);
    
    // Add colored materials (R, G, B, A)
    model.resources.materials.push(Material::with_color(1, 255, 0, 0, 255));  // Red
    model.resources.materials.push(Material::with_color(2, 0, 255, 0, 255));  // Green
    model.resources.materials.push(Material::with_color(3, 0, 0, 255, 255));  // Blue
    
    // Create a mesh with colored triangles
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));
    mesh.vertices.push(Vertex::new(5.0, 5.0, 10.0));
    
    // Create triangles with different colors
    mesh.triangles.push(Triangle::with_material(0, 1, 2, 1));  // Red triangle
    mesh.triangles.push(Triangle::with_material(0, 1, 3, 2));  // Green triangle
    mesh.triangles.push(Triangle::with_material(1, 2, 3, 3));  // Blue triangle
    
    let mut obj = Object::new(1);
    obj.mesh = Some(mesh);
    model.resources.objects.push(obj);
    
    model.build.items.push(BuildItem::new(1));
    
    let file = File::create("colored.3mf")?;
    model.to_writer(file)?;
    
    Ok(())
}
```

### Round-Trip Processing

```rust
use lib3mf::Model;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read existing file
    let input = File::open("input.3mf")?;
    let mut model = Model::from_reader(input)?;
    
    // Modify the model
    model.metadata.insert("Modified".to_string(), "true".to_string());
    
    // Write modified model
    let output = File::create("output.3mf")?;
    model.to_writer(output)?;
    
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

The repository includes complete examples demonstrating both reading and writing:

```bash
# Parse an existing 3MF file
cargo run --example parse_3mf

# Create and write 3MF files
cargo run --example write_3mf
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
- Full production extension support (UUID extraction, path references)
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
- ⚠️ **1.7% Negative Test Compliance**: 9 out of 543 invalid files are correctly rejected

The parser successfully handles files using all 3MF extensions including:
- Core Specification (1.4.0)
- Materials & Properties Extension (1.2.1)
- Production Extension (1.2.0)
- Slice Extension (1.0.2)
- Beam Lattice Extension (1.2.0)
- Secure Content Extension (1.0.2)
- Boolean Operations Extension (1.1.1)
- Displacement Extension (1.0.0)

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

