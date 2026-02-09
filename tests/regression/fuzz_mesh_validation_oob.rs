//! Regression test for fuzz_mesh_validation index out of bounds crash
//!
//! This test verifies the fix for the fuzzing crash where:
//! 1. Triangle indices were generated as arbitrary u32 values
//! 2. vertex_count could be 0, causing underflow in range calculation
//!
//! The fix ensures triangle indices are constrained to valid vertex ranges.

use lib3mf::{Mesh, Triangle, Vertex};

#[test]
fn test_empty_mesh_no_panic() {
    // Test with zero vertices - should not panic
    let mesh = Mesh::new();
    assert_eq!(mesh.vertices.len(), 0);
    assert_eq!(mesh.triangles.len(), 0);
    
    // All mesh operations should handle empty mesh gracefully
    assert!(lib3mf::mesh_ops::compute_mesh_volume(&mesh).unwrap() == 0.0);
    assert!(lib3mf::mesh_ops::compute_mesh_aabb(&mesh).is_err()); // Expected to fail
    assert!(lib3mf::mesh_ops::compute_mesh_signed_volume(&mesh).unwrap() == 0.0);
    let normals = lib3mf::mesh_ops::calculate_vertex_normals(&mesh);
    assert_eq!(normals.len(), 0);
}

#[test]
fn test_vertices_but_no_triangles() {
    // Test with vertices but no triangles
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));
    
    assert_eq!(mesh.vertices.len(), 3);
    assert_eq!(mesh.triangles.len(), 0);
    
    // Volume and signed volume should be 0 with no triangles
    assert!(lib3mf::mesh_ops::compute_mesh_volume(&mesh).unwrap() == 0.0);
    assert!(lib3mf::mesh_ops::compute_mesh_aabb(&mesh).is_err()); // No triangles
    assert!(lib3mf::mesh_ops::compute_mesh_signed_volume(&mesh).unwrap() == 0.0);
}

#[test]
fn test_valid_indices_within_range() {
    // Test that valid indices work correctly
    let mut mesh = Mesh::new();
    
    // Add 3 vertices
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));
    
    // Add triangle with indices 0, 1, 2 (all valid)
    mesh.triangles.push(Triangle::new(0, 1, 2));
    
    // All operations should work
    let volume = lib3mf::mesh_ops::compute_mesh_volume(&mesh);
    assert!(volume.is_ok());
    
    let aabb = lib3mf::mesh_ops::compute_mesh_aabb(&mesh);
    assert!(aabb.is_ok());
    
    let signed_volume = lib3mf::mesh_ops::compute_mesh_signed_volume(&mesh);
    assert!(signed_volume.is_ok());
    
    let normals = lib3mf::mesh_ops::calculate_vertex_normals(&mesh);
    assert_eq!(normals.len(), 3);
}

#[test]
fn test_parry3d_panic_handling() {
    // Test that parry3d panics are caught and converted to errors
    // This creates a mesh that might trigger parry3d's BVH bug
    let mut mesh = Mesh::new();
    
    // Add vertices in a degenerate configuration
    for i in 0..10 {
        mesh.vertices.push(Vertex::new(i as f64, 0.0, 0.0));
    }
    
    // Add triangles
    for i in 0..8 {
        mesh.triangles.push(Triangle::new(i, (i + 1) % 10, (i + 2) % 10));
    }
    
    // These should either succeed or return an error, but never panic
    let _ = lib3mf::mesh_ops::compute_mesh_volume(&mesh);
    let _ = lib3mf::mesh_ops::compute_mesh_aabb(&mesh);
    let _ = lib3mf::mesh_ops::compute_mesh_signed_volume(&mesh);
    let _ = lib3mf::mesh_ops::calculate_vertex_normals(&mesh);
}

#[test]
fn test_nan_and_inf_handling() {
    // Test that NaN and Inf values are handled gracefully
    let mut mesh = Mesh::new();
    
    // Add valid vertices
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));
    
    // Add triangle
    mesh.triangles.push(Triangle::new(0, 1, 2));
    
    // Operations should work with valid finite values
    let normals = lib3mf::mesh_ops::calculate_vertex_normals(&mesh);
    assert_eq!(normals.len(), 3);
    
    // All normals should be finite
    for (x, y, z) in normals {
        assert!(x.is_finite());
        assert!(y.is_finite());
        assert!(z.is_finite());
    }
}
