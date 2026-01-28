//! Polygon triangulation for filled 2D rendering
//!
//! This module provides polygon triangulation utilities to convert 2D polygon contours
//! (with or without holes) into triangles for filled rendering. This is essential for:
//! - Graphics APIs that render triangles, not arbitrary polygons
//! - Handling polygons with holes correctly
//! - Rasterization with the `image` crate
//!
//! The module uses the `earcutr` library, which is a Rust port of the well-tested
//! earcut.js polygon triangulation library from MapBox.

use crate::model::Vertex2D;

/// Error type for polygon triangulation operations
#[derive(Debug, thiserror::Error)]
pub enum TriangulationError {
    /// Polygon has too few vertices to triangulate
    #[error("Polygon has too few vertices: {0} (minimum 3 required)")]
    TooFewVertices(usize),

    /// Invalid hole specification
    #[error("Invalid hole: {0}")]
    InvalidHole(String),

    /// Triangulation failed
    #[error("Triangulation failed: {0}")]
    TriangulationFailed(String),
}

/// Triangulate a simple polygon without holes
///
/// Converts a 2D polygon into a set of triangles represented as indices into
/// the vertex array. The polygon should be specified as a sequence of vertices
/// in counter-clockwise order.
///
/// # Arguments
///
/// * `polygon` - Array of 2D vertices forming the polygon boundary
///
/// # Returns
///
/// A vector of triangle indices, where each consecutive triplet of indices
/// represents one triangle. For example, `[0, 1, 2, 1, 3, 2]` represents
/// two triangles: (0,1,2) and (1,3,2).
///
/// # Errors
///
/// Returns an error if:
/// - The polygon has fewer than 3 vertices
/// - The triangulation algorithm fails
///
/// # Example
///
/// ```
/// use lib3mf::model::Vertex2D;
/// use lib3mf::polygon_triangulation::triangulate_simple;
///
/// // Create a simple square
/// let vertices = vec![
///     Vertex2D::new(0.0, 0.0),
///     Vertex2D::new(10.0, 0.0),
///     Vertex2D::new(10.0, 10.0),
///     Vertex2D::new(0.0, 10.0),
/// ];
///
/// let triangles = triangulate_simple(&vertices)
///     .expect("Failed to triangulate");
///
/// assert_eq!(triangles.len(), 6); // 2 triangles × 3 indices
/// ```
pub fn triangulate_simple(polygon: &[Vertex2D]) -> Result<Vec<usize>, TriangulationError> {
    if polygon.len() < 3 {
        return Err(TriangulationError::TooFewVertices(polygon.len()));
    }

    // Convert vertices to flat coordinate array [x0, y0, x1, y1, ...]
    let mut coords = Vec::with_capacity(polygon.len() * 2);
    for vertex in polygon {
        coords.push(vertex.x);
        coords.push(vertex.y);
    }

    // No holes for simple polygon
    let hole_indices: Vec<usize> = Vec::new();

    // Perform triangulation using earcutr
    // The third parameter (2) indicates 2D coordinates (x, y pairs)
    let result = earcutr::earcut(&coords, &hole_indices, 2)
        .map_err(|e| TriangulationError::TriangulationFailed(format!("Earcut error: {}", e)))?;

    if result.is_empty() && polygon.len() >= 3 {
        return Err(TriangulationError::TriangulationFailed(
            "Earcut returned no triangles".to_string(),
        ));
    }

    Ok(result)
}

