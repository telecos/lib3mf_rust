//! Example demonstrating vertex normal calculation utilities
//!
//! This example shows how to use the calculate_face_normal and
//! calculate_vertex_normals functions to compute normals for meshes.

use lib3mf::mesh_ops::{calculate_face_normal, calculate_vertex_normals};
use lib3mf::{Mesh, Triangle, Vertex};

fn main() {
    println!("=== Vertex Normal Calculation Example ===\n");

    // Example 1: Calculate face normal for a single triangle
    println!("Example 1: Face Normal Calculation");
    let v0 = Vertex::new(0.0, 0.0, 0.0);
    let v1 = Vertex::new(1.0, 0.0, 0.0);
    let v2 = Vertex::new(0.0, 1.0, 0.0);

    let normal = calculate_face_normal(&v0, &v1, &v2);
    println!("  Triangle vertices:");
    println!("    v0: ({}, {}, {})", v0.x, v0.y, v0.z);
    println!("    v1: ({}, {}, {})", v1.x, v1.y, v1.z);
    println!("    v2: ({}, {}, {})", v2.x, v2.y, v2.z);
    println!("  Face normal: ({:.6}, {:.6}, {:.6})", normal.0, normal.1, normal.2);
    println!("  (Expected: pointing in +Z direction)\n");

    // Example 2: Calculate vertex normals for a simple mesh
    println!("Example 2: Vertex Normals for a Single Triangle Mesh");
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));

    let normals = calculate_vertex_normals(&mesh);
    println!("  Mesh has {} vertices and {} triangles", mesh.vertices.len(), mesh.triangles.len());
    for (i, normal) in normals.iter().enumerate() {
        println!("  Vertex {} normal: ({:.6}, {:.6}, {:.6})", i, normal.0, normal.1, normal.2);
    }
    println!();

    // Example 3: Calculate vertex normals for a cube
    println!("Example 3: Vertex Normals for a Cube");
    let mut cube = Mesh::new();

    // Cube vertices
    cube.vertices.push(Vertex::new(0.0, 0.0, 0.0)); // 0
    cube.vertices.push(Vertex::new(1.0, 0.0, 0.0)); // 1
    cube.vertices.push(Vertex::new(1.0, 1.0, 0.0)); // 2
    cube.vertices.push(Vertex::new(0.0, 1.0, 0.0)); // 3
    cube.vertices.push(Vertex::new(0.0, 0.0, 1.0)); // 4
    cube.vertices.push(Vertex::new(1.0, 0.0, 1.0)); // 5
    cube.vertices.push(Vertex::new(1.0, 1.0, 1.0)); // 6
    cube.vertices.push(Vertex::new(0.0, 1.0, 1.0)); // 7

    // Cube faces (counter-clockwise winding)
    // Bottom face
    cube.triangles.push(Triangle::new(0, 2, 1));
    cube.triangles.push(Triangle::new(0, 3, 2));
    // Top face
    cube.triangles.push(Triangle::new(4, 5, 6));
    cube.triangles.push(Triangle::new(4, 6, 7));
    // Front face
    cube.triangles.push(Triangle::new(0, 1, 5));
    cube.triangles.push(Triangle::new(0, 5, 4));
    // Back face
    cube.triangles.push(Triangle::new(3, 7, 6));
    cube.triangles.push(Triangle::new(3, 6, 2));
    // Left face
    cube.triangles.push(Triangle::new(0, 4, 7));
    cube.triangles.push(Triangle::new(0, 7, 3));
    // Right face
    cube.triangles.push(Triangle::new(1, 2, 6));
    cube.triangles.push(Triangle::new(1, 6, 5));

    let cube_normals = calculate_vertex_normals(&cube);
    println!("  Cube has {} vertices and {} triangles", cube.vertices.len(), cube.triangles.len());
    println!("  Corner vertex normals (averaged from 3 adjacent faces):");
    for (i, normal) in cube_normals.iter().enumerate() {
        let vertex = &cube.vertices[i];
        println!(
            "  Vertex {} at ({:.1}, {:.1}, {:.1}): normal = ({:.6}, {:.6}, {:.6})",
            i, vertex.x, vertex.y, vertex.z, normal.0, normal.1, normal.2
        );
    }

    println!("\n=== Example Complete ===");
}
