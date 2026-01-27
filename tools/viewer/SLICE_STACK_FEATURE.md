# Slice Stack Visualization Feature

## Overview

The 3MF viewer now supports comprehensive visualization of slice stack data from the 3MF Slice Extension. This feature allows users to navigate through pre-computed 2D slices, animate the slicing sequence, and view slices in both 2D and 3D modes.

## Features

### 1. Automatic Slice Stack Detection

When opening a 3MF file containing slice stack data and pressing **Z** to enable slice view, the viewer automatically:

- Detects slice stacks in the model
- Displays detailed statistics:
  - Total number of slices
  - Z-axis range (bottom to top)
  - Average layer height
  - Current slice information

### 2. Single Slice Visualization

Navigate through individual slices one at a time:

- **Up Arrow**: Move to next slice (higher Z)
- **Down Arrow**: Move to previous slice (lower Z)
- **Home**: Jump to first slice
- **End**: Jump to last slice

For each slice, the viewer displays:
- Slice number (e.g., "Slice 42 / 378")
- Z height
- Number of vertices
- Number of polygons

### 3. 3D Stack Visualization

View all slices simultaneously in 3D space:

- **K**: Toggle 3D stack visualization on/off
- Color gradient from blue (bottom) to red (top)
- Current slice highlighted in yellow
- Adjustable spread factor for better visibility

**Spread Control (when 3D mode is active):**
- **Shift+Up**: Increase spread factor (max 5.0x)
- **Shift+Down**: Decrease spread factor (min 1.0x)

### 4. Slice Animation

Automatically step through the slice stack:

- **Space**: Play/pause animation
- **[ (Left Bracket)**: Decrease animation speed
- **] (Right Bracket)**: Increase animation speed

Animation features:
- Speed range: 0.1 to 20.0 slices per second
- Automatic looping by default
- Smooth transitions between slices

### 5. Rendering Modes

- **N**: Toggle between filled and outline rendering modes
  - Outline: Show only polygon edges (default)
  - Filled: Add radial fill lines from polygon center

### 6. Mode Switching

- **S**: Toggle between slice stack mode and traditional mesh-based slicing
  - Slice Stack Mode: Navigate pre-computed slices from file
  - Traditional Mode: Compute slices from mesh geometry at arbitrary Z heights

## Keyboard Reference

### Essential Controls

| Key | Action |
|-----|--------|
| **Z** | Toggle slice view on/off |
| **Up/Down** | Navigate slices (or pan camera when not in slice mode) |
| **Space** | Play/pause slice animation |
| **K** | Toggle 3D stack visualization |
| **S** | Toggle slice stack mode / Take screenshot (fallback) |

### Advanced Controls

| Key | Action |
|-----|--------|
| **Home** | Jump to first slice (in stack mode) / Reset camera (otherwise) |
| **End** | Jump to last slice (in stack mode) |
| **[** | Decrease animation speed |
| **]** | Increase animation speed |
| **Shift+Up** | Increase spread factor (3D mode) / Increase Z height (traditional mode) |
| **Shift+Down** | Decrease spread factor (3D mode) / Decrease Z height (traditional mode) |
| **N** | Toggle filled/outline rendering |
| **L** | Toggle slice plane visibility |
| **X** | Export current slice to PNG |

## Technical Details

### Data Structure Support

The feature uses the following 3MF Slice Extension structures:

```rust
pub struct SliceStack {
    pub id: usize,
    pub zbottom: f64,
    pub slices: Vec<Slice>,
    pub slice_refs: Vec<SliceRef>,
}

pub struct Slice {
    pub ztop: f64,
    pub vertices: Vec<Vertex2D>,
    pub polygons: Vec<SlicePolygon>,
}

pub struct SlicePolygon {
    pub startv: usize,
    pub segments: Vec<SliceSegment>,
}
```

### Rendering Implementation

**Single Slice Mode:**
- Renders the current slice at its exact Z height
- Colors based on position in stack (gradient)
- Supports both outline and filled modes

**3D Stack Mode:**
- Renders all slices simultaneously
- Applies spread factor to Z coordinates for visibility
- Current slice highlighted in yellow
- Other slices use blue-to-red gradient

**Animation:**
- Updates at ~60 FPS
- Smooth progression through slice indices
- Configurable speed and loop behavior

### Color Scheme

- **Single Slice**: Gradient color based on position (blue → red)
- **3D Stack**: Blue (bottom) → Red (top) gradient
- **Current Slice (3D)**: Bright yellow highlight
- **Slice Plane**: Yellow outline
- **Contours (traditional)**: Red lines

## Example Usage

### Viewing a Sliced Model

1. Open a 3MF file with slice data:
   ```bash
   cargo run --bin lib3mf-viewer test_files/slices/box_sliced.3mf
   ```

2. Press **Z** to enable slice view
   - Slice stack information is displayed
   - Slice stack mode is automatically enabled

3. Navigate through slices:
   - Use **Up/Down** arrows to step through layers
   - Press **K** to see all slices in 3D

4. Animate the slicing:
   - Press **Space** to start animation
   - Adjust speed with **[** and **]**

5. Adjust visualization:
   - Use **Shift+Up/Down** to spread slices apart (in 3D mode)
   - Press **N** to toggle filled mode

### Working with Test Files

The viewer works with the official 3MF test files located in `test_files/slices/`:

```bash
cd tools/viewer
cargo run -- ../../test_files/slices/box_sliced.3mf
```

This file contains 378 slices representing a box model with 0.08mm layer height.

## Implementation Notes

### Performance

- Efficient rendering using kiss3d line primitives
- Animation runs at monitor refresh rate (~60 FPS)
- Handles hundreds of slices smoothly (tested with 378 slices)

### Limitations

- Currently supports the first slice stack in the model
- Filled mode uses simple radial triangulation (not true polygon fill)
- Spread factor limited to 5.0x to maintain usability
- Animation speed limited to 20 slices/second

### Future Enhancements

Potential improvements for future versions:

- [ ] Support for multiple slice stacks with selector
- [ ] True polygon fill rendering using tessellation
- [ ] Export animation to video/GIF
- [ ] Side-by-side view of slice + original mesh
- [ ] Slice comparison mode (overlay multiple slices)
- [ ] Custom color schemes for slices
- [ ] Slice thickness visualization

## Integration

This feature integrates seamlessly with existing viewer capabilities:

- Works alongside beam lattice visualization
- Compatible with displacement visualization
- Respects theme and print area settings
- Coexists with boolean operation modes

## Troubleshooting

**Q: Slice stack mode doesn't activate**
- Ensure the file contains slice stack data
- Check that you've pressed **Z** to enable slice view
- Verify the file is valid with `cargo run --example slice_extension_demo`

**Q: Animation is too fast/slow**
- Use **[** and **]** to adjust speed
- Speed is displayed in console (e.g., "2.0 slices/sec")

**Q: Can't see slices in 3D mode**
- Increase spread factor with **Shift+Up**
- Try fitting camera with **F** key
- Check slice plane is not obscuring view

**Q: Polygons not rendering correctly**
- This is expected for complex/concave polygons in filled mode
- Switch to outline mode with **N** for accurate representation

## See Also

- [Main Viewer Documentation](README.md)
- [Slice Extension Demo](../../examples/slice_extension_demo.rs)
- [3MF Slice Extension Specification](https://3mf.io/specification/)
