//! Integration tests for mesh slicing functionality
//!
//! These tests verify the complete mesh-plane slicing workflow:
//! - Triangle-plane intersection
//! - Segment collection
//! - Contour assembly

use lib3mf::{
    Mesh, Triangle, Vertex, assemble_contours, collect_intersection_segments,
    triangle_plane_intersection,
};

#[test]
fn test_slice_simple_cube() {
    let mut mesh = Mesh::new();

    // Create a simple 10x10x10 cube
    // Bottom vertices (Z=0)
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0)); // 0
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0)); // 1
    mesh.vertices.push(Vertex::new(10.0, 10.0, 0.0)); // 2
    mesh.vertices.push(Vertex::new(0.0, 10.0, 0.0)); // 3

    // Top vertices (Z=10)
    mesh.vertices.push(Vertex::new(0.0, 0.0, 10.0)); // 4
    mesh.vertices.push(Vertex::new(10.0, 0.0, 10.0)); // 5
    mesh.vertices.push(Vertex::new(10.0, 10.0, 10.0)); // 6
    mesh.vertices.push(Vertex::new(0.0, 10.0, 10.0)); // 7

    // Bottom face
    mesh.triangles.push(Triangle::new(0, 2, 1));
    mesh.triangles.push(Triangle::new(0, 3, 2));

    // Top face
    mesh.triangles.push(Triangle::new(4, 5, 6));
    mesh.triangles.push(Triangle::new(4, 6, 7));

    // Front face (Y=0)
    mesh.triangles.push(Triangle::new(0, 1, 5));
    mesh.triangles.push(Triangle::new(0, 5, 4));

    // Back face (Y=10)
    mesh.triangles.push(Triangle::new(3, 7, 6));
    mesh.triangles.push(Triangle::new(3, 6, 2));

    // Left face (X=0)
    mesh.triangles.push(Triangle::new(0, 4, 7));
    mesh.triangles.push(Triangle::new(0, 7, 3));

    // Right face (X=10)
    mesh.triangles.push(Triangle::new(1, 2, 6));
    mesh.triangles.push(Triangle::new(1, 6, 5));

    // Slice at Z=5 (middle)
    let segments = collect_intersection_segments(&mesh, 5.0);

    // Should get 8 segments (2 per side face, 4 side faces total)
    // Each side face is made of 2 triangles, each producing 1 segment
    assert_eq!(segments.len(), 8);

    // Assemble into contours
    let contours = assemble_contours(segments, 1e-6);

    // Should get exactly 1 square contour
    assert_eq!(contours.len(), 1);

    // Contour should have 8 vertices (2 per edge, since each edge has 2 segments)
    // Actually, after assembly with proper tolerance, adjacent segments should connect
    // The contour length depends on how segments are assembled
    assert!(
        contours[0].len() >= 4,
        "Contour should have at least 4 vertices"
    );

    // Verify all vertices are within the cube bounds
    for point in &contours[0] {
        assert!(
            point.0 >= -1e-6 && point.0 <= 10.0 + 1e-6,
            "X coordinate should be in range [0, 10], got {}",
            point.0
        );
        assert!(
            point.1 >= -1e-6 && point.1 <= 10.0 + 1e-6,
            "Y coordinate should be in range [0, 10], got {}",
            point.1
        );
    }
}

