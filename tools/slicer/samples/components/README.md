# Component Hierarchy Sample

This sample demonstrates the slicer's ability to process 3MF files with component hierarchies and transform composition.

## Model Structure

The sample 3MF file contains:

1. **Object 1**: Base cube (10×10×10mm)
   - Simple mesh object (not used directly in build)

2. **Object 2**: Sphere (radius 3mm)
   - Spherical mesh with 16 segments and 12 rings

3. **Object 3**: Component Assembly
   - References Object 2 (sphere) four times
   - Each component has a transform placing spheres at the corners of an imaginary 10×10mm square
   - Positions: (±5, ±5, 5)mm relative to the component origin

4. **Build Item**: References Object 3
   - Applies an additional transform moving the entire assembly up by 20mm in Z
   - Final sphere centers at approximately: (±5, ±5, 25)mm in world space

## Transform Composition

The slicer correctly composes transforms through the hierarchy:
- Each sphere component has a translation transform
- The build item applies an additional translation
- Final positions = BuildItem transform × Component transform

## Expected Output

The slicing should produce contours showing:
- 4 circular cross-sections (the spheres) at various Z heights
- Peak activity around Z=25mm where all 4 spheres intersect the plane
- Spheres appear from Z≈17mm to Z≈29mm (center at 25mm ± radius 3mm, accounting for build transform)

## Configuration

- **Slice thickness**: 100 μm (0.1mm)
- **Printable box**: (-15, -15, 15) to (15, 15, 31) mm
- **Resolution**: 150 DPI (177×177 pixels)
- **Number of layers**: 160

## Running the Sample

```bash
cd tools/slicer
cargo run -- samples/components/components.3mf samples/components/config.json -o samples/components/output
```

## Verifying Component Support

Check slice images around Z=25mm (layer ~100) to see all four spheres:
- `slice_00100_z25.000mm.png` should show 4 circular cross-sections

This validates:
1. ✓ Recursive component resolution
2. ✓ Transform composition through hierarchy
3. ✓ Mesh merging from multiple component instances
4. ✓ Proper world-space positioning
