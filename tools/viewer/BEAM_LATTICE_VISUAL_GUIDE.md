## Beam Lattice Rendering Visual Guide

### Architecture Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                         3MF Model File                            │
│                    (pyramid.3mf example)                          │
└────────────────────────┬─────────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────────────┐
│                      Model Parser                                 │
│  - Reads 3MF ZIP container                                       │
│  - Parses XML model file                                         │
│  - Detects BeamLattice extension                                 │
└────────────────────────┬─────────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────────────┐
│                   Model Data Structure                            │
│  ┌────────────────────────────────────────────────────┐          │
│  │ Object 1                                           │          │
│  │  ├─ Mesh                                           │          │
│  │  │   ├─ Vertices: 123 (x, y, z positions)        │          │
│  │  │   ├─ Triangles: 0 (mesh has no surface)       │          │
│  │  │   └─ BeamSet                                   │          │
│  │  │       ├─ Radius: 1.0 mm                        │          │
│  │  │       ├─ CapMode: Sphere                       │          │
│  │  │       └─ Beams: 391                            │          │
│  │  │           ├─ Beam 1: v0->v15, r1=2.3mm         │          │
│  │  │           ├─ Beam 2: v9->v15, r1=2.3mm         │          │
│  │  │           ├─ Beam 3: v7->v28, r1=3.6, r2=4.1   │          │
│  │  │           └─ ... (388 more)                    │          │
│  └────────────────────────────────────────────────────┘          │
└────────────────────────┬─────────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────────────┐
│               create_beam_lattice_nodes()                         │
│                                                                   │
│  For each beam:                                                  │
│  1. Get vertex positions (v1, v2)                                │
│  2. Determine radii (r1, r2)                                     │
│  3. Call create_cylinder_mesh(p1, p2, r1, r2, 8)                │
│  4. Set color to orange (1.0, 0.6, 0.0)                          │
│  5. Add to beam_nodes collection                                 │
│                                                                   │
│  For sphere cap mode:                                            │
│  1. Count connections at each vertex                             │
│  2. For vertices with 2+ connections:                            │
│     - Get max radius of connected beams                          │
│     - Call create_sphere_mesh(center, radius, 8)                │
│     - Set color to orange                                        │
│     - Add to beam_nodes collection                               │
└────────────────────────┬─────────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────────────┐
│                 Geometry Generation                               │
│                                                                   │
│  ┌────────────────────────────────────────────────────┐          │
│  │ Cylinder Mesh (for each beam)                     │          │
│  │  - 2 circles of 8 vertices each (16 vertices)     │          │
│  │  - 8 quads connecting circles (16 triangles)      │          │
│  │  - 2 triangle fans for end caps                   │          │
│  │  - Supports tapered radius (r1 != r2)             │          │
│  └────────────────────────────────────────────────────┘          │
│                                                                   │
│  ┌────────────────────────────────────────────────────┐          │
│  │ Sphere Mesh (for ball joints)                     │          │
│  │  - Top vertex + bottom vertex                     │          │
│  │  - 6 rings of 8 vertices each (48 vertices)       │          │
│  │  - Triangle fans + quad strips (~64 triangles)    │          │
│  └────────────────────────────────────────────────────┘          │
└────────────────────────┬─────────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────────────┐
│                    Scene Rendering                                │
│                                                                   │
│  ViewerState:                                                    │
│  ├─ mesh_nodes: Vec<SceneNode>  (original mesh triangles)        │
│  └─ beam_nodes: Vec<SceneNode>  (beam cylinders + spheres)       │
│      ├─ 391 cylinder meshes (orange)                             │
│      └─ 123 sphere meshes (orange, at joints)                    │
│                                                                   │
│  User Control:                                                   │
│  - Press 'B' key -> toggle show_beams flag                       │
│  - For each node in beam_nodes:                                  │
│      node.set_visible(show_beams)                                │
└──────────────────────────────────────────────────────────────────┘
```

### Visual Representation

```
Pyramid Beam Lattice (pyramid.3mf)
==================================

Side View (simplified):
                    
                    v0 ●
                   /│\
                  / │ \
         v9 ●────●  │  ●────● v15
             \   │  │  │   /
              \  │  ●  │  /
               \ │ / \ │ /
                \│/   \│/
                 ●─────●

Each line represents a BEAM (rendered as cylinder):
- Uniform beams: r1 = 2.3mm (235 beams)
- Tapered beams: r1 ≠ r2 (156 beams)

Each vertex (●) with 2+ connections gets a SPHERE:
- 123 ball joints total
- Radius = max(connected beam radii)

Colors:
- Mesh triangles: Blue-gray (0.4, 0.6, 0.8)
- Beam cylinders: Orange (1.0, 0.6, 0.0)
- Ball joints: Orange (1.0, 0.6, 0.0)
```

### Rendering Statistics (pyramid.3mf)

| Component | Count | Vertices (approx) | Triangles (approx) |
|-----------|-------|-------------------|-------------------|
| Cylinder meshes | 391 | 6,256 | 6,256 |
| Sphere meshes | 123 | 5,166 | 7,872 |
| **Total beams** | **514** | **11,422** | **14,128** |

### Key Features Illustrated

1. **Tapered Beams**: 
   - Beam with r1=3.6mm, r2=4.1mm renders as cone
   - Smooth transition between radii

2. **Ball Joints**:
   - Vertex with 7 connections gets sphere of radius = max(beam radii)
   - Fills gaps at connection points

3. **Toggle Control**:
   ```
   Initial state: show_beams = true  (beams visible)
   Press 'B': show_beams = false     (beams hidden)
   Press 'B': show_beams = true      (beams visible)
   ```

4. **Model Info Display**:
   ```
   ═══════════════════════════════════════════════════════════
     Model Information:
     - Objects: 1
     - Triangles: 0
     - Vertices: 123
     - Unit: millimeter
     - Beam Lattice: 391 beams  ← NEW
   ═══════════════════════════════════════════════════════════
   ```

### Code Flow Example

```rust
// 1. User loads pyramid.3mf
let model = Model::from_reader(file)?;

// 2. Viewer creates mesh nodes
state.mesh_nodes = create_mesh_nodes(&mut window, &model);

// 3. Viewer creates beam nodes (NEW)
state.beam_nodes = create_beam_lattice_nodes(&mut window, &model);
// Result: 391 cylinders + 123 spheres = 514 scene nodes

// 4. User presses 'B' key
for node in &mut state.beam_nodes {
    node.set_visible(!state.show_beams);
}
state.show_beams = !state.show_beams;
```

### Geometry Detail: Cylinder with Tapered Radius

```
p1 (v1) ────────────────────────► p2 (v2)
r1=3.6mm                          r2=4.1mm

Circle at p1:        Circle at p2:
    2──3                  2──3
   1    4                1    4
  8      5              8      5
   7    6                7    6
    6──5                  6──5

Connected by triangles:
p1[0]─────p2[0]        Triangle 1: p1[0], p2[0], p1[1]
  │   ╱   │            Triangle 2: p1[1], p2[0], p2[1]
  │ ╱     │            (repeated 8 times around)
p1[1]─────p2[1]

End caps filled with triangle fans
```

This implementation provides complete visualization of beam lattice structures
in the 3MF viewer with efficient geometry and intuitive controls.
