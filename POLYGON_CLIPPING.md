# Polygon Clipping for Self-Intersection Resolution

This document describes the integration of polygon clipping functionality into lib3mf_rust for resolving self-intersections in slice polygons.

## Background

In C++ lib3mf, the [polyclipping library](https://github.com/bimpp/polyclipping/tree/master/cpp) (also known as Clipper) is used to resolve self-intersections in slice polygons. Self-intersections can occur during the slicing process in 3D manufacturing workflows and need to be resolved for proper manufacturing.

## Rust Implementation

Instead of porting the C++ polyclipping library, we use the **clipper2** Rust crate, which is a Rust wrapper around Clipper2 - the modern successor to the original Clipper library.

### Why Clipper2?

- **Modern API**: Clipper2 is the actively maintained successor to the original Clipper library
- **Robust**: Used in production for polygon boolean operations across many industries
- **Well-tested**: Extensively tested for numerical robustness and edge cases
- **Pure Rust wrapper**: The `clipper2` crate provides a safe Rust interface to the C++ library
- **Feature parity**: Provides all the functionality needed for 2D polygon operations

### Alternative Considered

We also evaluated **geo-clipper**, another Rust wrapper around the original Clipper library that integrates with the `geo-types` ecosystem. However, we chose `clipper2` because:
- It uses the newer Clipper2 engine
- Has a simpler API for our use case
- Doesn't require the full `geo-types` ecosystem

## Module Overview

The `polygon_clipping` module (`src/polygon_clipping.rs`) provides:

### Core Functions

1. **`resolve_self_intersections`**
   - Removes self-intersections from a single polygon
   - Uses Clipper2's simplification algorithm
   - May split a self-intersecting polygon into multiple simple polygons

2. **`union_polygons`**
   - Combines multiple polygons into a unified set
   - Merges overlapping areas
   - Useful for combining adjacent slice regions

3. **`intersect_polygons`**
   - Finds the overlapping areas between polygon sets
   - Returns only the intersection region(s)

4. **`difference_polygons`**
   - Subtracts one polygon set from another
   - Useful for creating holes or removing regions

### Data Conversion

The module handles conversion between lib3mf's slice polygon format and Clipper2's path format:

- **lib3mf format**: `SlicePolygon` with indexed vertices and segments
- **Clipper2 format**: `Vec<(f64, f64)>` representing polygon paths

## Usage Examples

### Basic Self-Intersection Resolution

```rust
use lib3mf::polygon_clipping::resolve_self_intersections;
use lib3mf::model::{SlicePolygon, SliceSegment, Vertex2D};

// Create a polygon with potential self-intersections
let vertices = vec![
    Vertex2D::new(0.0, 0.0),
    Vertex2D::new(100.0, 0.0),
    Vertex2D::new(100.0, 100.0),
    Vertex2D::new(0.0, 100.0),
];

let mut polygon = SlicePolygon::new(0);
polygon.segments.push(SliceSegment::new(1));
polygon.segments.push(SliceSegment::new(2));
polygon.segments.push(SliceSegment::new(3));

let mut result_vertices = Vec::new();
let simplified = resolve_self_intersections(
    &polygon, 
    &vertices, 
    &mut result_vertices
).expect("Failed to resolve self-intersections");

println!("Resolved to {} polygon(s)", simplified.len());
```

### Union of Overlapping Slice Regions

```rust
use lib3mf::polygon_clipping::union_polygons;

// Union multiple overlapping polygons
let mut result_vertices = Vec::new();
let unified = union_polygons(
    &[polygon1, polygon2, polygon3],
    &vertices,
    &mut result_vertices
).expect("Failed to union polygons");

// Result contains the unified polygon(s)
```

### Working with 3MF Slice Data

```rust
use lib3mf::Model;
use lib3mf::polygon_clipping::resolve_self_intersections;
use std::fs::File;

// Load a 3MF file with slice data
let file = File::open("model_with_slices.3mf")?;
let model = Model::from_reader(file)?;

// Process each slice stack
for slice_stack in &model.resources.slice_stacks {
    for slice in &slice_stack.slices {
        // Resolve self-intersections in each polygon
        for polygon in &slice.polygons {
            let mut result_vertices = Vec::new();
            let clean_polygons = resolve_self_intersections(
                polygon,
                &slice.vertices,
                &mut result_vertices
            )?;
            
            // Use clean_polygons for further processing
        }
    }
}
```

## Integration with 3MF Workflow

The polygon clipping functionality integrates into the 3MF workflow at several points:

1. **Slice Import**: When loading 3MF files with slice data, you can optionally clean polygons
2. **Slice Generation**: When generating slices from 3D meshes, use clipping to resolve intersections
3. **Slice Export**: Before writing slice data to 3MF, clean polygons to ensure validity

## Performance Considerations

- **Clipper2** uses integer arithmetic internally for robustness, converting from floating-point
- Operations are generally O(n log n) for n vertices
- For large slice stacks with many polygons, consider processing in parallel
- The `simplify` function removes nearly-collinear points with an epsilon tolerance of 0.01 units

## Testing

The module includes comprehensive tests for:
- Simple polygon resolution
- Union of overlapping squares
- Intersection of overlapping regions
- Difference operations
- Error handling for invalid vertex indices

Run tests with:
```bash
cargo test polygon_clipping
```

Run the example demonstration:
```bash
cargo run --example polygon_clipping_demo
```

## Error Handling

The module defines `ClippingError` for error cases:
- **InvalidPolygon**: Vertex indices out of bounds or malformed polygon data
- **ClipperError**: Internal Clipper2 operation failures

All operations return `Result<Vec<SlicePolygon>, ClippingError>` for proper error handling.

## Dependencies

- **clipper2** (v0.2.3): Core polygon clipping functionality
  - No known security vulnerabilities
  - Actively maintained
  - Safe Rust wrapper around proven C++ library

## Future Enhancements

Potential improvements:
- Batch processing of multiple slices in parallel
- Additional polygon operations (XOR, offset/inflation)
- Performance optimizations for very large slice stacks
- Integration with slice generation from 3D meshes

## References

- [Clipper2 Library](http://www.angusj.com/clipper2/Docs/Overview.htm)
- [clipper2 Rust Crate](https://crates.io/crates/clipper2)
- [3MF Slice Extension Specification](https://github.com/3MFConsortium/spec_slice)
- [Original Polyclipping (C++)](https://github.com/bimpp/polyclipping)
