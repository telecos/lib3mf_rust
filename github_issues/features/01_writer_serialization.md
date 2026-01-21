---
name: Implement 3MF Writer/Serialization Support
about: MAJOR FEATURE - Add ability to create and write 3MF files
title: 'Implement 3MF Writer/Serialization Support'
labels: 'feature, priority:critical, enhancement'
assignees: ''
---

## Description

**MAJOR MISSING FEATURE**: The library is currently **read-only** with NO ability to create or write 3MF files. If the goal is "100% compliance parser AND writer implementation," this is a critical gap.

## Current State

- ✅ Full reading/parsing support for 3MF files
- ✅ Data structures represent complete 3MF models
- ❌ **NO writer/serialization functionality**
- ❌ Cannot create 3MF files programmatically
- ❌ Cannot modify and save existing models
- ❌ No round-trip capability (read→write→read)

## Impact

- Cannot generate 3MF files programmatically
- Cannot modify existing files
- Read-only limits use cases significantly
- Cannot test parser via round-trip validation

## Expected Outcome

Implement complete writing/serialization functionality:

1. **Model → XML Serialization**:
   ```rust
   impl Model {
       /// Serialize model to 3MF file
       pub fn to_writer<W: Write + Seek>(
           &self,
           writer: W
       ) -> Result<()> {
           // Serialize to 3MF format
       }
       
       /// Serialize to ZIP archive
       pub fn save<P: AsRef<Path>>(
           &self,
           path: P
       ) -> Result<()> {
           let file = File::create(path)?;
           self.to_writer(file)
       }
   }
   ```

2. **XML Generation**:
   - Serialize `<model>` element with namespaces
   - Write `<resources>` with objects, materials, color groups
   - Write `<build>` with build items
   - Handle extension namespaces (materials, production, etc.)
   - Generate well-formed XML with proper formatting

3. **ZIP/OPC Package Creation**:
   - Create ZIP archive structure
   - Generate `_rels/.rels` (relationships)
   - Generate `[Content_Types].xml`
   - Write `3D/3dmodel.model` file
   - Support additional files (textures, thumbnails)
   - Follow OPC conventions

4. **Validation Before Write**:
   - Ensure model is valid before serialization
   - Check all references are resolved
   - Validate required attributes present

## Implementation Notes

**New Module**: `src/writer.rs`

**Core Functions**:
- `write_model(model: &Model, writer: &mut ZipWriter) -> Result<()>`
- `write_content_types(writer: &mut ZipWriter) -> Result<()>`
- `write_relationships(writer: &mut ZipWriter) -> Result<()>`
- `serialize_to_xml(model: &Model) -> Result<String>`

**Dependencies** (already available):
- `zip::ZipWriter` for archive creation
- `quick-xml` for XML generation

**Extension Support**:
- Detect which extensions are used in model
- Include appropriate namespace declarations
- Write extension-specific data (UUIDs, slices, beams)

**Spec Compliance**:
- Follow 3MF Core Spec for structure
- Ensure XML is valid per schema
- Follow OPC conventions for package structure

## Test Strategy

1. **Round-Trip Testing**:
   ```rust
   let original = Model::from_reader(File::open("test.3mf")?)?;
   
   let mut buffer = Vec::new();
   original.to_writer(&mut buffer)?;
   
   let reloaded = Model::from_reader(Cursor::new(buffer))?;
   
   assert_eq!(original, reloaded);
   ```

2. **Conformance**:
   - Written files should pass conformance tests
   - Validate with external tools if available

3. **Extension Coverage**:
   - Test writing with each extension
   - Ensure namespace declarations correct

## Acceptance Criteria

- [ ] `src/writer.rs` module created
- [ ] `Model::to_writer()` implemented
- [ ] `Model::save()` convenience method added
- [ ] ZIP archive structure correct
- [ ] XML well-formed and valid
- [ ] Content types file generated
- [ ] Relationships file generated  
- [ ] Round-trip tests pass (read→write→read)
- [ ] Extension namespaces written correctly
- [ ] Documentation added with examples
- [ ] README updated with writing examples

## Example Usage

```rust
use lib3mf::{Model, Mesh, Object, Vertex, Triangle, BuildItem};

// Create a new model
let mut model = Model::new();

// Add a simple cube
let mut mesh = Mesh::new();
mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
// ... add more vertices
mesh.triangles.push(Triangle::new(0, 1, 2));
// ... add more triangles

let mut object = Object::new(1);
object.mesh = Some(mesh);
model.resources.objects.push(object);

// Add to build
model.build.items.push(BuildItem::new(1));

// Write to file
model.save("output.3mf")?;
```

## References

- [3MF Core Specification](https://3mf.io/specification/)
- [Open Packaging Conventions](https://en.wikipedia.org/wiki/Open_Packaging_Conventions)
- Original REMAINING_ISSUES.md, Issue #14

## Related Issues

- Conformance Testing
- Extension Data Extraction (needed for writing extension data)

## Priority

**CRITICAL** if goal is writer support, otherwise **LOW**.

Based on user comment "100% compliance parser and writer implementation," this should be **CRITICAL PRIORITY**.

## Effort Estimate

**Large (1-2 weeks)** - Significant implementation:
- XML serialization logic
- ZIP/OPC package creation
- Extension handling
- Comprehensive testing
- Documentation

## Notes

This could be broken into sub-issues:
1. Basic XML serialization (core spec only)
2. ZIP/OPC package structure
3. Extension support in writer
4. Round-trip testing
5. Documentation and examples
