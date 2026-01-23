//! Tests for Mesh helper methods
//!
//! These tests verify the mesh inspection methods work correctly,
//! particularly for beam lattice meshes that may have vertices but no triangles.

use lib3mf::{Mesh, Triangle, Vertex};

#[test]
fn test_empty_mesh() {
    let mesh = Mesh::new();
    
    assert!(!mesh.has_triangles(), "Empty mesh should have no triangles");
    assert!(!mesh.has_vertices(), "Empty mesh should have no vertices");
    assert!(mesh.is_empty(), "Empty mesh should be empty");
}

#[test]
fn test_mesh_with_vertices_only() {
    // This simulates a beam lattice mesh with vertices but no triangles
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));
    
    assert!(!mesh.has_triangles(), "Mesh with only vertices should have no triangles");
    assert!(mesh.has_vertices(), "Mesh should have vertices");
    assert!(!mesh.is_empty(), "Mesh with vertices is not empty");
}

#[test]
fn test_mesh_with_triangles() {
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));
    
    assert!(mesh.has_triangles(), "Mesh should have triangles");
    assert!(mesh.has_vertices(), "Mesh should have vertices");
    assert!(!mesh.is_empty(), "Mesh with data is not empty");
}

#[test]
fn test_mesh_with_capacity() {
    let mesh = Mesh::with_capacity(100, 50);
    
    assert!(!mesh.has_triangles(), "New mesh with capacity should have no triangles");
    assert!(!mesh.has_vertices(), "New mesh with capacity should have no vertices");
    assert!(mesh.is_empty(), "New mesh with capacity should be empty");
    
    // Verify capacity was set correctly (implementation detail)
    assert!(mesh.vertices.capacity() >= 100);
    assert!(mesh.triangles.capacity() >= 50);
}

#[test]
fn test_beam_lattice_mesh_scenario() {
    // Test a realistic beam lattice scenario from suite7 test files
    // where meshes have vertices for beam endpoints but no triangles
    let mut mesh = Mesh::new();
    
    // Add vertices for beam lattice structure
    for i in 0..10 {
        mesh.vertices.push(Vertex::new(i as f64, 0.0, 0.0));
    }
    
    // No triangles added (beam lattice only)
    
    // This should be safe to check before passing to external libraries
    assert!(!mesh.has_triangles(), "Expected no triangles in beam lattice mesh");
    
    // This is the expected case for beam lattice meshes
    assert!(mesh.has_vertices(), "Beam lattice should have vertices");
    assert_eq!(mesh.vertices.len(), 10, "Should have 10 vertices");
    assert_eq!(mesh.triangles.len(), 0, "Should have 0 triangles");
}
