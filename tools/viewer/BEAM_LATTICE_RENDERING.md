# Beam Lattice Rendering in 3MF Viewer

## Overview

This document describes the beam lattice rendering implementation in the lib3mf viewer. The viewer now supports visualization of beam lattice structures from the 3MF Beam Lattice extension.

## Features Implemented

### 1. Beam Detection and Rendering

The viewer automatically detects beam lattice data in loaded 3MF models and renders them as 3D cylinders:

- **Beam Cylinders**: Each beam is rendered as a 3D cylinder connecting two vertices
- **Tapered Beams**: Supports beams with different radii at each end (r1, r2)
- **Distinct Color**: Beams are rendered in orange (RGB: 1.0, 0.6, 0.0) to distinguish from mesh geometry
- **Segment Count**: Uses 8 segments around cylinder circumference for efficient rendering

### 2. Ball Joints

The viewer supports both explicit balls (from the balls extension) and inferred balls:

**Explicit Balls (Balls Extension)**:
- Renders balls explicitly defined in `<bb:balls>` elements
- Uses ball-specific radius or falls back to `ball_radius` or beamset default
- Takes precedence over inferred ball joints
- Supports per-ball properties (property_id, property_index)

**Inferred Ball Joints (Sphere Cap Mode)**:
- When no explicit balls are defined and cap mode is sphere
- Identifies vertices with 2+ beam connections
- Adaptive Radius: Uses the maximum radius of connected beams
- Ball Meshes: Generates sphere meshes with 8 segments

### 3. User Controls

- **Toggle Visibility**: Press 'B' key to show/hide beam lattice structures
- **Model Info**: Beam count is displayed in the model information panel
- **Visual Feedback**: Console output confirms beam visibility state

## Implementation Details

### Geometry Generation

#### Cylinder Mesh Creation

```rust
fn create_cylinder_mesh(p1: Point3<f32>, p2: Point3<f32>, r1: f32, r2: f32, segments: u32) -> TriMesh<f32>
```

- Calculates cylinder axis and perpendicular vectors
- Generates circle vertices at both ends
- Connects circles with triangle strips
- Adds end caps for non-zero radii
- Handles degenerate cases (zero-length beams)

#### Sphere Mesh Creation

```rust
fn create_sphere_mesh(center: Point3<f32>, radius: f32, segments: u32) -> TriMesh<f32>
```

- Generates sphere using latitude/longitude rings
- Creates triangle fan for top and bottom caps
- Connects rings with quad strips (split into triangles)

### Beam Lattice Node Creation

```rust
fn create_beam_lattice_nodes(window: &mut Window, model: &Model) -> Vec<SceneNode>
```

For each object with beam lattice data:

1. **Process Beams**:
   - Extract vertex positions (v1, v2)
   - Determine radii (use beam-specific r1/r2 or beamset default)
   - Generate cylinder mesh
   - Apply orange color
   - Add to scene

2. **Process Explicit Balls** (from Balls Extension):
   - Render all balls defined in `beamset.balls` vector
   - Use ball's radius, or fall back to beamset.ball_radius or beamset.radius
   - Apply orange color
   - Add to scene

3. **Process Inferred Ball Joints** (if no explicit balls and cap_mode == Sphere):
   - Count connections at each vertex
   - Generate spheres at vertices with 2+ connections
   - Use maximum radius of connected beams
   - Add to scene

## Usage Example

```bash
# Build and run viewer with beam lattice file
cd tools/viewer
cargo run --release -- --ui ../../test_files/beam_lattice/pyramid.3mf
```

### Controls

- **Left Mouse + Drag**: Rotate view
- **Right Mouse + Drag**: Pan view
- **Scroll Wheel**: Zoom
- **B**: Toggle beam lattice visibility
- **Ctrl+O**: Open new file
- **ESC**: Exit viewer

## Test Files

The implementation has been tested with:

- `test_files/beam_lattice/pyramid.3mf`: 391 beams, including 156 tapered beams
  - Default radius: 1.0 mm
  - Cap mode: Sphere
  - 123 vertices with multiple connections

## Performance Considerations

- **Segment Count**: Fixed at 8 segments for good balance of quality/performance
- **Geometry Caching**: Scene nodes are created once per file load
- **Efficient Structures**: Uses Vec for beams and nodes to minimize allocations

## Acceptance Criteria Status

- ✅ Beam lattice structures render as 3D cylinders
- ✅ Beam radius is respected (including tapered beams with r1 != r2)
- ✅ End caps render correctly (sphere mode implemented)
- ✅ Toggle to show/hide beam lattice (B key)
- ✅ Distinct visual styling from mesh geometry (orange color)
- ✅ Reasonable performance with many beams (tested with 391 beams)

## Future Enhancements

Potential improvements for future development:

1. **LOD (Level of Detail)**: Reduce segment count for distant beams
2. **Hemisphere/Butt Caps**: Full implementation of all cap modes
3. **Wireframe Mode**: Add wireframe rendering option for beams
4. **Transparency**: Add opacity control for beams
5. **Performance**: Frustum culling for beams outside view
6. **Color Customization**: Allow custom beam colors
7. **Clipping Support**: Implement clipping mesh visualization

## Code Structure

The beam lattice rendering code is located in:

- `tools/viewer/src/ui_viewer.rs`:
  - `create_cylinder_mesh()`: Generates cylinder geometry
  - `create_sphere_mesh()`: Generates sphere geometry
  - `create_beam_lattice_nodes()`: Creates scene nodes for beams
  - `count_beams()`: Utility to count total beams in model
  - `ViewerState`: Extended with `beam_nodes` and `show_beams` fields

## Dependencies

The beam lattice rendering uses:

- `kiss3d`: 3D rendering engine
- `nalgebra`: Vector/point math (via kiss3d)
- `lib3mf`: 3MF model parsing with beam lattice extension support
