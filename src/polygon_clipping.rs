//! Polygon clipping and self-intersection resolution for slices
//!
//! This module provides polygon clipping operations using the Clipper2 library,
//! which is a Rust port of the Angus Johnson's Clipper2 library (the successor
//! to the polyclipping library used in the C++ lib3mf implementation).
//!
//! Clipper2 provides robust polygon boolean operations including:
//! - Union: Combine multiple polygons
//! - Intersection: Find overlapping areas
//! - Difference: Subtract one polygon from another
//! - XOR: Symmetric difference
//! - Simplification: Remove self-intersections and degenerate vertices
//!
//! These operations are essential for resolving self-intersections in slice polygons,
//! which can occur during the slicing process in 3D manufacturing workflows.

use crate::model::{SlicePolygon, Vertex2D};
use clipper2::*;

/// Error type for polygon clipping operations
#[derive(Debug, thiserror::Error)]
pub enum ClippingError {
    /// Invalid polygon data
    #[error("Invalid polygon: {0}")]
    InvalidPolygon(String),

    /// Clipper operation failed
    #[error("Clipper operation failed: {0}")]
    ClipperError(String),
}

/// Convert slice polygons to Clipper2 paths
///
/// Converts the lib3mf slice polygon format (vertex indices with segments)
/// into Clipper2's path format (Vec<(f64, f64)>).
fn slice_polygon_to_paths(
    polygon: &SlicePolygon,
    vertices: &[Vertex2D],
) -> Result<Vec<(f64, f64)>, ClippingError> {
    let mut path = Vec::new();

    // Add starting vertex
    if polygon.startv >= vertices.len() {
        return Err(ClippingError::InvalidPolygon(format!(
            "Start vertex index {} out of bounds",
            polygon.startv
        )));
    }

    let start_vertex = &vertices[polygon.startv];
    path.push((start_vertex.x, start_vertex.y));

    // Add vertices from segments
    for segment in &polygon.segments {
        if segment.v2 >= vertices.len() {
            return Err(ClippingError::InvalidPolygon(format!(
                "Segment vertex index {} out of bounds",
                segment.v2
            )));
        }

        let vertex = &vertices[segment.v2];
        path.push((vertex.x, vertex.y));
    }

    Ok(path)
}

/// Convert Clipper2 paths back to slice polygons
///
/// Converts Clipper2's path format back into lib3mf slice polygon format.
/// This creates new vertices and polygon structures.
fn paths_to_slice_polygons(
    paths: Vec<Vec<(f64, f64)>>,
    vertices: &mut Vec<Vertex2D>,
) -> Vec<SlicePolygon> {
    let mut polygons = Vec::new();

    for path in paths {
        if path.len() < 3 {
            // Skip degenerate polygons (need at least 3 vertices)
            continue;
        }

        let start_idx = vertices.len();

        // Add the first vertex and create polygon
        let first_point = path[0];
        vertices.push(Vertex2D::new(first_point.0, first_point.1));

        let mut polygon = SlicePolygon::new(start_idx);

        // Add remaining vertices as segments
        for point in path.iter().skip(1) {
            let vertex_idx = vertices.len();
            vertices.push(Vertex2D::new(point.0, point.1));
            polygon
                .segments
                .push(crate::model::SliceSegment::new(vertex_idx));
        }

        polygons.push(polygon);
    }

    polygons
}

