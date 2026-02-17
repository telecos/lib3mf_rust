# Displacement Map Sample

This sample demonstrates slicing a 3MF file with displacement mapping applied to a simple box mesh.

## Model Description

- **Geometry**: 10mm × 10mm × 10mm box centered at origin
- **Displacement Extension**: Applied with radial gradient texture
- **Displacement Height**: 1.5mm displacement amplitude
- **Normal Vectors**: Bottom face vertices point downward (-Z), top face vertices point upward (+Z)
- **UV Mapping**: Standard 0-1 UV coordinates mapped to texture

## Displacement Mapping

The displacement map uses a radial gradient texture (256×256 pixels):
- White (255) at center = maximum displacement outward
- Black (0) at edges = no displacement
- Formula: `displacement = offset + (height × texture_value × factor)`

With the settings:
- `offset = 0.0mm`
- `height = 1.5mm`
- `factor = 1.0` (default)
- `texture_value = 0-255 normalized to 0-1`

This creates a "bulge" effect where the center of each face displaces outward by up to 1.5mm.

## Slicing Configuration

The `config.json` file is configured for:
- **Slice thickness**: 100 μm (0.1mm)
- **Printable volume**: (-10, -10, -2) to (10, 10, 12) mm
  - Includes extra Z range to capture displaced surfaces
- **Resolution**: 150 DPI

## Running the Slicer

From the `tools/slicer` directory:

```bash
cargo run --release -- samples/displacement/box_displaced.3mf samples/displacement/config.json -o slices_displacement
```

## Expected Output

The slicer should generate approximately 140 slice images:
- Bottom slices (Z < 0mm): Show the displaced bottom face bulging downward
- Middle slices (0mm ≤ Z ≤ 10mm): Show the box cross-section with displaced side walls
- Top slices (Z > 10mm): Show the displaced top face bulging upward

Each slice will show the effect of displacement mapping, with contours that differ from a standard non-displaced box.

## Notes

This example validates that the slicer correctly:
1. Detects objects with `DisplacementMesh` instead of regular `Mesh`
2. Loads PNG displacement textures from `/3D/Textures/`
3. Samples texture values using UV coordinates
4. Applies displacement along normal vectors
5. Handles tiling modes (wrap, mirror, clamp, none)
6. Respects displacement height and offset parameters
