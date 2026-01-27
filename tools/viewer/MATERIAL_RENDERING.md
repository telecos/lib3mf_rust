# Material and Color Rendering Feature

## Overview

The 3MF viewer now supports rendering materials and colors from the 3MF Materials extension, allowing you to view models with their actual colors as defined in the 3MF file.

## Features

### Supported Material Types

1. **Base Materials** - Single-color materials defined with `<base>` elements
2. **Color Groups** - Collections of colors defined with `<colorgroup>` elements  
3. **Base Material Groups** - Groups of base materials with `<basematerialgroup>` elements

### Per-Triangle Coloring

The viewer supports per-triangle material properties:
- Each triangle can reference a material via the `pid` attribute
- Color indices can be specified via `pindex` (for the whole triangle) or `p1`, `p2`, `p3` (per-vertex)
- When triangles in an object have different colors, the viewer creates separate meshes for each color group

### Material Property Resolution

The viewer resolves colors in the following priority order:
1. **Triangle-level**: Check `triangle.pid` and color indices
2. **Object-level**: Fall back to `object.pid` if no triangle-level material
3. **Default**: Use default gray color if no materials specified

## Usage

### Keyboard Control

- **R Key**: Toggle material rendering ON/OFF
  - ON: Show materials and colors from the 3MF file
  - OFF: Show default gray color for all meshes

### Visual Feedback

- **Model Info**: Shows counts of materials, color groups, and base material groups
- **Menu Display**: Press 'M' to see current material rendering status
- **Console Output**: Material mode changes are logged to the console

## Technical Implementation

### Color Lookup

The implementation uses a two-level color lookup system:

```rust
fn get_triangle_color(model: &Model, obj: &Object, triangle: &Triangle) -> (f32, f32, f32) {
    // 1. Check triangle-level material (pid + pindex/p1)
    // 2. Look in base materials, color groups, or base material groups
    // 3. Fall back to object-level color
    // 4. Use default gray if no material found
}
```

### Mesh Construction

For objects with per-triangle colors:
- Triangles are grouped by their resolved color
- A separate mesh is created for each color group
- All meshes share the same vertex data but have different triangle indices

For objects with uniform colors:
- A single mesh is created with one color
- More efficient rendering

## Examples

### Test Files

The viewer has been tested with:
- `test_files/material/kinect_scan.3mf` - 3D scan with 59,357 colors and 282,544 triangles

### Command Line

```bash
# Launch viewer with material file
cd tools/viewer
cargo run --release -- ../../test_files/material/kinect_scan.3mf --ui

# Extract color information
cd ../..
cargo run --example extract_colors test_files/material/kinect_scan.3mf
```

### In the Viewer

1. Load a 3MF file with materials
2. Press 'R' to toggle between colored and default gray rendering
3. Press 'M' to see the menu with material information
4. Use normal camera controls to view the model

## Integration with Other Features

Material rendering works seamlessly with:
- **Boolean Mode**: Material rendering is temporarily disabled in boolean visualization modes
- **Displacement Mode**: Material rendering is temporarily disabled when viewing displacement data
- **Theme Switching**: Material colors are preserved when changing background themes
- **File Loading**: Material state persists when loading new files

## Limitations

### Current Version

1. **Per-Vertex Interpolation**: Colors are uniform per triangle face; true per-vertex color interpolation is not supported due to kiss3d limitations
2. **Textures**: Texture mapping (Texture2D, Texture2DGroup) is not yet implemented
3. **Transparency**: Alpha channel is parsed but not rendered; all materials appear opaque
4. **Composite Materials**: Not yet supported

### Future Enhancements

Potential future improvements:
- Per-vertex color interpolation with custom shader support
- Texture mapping with UV coordinates
- Transparency/alpha rendering with depth sorting
- Composite material support
- Material preview thumbnails

## Implementation Details

### File Changes

- `tools/viewer/src/ui_viewer.rs`:
  - Added `show_materials` field to `ViewerState`
  - Implemented `get_triangle_color()` for per-triangle material lookup
  - Enhanced `get_object_color()` to support BaseMaterialGroups
  - Created `create_mesh_nodes_with_materials()` for material-aware mesh creation
  - Created `create_mesh_nodes_with_triangle_colors()` for per-triangle coloring
  - Added 'R' key handler for material toggle
  - Updated menu and model info displays

### Data Structures

Materials are accessed from the model's resources:
```rust
model.resources.materials          // Vec<Material>
model.resources.color_groups        // Vec<ColorGroup>
model.resources.base_material_groups // Vec<BaseMaterialGroup>
```

Each triangle can reference materials:
```rust
triangle.pid     // Property group ID
triangle.pindex  // Property index for entire triangle
triangle.p1      // Property index for vertex 1
triangle.p2      // Property index for vertex 2
triangle.p3      // Property index for vertex 3
```

## Testing

### Automated Tests

The implementation passes:
- Cargo build (release and debug)
- Cargo clippy with `-D warnings`
- Color extraction example with material files

### Manual Testing

To manually verify material rendering:

1. **Load a colored model**:
   ```bash
   cargo run --release -- ../../test_files/material/kinect_scan.3mf --ui
   ```

2. **Check material info**: 
   - Model info should show material/color group counts
   - Press 'M' to see material status in menu

3. **Toggle rendering**:
   - Press 'R' to toggle between colored and gray
   - Verify colors change appropriately

4. **Check console output**:
   - Material mode changes should be logged
   - No errors should appear

## References

- 3MF Materials Extension Specification: https://3mf.io/specification/
- lib3mf_rust Material Types: `src/model/material.rs`
- Example: `examples/extract_colors.rs`