/// Resolve self-intersections in a slice polygon
///
/// Uses Clipper2's simplify operation to remove self-intersections and
/// degenerate vertices from a polygon. This is useful for cleaning up
/// slice data that may have been generated with numerical errors or
/// topology issues.
///
/// # Arguments
///
/// * `polygon` - The slice polygon to simplify
/// * `vertices` - The vertex buffer containing the polygon's vertices
/// * `result_vertices` - Output vertex buffer for the simplified polygon(s)
///
/// # Returns
///
/// A vector of simplified polygons. Note that resolving self-intersections
/// may result in multiple output polygons if the self-intersection divides
/// the polygon into separate regions.
///
/// # Example
///
/// ```ignore
/// use lib3mf::polygon_clipping::resolve_self_intersections;
/// use lib3mf::model::{SlicePolygon, Vertex2D};
///
/// let polygon = SlicePolygon::new(0);
/// let vertices = vec![
///     Vertex2D::new(0.0, 0.0),
///     Vertex2D::new(10.0, 0.0),
///     Vertex2D::new(10.0, 10.0),
///     Vertex2D::new(0.0, 10.0),
/// ];
/// let mut result_vertices = Vec::new();
///
/// let simplified = resolve_self_intersections(&polygon, &vertices, &mut result_vertices)
///     .expect("Failed to resolve self-intersections");
/// ```
pub fn resolve_self_intersections(
    polygon: &SlicePolygon,
    vertices: &[Vertex2D],
    result_vertices: &mut Vec<Vertex2D>,
) -> Result<Vec<SlicePolygon>, ClippingError> {
    // Convert to Clipper2 path
    let path = slice_polygon_to_paths(polygon, vertices)?;

    // Simplify the polygon to remove self-intersections and nearly collinear points
    // epsilon=0.01 removes points within 0.01 units of a line
    // is_open=false because slice polygons are closed
    let simplified = simplify::<Centi>(vec![path], 0.01, false);

    // Convert back to slice polygons
    let result: Vec<Vec<(f64, f64)>> = simplified.into();
    Ok(paths_to_slice_polygons(result, result_vertices))
}

/// Perform union operation on multiple slice polygons
///
/// Combines multiple polygons into a single unified set of polygons,
/// merging overlapping areas.
///
/// # Arguments
///
/// * `polygons` - The slice polygons to union
/// * `vertices` - The vertex buffer containing the polygons' vertices
/// * `result_vertices` - Output vertex buffer for the result polygon(s)
///
/// # Returns
///
/// A vector of polygons representing the union of all input polygons.
pub fn union_polygons(
    polygons: &[SlicePolygon],
    vertices: &[Vertex2D],
    result_vertices: &mut Vec<Vertex2D>,
) -> Result<Vec<SlicePolygon>, ClippingError> {
    if polygons.is_empty() {
        return Ok(Vec::new());
    }

    // Convert all polygons to paths
    let mut all_paths = Vec::new();
    for polygon in polygons {
        all_paths.push(slice_polygon_to_paths(polygon, vertices)?);
    }

    // If only one polygon, simplify it and return
    if all_paths.len() == 1 {
        let simplified = simplify::<Centi>(all_paths, 0.01, false);
        let result_paths: Vec<Vec<(f64, f64)>> = simplified.into();
        return Ok(paths_to_slice_polygons(result_paths, result_vertices));
    }

    // Perform union operation on multiple polygons
    let first_path = all_paths.remove(0);
    let result = union::<Centi>(vec![first_path], all_paths, FillRule::default())
        .map_err(|e| ClippingError::ClipperError(format!("{:?}", e)))?;

    // Convert back to slice polygons
    let result_paths: Vec<Vec<(f64, f64)>> = result.into();
    Ok(paths_to_slice_polygons(result_paths, result_vertices))
}

/// Perform intersection operation on two sets of slice polygons
///
/// Finds the overlapping areas between two sets of polygons.
///
/// # Arguments
///
/// * `subject_polygons` - The first set of polygons
/// * `clip_polygons` - The second set of polygons
/// * `vertices` - The vertex buffer containing the polygons' vertices
/// * `result_vertices` - Output vertex buffer for the result polygon(s)
///
/// # Returns
///
/// A vector of polygons representing the intersection of the two sets.
pub fn intersect_polygons(
    subject_polygons: &[SlicePolygon],
    clip_polygons: &[SlicePolygon],
    vertices: &[Vertex2D],
    result_vertices: &mut Vec<Vertex2D>,
) -> Result<Vec<SlicePolygon>, ClippingError> {
    if subject_polygons.is_empty() || clip_polygons.is_empty() {
        return Ok(Vec::new());
    }

    // Convert subject polygons to paths
    let mut subject_paths = Vec::new();
    for polygon in subject_polygons {
        subject_paths.push(slice_polygon_to_paths(polygon, vertices)?);
    }

    // Convert clip polygons to paths
    let mut clip_paths = Vec::new();
    for polygon in clip_polygons {
        clip_paths.push(slice_polygon_to_paths(polygon, vertices)?);
    }

    // Perform intersection operation
    let result = intersect::<Centi>(subject_paths, clip_paths, FillRule::default())
        .map_err(|e| ClippingError::ClipperError(format!("{:?}", e)))?;

    // Convert back to slice polygons
    let result_paths: Vec<Vec<(f64, f64)>> = result.into();
    Ok(paths_to_slice_polygons(result_paths, result_vertices))
}

