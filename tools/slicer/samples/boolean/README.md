# Boolean Operations Sample

This sample demonstrates the slicer's ability to detect 3MF files with boolean operations (Boolean Operations extension).

## Model Structure

The sample 3MF file contains:

1. **Object 1**: Cube A - Base cube (20×20×20mm) centered at origin
   - Main object for the boolean operation

2. **Object 2**: Cube B - Smaller cube (12×12×12mm) offset at (5, 5, 0)mm
   - Object to be subtracted from Cube A

3. **Object 3**: Boolean Difference Operation
   - Defines: Object 1 - Object 2 (difference operation)
   - Has no direct mesh - only boolean shape definition

4. **Build Item**: References Object 3
   - Applies transform moving the result up by 10mm in Z
   - Final position: Z range from 0mm to 20mm

## Boolean Operations Extension

This file uses the 3MF Boolean Operations extension which allows defining CSG (Constructive Solid Geometry) operations:
- **Union**: Combines two volumes
- **Intersection**: Keeps only overlapping volume
- **Difference**: Subtracts second volume from first

## Current Slicer Limitation

**Important**: The lib3mf-slicer currently **does not implement** boolean mesh operations (CSG).

When slicing this file, the slicer will:
1. Detect the boolean shape
2. Log a warning message (once per object)
3. Slice only the base mesh (Cube A) without applying the boolean operation
4. Generate slice images showing the complete Cube A (20×20×20mm)

Expected behavior when fully implemented:
- The slicer would compute Cube A - Cube B
- Result would show Cube A with a notch/cavity where Cube B overlaps
- Slice images would show the difference geometry

## Configuration

- **Slice thickness**: 100 μm (0.1mm)
- **Printable box**: (-15, -15, 0) to (15, 15, 22) mm
- **Resolution**: 150 DPI (177×177 pixels)
- **Number of layers**: 220

## Running the Sample

```bash
cd tools/slicer
cargo run -- samples/boolean/boolean_diff.3mf samples/boolean/config.json -o samples/boolean/output
```

## Expected Output

The slicer will:
1. Load the model successfully
2. Print: "Warning: Object 3 has boolean shape (operation: Difference) which is not yet supported by the slicer."
3. Generate 220 slice images showing the base cube (without boolean subtraction applied)
4. Slices will show a square cross-section (20×20mm) from Z=0 to Z=20mm

## Future Enhancements

Boolean mesh operations require:
- CSG mesh processing library (e.g., leveraging parry3d or a dedicated CSG library)
- Mesh intersection, union, and difference calculations
- Robust handling of edge cases and degenerate geometries

This sample validates that:
1. ✓ Boolean operation parsing works correctly
2. ✓ 3MF files with boolean shapes can be loaded
3. ✓ The slicer detects boolean operations
4. ✓ Appropriate warnings are displayed
5. ✗ Boolean operations are applied (not yet implemented)
