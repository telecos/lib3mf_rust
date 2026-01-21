---
name: Support Advanced Material Properties
about: Implement textures, composites, and multi-properties from Materials extension
title: 'Support Advanced Material Properties (Textures, Composites)'
labels: 'feature, priority:medium, materials'
assignees: ''
---

## Description

The Materials extension supports advanced properties beyond color groups: Texture2D, composite materials, and multi-property groups. Currently only color groups and basic base materials are implemented.

## Current State

- ✅ Color groups fully supported
- ✅ Basic base materials parsing
- ❌ Texture2D not supported
- ❌ Texture coordinates not parsed
- ❌ Composite materials not supported
- ❌ Multi-property groups not supported

## Impact

- Cannot work with textured models
- Cannot handle material blending/composites
- Missing advanced manufacturing capabilities
- Limited Materials extension support

## Expected Outcome

1. **Texture2D Support**:
   ```rust
   pub struct Texture2D {
       pub id: usize,
       pub path: String,  // Path in package
       pub content_type: String,
       pub tilestyleu: TileStyle,
       pub tilestylev: TileStyle,
   }
   
   pub enum TileStyle {
       Wrap,
       Mirror,
       Clamp,
       None,
   }
   
   pub struct TextureCoordinate {
       pub u: f64,
       pub v: f64,
   }
   ```

2. **Composite Materials**:
   ```rust
   pub struct Composite {
       pub material_ids: Vec<usize>,
       pub values: Vec<f64>,  // Mix proportions
   }
   ```

3. **Multi-Properties**:
   ```rust
   pub struct MultiPropertyGroup {
       pub id: usize,
       pub pids: Vec<usize>,  // Property indices
   }
   ```

4. **Parser Updates**:
   - Parse `<m:texture2d>` elements
   - Extract texture paths from package
   - Parse `<m:composite>` materials
   - Handle `<m:multiproperties>` groups
   - Store texture coordinates on triangles

## Implementation Notes

**Materials Extension Elements**:
- `m:texture2d` - Texture image definition
- `m:tex2coord` - Texture coordinates (u, v)
- `m:composite` - Blended material
- `m:multiproperties` - Multi-property group
- Triangle attributes: `tex2coord`, `p1tex2coord`, etc.

**Package Integration**:
- Textures stored as separate files in 3MF package
- Parser must extract texture image data
- Writer must embed texture files

**Spec Reference**: [Materials Extension 1.2.1](https://github.com/3MFConsortium/spec_materials)

## Test Files

Advanced materials used in conformance test suite - look for files with texture or composite elements.

## Acceptance Criteria

- [ ] `Texture2D` struct and enums added
- [ ] Parser extracts texture definitions
- [ ] Texture coordinates parsed on triangles
- [ ] Texture image data accessible from package
- [ ] Composite materials parsed
- [ ] Multi-property groups parsed
- [ ] Data accessible via Model API
- [ ] Tests added for advanced materials
- [ ] Documentation updated
- [ ] README shows full Materials extension support

## Related Issues

- Production Extension (#1)
- Writer Support (#1 in features) - Needed to write textured models

## Priority

**Medium** - Advanced materials are important for realistic rendering and multi-material manufacturing, but basic materials cover common use cases.

## Effort Estimate

**Medium (5-7 days)** - Multiple data structures, texture file handling, comprehensive parsing.

## Notes

Texture support may require image decoding/encoding if validation is needed. Consider using `image` crate for this.
