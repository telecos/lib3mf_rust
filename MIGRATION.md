# Migration Guide: From lib3mf (C++) to lib3mf_rust

This guide helps you migrate from the official C++ lib3mf library to this pure Rust implementation.

## Table of Contents

- [Overview](#overview)
- [Key Differences](#key-differences)
- [API Comparison](#api-comparison)
- [Code Examples](#code-examples)
- [Feature Parity](#feature-parity)
- [Common Migration Patterns](#common-migration-patterns)
- [Error Handling](#error-handling)
- [Extension Support](#extension-support)
- [Performance Considerations](#performance-considerations)

## Overview

lib3mf_rust is a **pure Rust reimplementation** of the 3MF file format parser, designed with safety and simplicity in mind. Unlike the C++ lib3mf, this implementation:

- ✅ Contains **no unsafe code** (enforced with `#![forbid(unsafe_code)]`)
- ✅ Has **no C/C++ dependencies** (pure Rust)
- ✅ Leverages Rust's **type system and ownership** for memory safety
- ✅ Provides **simpler, more idiomatic Rust APIs**
- ⚠️ Currently **read-only** (writing 3MF files is not yet supported)

## Key Differences

### Philosophy

| Aspect | C++ lib3mf | lib3mf_rust |
|--------|------------|-------------|
| **Language** | C++ with C bindings | Pure Rust |
| **Safety** | Manual memory management | Automatic (ownership system) |
| **API Style** | Object-oriented, stateful | Functional, immutable |
| **Error Handling** | Return codes, exceptions | `Result<T, E>` types |
| **Operations** | Read and write | Read-only (for now) |
| **Dependencies** | External C++ libraries | Pure Rust crates |

### Architecture

**C++ lib3mf:**
- Uses wrapper classes and interfaces
- Stateful objects with setters/getters
- Explicit resource management
- Complex class hierarchies

**lib3mf_rust:**
- Simple data structures
- Immutable by default
- Automatic resource cleanup
- Flat module organization

## API Comparison

### Initialization and Model Loading

**C++ lib3mf:**
```cpp
#include <lib3mf_implicit.hpp>

// Initialize wrapper
Lib3MF::PWrapper wrapper = Lib3MF::CWrapper::loadLibrary();

// Create a model
Lib3MF::PModel model = wrapper->CreateModel();

// Read from file
Lib3MF::PReader reader = model->QueryReader("3mf");
reader->ReadFromFile("model.3mf");
```

**lib3mf_rust:**
```rust
use lib3mf::Model;
use std::fs::File;

// Open and parse in one step
let file = File::open("model.3mf")?;
let model = Model::from_reader(file)?;
```

### Accessing Model Data

**C++ lib3mf:**
```cpp
// Get build items
Lib3MF::PBuildItemIterator buildItems = model->GetBuildItems();

while (buildItems->MoveNext()) {
    Lib3MF::PBuildItem buildItem = buildItems->GetCurrent();
    Lib3MF::PObject object = buildItem->GetObjectResource();
    
    if (object->IsMeshObject()) {
        Lib3MF::PMeshObject meshObj = model->GetMeshObjectByID(object->GetResourceID());
        
        // Get vertex count
        Lib3MF_uint32 vertexCount = meshObj->GetVertexCount();
        
        // Get vertices
        std::vector<Lib3MF::sPosition> vertices;
        vertices.resize(vertexCount);
        meshObj->GetVertices(vertices.data());
    }
}
```

**lib3mf_rust:**
```rust
// Direct access to build items
for item in &model.build.items {
    // Find the corresponding object
    let object = model.resources.objects.iter()
        .find(|obj| obj.id == item.objectid);
    
    if let Some(obj) = object {
        if let Some(ref mesh) = obj.mesh {
            // Direct access to vertices
            let vertex_count = mesh.vertices.len();
            
            // Iterate vertices
            for vertex in &mesh.vertices {
                println!("Vertex: ({}, {}, {})", vertex.x, vertex.y, vertex.z);
            }
        }
    }
}
```

### Mesh Data

**C++ lib3mf:**
```cpp
// Get mesh object
Lib3MF::PMeshObject mesh = model->AddMeshObject();

// Add vertices
std::vector<Lib3MF::sPosition> vertices = {
    {0.0f, 0.0f, 0.0f},
    {10.0f, 0.0f, 0.0f},
    {5.0f, 10.0f, 0.0f}
};
mesh->SetVertices(vertices);

// Add triangles
std::vector<Lib3MF::sTriangle> triangles = {
    {0, 1, 2}
};
mesh->SetTriangles(triangles);
```

**lib3mf_rust:**
```rust
// Read-only: Access existing mesh data
if let Some(ref mesh) = object.mesh {
    // Access vertices (immutable)
    for vertex in &mesh.vertices {
        println!("x: {}, y: {}, z: {}", vertex.x, vertex.y, vertex.z);
    }
    
    // Access triangles (immutable)
    for triangle in &mesh.triangles {
        println!("Indices: {}, {}, {}", triangle.v1, triangle.v2, triangle.v3);
    }
}
```

### Metadata

**C++ lib3mf:**
```cpp
// Get metadata
Lib3MF::PMetaDataGroup metaData = model->GetMetaDataGroup();
Lib3MF_uint32 count = metaData->GetMetaDataCount();

for (Lib3MF_uint32 i = 0; i < count; i++) {
    Lib3MF::PMetaData item = metaData->GetMetaData(i);
    std::string name = item->GetName();
    std::string value = item->GetValue();
}
```

**lib3mf_rust:**
```rust
// Direct access via HashMap
for (name, value) in &model.metadata {
    println!("{}: {}", name, value);
}
```

## Code Examples

### Example 1: Reading a Simple 3MF File

**C++ lib3mf:**
```cpp
#include <lib3mf_implicit.hpp>
#include <iostream>

int main() {
    try {
        auto wrapper = Lib3MF::CWrapper::loadLibrary();
        auto model = wrapper->CreateModel();
        auto reader = model->QueryReader("3mf");
        
        reader->ReadFromFile("cube.3mf");
        
        auto buildItems = model->GetBuildItems();
        std::cout << "Build items: " << buildItems->Count() << std::endl;
        
        return 0;
    } catch (Lib3MF::ELib3MFException &e) {
        std::cerr << "Error: " << e.what() << std::endl;
        return 1;
    }
}
```

**lib3mf_rust:**
```rust
use lib3mf::Model;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("cube.3mf")?;
    let model = Model::from_reader(file)?;
    
    println!("Build items: {}", model.build.items.len());
    
    Ok(())
}
```

### Example 2: Extracting All Mesh Geometry

**C++ lib3mf:**
```cpp
auto buildItems = model->GetBuildItems();

while (buildItems->MoveNext()) {
    auto item = buildItems->GetCurrent();
    auto object = item->GetObjectResource();
    
    if (object->IsMeshObject()) {
        auto meshObj = model->GetMeshObjectByID(object->GetResourceID());
        
        // Get vertices
        auto vertexCount = meshObj->GetVertexCount();
        std::vector<Lib3MF::sPosition> vertices(vertexCount);
        meshObj->GetVertices(vertices.data());
        
        // Get triangles
        auto triangleCount = meshObj->GetTriangleCount();
        std::vector<Lib3MF::sTriangle> triangles(triangleCount);
        meshObj->GetTriangleIndices(triangles.data());
        
        // Process mesh data
        for (const auto& v : vertices) {
            std::cout << "V: " << v.m_Coordinates[0] << ", "
                      << v.m_Coordinates[1] << ", "
                      << v.m_Coordinates[2] << std::endl;
        }
    }
}
```

**lib3mf_rust:**
```rust
for item in &model.build.items {
    if let Some(object) = model.resources.objects.iter()
        .find(|obj| obj.id == item.objectid) {
        
        if let Some(ref mesh) = object.mesh {
            // Process vertices
            for vertex in &mesh.vertices {
                println!("V: {}, {}, {}", vertex.x, vertex.y, vertex.z);
            }
            
            // Process triangles
            for triangle in &mesh.triangles {
                println!("T: {}, {}, {}", 
                    triangle.v1, triangle.v2, triangle.v3);
            }
        }
    }
}
```

### Example 3: Materials and Colors

**C++ lib3mf:**
```cpp
auto baseMaterialGroup = model->AddBaseMaterialGroup();
baseMaterialGroup->AddMaterial("Red", Lib3MF::sColor{255, 0, 0, 255});

auto colorGroup = model->AddColorGroup();
colorGroup->AddColor(Lib3MF::sColor{0, 255, 0, 255});

// Assign to triangle
auto meshObj = model->AddMeshObject();
// ... add vertices ...
auto triangle = meshObj->AddTriangle({0, 1, 2});
meshObj->SetTriangleProperties(0, baseMaterialGroup->GetResourceID(), 0);
```

**lib3mf_rust:**
```rust
// Read materials
for material in &model.resources.materials {
    println!("Material ID {}: {}", material.id, material.name);
    if let Some(color) = material.color {
        println!("  Color: #{:02x}{:02x}{:02x}{:02x}", 
            color.red, color.green, color.blue, color.alpha);
    }
}

// Check triangle materials
if let Some(ref mesh) = object.mesh {
    for triangle in &mesh.triangles {
        if let Some(pid) = triangle.pid {
            println!("Triangle uses material property: {}", pid);
        }
    }
}
```

## Feature Parity

### Currently Supported Features

| Feature | C++ lib3mf | lib3mf_rust | Notes |
|---------|------------|-------------|-------|
| **Read 3MF files** | ✅ | ✅ | Full support |
| **Write 3MF files** | ✅ | ❌ | Not yet implemented |
| **Mesh geometry** | ✅ | ✅ | Vertices, triangles |
| **Materials** | ✅ | ✅ | Basic materials, color groups |
| **Metadata** | ✅ | ✅ | Key-value pairs |
| **Build items** | ✅ | ✅ | Object placement |
| **Transformations** | ✅ | ✅ | 4x3 matrix support |
| **Components** | ✅ | ⚠️ | Partial (read but not fully structured) |
| **Textures** | ✅ | ❌ | Not yet implemented |
| **Production extension** | ✅ | ⚠️ | Files parse, data not extracted |
| **Slice extension** | ✅ | ⚠️ | Files parse, data not extracted |
| **Beam lattice** | ✅ | ⚠️ | Files parse, data not extracted |

### Extension Support Comparison

Both libraries support extension validation, but with different approaches:

**C++ lib3mf:**
- Extensions are validated during parsing
- You register which extensions you support
- Can create files with specific extensions

**lib3mf_rust:**
- Configurable extension support via `ParserConfig`
- Unknown extensions are handled gracefully (warning vs. error)
- Extension-specific data structures are being developed

## Common Migration Patterns

### Pattern 1: File Reading Workflow

**Migration steps:**
1. Replace `CWrapper::loadLibrary()` → remove (no wrapper needed)
2. Replace `model->QueryReader("3mf")` → `Model::from_reader(file)`
3. Replace iterator patterns → use standard Rust iterators
4. Replace getter methods → direct field access

### Pattern 2: Error Handling

**C++ (exceptions):**
```cpp
try {
    auto model = wrapper->CreateModel();
    auto reader = model->QueryReader("3mf");
    reader->ReadFromFile("file.3mf");
} catch (Lib3MF::ELib3MFException &e) {
    std::cerr << "Error: " << e.what() << std::endl;
}
```

**Rust (Result type):**
```rust
match Model::from_reader(file) {
    Ok(model) => {
        // Use model
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}

// Or use ? operator
let model = Model::from_reader(file)?;
```

### Pattern 3: Resource Iteration

**C++ (iterator pattern):**
```cpp
auto buildItems = model->GetBuildItems();
while (buildItems->MoveNext()) {
    auto item = buildItems->GetCurrent();
    // Process item
}
```

**Rust (for loop):**
```rust
for item in &model.build.items {
    // Process item
}
```

### Pattern 4: Memory Management

**C++ (smart pointers):**
```cpp
Lib3MF::PModel model = wrapper->CreateModel();
// Model is freed when PModel goes out of scope
```

**Rust (ownership):**
```rust
let model = Model::from_reader(file)?;
// Model is automatically dropped when it goes out of scope
// No manual cleanup needed
```

## Error Handling

### Error Types

**C++ lib3mf:**
- `ELib3MFException` - Base exception class
- Various derived exception types
- Return codes for C bindings

**lib3mf_rust:**
- `lib3mf::Error` - Enum with specific error variants
- `lib3mf::Result<T>` - Type alias for `Result<T, Error>`
- Detailed error messages with context

### Error Variants

```rust
use lib3mf::Error;

match model_result {
    Err(Error::Io(e)) => {
        // I/O error (file not found, permissions, etc.)
    }
    Err(Error::Zip(e)) => {
        // Invalid 3MF container (corrupt ZIP)
    }
    Err(Error::Parse(msg)) => {
        // XML parsing error
    }
    Err(Error::InvalidModel(msg)) => {
        // Invalid 3MF structure
    }
    Err(Error::UnsupportedExtension(ext)) => {
        // File requires unsupported extension
    }
    Ok(model) => {
        // Success
    }
}
```

## Extension Support

### Configuring Extension Support

**C++ lib3mf:**
```cpp
auto model = wrapper->CreateModel();
// Extensions are handled implicitly
// You can check required extensions after reading
```

**lib3mf_rust:**
```rust
use lib3mf::{Model, ParserConfig, Extension};

// Default: Accept all known extensions
let model = Model::from_reader(file)?;

// Strict: Only accept core specification
let config = ParserConfig::new();
let model = Model::from_reader_with_config(file, config)?;

// Custom: Accept specific extensions
let config = ParserConfig::new()
    .with_extension(Extension::Material)
    .with_extension(Extension::Production);
let model = Model::from_reader_with_config(file, config)?;

// Permissive: Accept all extensions
let config = ParserConfig::with_all_extensions();
let model = Model::from_reader_with_config(file, config)?;
```

### Checking Required Extensions

**C++ lib3mf:**
```cpp
auto requiredExts = model->GetRequiredExtensions();
// Returns list of extension URIs
```

**lib3mf_rust:**
```rust
for ext in &model.required_extensions {
    println!("File requires: {}", ext.name());
    println!("Namespace: {}", ext.namespace());
}
```

## Performance Considerations

### Memory Usage

**C++ lib3mf:**
- Manual memory management
- Potential for memory leaks if not careful
- Smart pointers help but add overhead

**lib3mf_rust:**
- Zero-cost abstractions
- No garbage collection overhead
- Compiler-enforced memory safety
- Predictable performance

### Parsing Speed

Both libraries have comparable parsing speeds for typical 3MF files:

- **Small files (< 1MB)**: Negligible difference
- **Large files (> 10MB)**: lib3mf_rust may be slightly faster due to optimized Rust XML parsing
- **Memory-mapped files**: C++ lib3mf may have an advantage with custom readers

### Threading

**C++ lib3mf:**
- Thread safety depends on usage
- Models are generally not thread-safe
- Requires manual synchronization

**lib3mf_rust:**
- Models are `Send` (can be moved between threads)
- Immutable access is `Sync` (can be shared read-only)
- Compiler enforces thread safety

## Migration Checklist

When migrating from C++ lib3mf to lib3mf_rust:

- [ ] **Install Rust toolchain** (`rustup`)
- [ ] **Add dependency** to `Cargo.toml`
- [ ] **Replace initialization code** (no wrapper needed)
- [ ] **Update file reading** (use `Model::from_reader`)
- [ ] **Convert iterator patterns** to Rust for loops
- [ ] **Replace getter/setter calls** with field access
- [ ] **Update error handling** (exceptions → `Result`)
- [ ] **Remove manual cleanup** (trust Rust ownership)
- [ ] **Verify extension support** (configure `ParserConfig` if needed)
- [ ] **Test with your 3MF files**
- [ ] **Note: Writing not yet supported** - keep C++ version if you need write operations

## Limitations and Future Work

### Current Limitations

1. **Read-only**: Cannot create or modify 3MF files (write support planned)
2. **Extension data**: Some extension-specific data not yet extracted:
   - Production: UUIDs, paths
   - Slice: Slice stacks, layer data
   - Beam lattice: Beam definitions
3. **Components**: Component hierarchies are partially supported
4. **Textures**: Not yet implemented
5. **Custom extensions**: Not yet supported

### Future Enhancements

The roadmap includes:

- ✅ Full read support (DONE)
- 🚧 Extension data extraction (IN PROGRESS)
- 📋 Write support (PLANNED)
- 📋 Component hierarchies (PLANNED)
- 📋 Texture support (PLANNED)
- 📋 Custom extension API (PLANNED)

## Getting Help

- **Examples**: See `examples/` directory in the repository
- **Issues**: [GitHub Issues](https://github.com/telecos/lib3mf_rust/issues)
- **3MF Spec**: [3mf.io/specification](https://3mf.io/specification/)
- **C++ Reference**: [https://lib3mf.readthedocs.io/](https://lib3mf.readthedocs.io/)

## Conclusion

Migrating from C++ lib3mf to lib3mf_rust offers:

✅ **Improved safety** - No unsafe code, memory safety guaranteed  
✅ **Simpler API** - Direct field access, standard Rust patterns  
✅ **Better errors** - Descriptive error messages with context  
✅ **No dependencies** - Pure Rust, no C++ runtime needed  
✅ **Modern tooling** - Cargo, clippy, rustfmt built-in  

⚠️ **Current trade-offs**:
- Read-only (write support coming)
- Some extension data not yet extracted

For **reading 3MF files**, lib3mf_rust is a complete solution. For **writing**, continue using C++ lib3mf until write support is implemented.

---

**Questions or feedback?** Please open an issue on GitHub!
