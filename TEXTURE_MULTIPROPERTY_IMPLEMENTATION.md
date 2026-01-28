# Texture and Multiproperty Rendering Implementation

## Summary

This implementation adds full support for rendering textures, multiproperties, and composite materials from the 3MF Materials extension in the viewer.

## Changes Made

### 1. Texture Loading and Rendering (`load_textures_from_package`)

Loads texture images from the 3MF package ZIP archive:
- Opens the 3MF file as a ZIP archive
- Extracts texture image files referenced by Texture2D resources
- Loads images using the `image` crate
- Returns a HashMap mapping texture IDs to loaded image data

### 2. UV Coordinate Mapping

Properly maps UV coordinates to triangle vertices:
- Duplicates vertices per triangle to assign correct UV coordinates
- Maps vertex indices to UV coordinates from Tex2DGroup
- Creates TriMesh with UV data using `TriMesh::new()` with uvs parameter
- Applies textures using `set_texture_from_memory()`

### 3. Multiproperty Color Resolution (`resolve_multiproperty_color`)

Blends colors from multiple property groups referenced by a multiproperty:
- Retrieves colors from each referenced property group (color groups or base material groups)
- Averages the colors (simplified Mix blend method)
- Falls back to magenta if resolution fails (for debugging)

### 4. Composite Material Color Resolution (`resolve_composite_color`)

Blends base materials according to composite mixing ratios:
- Finds the base material group referenced by the composite
- Blends materials using the composite values (proportions)
- Normalizes by sum of values to handle non-unity proportions
- Falls back to purple if resolution fails (for debugging)

### 5. Enhanced Mesh Creation

Updated mesh creation pipeline to support textures:
- Groups triangles by property type (color vs texture)
- Creates separate meshes for textured and colored triangles
- Passes file path through rendering pipeline for texture loading
- Applies loaded textures or falls back to teal indicator color

### 6. Code Quality

- Fixed clippy warning in `menu_ui.rs` (collapsible if statement)
- All existing tests pass
- Added examples to create test files with multiproperties and composites
- Added zip dependency for texture extraction

## Testing

Created two example programs to generate test 3MF files:

### `examples/create_multiproperty_test.rs`
Creates a file with 3 triangles using multiproperties:
- Triangle 1: Red + Yellow blend
- Triangle 2: Green + Cyan blend  
- Triangle 3: Blue + Yellow blend

Usage:
```bash
cargo run --example create_multiproperty_test output.3mf
```

### `examples/create_composite_test.rs`
Creates a file with 3 triangles using composite materials:
- Triangle 1: 70% Red + 30% Green = Orange
- Triangle 2: 60% Green + 40% Blue = Teal
- Triangle 3: 50% Red + 50% Blue = Purple

Usage:
```bash
cargo run --example create_composite_test output.3mf
```

## Verification

The viewer successfully loads and displays material information:
```
┌─ Resources ────────────────────────────────────────────┐
│ Objects:              1                                  │
│ Base Materials:       3                                  │
│ Color Groups:         1                                  │
│ Texture 2D Groups:    0                                  │
│ Composite Materials:  0                                  │
│ Multi-Properties:     1                                  │
└────────────────────────────────────────────────────────┘
```

## Texture Support

### Implemented Features
- ✅ Texture image loading from 3MF package
- ✅ UV coordinate mapping from Tex2DGroup
- ✅ Texture application to mesh surfaces
- ✅ Support for PNG and JPEG texture formats
- ✅ Multiple textures in same model
- ⚠️ Basic tile style support (via kiss3d defaults)

### Technical Implementation Details

**Texture Loading Process:**
1. Open 3MF file as ZIP archive using `zip` crate
2. Iterate through Texture2D resources in the model
3. Extract image files from the package
4. Load images using `image::load_from_memory()`
5. Store in HashMap for quick lookup during rendering

**UV Coordinate Mapping:**
- Each triangle with textures gets its vertices duplicated
- UV coordinates from Tex2DGroup are mapped to each vertex
- TriMesh created with uvs parameter: `TriMesh::new(vertices, normals, Some(uvs), indices)`
- Textures applied using `set_texture_from_memory()` with RGBA8 format

**Fallback Behavior:**
- If texture file not found: teal color indicator (#00CCCC)
- If texture fails to load: teal color indicator
- If no UV coordinates: teal color indicator

### Blend Methods
Currently only the "Mix" blend method is fully implemented (simple averaging).
The "Multiply" blend method could be added in the future.

## Files Modified

- `tools/viewer/src/ui_viewer.rs`: Added texture loading, UV mapping, multiproperty/composite support
- `tools/viewer/src/menu_ui.rs`: Fixed clippy warning
- `tools/viewer/Cargo.toml`: Added zip dependency
- `examples/create_multiproperty_test.rs`: New test file generator
- `examples/create_composite_test.rs`: New test file generator
