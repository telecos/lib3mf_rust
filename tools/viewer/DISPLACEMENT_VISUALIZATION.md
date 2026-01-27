# Displacement Map Visualization

This document describes the displacement map visualization feature in the 3MF viewer.

## Overview

The displacement visualization feature allows you to identify and highlight objects in a 3MF model that use the Displacement extension. Objects with displacement data are rendered with a distinctive bright cyan color to make them easily identifiable.

## Features

### Automatic Detection
The viewer automatically detects when a model contains displacement extension data:
- Displacement maps (displacement2d resources)
- Normal vector groups (normvectorgroup resources)
- Displacement coordinate groups (disp2dgroup resources)
- Objects with displacement meshes

### Visual Highlighting
When displacement visualization is enabled (press 'D'), objects with displacement meshes are rendered in bright cyan (RGB: 0, 255, 255) instead of their normal colors. This makes it easy to identify which parts of your model use displacement mapping.

### Information Display

#### Model Info Panel
When a model with displacement data is loaded, the model information panel shows:
```
Model Information:
  - Objects: 1
  - Triangles: 12
  - Vertices: 8
  - Unit: millimeter
  - Displacement:
      Maps: 1
      Normal Vector Groups: 1
      Displacement Groups: 1
      Objects with Displacement: 1
```

#### Menu Display
Press 'M' to show the menu, which includes displacement status:
```
Menu - Current Settings
  Theme:           Dark
  Print Area:      ON
    Width (X):     200 mm
    Depth (Y):     200 mm
    Height (Z):    200 mm
  Displacement:    ON
    Maps:          1
    Groups:        1
    Objects:       1
  File:            test.3mf
```

## Controls

| Key | Action |
|-----|--------|
| D | Toggle displacement visualization on/off |
| M | Show/hide menu with displacement status |

## Usage

1. **Load a 3MF file** with displacement data (Ctrl+O or provide file path at startup)

2. **Check model info** - If the model contains displacement data, it will be shown in the model information panel

3. **Toggle visualization** - Press 'D' to enable displacement highlighting
   - Displaced objects will turn bright cyan
   - Console shows displacement statistics

4. **View details** - Press 'M' to see the menu with full displacement information

## Example Console Output

When toggling displacement visualization on:
```
Displacement Visualization: ON
  Displacement Maps: 1
  Normal Vector Groups: 1
  Displacement Groups: 1
  Objects with Displacement: 1
```

When toggling displacement visualization off:
```
Displacement Visualization: OFF
```

If no displacement data exists:
```
No displacement data in this model
```

## Technical Details

### Displacement Extension
The 3MF Displacement extension allows texture-based surface displacement that modifies mesh geometry. It includes:

- **Displacement Maps**: PNG textures that define displacement values
- **Normal Vector Groups**: Normalized vectors for displacement direction
- **Displacement Groups**: Coordinate groups linking vertices to displacement values
- **Displacement Meshes**: Alternative mesh definition with displacement properties

### Visualization Implementation
The viewer identifies objects with `displacement_mesh` data and renders them with distinct coloring. This is a visual indication only - the actual displacement calculation and mesh subdivision are not performed in the current implementation.

## Future Enhancements

Potential future improvements could include:
- Heat map visualization of displacement intensity
- Preview of actual displaced geometry with mesh subdivision
- Display displacement texture in info panel
- Before/after comparison toggle
- Displacement scale slider for preview

## Compatibility

This feature works with:
- 3MF files using the Displacement extension (http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07)
- Compatible with other viewer features (beam lattice, boolean operations, slice view)
- Works in all theme modes (Dark, Light, Blue, White, Black)

## See Also

- [3MF Displacement Extension Specification](https://github.com/3MFConsortium/spec_displacement)
- Viewer README for general usage
- Other visualization features (Beam Lattice, Boolean Operations, Slice View)
