# Object Selection and Highlighting Feature

This document describes the object selection and highlighting feature added to the 3MF viewer.

## Overview

The 3D viewer now supports interactive object selection with visual highlighting and detailed object information display. Users can click on objects to select them, view their properties, and perform actions like focusing the camera or hiding/showing objects.

## Features

### 1. Object Selection

- **Click to Select**: Left-click on any mesh object in the 3D view to select it
- **Multi-Select**: Hold `Ctrl` while clicking to select multiple objects
- **Deselect**: Click on empty space or press `Escape` to clear selection
- **Visual Feedback**: Selected objects are highlighted with a yellow tint

### 2. Selection Information

When objects are selected, the Model Information Panel (press `I` to toggle) displays:

- Number of selected objects
- Object ID and name (if available)
- Vertex count
- Triangle count
- Material/color information
- Transform status (if non-identity transform is applied)

### 3. Selection Actions

| Key | Action | Description |
|-----|--------|-------------|
| `F` | Focus | Focuses camera on selected object (or fits whole model if no selection) |
| `J` | Hide/Show | Toggles visibility of selected objects |
| `Y` | Isolate | Shows only selected objects, hiding all others (press `Y` again with no selection to show all) |
| `Escape` | Clear | Clears current selection |

## Technical Implementation

### Ray Casting

The selection system uses ray casting to detect which object the user clicked on:

1. Mouse click position is converted to a ray in world space
2. The ray is tested against all triangles in all mesh objects
3. The closest intersected object is selected

The implementation uses the Möller-Trumbore ray-triangle intersection algorithm for accurate and efficient picking.

### Selection State

Selection is tracked using a `SelectionState` struct that maintains:
- Set of selected object indices
- Selection highlight color (yellow by default)
- Original colors for restoration after deselection

### Visual Highlighting

Selected objects are highlighted by blending their original color with the selection color:
- 50% blend between original color and yellow
- Highlighting is applied dynamically when selection changes
- Original colors are restored on deselection

### Object Information

The model info panel was extended to show a "Selection" section when objects are selected, displaying:
- Total count of selected objects
- Detailed information for the first selected object (in multi-select scenarios)
- Material properties and transform status

## Usage Examples

### Basic Selection

1. Load a 3MF file with multiple objects (e.g., `test_files/components/assembly.3mf`)
2. Left-click on an object to select it
3. The object will be highlighted in yellow
4. Press `I` to see detailed object information in the info panel

### Multi-Selection

1. Click on first object to select it
2. Hold `Ctrl` and click on additional objects to add them to selection
3. Selected count shown in info panel
4. Click on already-selected object while holding `Ctrl` to deselect it

### Focus on Object

1. Select an object
2. Press `F` to focus the camera on that object
3. The camera will zoom and center on the selected object's bounding box

### Isolate Selected

1. Select one or more objects
2. Press `Y` to hide all other objects
3. Only selected objects remain visible
4. Press `Y` again with no selection to show all objects

### Hide Selected

1. Select one or more objects
2. Press `J` to toggle their visibility
3. Press `J` again to show them

## Code Structure

The selection feature is implemented in `tools/viewer/src/ui_viewer.rs`:

- **SelectionState**: Struct for tracking selection state (~lines 360-410)
- **Ray and ray_triangle_intersection**: Ray casting structures and functions (~lines 36-100)
- **pick_object**: Main object picking function using ray casting (~lines 103-205)
- **apply_selection_highlight**: Applies visual highlighting to selected objects (~lines 2033-2075)
- **restore_mesh_colors**: Restores original colors after deselection (~lines 2077-2091)
- **focus_camera_on_object**: Focuses camera on selected object (~lines 2127-2184)
- **Event Handlers**: Mouse and keyboard event handling for selection (~lines 858-920, 1457-1503)
- **Info Panel**: Selection information display (~lines 3285-3374)

Keybindings are documented in `tools/viewer/src/keybindings.rs` with a new "SELECTION" category.

## Performance Considerations

- Ray casting is performed only on mouse click, not continuously
- Intersection tests are performed against raw triangle data
- For models with many objects/triangles, selection may take a moment
- Tested and working well with typical CAD models (< 100K triangles)

## Future Enhancements

Potential improvements that could be added:

1. **Better Highlighting**: Use shader-based outline rendering instead of color tint
2. **Bounding Box Acceleration**: Use object bounding boxes for faster initial intersection tests
3. **Hover Preview**: Show object info on hover before clicking
4. **Copy to Clipboard**: Add ability to copy object info to clipboard
5. **Selection Box**: Add drag-select functionality for selecting multiple objects
6. **Selection History**: Remember previous selections for undo/redo

## Testing

The feature has been tested with:
- Single object selection
- Multi-object selection
- Selection clearing
- Camera focus on selected objects
- Hide/show and isolate operations
- Info panel display of selection details

Manual testing recommended with various 3MF files containing multiple objects.
