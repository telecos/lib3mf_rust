# Migration Guide: From C++ lib3mf to lib3mf_rust

This comprehensive guide helps you migrate from the official C++ lib3mf library to this pure Rust implementation. Whether you're porting existing code or starting a new project, this guide provides detailed API mappings, code examples, and best practices.

## Table of Contents

- [Overview](#overview)
- [Installation and Setup](#installation-and-setup)
- [Key Differences](#key-differences)
- [API Mapping](#api-mapping)
- [Side-by-Side Examples](#side-by-side-examples)
- [Feature Parity Matrix](#feature-parity-matrix)
- [Common Migration Patterns](#common-migration-patterns)
- [Error Handling](#error-handling)
- [Extension Support](#extension-support)
- [Performance Considerations](#performance-considerations)
- [Memory Usage and Threading](#memory-usage-and-threading)
- [Migration Checklist](#migration-checklist)
- [FAQ and Troubleshooting](#faq-and-troubleshooting)

## Overview

**lib3mf_rust** is a **pure Rust reimplementation** of the 3MF file format parser and writer, designed with safety, simplicity, and modern development practices in mind.

### Why Migrate?

✅ **Memory Safety** - No unsafe code, guaranteed by Rust's compiler  
✅ **No C/C++ Dependencies** - Pure Rust, simpler deployment  
✅ **Simpler API** - Direct field access, no getter/setter boilerplate  
✅ **Modern Error Handling** - `Result<T, E>` instead of exceptions or return codes  
✅ **Better Tooling** - Cargo, clippy, rustfmt built-in  
✅ **Thread Safety** - Compiler-enforced safe concurrency  
✅ **Read & Write Support** - Full round-trip capability  

### When to Migrate

**Good fit for:**
- New projects starting from scratch
- Applications primarily reading 3MF files
- Projects requiring memory safety guarantees
- Rust-based tool chains
- Applications that need thread-safe 3MF processing

**Consider C++ lib3mf if:**
- You need C language bindings
- You require features not yet implemented in Rust version
- You have extensive existing C++ codebase

## Installation and Setup

### C++ lib3mf

**C++ lib3mf:**
```bash
# Download prebuilt binaries or build from source
git clone https://github.com/3MFConsortium/lib3mf.git
cd lib3mf
mkdir build && cd build
cmake ..
make
make install
```

**In your C++ project:**
```cpp
#include <lib3mf_implicit.hpp>

// Link against lib3mf library
// -l3mf or similar depending on your build system
```

### lib3mf_rust

**Installation:**
```bash
# Just add to Cargo.toml
```

**Cargo.toml:**
```toml
[dependencies]
lib3mf = "0.1"
```

**In your Rust code:**
```rust
use lib3mf::Model;
use std::fs::File;
```

**That's it!** No external dependencies, no build scripts, no linking issues.

## Key Differences

### Philosophy and Design

| Aspect | C++ lib3mf | lib3mf_rust |
|--------|------------|-------------|
| **Language** | C++ with C bindings | Pure Rust |
| **Safety** | Manual memory management | Automatic (ownership system) |
| **API Style** | Object-oriented, stateful | Functional, data-oriented |
| **Error Handling** | Exceptions or return codes | `Result<T, E>` types |
| **Operations** | Read and write | Read and write |
| **Dependencies** | C++ libraries, system libs | Pure Rust crates only |
| **Memory Model** | Smart pointers, manual | Ownership & borrowing |
| **Concurrency** | Manual synchronization | Compiler-enforced safety |
| **Binary Size** | Larger (C++ runtime) | Smaller (static linking) |

### Architecture Comparison

**C++ lib3mf:**
- Complex class hierarchies
- Wrapper classes and interfaces
- Stateful objects with setters/getters
- Iterator-based traversal
- Explicit resource management

**lib3mf_rust:**
- Simple data structures (structs and enums)
- Direct field access
- Immutable by default
- Standard Rust iteration
- Automatic resource cleanup

## API Mapping

### Core Operations

| Operation | C++ lib3mf | lib3mf_rust | Notes |
|-----------|------------|-------------|-------|
| **Initialize** | `CWrapper::loadLibrary()` | Not needed | No wrapper required |
| **Create Model** | `wrapper->CreateModel()` | `Model::new()` | Direct constructor |
| **Read File** | `reader->ReadFromFile(path)` | `Model::from_reader(file)` | Uses Rust `Read` trait |
| **Write File** | `writer->WriteToFile(path)` | `model.write_to_file(path)` | Single method call |
| **Get Objects** | `model->GetObjects()` | `model.resources.objects` | Direct Vec access |
| **Get Build Items** | `model->GetBuildItems()` | `model.build.items` | Direct Vec access |
| **Get Metadata** | `model->GetMetaDataGroup()` | `model.metadata` | Vec of entries |

### Model Access

| Feature | C++ lib3mf | lib3mf_rust |
|---------|------------|-------------|
| **Unit** | `model->GetUnit()` / `SetUnit()` | `model.unit` (String) |
| **Language** | `model->GetLanguage()` | `model.lang` (Option<String>) |
| **Namespace** | Implicit | `model.xmlns` (String) |
| **Metadata** | Iterator pattern | `model.metadata` (Vec) |
| **Resources** | Various methods | `model.resources` (struct) |
| **Build** | `GetBuildItems()` | `model.build` (struct) |

### Mesh Operations

| Operation | C++ lib3mf | lib3mf_rust |
|-----------|------------|-------------|
| **Get Vertices** | `meshObj->GetVertices(buffer)` | `mesh.vertices` (Vec) |
| **Add Vertex** | `meshObj->AddVertex(pos)` | `mesh.vertices.push(vertex)` |
| **Vertex Count** | `meshObj->GetVertexCount()` | `mesh.vertices.len()` |
| **Get Triangles** | `meshObj->GetTriangleIndices(buffer)` | `mesh.triangles` (Vec) |
| **Add Triangle** | `meshObj->AddTriangle(indices)` | `mesh.triangles.push(tri)` |
| **Triangle Count** | `meshObj->GetTriangleCount()` | `mesh.triangles.len()` |

### Material Operations

| Operation | C++ lib3mf | lib3mf_rust |
|-----------|------------|-------------|
| **Add Base Material** | `group->AddMaterial(name, color)` | `group.materials.push(material)` |
| **Get Material** | `group->GetMaterial(index)` | `group.materials[index]` |
| **Set Triangle Material** | `mesh->SetTriangleProperties(idx, pid, pindex)` | `triangle.pid = Some(pid); triangle.pindex = Some(pindex)` |

## Side-by-Side Examples

### Example 1: Reading a 3MF File

**C++ lib3mf:**
```cpp
#include <lib3mf_implicit.hpp>
#include <iostream>

int main() {
    try {
        // Initialize wrapper
        auto wrapper = Lib3MF::CWrapper::loadLibrary();
        
        // Create model
        auto model = wrapper->CreateModel();
        
        // Create reader and read file
        auto reader = model->QueryReader("3mf");
        reader->ReadFromFile("model.3mf");
        
        // Get unit
        std::string unit = model->GetUnit();
        std::cout << "Unit: " << unit << std::endl;
        
        // Count objects
        auto objects = model->GetObjects();
        std::cout << "Objects: " << objects->Count() << std::endl;
        
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
    // Open and parse in one step
    let file = File::open("model.3mf")?;
    let model = Model::from_reader(file)?;
    
    // Direct field access
    println!("Unit: {}", model.unit);
    println!("Objects: {}", model.resources.objects.len());
    
    Ok(())
}
```

**Key Differences:**
- No wrapper or initialization needed
- Single method call to read
- Direct field access instead of getters
- `?` operator for error handling
- Simpler, more concise code

### Example 2: Creating and Writing a 3MF File

**C++ lib3mf:**
```cpp
#include <lib3mf_implicit.hpp>

int main() {
    try {
        auto wrapper = Lib3MF::CWrapper::loadLibrary();
        auto model = wrapper->CreateModel();
        
        // Set model properties
        model->SetUnit("millimeter");
        
        // Create mesh object
        auto meshObj = model->AddMeshObject();
        meshObj->SetName("Triangle");
        
        // Add vertices
        Lib3MF::sPosition v1 = {0.0f, 0.0f, 0.0f};
        Lib3MF::sPosition v2 = {10.0f, 0.0f, 0.0f};
        Lib3MF::sPosition v3 = {5.0f, 10.0f, 0.0f};
        
        meshObj->AddVertex(v1);
        meshObj->AddVertex(v2);
        meshObj->AddVertex(v3);
        
        // Add triangle
        Lib3MF::sTriangle tri = {0, 1, 2};
        meshObj->AddTriangle(tri);
        
        // Add to build
        model->AddBuildItem(meshObj.get(), wrapper->GetIdentityTransform());
        
        // Write to file
        auto writer = model->QueryWriter("3mf");
        writer->WriteToFile("output.3mf");
        
        return 0;
    } catch (Lib3MF::ELib3MFException &e) {
        std::cerr << "Error: " << e.what() << std::endl;
        return 1;
    }
}
```

**lib3mf_rust:**
```rust
use lib3mf::{Model, Object, Mesh, Vertex, Triangle, BuildItem};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create new model
    let mut model = Model::new();
    model.unit = "millimeter".to_string();
    
    // Create mesh
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));
    
    // Create object
    let mut object = Object::new(1);
    object.name = Some("Triangle".to_string());
    object.mesh = Some(mesh);
    
    // Add to resources and build
    model.resources.objects.push(object);
    model.build.items.push(BuildItem::new(1));
    
    // Write to file
    model.write_to_file("output.3mf")?;
    
    Ok(())
}
```

**Key Differences:**
- Direct struct manipulation
- `push()` instead of `Add*()` methods
- No identity transform needed (identity is default)
- Single method to write file
- Type-safe with Option<T> for optional fields

### Example 3: Iterating Through Mesh Data

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
        
        // Process vertices
        for (const auto& v : vertices) {
            std::cout << "Vertex: (" 
                      << v.m_Coordinates[0] << ", "
                      << v.m_Coordinates[1] << ", "
                      << v.m_Coordinates[2] << ")" << std::endl;
        }
        
        // Get triangles
        auto triangleCount = meshObj->GetTriangleCount();
        std::vector<Lib3MF::sTriangle> triangles(triangleCount);
        meshObj->GetTriangleIndices(triangles.data());
        
        // Process triangles
        for (const auto& t : triangles) {
            std::cout << "Triangle: ("
                      << t.m_Indices[0] << ", "
                      << t.m_Indices[1] << ", "
                      << t.m_Indices[2] << ")" << std::endl;
        }
    }
}
```

**lib3mf_rust:**
```rust
for item in &model.build.items {
    // Find corresponding object
    if let Some(object) = model.resources.objects.iter()
        .find(|obj| obj.id == item.objectid) {
        
        if let Some(ref mesh) = object.mesh {
            // Process vertices - direct iteration
            for vertex in &mesh.vertices {
                println!("Vertex: ({}, {}, {})", 
                    vertex.x, vertex.y, vertex.z);
            }
            
            // Process triangles - direct iteration
            for triangle in &mesh.triangles {
                println!("Triangle: ({}, {}, {})",
                    triangle.v1, triangle.v2, triangle.v3);
            }
        }
    }
}
```

**Key Differences:**
- Standard Rust iteration (for loops)
- No manual buffer allocation
- Direct field access (`.x`, `.y`, `.z`)
- Pattern matching for optional data

### Example 4: Working with Materials

**C++ lib3mf:**
```cpp
// Create base material group
auto matGroup = model->AddBaseMaterialGroup();

// Add materials with colors
Lib3MF::sColor red = {255, 0, 0, 255};
Lib3MF::sColor blue = {0, 0, 255, 255};

matGroup->AddMaterial("Red Material", red);
matGroup->AddMaterial("Blue Material", blue);

// Create mesh and assign materials to triangles
auto meshObj = model->AddMeshObject();
// ... add vertices ...

Lib3MF::sTriangle tri1 = {0, 1, 2};
meshObj->AddTriangle(tri1);
meshObj->SetTriangleProperties(0, matGroup->GetResourceID(), 0); // Red

Lib3MF::sTriangle tri2 = {2, 3, 0};
meshObj->AddTriangle(tri2);
meshObj->SetTriangleProperties(1, matGroup->GetResourceID(), 1); // Blue
```

**lib3mf_rust:**
```rust
use lib3mf::{BaseMaterialGroup, BaseMaterial};

// Create material group
let mut mat_group = BaseMaterialGroup::new(1);
mat_group.materials.push(
    BaseMaterial::new("Red Material".to_string(), (255, 0, 0, 255))
);
mat_group.materials.push(
    BaseMaterial::new("Blue Material".to_string(), (0, 0, 255, 255))
);
model.resources.base_material_groups.push(mat_group);

// Create mesh with colored triangles
let mut mesh = Mesh::new();
// ... add vertices ...

let mut tri1 = Triangle::new(0, 1, 2);
tri1.pid = Some(1);      // Material group ID
tri1.pindex = Some(0);   // Red material
mesh.triangles.push(tri1);

let mut tri2 = Triangle::new(2, 3, 0);
tri2.pid = Some(1);
tri2.pindex = Some(1);   // Blue material
mesh.triangles.push(tri2);
```

**Key Differences:**
- Materials are structs with direct field assignment
- Colors as tuples `(r, g, b, a)`
- Triangle properties set directly on triangle struct
- Option<T> for optional material references

### Example 5: Handling Metadata

**C++ lib3mf:**
```cpp
// Add metadata
auto metaDataGroup = model->GetMetaDataGroup();
metaDataGroup->AddMetaData("Title", "My Model");
metaDataGroup->AddMetaData("Designer", "John Doe");

// Read metadata
auto count = metaDataGroup->GetMetaDataCount();
for (uint32_t i = 0; i < count; i++) {
    auto metaData = metaDataGroup->GetMetaData(i);
    std::string name = metaData->GetName();
    std::string value = metaData->GetValue();
    std::cout << name << ": " << value << std::endl;
}
```

**lib3mf_rust:**
```rust
use lib3mf::MetadataEntry;

// Add metadata
model.metadata.push(MetadataEntry::new(
    "Title".to_string(),
    "My Model".to_string()
));
model.metadata.push(MetadataEntry::new(
    "Designer".to_string(),
    "John Doe".to_string()
));

// Read metadata - simple iteration
for entry in &model.metadata {
    println!("{}: {}", entry.name, entry.value);
}
```

**Key Differences:**
- Vec instead of specialized container
- Simple push() to add entries
- Direct for loop iteration
- No index-based access needed

### Example 6: Round-Trip (Read, Modify, Write)

**C++ lib3mf:**
```cpp
try {
    auto wrapper = Lib3MF::CWrapper::loadLibrary();
    auto model = wrapper->CreateModel();
    
    // Read
    auto reader = model->QueryReader("3mf");
    reader->ReadFromFile("input.3mf");
    
    // Modify - add metadata
    auto metaData = model->GetMetaDataGroup();
    metaData->AddMetaData("Modified", "true");
    
    // Write
    auto writer = model->QueryWriter("3mf");
    writer->WriteToFile("output.3mf");
    
} catch (Lib3MF::ELib3MFException &e) {
    std::cerr << "Error: " << e.what() << std::endl;
}
```

**lib3mf_rust:**
```rust
// Read
let file = File::open("input.3mf")?;
let mut model = Model::from_reader(file)?;

// Modify - add metadata
model.metadata.push(MetadataEntry::new(
    "Modified".to_string(),
    "true".to_string()
));

// Write
model.write_to_file("output.3mf")?;
```

**Key Differences:**
- Mutable model for modifications
- Simple field manipulation
- Cleaner, more readable code

## Feature Parity Matrix

### Core Features

| Feature | C++ lib3mf | lib3mf_rust | Status |
|---------|:----------:|:-----------:|:------:|
| **Read 3MF files** | ✅ | ✅ | **Complete** |
| **Write 3MF files** | ✅ | ✅ | **Complete** |
| **Mesh geometry** | ✅ | ✅ | **Complete** |
| **Vertices** | ✅ | ✅ | **Complete** |
| **Triangles** | ✅ | ✅ | **Complete** |
| **Build items** | ✅ | ✅ | **Complete** |
| **Transformations** | ✅ | ✅ | **Complete** |
| **Metadata** | ✅ | ✅ | **Complete** |
| **Units** | ✅ | ✅ | **Complete** |
| **Language tags** | ✅ | ✅ | **Complete** |

### Materials & Properties

| Feature | C++ lib3mf | lib3mf_rust | Status |
|---------|:----------:|:-----------:|:------:|
| **Base materials** | ✅ | ✅ | **Complete** |
| **Color groups** | ✅ | ✅ | **Complete** |
| **Texture2D** | ✅ | ✅ | **Complete** |
| **Texture2D groups** | ✅ | ✅ | **Complete** |
| **Composite materials** | ✅ | ✅ | **Complete** |
| **Multi-properties** | ✅ | ✅ | **Complete** |
| **Per-triangle materials** | ✅ | ✅ | **Complete** |
| **Per-vertex colors** | ✅ | ⚠️ | *Partial* |

### Extensions

| Extension | C++ lib3mf | lib3mf_rust | Status |
|-----------|:----------:|:-----------:|:------:|
| **Core specification** | ✅ | ✅ | **Complete** |
| **Materials extension** | ✅ | ✅ | **Complete** |
| **Production extension** | ✅ | ✅ | **Complete** |
| **Beam lattice** | ✅ | ✅ | **Complete** |
| **Slice extension** | ✅ | ⚠️ | *Recognized* |
| **Secure content** | ✅ | ⚠️ | *Validation only* |
| **Boolean operations** | ✅ | ⚠️ | *Recognized* |
| **Displacement** | ✅ | ✅ | **Complete** |
| **Custom extensions** | ✅ | ✅ | **Complete** |

### Advanced Features

| Feature | C++ lib3mf | lib3mf_rust | Status |
|---------|:----------:|:-----------:|:------:|
| **Components** | ✅ | ⚠️ | *Partial* |
| **Component hierarchies** | ✅ | ❌ | *Planned* |
| **Thumbnails** | ✅ | ✅ | **Complete** |
| **Streaming parser** | ❌ | ✅ | **Rust-only** |
| **Custom extension handlers** | ⚠️ | ✅ | **Rust-only** |
| **Validation** | ✅ | ✅ | **Complete** |

**Legend:**
- ✅ **Complete** - Fully supported with feature parity
- ⚠️ **Partial** - Recognized but not fully implemented
- ❌ **Not available** - Not yet implemented
- **Rust-only** - Feature unique to Rust implementation

## Common Migration Patterns

### Pattern 1: Initialization

**C++ (multi-step):**
```cpp
auto wrapper = Lib3MF::CWrapper::loadLibrary();
auto model = wrapper->CreateModel();
```

**Rust (direct):**
```rust
let mut model = Model::new();
```

**Migration:** Remove wrapper, call constructor directly.

---

### Pattern 2: Error Handling

**C++ (exceptions):**
```cpp
try {
    auto model = wrapper->CreateModel();
    auto reader = model->QueryReader("3mf");
    reader->ReadFromFile("file.3mf");
    // ... use model ...
} catch (Lib3MF::ELib3MFException &e) {
    std::cerr << "Error: " << e.what() << std::endl;
    return 1;
}
```

**Rust (Result type):**
```rust
// Option 1: Pattern matching
match Model::from_reader(file) {
    Ok(model) => {
        // ... use model ...
    }
    Err(e) => {
        eprintln!("Error: {}", e);
        return Err(e.into());
    }
}

// Option 2: ? operator (recommended)
let model = Model::from_reader(file)?;
// ... use model ...
```

**Migration:** Replace try-catch with Result handling. Use `?` operator for propagation.

---

### Pattern 3: Iteration

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

**Migration:** Replace iterator pattern with standard Rust for loop.

---

### Pattern 4: Getters/Setters

**C++ (methods):**
```cpp
std::string unit = model->GetUnit();
model->SetUnit("millimeter");

std::string name = object->GetName();
object->SetName("MyObject");
```

**Rust (field access):**
```rust
let unit = &model.unit;
model.unit = "millimeter".to_string();

let name = object.name.as_ref();
object.name = Some("MyObject".to_string());
```

**Migration:** Replace getter/setter calls with direct field access. Use Option<T> for optional fields.

---

### Pattern 5: Resource Lookup

**C++ (method calls):**
```cpp
auto object = model->GetObjectByID(objectId);
auto resource = model->GetResourceByID(resourceId);
```

**Rust (iterator methods):**
```rust
let object = model.resources.objects.iter()
    .find(|obj| obj.id == object_id);

// Or with filter
let matching_objects: Vec<_> = model.resources.objects.iter()
    .filter(|obj| obj.id > 5)
    .collect();
```

**Migration:** Use standard Rust iterator methods (find, filter, etc.).

---

### Pattern 6: Memory Management

**C++ (smart pointers):**
```cpp
{
    Lib3MF::PModel model = wrapper->CreateModel();
    // ... use model ...
} // Model freed when PModel goes out of scope
```

**Rust (ownership):**
```rust
{
    let model = Model::new();
    // ... use model ...
} // Model automatically dropped
```

**Migration:** Remove explicit memory management. Trust Rust's ownership system.

---

### Pattern 7: Optional Values

**C++ (pointers or special values):**
```cpp
if (object->GetName() != "") {
    std::string name = object->GetName();
    // Use name
}
```

**Rust (Option type):**
```rust
if let Some(ref name) = object.name {
    // Use name
}

// Or with match
match object.name {
    Some(ref name) => println!("Name: {}", name),
    None => println!("No name"),
}
```

**Migration:** Use Option<T> for optional fields. Use pattern matching or if-let.

## Error Handling

### Error Types Comparison

**C++ lib3mf:**
```cpp
// Exception-based
try {
    // ... operations ...
} catch (Lib3MF::ELib3MFException &e) {
    // Error code available via e.getErrorCode()
    // Message via e.what()
}
```

**lib3mf_rust:**
```rust
// Result-based
use lib3mf::Error;

match model_result {
    Err(Error::Io(e)) => {
        // File I/O error
        eprintln!("I/O error: {}", e);
    }
    Err(Error::Zip(e)) => {
        // ZIP container error
        eprintln!("Invalid 3MF container: {}", e);
    }
    Err(Error::Parse(msg)) => {
        // XML parsing error
        eprintln!("Parse error: {}", msg);
    }
    Err(Error::InvalidModel(msg)) => {
        // Model validation error
        eprintln!("Invalid model: {}", msg);
    }
    Err(Error::UnsupportedExtension(ext)) => {
        // Required extension not supported
        eprintln!("Unsupported extension: {}", ext);
    }
    Ok(model) => {
        // Success
    }
}
```

### Error Handling Patterns

**Pattern 1: Propagate errors (recommended)**
```rust
fn process_3mf(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let model = Model::from_reader(file)?;
    // ... process model ...
    Ok(())
}
```

**Pattern 2: Handle specific errors**
```rust
match Model::from_reader(file) {
    Ok(model) => { /* ... */ }
    Err(Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
        eprintln!("File not found");
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

**Pattern 3: Provide defaults**
```rust
let model = Model::from_reader(file)
    .unwrap_or_else(|e| {
        eprintln!("Warning: {}", e);
        Model::new() // Return default
    });
```

## Extension Support

### Configuring Extensions

**C++ lib3mf:**
```cpp
// Extensions validated implicitly during parsing
auto reader = model->QueryReader("3mf");
reader->ReadFromFile("file.3mf");

// Check required extensions after reading
// (implementation specific)
```

**lib3mf_rust:**
```rust
use lib3mf::{ParserConfig, Extension};

// Option 1: Default (accepts all known extensions)
let model = Model::from_reader(file)?;

// Option 2: Core only (strict)
let config = ParserConfig::new();
let model = Model::from_reader_with_config(file, config)?;

// Option 3: Specific extensions
let config = ParserConfig::new()
    .with_extension(Extension::Material)
    .with_extension(Extension::Production)
    .with_extension(Extension::BeamLattice);
let model = Model::from_reader_with_config(file, config)?;

// Option 4: All extensions (permissive)
let config = ParserConfig::with_all_extensions();
let model = Model::from_reader_with_config(file, config)?;
```

### Custom Extension Handlers

**lib3mf_rust** provides unique support for custom extensions:

```rust
use lib3mf::{ParserConfig, CustomExtensionContext, CustomElementResult};
use std::sync::Arc;

let config = ParserConfig::new()
    .with_custom_extension_handlers(
        "http://example.com/myext/2024/01",
        "MyExtension",
        // Element handler
        Arc::new(|ctx: &CustomExtensionContext| -> Result<CustomElementResult, String> {
            println!("Custom element: {}", ctx.element_name);
            // Process custom elements
            Ok(CustomElementResult::Handled)
        }),
        // Validation handler
        Arc::new(|model| -> Result<(), String> {
            // Custom validation
            Ok(())
        })
    );

let model = Model::from_reader_with_config(file, config)?;
```

## Performance Considerations

### Parsing Speed

Both libraries offer comparable performance:

| File Size | C++ lib3mf | lib3mf_rust | Winner |
|-----------|:----------:|:-----------:|:------:|
| Small (< 1 MB) | ~1-5 ms | ~1-5 ms | Tie |
| Medium (1-10 MB) | ~10-50 ms | ~10-50 ms | Tie |
| Large (10-100 MB) | ~100-500 ms | ~90-450 ms | Slight edge to Rust |
| Very Large (> 100 MB) | ~500+ ms | ~450+ ms | Rust |

**Note:** Performance depends on hardware, file structure, and XML complexity.

### Optimization Tips

**For lib3mf_rust:**

1. **Use streaming parser for large files:**
   ```rust
   use lib3mf::streaming::StreamingParser;
   
   let parser = StreamingParser::new(file)?;
   for object in parser.objects() {
       // Process one object at a time
       // Previous objects are dropped, freeing memory
   }
   ```

2. **Pre-allocate when building models:**
   ```rust
   let mut mesh = Mesh::new();
   mesh.vertices.reserve(10000);  // If you know the size
   mesh.triangles.reserve(20000);
   ```

3. **Avoid unnecessary cloning:**
   ```rust
   // Good: Borrow
   for obj in &model.resources.objects {
       process_object(obj);
   }
   
   // Bad: Clone
   for obj in model.resources.objects.clone() {
       process_object(&obj);
   }
   ```

## Memory Usage and Threading

### Memory Model

**C++ lib3mf:**
- Manual memory management with smart pointers
- Risk of memory leaks if not careful
- Reference counting overhead

**lib3mf_rust:**
- Automatic memory management via ownership
- Zero overhead (no garbage collection)
- Guaranteed memory safety
- Deterministic deallocation

### Thread Safety

**C++ lib3mf:**
```cpp
// Manual synchronization required
std::mutex modelMutex;

void processModel(Lib3MF::PModel model) {
    std::lock_guard<std::mutex> lock(modelMutex);
    // ... access model ...
}
```

**lib3mf_rust:**
```rust
use std::sync::Arc;
use std::thread;

// Models are Send (can move between threads)
let model = Model::from_reader(file)?;
let handle = thread::spawn(move || {
    // Model moved to this thread
    process_model(model);
});

// Models are Sync when immutable (can share read-only)
let model = Arc::new(Model::from_reader(file)?);
let model_clone = Arc::clone(&model);

let handle = thread::spawn(move || {
    // Shared immutable access
    read_model(&model_clone);
});
```

**Key Points:**
- **Send**: Can move between threads (enforced by compiler)
- **Sync**: Can share immutable references across threads
- No data races possible (guaranteed by compiler)

### Concurrent Processing Example

**lib3mf_rust:**
```rust
use rayon::prelude::*;

// Process multiple 3MF files in parallel
let files = vec!["model1.3mf", "model2.3mf", "model3.3mf"];

let results: Vec<_> = files.par_iter()
    .map(|path| {
        let file = File::open(path)?;
        Model::from_reader(file)
    })
    .collect();

// Or process objects in parallel
model.resources.objects.par_iter()
    .for_each(|obj| {
        // Process each object in parallel
        process_object(obj);
    });
```

## Migration Checklist

Use this checklist to track your migration progress:

### Preparation
- [ ] Install Rust toolchain (`rustup`)
- [ ] Add `lib3mf = "0.1"` to `Cargo.toml`
- [ ] Review your C++ code to understand dependencies
- [ ] Identify which lib3mf features you use

### Code Migration
- [ ] Replace `#include <lib3mf_implicit.hpp>` with `use lib3mf::Model;`
- [ ] Remove wrapper initialization (`CWrapper::loadLibrary()`)
- [ ] Replace `QueryReader()` / `ReadFromFile()` with `Model::from_reader()`
- [ ] Replace `QueryWriter()` / `WriteToFile()` with `model.write_to_file()`
- [ ] Convert iterator patterns to Rust for loops
- [ ] Replace getter/setter methods with direct field access
- [ ] Update exception handling to use `Result<T, E>`
- [ ] Replace smart pointers with Rust ownership
- [ ] Update optional value handling to use `Option<T>`

### Materials & Extensions
- [ ] Migrate material creation code
- [ ] Update extension configuration if needed
- [ ] Test custom extensions if applicable
- [ ] Verify production data extraction if used

### Testing & Validation
- [ ] Test with your sample 3MF files
- [ ] Verify round-trip (read-write-read) works
- [ ] Benchmark performance if critical
- [ ] Test edge cases and error handling
- [ ] Validate metadata preservation
- [ ] Check material and color fidelity

### Cleanup
- [ ] Remove C++ lib3mf dependencies
- [ ] Update build scripts (remove CMake, etc.)
- [ ] Update documentation
- [ ] Run `cargo clippy` for linting
- [ ] Run `cargo fmt` for formatting

### Known Limitations
- [ ] Note: Component hierarchies partially supported
- [ ] Note: Some extensions recognized but not fully extracted
- [ ] Plan workarounds if needed for unsupported features

## FAQ and Troubleshooting

### Q: Do I need to install any C/C++ libraries?

**A:** No! lib3mf_rust is pure Rust with no external dependencies. Just add it to your `Cargo.toml`.

---

### Q: My C++ code uses components extensively. Is this supported?

**A:** Components are partially supported. Basic components work, but complex hierarchies may not be fully supported yet. Check the [IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md) for details.

---

### Q: How do I handle files that require specific extensions?

**A:** Use `ParserConfig` to specify supported extensions:

```rust
let config = ParserConfig::new()
    .with_extension(Extension::Material)
    .with_extension(Extension::Production);
let model = Model::from_reader_with_config(file, config)?;
```

---

### Q: Can I use lib3mf_rust from C or Python?

**A:** Not directly yet. You could create FFI bindings using `cbindgen` for C or `PyO3` for Python. These are not currently provided but could be added.

---

### Q: Performance seems slower than C++. What can I do?

**A:** Try these optimizations:
1. Use release builds (`cargo build --release`)
2. Use streaming parser for very large files
3. Pre-allocate vectors when building models
4. Avoid unnecessary cloning

---

### Q: I'm getting "UnsupportedExtension" errors

**A:** The file requires extensions not in your `ParserConfig`. Either add the extension to your config or use `ParserConfig::with_all_extensions()`.

---

### Q: How do I debug parsing errors?

**A:** Errors include detailed messages:

```rust
match Model::from_reader(file) {
    Err(e) => {
        eprintln!("Detailed error: {:?}", e);  // Debug format shows full details
    }
    Ok(model) => { /* ... */ }
}
```

---

### Q: Can I contribute to lib3mf_rust?

**A:** Yes! Contributions are welcome. Check the repository's issue tracker for areas needing help.

---

## Getting Help

- **Documentation**: This guide and the [README.md](README.md)
- **Examples**: See `examples/` directory for working code
- **API Reference**: Run `cargo doc --open`
- **Issues**: [GitHub Issues](https://github.com/telecos/lib3mf_rust/issues)
- **3MF Specification**: [https://3mf.io/specification/](https://3mf.io/specification/)
- **C++ Reference**: [lib3mf documentation](https://lib3mf.readthedocs.io/)

## Additional Resources

- [IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md) - Current implementation status
- [EXTENSION_SUPPORT_SUMMARY.md](EXTENSION_SUPPORT_SUMMARY.md) - Extension support details
- [ERROR_HANDLING.md](ERROR_HANDLING.md) - Error handling guide
- [PERFORMANCE.md](PERFORMANCE.md) - Performance benchmarks and tuning
- [CONFORMANCE_REPORT.md](CONFORMANCE_REPORT.md) - 3MF conformance test results

## Conclusion

Migrating from C++ lib3mf to lib3mf_rust offers significant benefits in safety, simplicity, and maintainability. The API is cleaner, errors are more explicit, and memory safety is guaranteed by the compiler.

### Summary of Benefits

✅ **Safety First** - No unsafe code, memory safety guaranteed  
✅ **Simpler Code** - Less boilerplate, direct field access  
✅ **Better Errors** - Descriptive Result types instead of exceptions  
✅ **No Dependencies** - Pure Rust, easy deployment  
✅ **Modern Tooling** - Cargo, clippy, rustfmt built-in  
✅ **Thread Safe** - Compiler-enforced safety  
✅ **Full Round-Trip** - Read and write support  

### When to Use Each Library

**Use lib3mf_rust for:**
- ✅ New projects
- ✅ Rust applications
- ✅ Safety-critical code
- ✅ Modern development practices
- ✅ Thread-safe processing

**Continue using C++ lib3mf for:**
- ⚠️ Existing large C++ codebases
- ⚠️ C language bindings required
- ⚠️ Features not yet in Rust (complex components)

### Next Steps

1. Follow the [Migration Checklist](#migration-checklist)
2. Review [Side-by-Side Examples](#side-by-side-examples)
3. Test with your 3MF files
4. Consult [FAQ](#faq-and-troubleshooting) if issues arise

**Welcome to the Rust 3MF ecosystem!** 🦀

---

*For questions or feedback, please open an issue on GitHub.*