/// Triangulate a polygon with holes
///
/// Converts a 2D polygon with interior holes into a set of triangles.
/// The outer boundary should be specified in counter-clockwise order,
/// and holes should be specified in clockwise order.
///
/// # Arguments
///
/// * `outer` - Array of 2D vertices forming the outer polygon boundary
/// * `holes` - Array of hole polygons, where each hole is an array of vertices
///
/// # Returns
///
/// A vector of triangle indices referring to vertices in the combined vertex array.
/// The outer polygon vertices come first, followed by hole vertices in order.
/// For example, if outer has 4 vertices and holes\[0\] has 3 vertices, then:
/// - Indices 0-3 refer to outer vertices
/// - Indices 4-6 refer to first hole vertices
///
/// # Errors
///
/// Returns an error if:
/// - The outer polygon has fewer than 3 vertices
/// - Any hole has fewer than 3 vertices
/// - The triangulation algorithm fails
///
/// # Example
///
/// ```
/// use lib3mf::model::Vertex2D;
/// use lib3mf::polygon_triangulation::triangulate_with_holes;
///
/// // Create a square with a triangular hole
/// let outer = vec![
///     Vertex2D::new(0.0, 0.0),
///     Vertex2D::new(100.0, 0.0),
///     Vertex2D::new(100.0, 100.0),
///     Vertex2D::new(0.0, 100.0),
/// ];
///
/// let hole = vec![
///     Vertex2D::new(25.0, 25.0),
///     Vertex2D::new(75.0, 25.0),
///     Vertex2D::new(50.0, 75.0),
/// ];
///
/// let triangles = triangulate_with_holes(&outer, &[hole])
///     .expect("Failed to triangulate");
///
/// // Should produce multiple triangles that avoid the hole
/// assert!(!triangles.is_empty());
/// ```
pub fn triangulate_with_holes(
    outer: &[Vertex2D],
    holes: &[Vec<Vertex2D>],
) -> Result<Vec<usize>, TriangulationError> {
    if outer.len() < 3 {
        return Err(TriangulationError::TooFewVertices(outer.len()));
    }

    // Validate holes
    for (i, hole) in holes.iter().enumerate() {
        if hole.len() < 3 {
            return Err(TriangulationError::InvalidHole(format!(
                "Hole {} has only {} vertices (minimum 3 required)",
                i,
                hole.len()
            )));
        }
    }

    // Convert vertices to flat coordinate array
    // Format: [outer_x0, outer_y0, ..., hole1_x0, hole1_y0, ..., hole2_x0, ...]
    let mut coords =
        Vec::with_capacity((outer.len() + holes.iter().map(|h| h.len()).sum::<usize>()) * 2);

    // Add outer boundary
    for vertex in outer {
        coords.push(vertex.x);
        coords.push(vertex.y);
    }

    // Track where each hole starts in the vertex list
    let mut hole_indices = Vec::with_capacity(holes.len());
    let mut current_index = outer.len();

    // Add holes
    for hole in holes {
        hole_indices.push(current_index);
        for vertex in hole {
            coords.push(vertex.x);
            coords.push(vertex.y);
        }
        current_index += hole.len();
    }

    // Perform triangulation using earcutr
    let result = earcutr::earcut(&coords, &hole_indices, 2)
        .map_err(|e| TriangulationError::TriangulationFailed(format!("Earcut error: {}", e)))?;

    if result.is_empty() && outer.len() >= 3 {
        return Err(TriangulationError::TriangulationFailed(
            "Earcut returned no triangles".to_string(),
        ));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangulate_simple_square() {
        // Create a simple square
        let vertices = vec![
            Vertex2D::new(0.0, 0.0),
            Vertex2D::new(10.0, 0.0),
            Vertex2D::new(10.0, 10.0),
            Vertex2D::new(0.0, 10.0),
        ];

        let triangles = triangulate_simple(&vertices).expect("Failed to triangulate square");

        // A square should produce 2 triangles (6 indices)
        assert_eq!(triangles.len(), 6, "Expected 2 triangles (6 indices)");

        // Verify all indices are valid (0-3 for 4 vertices)
        for &idx in &triangles {
            assert!(idx < 4, "Triangle index {} out of bounds", idx);
        }
    }

    #[test]
    fn test_triangulate_simple_triangle() {
        // Create a simple triangle (minimum valid polygon)
        let vertices = vec![
            Vertex2D::new(0.0, 0.0),
            Vertex2D::new(10.0, 0.0),
            Vertex2D::new(5.0, 10.0),
        ];

        let triangles = triangulate_simple(&vertices).expect("Failed to triangulate triangle");

        // A triangle should produce 1 triangle (3 indices)
        assert_eq!(triangles.len(), 3, "Expected 1 triangle (3 indices)");

        // Verify all indices are valid (0-2 for 3 vertices)
        for &idx in &triangles {
            assert!(idx < 3, "Triangle index {} out of bounds", idx);
        }
    }

    #[test]
    fn test_triangulate_simple_pentagon() {
        // Create a regular pentagon
        let vertices = vec![
            Vertex2D::new(0.0, 5.0),
            Vertex2D::new(4.75, 1.54),
            Vertex2D::new(2.94, -4.05),
            Vertex2D::new(-2.94, -4.05),
            Vertex2D::new(-4.75, 1.54),
        ];

        let triangles = triangulate_simple(&vertices).expect("Failed to triangulate pentagon");

        // A pentagon should produce 3 triangles (9 indices)
        assert_eq!(triangles.len(), 9, "Expected 3 triangles (9 indices)");

        // Verify all indices are valid
        for &idx in &triangles {
            assert!(idx < 5, "Triangle index {} out of bounds", idx);
        }
    }

    #[test]
    fn test_triangulate_too_few_vertices() {
        // Try with 2 vertices (invalid)
        let vertices = vec![Vertex2D::new(0.0, 0.0), Vertex2D::new(10.0, 0.0)];

        let result = triangulate_simple(&vertices);
        assert!(result.is_err(), "Should fail with too few vertices");

        match result {
            Err(TriangulationError::TooFewVertices(n)) => {
                assert_eq!(n, 2);
            }
            _ => panic!("Expected TooFewVertices error"),
        }
    }

    #[test]
    fn test_triangulate_with_one_hole() {
        // Create a square with a triangular hole
        let outer = vec![
            Vertex2D::new(0.0, 0.0),
            Vertex2D::new(100.0, 0.0),
            Vertex2D::new(100.0, 100.0),
            Vertex2D::new(0.0, 100.0),
        ];

        // Clockwise hole in the center
        let hole = vec![
            Vertex2D::new(25.0, 25.0),
            Vertex2D::new(75.0, 25.0),
            Vertex2D::new(50.0, 75.0),
        ];

        let triangles = triangulate_with_holes(&outer, &[hole])
            .expect("Failed to triangulate polygon with hole");

        // Should produce multiple triangles
        assert!(!triangles.is_empty(), "Expected at least one triangle");

        // All indices should be valid (0-6: 4 outer + 3 hole vertices)
        for &idx in &triangles {
            assert!(idx < 7, "Triangle index {} out of bounds", idx);
        }

        // Number of indices should be divisible by 3
        assert_eq!(
            triangles.len() % 3,
            0,
            "Number of indices should be divisible by 3"
        );
    }

    #[test]
    fn test_triangulate_with_multiple_holes() {
        // Create a large square with two smaller square holes
        let outer = vec![
            Vertex2D::new(0.0, 0.0),
            Vertex2D::new(100.0, 0.0),
            Vertex2D::new(100.0, 100.0),
            Vertex2D::new(0.0, 100.0),
        ];

        // First hole (left side)
        let hole1 = vec![
            Vertex2D::new(10.0, 10.0),
            Vertex2D::new(30.0, 10.0),
            Vertex2D::new(30.0, 30.0),
            Vertex2D::new(10.0, 30.0),
        ];

        // Second hole (right side)
        let hole2 = vec![
            Vertex2D::new(70.0, 70.0),
            Vertex2D::new(90.0, 70.0),
            Vertex2D::new(90.0, 90.0),
            Vertex2D::new(70.0, 90.0),
        ];

        let triangles = triangulate_with_holes(&outer, &[hole1, hole2])
            .expect("Failed to triangulate polygon with two holes");

        // Should produce multiple triangles
        assert!(!triangles.is_empty(), "Expected at least one triangle");

        // All indices should be valid (0-11: 4 outer + 4 hole1 + 4 hole2 vertices)
        for &idx in &triangles {
            assert!(idx < 12, "Triangle index {} out of bounds", idx);
        }

        // Number of indices should be divisible by 3
        assert_eq!(
            triangles.len() % 3,
            0,
            "Number of indices should be divisible by 3"
        );
    }

    #[test]
    fn test_triangulate_invalid_hole() {
        let outer = vec![
            Vertex2D::new(0.0, 0.0),
            Vertex2D::new(100.0, 0.0),
            Vertex2D::new(100.0, 100.0),
            Vertex2D::new(0.0, 100.0),
        ];

        // Invalid hole with only 2 vertices
        let invalid_hole = vec![Vertex2D::new(25.0, 25.0), Vertex2D::new(75.0, 25.0)];

        let result = triangulate_with_holes(&outer, &[invalid_hole]);
        assert!(result.is_err(), "Should fail with invalid hole");

        match result {
            Err(TriangulationError::InvalidHole(msg)) => {
                assert!(msg.contains("Hole 0"));
            }
            _ => panic!("Expected InvalidHole error"),
        }
    }

    #[test]
    fn test_triangulate_empty_outer() {
        let outer = vec![];
        let result = triangulate_simple(&outer);
        assert!(result.is_err(), "Should fail with empty polygon");
    }
}
