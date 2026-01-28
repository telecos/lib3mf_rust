# Model Information Panel

## Overview

The Model Information Panel displays detailed statistics and metadata about the loaded 3MF model in the viewer. It provides a quick overview of the file's properties, geometry, extensions used, and objects contained in the model.

## Usage

### Toggle Panel
- **Keyboard**: Press `I` to toggle the panel on/off
- **Menu**: View → Model Information
- **Default**: Hidden (press `I` to show)

### Visual Layout

```
┌─ Model Information ─────────────────────┐
│                                         │
│  File: cube_gears.3mf                  │
│  Size: 2.45 MB                         │
│                                         │
│  Geometry                               │
│    Vertices: 12,450                    │
│    Triangles: 24,896                   │
│    Objects: 3                          │
│    Components: 5                       │
│    Bounds: 150.0 × 100.0 × 80.0 mm    │
│                                         │
│  Extensions                             │
│    ✓ Materials (3 mats, 2 groups)      │
│    ✓ Beam Lattice (45 beams)          │
│    ✓ Slice (127 slices)                │
│    ✓ Production (2 items)              │
│                                         │
│  Objects                                │
│    Main Body (mesh)                    │
│    Support (mesh)                      │
│    Logo (component)                    │
│    ... and 2 more                      │
│                                         │
└─────────────────────────────────────────┘
```

## Features

### 1. File Metadata
- **File name**: Displays the name of the loaded 3MF file
- **File size**: Shows size in MB

### 2. Geometry Statistics
- **Vertices**: Total count of all vertices across all meshes in build items
- **Triangles**: Total count of all triangles across all meshes in build items
- **Objects**: Count of all object resources in the model
- **Components**: Total count of component references
- **Bounds**: Bounding box dimensions (X × Y × Z) with model units

### 3. Extension Information
Shows which 3MF extensions are used in the file:
- **Materials**: Count of base materials and color groups
- **Beam Lattice**: Count of beams if present
- **Slice**: Count of slices in slice stacks if present
- **Production**: Count of production items with UUIDs

### 4. Object List
- Shows first 5 objects with:
  - Object name (or "Object {id}" if no name)
  - Object type: mesh, component, or other
- If more than 5 objects, shows "... and X more"

## Implementation Details

### Code Location
- **Panel struct**: `tools/viewer/src/ui_viewer.rs` - `ModelInfoPanel`
- **Rendering**: `render_model_info_panel()` function
- **Menu action**: Added `ToggleModelInfo` to `menu_ui.rs`
- **Keybinding**: Registered in `keybindings.rs`

### Statistics Functions
Uses existing helper functions:
- `count_vertices(model)` - Total vertex count
- `count_triangles(model)` - Total triangle count
- `count_beams(model)` - Total beam count
- `calculate_model_bounds(model)` - Bounding box calculation

### Rendering Style
- **Font**: Default kiss3d font
- **Font size**: 13pt for content, 15pt for main title
- **Colors**: 
  - Text: Light gray (0.9, 0.9, 0.9)
  - Headers: Yellow (1.0, 1.0, 0.6)
- **Layout**: Left-aligned at (10, 40) pixels from top-left
- **Line height**: 16 pixels
- **Section spacing**: 8 pixels

## Future Enhancements

Potential improvements for future versions:
- [ ] Surface area calculation
- [ ] Volume estimation
- [ ] Expandable object tree (click to show mesh details)
- [ ] Clickable objects to highlight in 3D view
- [ ] Validation status/warnings
- [ ] Custom metadata display
- [ ] Thumbnail preview
- [ ] 3MF package contents list
- [ ] Draggable panel position
- [ ] Resizable panel
- [ ] Export statistics to CSV/JSON

## Testing

The feature includes:
- Unit tests for panel state management
- Integration with existing viewer tests
- Clippy clean (no warnings)
- All 24 tests passing

## Related Issues

- telecos/lib3mf_rust#278 - GUI menu bar implementation
- This feature adds the Model Information panel accessible via the View menu
