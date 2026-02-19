# Boolean Operations Sample

This sample demonstrates the slicer's ability to apply 3MF Boolean Operations (Boolean Operations extension) at slice time using 2D polygon operations.

## Model Structure

The sample 3MF file (`boolean_diff.3mf`) contains:

1. **Object 1**: Cube A - Base cube (20×20×20mm) centered at origin
   - Main object for the boolean operation

2. **Object 2**: Cube B - Smaller cube (12×12×12mm) offset at (−1,−1,−6) to (11,11,6) mm
   - Object to be subtracted from Cube A

3. **Object 3**: Boolean Difference Operation
   - Defines: Object 1 − Object 2 (difference operation)
   - Has no direct mesh — only a boolean shape definition

4. **Build Item**: References Object 3
   - Applies a transform moving the result up by 10mm in Z
   - Final world-space Z range: 0mm to 20mm (Cube A); 4mm to 16mm (Cube B overlap)

## Boolean Operations Extension

This file uses the 3MF Boolean Operations extension which allows defining CSG (Constructive Solid Geometry) operations:
- **Union**: Combines two volumes
- **Intersection**: Keeps only overlapping volume
- **Difference**: Subtracts second volume from first

## Slicer Implementation

The lib3mf-slicer implements boolean operations at the **2D slice level** using polygon clipping:

1. The base object (Cube A) is sliced at each Z height to get its 2D cross-section
2. Each operand (Cube B) is sliced at the same Z height to get its 2D cross-section
3. The 2D polygon boolean operation (difference, union, or intersection) is applied using Clipper2
4. The resulting 2D polygon is rendered in the slice image

### What to Expect

- **Z < 4mm** (below Cube B): Slices show the full 20×20mm Cube A square cross-section (400mm²)
- **4mm ≤ Z ≤ 16mm** (Cube B overlap zone): Slices show the L-shaped region after subtracting Cube B's cross-section (≈279mm²)
- **Z > 16mm** (above Cube B): Slices show the full Cube A square again (400mm²)

The L-shaped cross-section at mid-height has corners at approximately:
(-10,−10), (10,−10), (10,−1), (−1,−1), (−1,10), (−10,10)

### Enabling / Disabling Boolean Operations

Boolean operations support can be controlled via the `spec_support` configuration:

```json
{
  "spec_support": {
    "boolean_ops": false
  }
}
```

When `boolean_ops` is `false`, the slicer falls back to slicing only the base object mesh and displays a warning.

## Configuration

- **Slice thickness**: 100 μm (0.1mm)
- **Printable box**: (−15, −15, 0) to (15, 15, 22) mm
- **Resolution**: 150 DPI (177×177 pixels)
- **Number of layers**: 220

## Running the Sample

First, generate the sample 3MF file:

```bash
cd ../..  # from tools/slicer
cargo run --example create_boolean_sample
```

Then run the slicer:

```bash
cd tools/slicer
cargo run -- samples/boolean/boolean_diff.3mf samples/boolean/config.json -o samples/boolean/output
```

## Expected Output

The slicer will:
1. Load the model successfully (no warnings)
2. Generate 220 slice images
3. Slices at Z < 4mm: 20×20mm filled square
4. Slices at 4mm ≤ Z ≤ 16mm: L-shaped cross-section (Cube A minus Cube B)
5. Slices at Z > 16mm: 20×20mm filled square again

