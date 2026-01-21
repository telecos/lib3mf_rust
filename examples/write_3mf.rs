//! Example demonstrating how to create and write 3MF files
//!
//! This example shows:
//! - Creating a new Model from scratch
//! - Adding mesh geometry (vertices and triangles)
//! - Adding metadata
//! - Writing to a 3MF file
//! - Round-trip verification (read back what we wrote)

use lib3mf::{BuildItem, Material, Mesh, Model, Object, Triangle, Vertex, Extension};
use std::fs::File;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("3MF File Writing Example\n");
    
    // Example 1: Simple pyramid
    println!("1. Creating a simple pyramid...");
    create_pyramid()?;
    println!("   ✓ Saved to 'pyramid.3mf'\n");
    
    // Example 2: Colored cube
    println!("2. Creating a colored cube...");
    create_colored_cube()?;
    println!("   ✓ Saved to 'colored_cube.3mf'\n");
    
    // Example 3: Round-trip test
    println!("3. Testing round-trip (write then read)...");
    test_round_trip()?;
    println!("   ✓ Round-trip successful!\n");
    
    println!("All examples completed successfully!");
    Ok(())
}

/// Create a simple pyramid and save it
fn create_pyramid() -> Result<(), Box<dyn std::error::Error>> {
    let mut model = Model::new();
    model.metadata.insert("Title".to_string(), "Simple Pyramid".to_string());
    model.metadata.insert("Designer".to_string(), "lib3mf_rust example".to_string());
    
    // Create pyramid mesh
    let mut mesh = Mesh::new();
    
    // Base vertices (square base)
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(10.0, 10.0, 0.0));
    mesh.vertices.push(Vertex::new(0.0, 10.0, 0.0));
    
    // Apex
    mesh.vertices.push(Vertex::new(5.0, 5.0, 10.0));
    
    // Side faces (4 triangles)
    mesh.triangles.push(Triangle::new(0, 1, 4));
    mesh.triangles.push(Triangle::new(1, 2, 4));
    mesh.triangles.push(Triangle::new(2, 3, 4));
    mesh.triangles.push(Triangle::new(3, 0, 4));
    
    // Base (2 triangles)
    mesh.triangles.push(Triangle::new(0, 2, 1));
    mesh.triangles.push(Triangle::new(0, 3, 2));
    
    let mut obj = Object::new(1);
    obj.name = Some("Pyramid".to_string());
    obj.mesh = Some(mesh);
    model.resources.objects.push(obj);
    
    model.build.items.push(BuildItem::new(1));
    
    let file = File::create("pyramid.3mf")?;
    model.to_writer(file)?;
    
    Ok(())
}

