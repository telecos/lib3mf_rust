# Box with Slice Stack Sample

This sample demonstrates the slicer's ability to work with objects that contain **slice stacks** from the Slice extension.

## File Description

`box_sliced.3mf` contains:
- A simple box object (10.104 × 20.207 × 30.308 mm)
- A slice stack with 378 pre-computed slices at 0.08mm intervals
- A build item with transform that positions the box at approximately (33, 31.5, 15) mm

## What Makes This Special

Instead of computing slices by intersecting the 3D mesh with planes, the slicer extracts pre-computed slice data from the slice stack. This demonstrates:

1. **Slice Stack Detection**: The slicer recognizes that the object references a slice stack
2. **Transform Handling**: Build item transforms are correctly applied to the 2D slice vertices
3. **Z-Coordinate Mapping**: World-space Z coordinates are converted to object-space Z to select the appropriate slice from the stack
4. **Contour Extraction**: 2D polygons from the slice are converted into contours for rendering

## Running the Sample

```bash
lib3mf-slicer box_sliced.3mf config.json -o output
```

## Configuration

The `config.json` file specifies:
- **Slice thickness**: 80 μm (0.08 mm) - matches the slice stack's layer height
- **Printable box**: (30, 30, 14) to (45, 55, 46) mm - encompasses the transformed box
- **Resolution**: 150 DPI - produces 89×148 pixel images

## Expected Output

The slicer should generate approximately 400 slice images covering the Z range from 14.0 to 46.0 mm. Slices with contours appear from approximately Z=15.6 mm to Z=45.2 mm, corresponding to the transformed box bounds.

Example output:
```
Progress: 21/400 layers (1 contours at Z=15.600 mm)
Progress: 231/400 layers (1 contours at Z=32.400 mm)
Progress: 381/400 layers (1 contours at Z=44.400 mm)
```

Each contour represents the rectangular cross-section of the box at that Z height.

## Technical Notes

### Slice Stack Structure

The slice stack in this file:
- Has `zbottom=0.0` and slices up to `ztop=30.308` (in object space)
- Contains rectangular polygons with 4 vertices
- Uses the Slice extension (xmlns:s="http://schemas.microsoft.com/3dmanufacturing/slice/2015/07")

### Transform Application

The build item has a transform:
```
transform="1.0 0.0 0.0 0.0 1.0 0.0 0.0 0.0 1.0 33.0327 31.5297 14.9670"
```

This is an identity rotation/scale with translation (33.0327, 31.5297, 14.9670) mm. The slicer:
1. Transforms the slice stack's Z bounds to world space for intersection testing
2. Converts world-space Z back to object-space Z to find the correct slice
3. Transforms each 2D vertex from the slice to world space for rendering

## Source

This file (`box_sliced.3mf`) is from the 3MF Consortium's official conformance test suite for the Slice extension. It demonstrates a compliant implementation of pre-computed slice data according to the 3MF Slice specification (http://schemas.microsoft.com/3dmanufacturing/slice/2015/07).
