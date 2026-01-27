# 2D Slice View Feature

## Overview

The 2D Slice View feature allows you to visualize cross-sections of your 3D model at any Z height. This is particularly useful for:
- Analyzing internal structures
- Verifying wall thickness
- Understanding layer-by-layer composition
- Preparing for 3D printing slice analysis

## Features

### 1. Interactive Slice Plane
- **Toggle slice view**: Press `Z` to enable/disable the slice view
- **Adjust Z height**: Use `Shift+Up` and `Shift+Down` to move the slice plane up or down
- **Toggle slice plane visibility**: Press `L` to show/hide the yellow slice plane rectangle in the 3D view

### 2. 2D Contour Visualization
- Red contour lines show where the model intersects the slice plane
- Contours are rendered in real-time as you adjust the Z height
- The number of intersection segments is displayed in the console

### 3. Export to PNG
- Press `X` to export the current slice view to a PNG file
- Exported images include:
  - A coordinate grid for reference
  - Red contour lines showing the model intersection
  - Z height in the filename (e.g., `slice_z_25.50mm_20260127_163000.png`)

## Usage

### Basic Workflow

1. **Load a 3MF file**
   - Use `Ctrl+O` to open a file dialog, or
   - Launch the viewer with a file: `lib3mf-viewer -u model.3mf`

2. **Enable slice view**
   - Press `Z` to toggle the slice view on
   - The console will display:
     - Current Z height
     - Z range (min to max)
     - Number of contour segments

3. **Adjust the slice plane**
   - Press `Shift+Up` to move the plane up (2% of model height per press)
   - Press `Shift+Down` to move the plane down
   - The console shows the updated Z value and segment count

4. **Export the slice**
   - Press `X` to export the current slice view to PNG
   - The file will be saved in the current directory
   - Filename includes Z height and timestamp

### Keyboard Controls

| Key | Action |
|-----|--------|
| `Z` | Toggle 2D slice view on/off |
| `Shift+Up` | Move slice plane up |
| `Shift+Down` | Move slice plane down |
| `L` | Toggle slice plane visibility in 3D view |
| `X` | Export current slice to PNG |

## Technical Details

### Slice Computation
- Uses triangle-plane intersection algorithm
- Computes intersection for each triangle edge with the horizontal Z plane
- Groups intersection points into line segments representing the 2D contour

### Export Format
- **Image format**: PNG
- **Grid spacing**: 10 units (configurable in code)
- **Scale**: 10 pixels per unit (configurable in code)
- **Colors**:
  - White background
  - Light gray grid (220, 220, 220)
  - Red contour lines (255, 0, 0)

### Performance
- Slice computation is performed in real-time
- For large models (10,000+ triangles), there may be a brief delay when adjusting Z height
- Export is fast, typically completing in under a second

## Examples

### Example 1: Analyzing a Box
```bash
lib3mf-viewer -u test_files/core/box.3mf
# Press Z to enable slice view
# Use Shift+Up/Down to move through the model
# Press X to export interesting cross-sections
```

### Example 2: Complex Geometry
```bash
lib3mf-viewer -u test_files/core/torus.3mf
# Enable slice view with Z
# Move to middle of the torus
# Export to see the characteristic donut shape cross-section
```

## Limitations

- Currently only supports horizontal (XY plane) slices at varying Z heights
- Slice plane is always aligned with the model's coordinate system
- Export is limited to PNG format (no SVG or other vector formats)
- Grid and scale are fixed (not adjustable via UI)

## Future Enhancements

Potential improvements for future versions:
- [ ] Interactive slider for precise Z height control
- [ ] Separate 2D window showing just the slice (orthographic top-down view)
- [ ] Support for vertical slice planes (XZ and YZ)
- [ ] Adjustable export settings (resolution, colors, grid spacing)
- [ ] SVG export for vector graphics
- [ ] Multiple simultaneous slice planes
- [ ] Animation of slice plane sweeping through model

## Tips

1. **Finding the right Z height**: Start by enabling the slice view, which positions the plane at the model's midpoint. Then use Shift+Up/Down to explore.

2. **Complex models**: For models with many internal features, try exporting slices at regular intervals (e.g., every 10mm) to build a complete picture.

3. **Export organization**: Exported PNG files include timestamps, so you can safely export multiple slices without overwriting previous exports.

4. **Visibility**: If the red contours are hard to see against the model, try rotating the view or temporarily hiding the slice plane with `L`.

## Troubleshooting

**Q: I pressed Z but nothing appears**
- Make sure a model is loaded
- Check the console output for Z range information
- The slice plane might be at a Z height with no geometry - try adjusting with Shift+Up/Down

**Q: The exported PNG is empty or has no contours**
- Ensure the slice plane intersects the model geometry
- Check the console for the number of segments (should be > 0)
- Try adjusting the Z height

**Q: The grid is too fine/coarse in exported images**
- Currently the grid spacing is fixed at 10 units
- You can modify the `grid_spacing` variable in the `export_slice_to_png` function

**Q: Performance is slow on large models**
- The slice computation is O(n) where n is the number of triangles
- Consider using a lower-resolution model for interactive exploration
- Export is always fast regardless of model size