/// Perform difference operation on two sets of slice polygons
///
/// Subtracts the clip polygons from the subject polygons.
///
/// # Arguments
///
/// * `subject_polygons` - The polygons to subtract from
/// * `clip_polygons` - The polygons to subtract
/// * `vertices` - The vertex buffer containing the polygons' vertices
/// * `result_vertices` - Output vertex buffer for the result polygon(s)
///
/// # Returns
///
/// A vector of polygons representing the difference.
pub fn difference_polygons(
    subject_polygons: &[SlicePolygon],
    clip_polygons: &[SlicePolygon],
    vertices: &[Vertex2D],
    result_vertices: &mut Vec<Vertex2D>,
) -> Result<Vec<SlicePolygon>, ClippingError> {
    if subject_polygons.is_empty() {
        return Ok(Vec::new());
    }

    // Convert subject polygons to paths
    let mut subject_paths = Vec::new();
    for polygon in subject_polygons {
        subject_paths.push(slice_polygon_to_paths(polygon, vertices)?);
    }

    if clip_polygons.is_empty() {
        // Nothing to subtract, simplify and return subject
        let simplified = simplify::<Centi>(subject_paths, 0.01, false);
        let result_paths: Vec<Vec<(f64, f64)>> = simplified.into();
        return Ok(paths_to_slice_polygons(result_paths, result_vertices));
    }

    // Convert clip polygons to paths
    let mut clip_paths = Vec::new();
    for polygon in clip_polygons {
        clip_paths.push(slice_polygon_to_paths(polygon, vertices)?);
    }

    // Perform difference operation
    let result = difference::<Centi>(subject_paths, clip_paths, FillRule::default())
        .map_err(|e| ClippingError::ClipperError(format!("{:?}", e)))?;

    // Convert back to slice polygons
    let result_paths: Vec<Vec<(f64, f64)>> = result.into();
    Ok(paths_to_slice_polygons(result_paths, result_vertices))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SliceSegment;

    #[test]
    fn test_resolve_simple_polygon() {
        // Create a simple square polygon without self-intersections
        let vertices = vec![
            Vertex2D::new(0.0, 0.0),
            Vertex2D::new(10.0, 0.0),
            Vertex2D::new(10.0, 10.0),
            Vertex2D::new(0.0, 10.0),
        ];

        let mut polygon = SlicePolygon::new(0);
        polygon.segments.push(SliceSegment::new(1));
        polygon.segments.push(SliceSegment::new(2));
        polygon.segments.push(SliceSegment::new(3));

        let mut result_vertices = Vec::new();
        let result = resolve_self_intersections(&polygon, &vertices, &mut result_vertices)
            .expect("Failed to resolve polygon");

        // Should return a single polygon
        assert_eq!(result.len(), 1, "Expected one polygon");
        assert!(!result_vertices.is_empty(), "Expected vertices in result");
    }

    #[test]
    fn test_union_two_squares() {
        // Create two overlapping squares
        let vertices = vec![
            // First square (0,0) to (10,10)
            Vertex2D::new(0.0, 0.0),
            Vertex2D::new(10.0, 0.0),
            Vertex2D::new(10.0, 10.0),
            Vertex2D::new(0.0, 10.0),
            // Second square (5,5) to (15,15)
            Vertex2D::new(5.0, 5.0),
            Vertex2D::new(15.0, 5.0),
            Vertex2D::new(15.0, 15.0),
            Vertex2D::new(5.0, 15.0),
        ];

        let mut polygon1 = SlicePolygon::new(0);
        polygon1.segments.push(SliceSegment::new(1));
        polygon1.segments.push(SliceSegment::new(2));
        polygon1.segments.push(SliceSegment::new(3));

        let mut polygon2 = SlicePolygon::new(4);
        polygon2.segments.push(SliceSegment::new(5));
        polygon2.segments.push(SliceSegment::new(6));
        polygon2.segments.push(SliceSegment::new(7));

        let mut result_vertices = Vec::new();
        let result = union_polygons(&[polygon1, polygon2], &vertices, &mut result_vertices)
            .expect("Failed to union polygons");

        // Should return at least one polygon (the union)
        assert!(!result.is_empty(), "Expected at least one polygon in union");
        assert!(!result_vertices.is_empty(), "Expected vertices in result");
    }

    #[test]
    fn test_intersection_two_squares() {
        // Create two overlapping squares
        let vertices = vec![
            // First square (0,0) to (10,10)
            Vertex2D::new(0.0, 0.0),
            Vertex2D::new(10.0, 0.0),
            Vertex2D::new(10.0, 10.0),
            Vertex2D::new(0.0, 10.0),
            // Second square (5,5) to (15,15)
            Vertex2D::new(5.0, 5.0),
            Vertex2D::new(15.0, 5.0),
            Vertex2D::new(15.0, 15.0),
            Vertex2D::new(5.0, 15.0),
        ];

        let mut polygon1 = SlicePolygon::new(0);
        polygon1.segments.push(SliceSegment::new(1));
        polygon1.segments.push(SliceSegment::new(2));
        polygon1.segments.push(SliceSegment::new(3));

        let mut polygon2 = SlicePolygon::new(4);
        polygon2.segments.push(SliceSegment::new(5));
        polygon2.segments.push(SliceSegment::new(6));
        polygon2.segments.push(SliceSegment::new(7));

        let mut result_vertices = Vec::new();
        let result = intersect_polygons(&[polygon1], &[polygon2], &vertices, &mut result_vertices)
            .expect("Failed to intersect polygons");

        // Should return the overlapping region (5,5) to (10,10)
        assert!(
            !result.is_empty(),
            "Expected at least one polygon in intersection"
        );
    }

    #[test]
    fn test_difference_two_squares() {
        // Create two overlapping squares
        let vertices = vec![
            // First square (0,0) to (10,10)
            Vertex2D::new(0.0, 0.0),
            Vertex2D::new(10.0, 0.0),
            Vertex2D::new(10.0, 10.0),
            Vertex2D::new(0.0, 10.0),
            // Second square (5,5) to (15,15)
            Vertex2D::new(5.0, 5.0),
            Vertex2D::new(15.0, 5.0),
            Vertex2D::new(15.0, 15.0),
            Vertex2D::new(5.0, 15.0),
        ];

        let mut polygon1 = SlicePolygon::new(0);
        polygon1.segments.push(SliceSegment::new(1));
        polygon1.segments.push(SliceSegment::new(2));
        polygon1.segments.push(SliceSegment::new(3));

        let mut polygon2 = SlicePolygon::new(4);
        polygon2.segments.push(SliceSegment::new(5));
        polygon2.segments.push(SliceSegment::new(6));
        polygon2.segments.push(SliceSegment::new(7));

        let mut result_vertices = Vec::new();
        let result = difference_polygons(&[polygon1], &[polygon2], &vertices, &mut result_vertices)
            .expect("Failed to compute difference");

        // Should return the first square minus the overlapping region
        assert!(
            !result.is_empty(),
            "Expected at least one polygon in difference"
        );
    }

    #[test]
    fn test_invalid_vertex_index() {
        // Create a polygon with an out-of-bounds vertex index
        let vertices = vec![Vertex2D::new(0.0, 0.0), Vertex2D::new(10.0, 0.0)];

        let mut polygon = SlicePolygon::new(0);
        polygon.segments.push(SliceSegment::new(1));
        polygon.segments.push(SliceSegment::new(5)); // Out of bounds!

        let mut result_vertices = Vec::new();
        let result = resolve_self_intersections(&polygon, &vertices, &mut result_vertices);

        // Should return an error
        assert!(result.is_err(), "Expected error for out-of-bounds vertex");
    }
}