/// Create a colored cube using the materials extension
fn create_colored_cube() -> Result<(), Box<dyn std::error::Error>> {
    let mut model = Model::new();
    model.metadata.insert("Title".to_string(), "Colored Cube".to_string());
    model.metadata.insert("Description".to_string(), "A cube with colored faces".to_string());
    
    // Enable materials extension
    model.required_extensions.push(Extension::Material);
    
    // Add materials (6 colors for 6 faces)
    model.resources.materials.push(Material::with_color(1, 255, 0, 0, 255));    // Red
    model.resources.materials.push(Material::with_color(2, 0, 255, 0, 255));    // Green
    model.resources.materials.push(Material::with_color(3, 0, 0, 255, 255));    // Blue
    model.resources.materials.push(Material::with_color(4, 255, 255, 0, 255));  // Yellow
    model.resources.materials.push(Material::with_color(5, 255, 0, 255, 255));  // Magenta
    model.resources.materials.push(Material::with_color(6, 0, 255, 255, 255));  // Cyan
    
    // Create cube mesh
    let mut mesh = Mesh::new();
    
    // 8 vertices of a cube
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));  // 0: front-bottom-left
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0)); // 1: front-bottom-right
    mesh.vertices.push(Vertex::new(10.0, 10.0, 0.0));// 2: front-top-right
    mesh.vertices.push(Vertex::new(0.0, 10.0, 0.0)); // 3: front-top-left
    mesh.vertices.push(Vertex::new(0.0, 0.0, 10.0)); // 4: back-bottom-left
    mesh.vertices.push(Vertex::new(10.0, 0.0, 10.0));// 5: back-bottom-right
    mesh.vertices.push(Vertex::new(10.0, 10.0, 10.0));// 6: back-top-right
    mesh.vertices.push(Vertex::new(0.0, 10.0, 10.0));// 7: back-top-left
    
    // Front face (red) - triangles 0, 1
    mesh.triangles.push(Triangle::with_material(0, 1, 2, 1));
    mesh.triangles.push(Triangle::with_material(0, 2, 3, 1));
    
    // Back face (green) - triangles 2, 3
    mesh.triangles.push(Triangle::with_material(5, 4, 7, 2));
    mesh.triangles.push(Triangle::with_material(5, 7, 6, 2));
    
    // Left face (blue) - triangles 4, 5
    mesh.triangles.push(Triangle::with_material(4, 0, 3, 3));
    mesh.triangles.push(Triangle::with_material(4, 3, 7, 3));
    
    // Right face (yellow) - triangles 6, 7
    mesh.triangles.push(Triangle::with_material(1, 5, 6, 4));
    mesh.triangles.push(Triangle::with_material(1, 6, 2, 4));
    
    // Top face (magenta) - triangles 8, 9
    mesh.triangles.push(Triangle::with_material(3, 2, 6, 5));
    mesh.triangles.push(Triangle::with_material(3, 6, 7, 5));
    
    // Bottom face (cyan) - triangles 10, 11
    mesh.triangles.push(Triangle::with_material(4, 5, 1, 6));
    mesh.triangles.push(Triangle::with_material(4, 1, 0, 6));
    
    let mut obj = Object::new(1);
    obj.name = Some("ColoredCube".to_string());
    obj.mesh = Some(mesh);
    model.resources.objects.push(obj);
    
    model.build.items.push(BuildItem::new(1));
    
    let file = File::create("colored_cube.3mf")?;
    model.to_writer(file)?;
    
    Ok(())
}

/// Test round-trip: write a model and read it back
fn test_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    // Create a test model
    let mut model = Model::new();
    model.metadata.insert("Test".to_string(), "RoundTrip".to_string());
    
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(1.5, 2.5, 3.5));
    mesh.vertices.push(Vertex::new(4.5, 5.5, 6.5));
    mesh.vertices.push(Vertex::new(7.5, 8.5, 9.5));
    mesh.triangles.push(Triangle::new(0, 1, 2));
    
    let mut obj = Object::new(42);
    obj.name = Some("TestObject".to_string());
    obj.mesh = Some(mesh);
    model.resources.objects.push(obj);
    
    model.build.items.push(BuildItem::new(42));
    
    // Write to buffer
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    model.to_writer(cursor)?;
    
    // Read back from buffer
    let cursor = Cursor::new(buffer);
    let model2 = Model::from_reader(cursor)?;
    
    // Verify key properties
    assert_eq!(model2.unit, "millimeter");
    assert_eq!(model2.metadata.get("Test"), Some(&"RoundTrip".to_string()));
    assert_eq!(model2.resources.objects.len(), 1);
    assert_eq!(model2.resources.objects[0].id, 42);
    assert_eq!(model2.resources.objects[0].name, Some("TestObject".to_string()));
    
    let mesh2 = model2.resources.objects[0].mesh.as_ref().unwrap();
    assert_eq!(mesh2.vertices.len(), 3);
    assert_eq!(mesh2.triangles.len(), 1);
    
    // Check vertex precision
    assert!((mesh2.vertices[0].x - 1.5).abs() < 0.001);
    assert!((mesh2.vertices[0].y - 2.5).abs() < 0.001);
    assert!((mesh2.vertices[0].z - 3.5).abs() < 0.001);
    
    Ok(())
}
