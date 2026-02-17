# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-02-17

### Added

#### Core Slicing Engine
- **Mesh-plane intersection slicing** — Generates 2D contour slices from 3D mesh
  geometry at configurable Z-height intervals using mesh-plane intersection.
- **Closed contour assembly** — Intersection segments are assembled into closed
  contour loops for accurate geometry representation.
- **Scanline rasterization** — Contours are rendered into PNG images using a
  scanline fill algorithm with proper winding-number handling.
- **Configurable slice thickness** — Layer height specified in micrometers (μm)
  for precise control over slice resolution.
- **Configurable printable box** — Define the printable volume origin and extents
  in millimeters; only geometry within the box is sliced.
- **Resolution control** — Output image resolution specified in DPI (dots per
  inch) with automatic pixel dimension calculation.

#### 3MF Extension Support
- **Component hierarchy traversal** — Recursively resolves component references
  and composes transforms through the entire object hierarchy.
- **Beam lattice rendering** — Slices beam lattice structures by generating
  circular cross-sections at each beam/plane intersection point, sized
  according to beam radius with endpoint cap interpolation.
- **Displacement map support** — Applies texture-based surface displacement from
  the 3MF Displacement Extension, offsetting vertices along normal vectors based
  on grayscale texture values. Supports wrap, mirror, clamp, and none tile modes.
- **Slice stack support** — Extracts pre-computed 2D slice data from the 3MF
  Slice Extension instead of computing mesh intersections, with proper
  object-space to world-space Z coordinate mapping and vertex transformation.
- **Color/material-aware rendering** — Detects color groups and texture
  coordinates, rendering contour borders with interpolated per-vertex colors
  from material properties. Textured models show colored borders with mid-gray
  fill; plain models use solid black fill.
- **Boolean operations detection** — Recognizes boolean shape definitions
  (union, intersection, difference) and displays a warning; CSG mesh evaluation
  is not yet implemented.

#### CLI & Configuration
- **JSON configuration file** — All slicing parameters (thickness, printable box,
  resolution, feature toggles) specified via a JSON config file.
- **Command-line interface** — Built with `clap` for argument parsing with
  `--output`, `--verbose`, and `--help` options.
- **Verbose model info** — Optional `--verbose` flag displays object count,
  bounding boxes, material groups, and slice stack details.
- **Progress reporting** — Prints progress updates every 10 layers with contour
  count at each Z height.
- **Optional crypto support** — Encrypted 3MF files supported via the `crypto`
  feature flag, using `lib3mf`'s Secure Content decryption.

#### Samples & Documentation
- **Sample configurations** — Pre-built sample configs and 3MF files for:
  - `pyramid` — Beam lattice pyramid (10mm layers)
  - `cube_gears` — Multi-part gear assembly (100μm layers, 300 DPI)
  - `box_sliced` — Pre-computed slice stack from Slice Extension (80μm layers)
  - `components` — Component hierarchy with transform composition
  - `displacement` — Texture-based displacement mapping
  - `multipletextures` — Multi-texture color rendering
  - `boolean` — Boolean operations detection
- **Visual examples** — Reference slice images in `images/` directory showing
  cube_gears, multipletextures, and displacement results.
- **README** with full usage documentation, configuration reference, and
  architecture overview.
