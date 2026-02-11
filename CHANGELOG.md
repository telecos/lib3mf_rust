# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Documentation improvements:
  - Added CONTRIBUTING.md with contribution guidelines
  - Added CHANGELOG.md for tracking changes
  - Cleaned up non-standard implementation documentation
  - Reorganized viewer documentation

## [0.1.3] - 2026-02-11

### Added

- **Feature flags for optional dependencies** — Dependencies are now organized
  into feature groups so users only pull in what they need:
  - `mesh-ops` (default) — Enables advanced mesh operations (`parry3d`,
    `nalgebra`). Disable with `default-features = false` if you only need
    parsing/writing.
  - `polygon-ops` (default) — Enables polygon clipping and triangulation
    (`clipper2`, `earcutr`). Disable with `default-features = false` if not
    processing slice polygon data.
  - `crypto` (opt-in) — Enables Secure Content extension decryption (`rsa`,
    `aes-gcm`, `base64`, `flate2`, `sha1`, `sha2`). Enable with
    `features = ["crypto"]` when you need to decrypt encrypted 3MF files.
- Examples that require optional features now declare `required-features` in
  `Cargo.toml` (`calculate_normals`, `mesh_analysis`, `mesh_slicing_demo`,
  `mesh_subdivision`, `polygon_clipping_demo`, `polygon_triangulation_demo`).

### Fixed

- **Critical: parry3d BVH panic on extremely small mesh coordinates** — Added
  validation for subnormal/extremely small vertex coordinates that caused
  parry3d's BVH construction to panic. Meshes with coordinates below `1e-35`
  are now rejected with a clear error. Added a `safe_create_trimesh` wrapper
  for additional robustness.

### Changed

- Bump `tempfile` dev-dependency from 3.24 to 3.25.

### Removed

- Removed unused `regex` dependency from runtime dependencies.

## [0.1.2] - 2026-02-09

### Fixed

- **Critical: Index out-of-bounds in mesh subdivision** — Added bounds check for
  triangle vertex indices in `subdivide_midpoint()` before indexing into the
  original vertex array, preventing a panic on malformed meshes.
- **Decompression bomb protection** — Added a 256 MB size limit to
  `decompress_deflate()` in the SecureContent decryption module via
  `Read::take()`, preventing denial-of-service from crafted compressed payloads.
- **Unbounded subdivision levels** — Capped subdivision levels to a maximum of 10
  to prevent exponential memory growth (each level multiplies triangle count by 4×).
  Used saturating arithmetic for capacity estimation to prevent integer overflow.
- **Non-finite vertex coordinates** — Added `is_finite()` and f32 range validation
  in `validate_mesh_for_parry3d()` to reject vertices that would become infinity
  after `f64 → f32` cast.
- **Silent parse failures in material extension** — Replaced four instances of
  `filter_map(|s| s.parse().ok())` in compositematerials/multiproperties parsing
  with proper error propagation. Malformed numeric values now produce clear parse
  errors instead of being silently dropped.
- **Integer truncation in beam lattice validator** — Replaced all `id as u32`
  truncation casts with safe `u32 as usize` widening casts when comparing beam
  lattice property IDs against resource IDs. Changed `HashSet<u32>` to
  `HashSet<usize>` to match the resource ID type.
- **Latent underflow in validator error messages** — Replaced `num_vertices - 1`
  with `saturating_sub(1)` in error format strings in core and slice validators.
- **Integer overflow in polygon triangulation** — Used `saturating_mul` and
  `saturating_add` for `Vec::with_capacity()` calculation.

## [0.1.1] - 2026-02-08

### Added

- **Volumetric Extension** - Full support for volumetric data in 3MF files:
  - Parsing of `<v:volumetricdata>`, `<v:boundary>`, `<v:voxels>`, `<v:voxel>`,
    `<v:volumetricpropertygroup>`, `<v:property>`, and `<v:implicit>` elements
  - Writing/serialization of all volumetric extension elements
  - Complete round-trip (parse → modify → write) support
  - Data types: `VolumetricData`, `VoxelGrid`, `Voxel`, `VolumetricBoundary`,
    `ImplicitVolume`, `VolumetricPropertyGroup`, `VolumetricProperty`
  - Extension handler (`VolumetricExtensionHandler`) registered in default registry
  - Validation: boundary checks, voxel bounds, spacing, dimension, duplicate-ID,
    and required-extension declaration
  - `volumetric_demo` example for creating, reading, and round-tripping volumetric data
  - Integration tests including file parsing and round-trip verification

