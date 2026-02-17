# Pyramid Beam Lattice Sample

This sample demonstrates slicing a 3MF file containing beam lattice structures according to the 3MF Beam Lattice Extension specification.

## Model Description

The `pyramid.3mf` file contains a pyramid-shaped beam lattice structure with:
- **123 vertices** positioned to form a pyramid lattice
- **~300 beams** connecting the vertices with cylindrical cross-sections
- **Z-range**: 0 to 100mm
- **Default beam radius**: 1mm (configurable per beam)
- **Supports tapered beams** with different radii at endpoints

## Configuration

The `config.json` file specifies:
- **Slice thickness**: 10mm (10000 μm) for clear visualization of beam cross-sections
- **Printable box**: Encompasses the entire pyramid structure with some margin
- **Resolution**: 150 DPI, resulting in 915×974 pixel images

## Expected Output

Running the slicer generates **11 slice images** from Z=0mm to Z=100mm:

- **Z=0mm** (base): 58 contours showing beam cross-sections at the pyramid base
- **Z=10-90mm** (middle layers): Varying contour counts as the pyramid narrows
- **Z=100mm** (apex): 6 contours showing the pyramid converging to its top

Each beam appears as a circular cross-section in the slice images, with the circles approximated as 16-segment polygons for accurate rendering.

## How to Run

From the `tools/slicer` directory:

```bash
# Build the slicer
cargo build --release

# Generate slices
../../target/release/lib3mf-slicer \
    samples/pyramid/pyramid.3mf \
    samples/pyramid/config.json \
    -o samples/pyramid/output

# View output
ls samples/pyramid/output/*.png
```

## Verification

The successful slicing of this model verifies that:

1. ✅ Beam lattice structures are correctly detected and processed
2. ✅ Beam-plane intersections are computed accurately
3. ✅ Circular cross-sections are rendered at appropriate Z-heights
4. ✅ Tapered beams with varying radii are handled properly
5. ✅ Ball joints (if present) are also rendered as circular cross-sections
6. ✅ Output integrates seamlessly with triangle mesh slicing pipeline

## Beam Lattice Features Demonstrated

- **Cylindrical beams**: Each beam is represented as a cylinder connecting two vertices
- **Beam radii**: Default radius from beamset with per-beam override capability
- **Tapered beams**: Radius interpolation between endpoints (r1 and r2)
- **Cap modes**: Sphere/butt/hemisphere (visual only, not affecting slicing)
- **Ball joints**: Spherical nodes at vertices (rendered as circles in slices)

## Technical Details

The slicing algorithm:
1. Computes beam-plane intersections using parametric line-plane intersection
2. Interpolates radius at intersection point for tapered beams: `r = r1 + t*(r2-r1)`
3. For ball joints, uses sphere geometry: `r_slice² = r² - dz²`
4. Approximates each circular cross-section as a 16-segment polygon
5. Integrates beam segments with triangle mesh segments via `assemble_contours()`
