## XYZ Axis Visualization - Visual Reference

This document provides a textual description of what the XYZ axes look like in the viewer.

### Coordinate System

The viewer uses a standard right-handed 3D coordinate system:

```
        Y (Green)
        |
        |
        |
        O -------- X (Red)
       /
      /
     Z (Blue)
```

### Axis Rendering

When enabled (default), three colored lines are drawn from the origin:

1. **X Axis (Red)**
   - Start: (0, 0, 0)
   - End: (length, 0, 0)
   - Color: RGB(1.0, 0.0, 0.0) - Pure Red

2. **Y Axis (Green)**
   - Start: (0, 0, 0)
   - End: (0, length, 0)
   - Color: RGB(0.0, 1.0, 0.0) - Pure Green

3. **Z Axis (Blue)**
   - Start: (0, 0, 0)
   - End: (0, 0, length)
   - Color: RGB(0.0, 0.0, 1.0) - Pure Blue

### Axis Length

The length of each axis is automatically calculated as:
- `length = max_model_dimension * 0.5`

This ensures the axes are:
- Visible but not overwhelming
- Proportional to the model being viewed
- Useful for understanding scale

### Toggle Behavior

**Default State: ON (Visible)**

```
Press 'A' → Axes disappear → Console: "XYZ Axes: OFF"
Press 'A' → Axes appear    → Console: "XYZ Axes: ON"
```

### Example Usage Scenarios

#### Scenario 1: Small Box Model (10mm x 10mm x 10mm)
- Model max dimension: 10mm
- Axis length: 5mm
- Result: Axes extend 5mm in each direction from origin

#### Scenario 2: Large Building Model (1000mm x 500mm x 300mm)
- Model max dimension: 1000mm
- Axis length: 500mm
- Result: Axes extend 500mm in each direction from origin

### Visual Benefits

1. **Orientation Reference**
   - Quickly understand model orientation in 3D space
   - Identify which way is "up" (typically Y or Z depending on model)

2. **Scale Understanding**
   - Axes provide a reference for model size
   - Helpful when working with models of unknown dimensions

3. **Debugging**
   - Verify coordinate system handedness
   - Confirm model is positioned correctly relative to origin

4. **Navigation Aid**
   - Provides fixed reference while rotating view
   - Helps maintain spatial awareness during complex manipulations

### Console Output Example

```
═══════════════════════════════════════════════════════════
  Interactive 3D Viewer Controls
═══════════════════════════════════════════════════════════

  🖱️  Left Mouse + Drag  : Rotate view
  🖱️  Right Mouse + Drag : Pan view
  🖱️  Scroll Wheel       : Zoom in/out
  ⌨️  Arrow Keys         : Pan view
  ⌨️  A Key              : Toggle XYZ axes
  ⌨️  ESC / Close Window : Exit viewer

═══════════════════════════════════════════════════════════

  Model Information:
  - Objects: 1
  - Triangles: 12
  - Vertices: 8
  - Unit: millimeter

═══════════════════════════════════════════════════════════

[User presses 'A']
XYZ Axes: OFF

[User presses 'A' again]
XYZ Axes: ON
```

### Color Standards Reference

The RGB color scheme follows the common convention in 3D graphics:
- **Red (X)**: Horizontal left-right axis
- **Green (Y)**: Vertical up-down axis  
- **Blue (Z)**: Depth front-back axis

This is the same convention used by many 3D modeling tools:
- Blender (default)
- Maya
- 3ds Max (with Y-up)
- Unity (with Y-up)