## [0.1.0] - 2024-01-01

### Added

#### Core Features
- Pure Rust implementation for parsing 3MF files
- No unsafe code (enforced with `#![forbid(unsafe_code)]`)
- Write/Serialize 3MF files with full round-trip support
- Streaming parser for large files
- Comprehensive error handling with detailed messages

#### 3MF Specification Support
- **Core Specification** - Full support for:
  - Model structure (resources, build, metadata)
  - Mesh geometry (vertices, triangles)
  - Object definitions and component hierarchies
  - Build items with transformations
  - Basic materials and colors

#### Extension Support
- **Materials Extension** (1.2.1) - Complete support:
  - Color groups (m:colorgroup)
  - Per-triangle material references
  - Base materials with display colors
  - Texture2D resources with UV coordinates
  - Composite materials with mixing ratios
  - Multi-properties for layered materials

- **Production Extension** (1.2.0):
  - UUID extraction from objects, build, and build items
  - Production path information

- **Slice Extension** (1.0.2):
  - Slice stacks and polygons
  - Polygon clipping operations
  - Self-intersection resolution

- **Beam Lattice Extension** (1.2.0):
  - BeamSet structures with radius and cap modes
  - Individual beam definitions with variable radii

- **Secure Content Extension** (1.0.2):
  - Keystore metadata extraction
  - Test-only decryption with Suite 8 keys
  - Access rights and encryption parameters

- **Boolean Operations Extension** (1.1.1):
  - Recognition and validation

- **Displacement Extension** (1.0.0):
  - Displacement maps with PNG textures
  - NormVectorGroup (normalized displacement vectors)
  - Disp2DGroup (displacement coordinate groups)
  - Texture filtering and tiling modes

#### Mesh Operations
- Volume computation (signed and absolute)
- Axis-aligned bounding box (AABB) calculation
- Transformation support with transformed AABB
- Build volume computation
- Mesh subdivision utilities for displacement mapping
- Polygon clipping operations using Clipper2

#### Export Capabilities
- STL export for 3D printing
- OBJ export with MTL material files
- Thumbnail extraction

#### Developer Tools
- Comprehensive example suite (20+ examples)
- 3MF Viewer tool with interactive visualization
- Official 3MF Consortium conformance test support
- Performance benchmarks with criterion.rs
- Custom extension handler support

#### Viewer Features
- 3D model visualization with kiss3d
- Material and color display
- Build transformation visualization
- Model information panel
- Keyboard controls for navigation
- Live slice preview with 2D window
- Drag-and-drop file loading
- Texture rendering with UV mapping

### Testing
- 100% positive test compliance (1,719+ valid 3MF files)
- ~90% negative test compliance
- Overall ~97.6% conformance with official test suites
- Comprehensive unit and integration tests
- Property-based testing with `proptest`

### Performance
- Linear scaling with file size
- Memory-efficient streaming XML parsing
- Benchmarked performance:
  - Small files (1,000 vertices): ~1 ms
  - Medium files (10,000 vertices): ~7 ms
  - Large files (100,000 vertices): ~70 ms

### Dependencies
- `zip` - ZIP archive handling
- `quick-xml` - XML parsing
- `thiserror` - Error handling
- `parry3d` - Mesh geometric operations
- `nalgebra` - Linear algebra
- `clipper2` - Polygon clipping
- Cryptographic libraries for Secure Content extension

---

## Release Notes

### Version 0.1.0

This is the initial release of lib3mf_rust, providing a complete, safe, and production-ready Rust implementation of the 3MF format specification. The library has been validated against the official 3MF Consortium test suites with high conformance rates.

Key highlights:
- **Zero unsafe code** - Memory safety guaranteed by Rust
- **Extensive extension support** - All major 3MF extensions implemented
- **High conformance** - 97.6% compliance with official test suites
- **Production ready** - Battle-tested with comprehensive test coverage
- **Developer friendly** - Rich examples and documentation

---

[Unreleased]: https://github.com/telecos/lib3mf_rust/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/telecos/lib3mf_rust/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/telecos/lib3mf_rust/releases/tag/v0.1.0