#[test]
fn test_slice_pyramid_at_different_heights() {
    let mut mesh = Mesh::new();

    // Base vertices (Z=0)
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0)); // 0
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0)); // 1
    mesh.vertices.push(Vertex::new(10.0, 10.0, 0.0)); // 2
    mesh.vertices.push(Vertex::new(0.0, 10.0, 0.0)); // 3

    // Apex (Z=10)
    mesh.vertices.push(Vertex::new(5.0, 5.0, 10.0)); // 4

    // Base triangles
    mesh.triangles.push(Triangle::new(0, 2, 1));
    mesh.triangles.push(Triangle::new(0, 3, 2));

    // Side triangles
    mesh.triangles.push(Triangle::new(0, 1, 4));
    mesh.triangles.push(Triangle::new(1, 2, 4));
    mesh.triangles.push(Triangle::new(2, 3, 4));
    mesh.triangles.push(Triangle::new(3, 0, 4));

    // Test slice at Z=0 (base)
    let segments = collect_intersection_segments(&mesh, 0.0);
    assert!(
        segments.len() >= 2,
        "Should have segments at base (edges on plane)"
    );

    // Test slice at Z=5 (middle)
    let segments = collect_intersection_segments(&mesh, 5.0);
    assert_eq!(segments.len(), 4, "Should have 4 segments at middle");

    let contours = assemble_contours(segments, 1e-6);
    assert_eq!(contours.len(), 1, "Should have 1 contour at middle");
    assert_eq!(contours[0].len(), 4, "Contour should have 4 vertices");

    // Test slice at Z=10 (apex)
    let segments = collect_intersection_segments(&mesh, 10.0);
    // At apex, only edges touch the plane
    assert!(segments.len() >= 4, "Should have segments at apex");

    // Test slice above pyramid (no intersection)
    let segments = collect_intersection_segments(&mesh, 15.0);
    assert_eq!(segments.len(), 0, "Should have no segments above pyramid");

    // Test slice below pyramid (no intersection)
    let segments = collect_intersection_segments(&mesh, -5.0);
    assert_eq!(segments.len(), 0, "Should have no segments below pyramid");
}

#[test]
fn test_triangle_intersection_edge_cases() {
    // Triangle with one vertex on the plane
    let v0 = Vertex::new(0.0, 0.0, 5.0); // On plane
    let v1 = Vertex::new(10.0, 0.0, 0.0); // Below
    let v2 = Vertex::new(5.0, 10.0, 10.0); // Above

    let result = triangle_plane_intersection(&v0, &v1, &v2, 5.0);
    assert!(result.is_some(), "Should intersect when vertex is on plane");

    // Triangle with two vertices on the plane (edge on plane)
    let v0 = Vertex::new(0.0, 0.0, 5.0); // On plane
    let v1 = Vertex::new(10.0, 0.0, 5.0); // On plane
    let v2 = Vertex::new(5.0, 10.0, 10.0); // Above

    let result = triangle_plane_intersection(&v0, &v1, &v2, 5.0);
    assert!(
        result.is_some(),
        "Should handle edge on plane (2 vertices on plane)"
    );

    // Triangle parallel to plane but above it
    let v0 = Vertex::new(0.0, 0.0, 10.0);
    let v1 = Vertex::new(10.0, 0.0, 10.0);
    let v2 = Vertex::new(5.0, 10.0, 10.0);

    let result = triangle_plane_intersection(&v0, &v1, &v2, 5.0);
    assert!(
        result.is_none(),
        "Should not intersect when triangle is parallel and above"
    );
}

#[test]
fn test_contour_assembly_with_tolerance() {
    // Create segments that are slightly disconnected
    let segments = vec![
        ((0.0, 0.0), (1.0, 0.0)),
        ((1.000001, 0.0), (1.0, 1.0)), // Small gap
        ((1.0, 1.0), (0.0, 1.0)),
        ((0.0, 1.0), (0.0, 0.0)),
    ];

    // With small tolerance, should not assemble
    let contours = assemble_contours(segments.clone(), 1e-9);
    assert!(
        contours.len() > 1 || contours[0].len() != 4,
        "Small tolerance should not connect slightly disconnected segments"
    );

    // With larger tolerance, should assemble
    let contours = assemble_contours(segments, 1e-5);
    assert_eq!(
        contours.len(),
        1,
        "Larger tolerance should connect segments"
    );
    assert_eq!(contours[0].len(), 4, "Should have 4 vertices in square");
}

#[test]
fn test_multiple_contours() {
    // Create two separate squares
    let segments = vec![
        // First square at origin
        ((0.0, 0.0), (1.0, 0.0)),
        ((1.0, 0.0), (1.0, 1.0)),
        ((1.0, 1.0), (0.0, 1.0)),
        ((0.0, 1.0), (0.0, 0.0)),
        // Second square offset
        ((5.0, 5.0), (6.0, 5.0)),
        ((6.0, 5.0), (6.0, 6.0)),
        ((6.0, 6.0), (5.0, 6.0)),
        ((5.0, 6.0), (5.0, 5.0)),
    ];

    let contours = assemble_contours(segments, 1e-6);

    assert_eq!(contours.len(), 2, "Should have 2 separate contours");
    for contour in &contours {
        assert_eq!(contour.len(), 4, "Each contour should have 4 vertices");
    }
}
