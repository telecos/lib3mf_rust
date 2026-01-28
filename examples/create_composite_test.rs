//! Create a test 3MF file with composite materials for viewer testing

use lib3mf::*;
use std::fs::File;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Create a model with composite materials
    let mut model = Model::new();
    model.unit = "millimeter".to_string();

    // Add base material group with Red, Green, Blue
    let mut base_group = BaseMaterialGroup::new(1);
    base_group
        .materials
        .push(BaseMaterial::new("Red".to_string(), (255, 0, 0, 255)));
    base_group
        .materials
        .push(BaseMaterial::new("Green".to_string(), (0, 255, 0, 255)));
    base_group
        .materials
        .push(BaseMaterial::new("Blue".to_string(), (0, 0, 255, 255)));
    model.resources.base_material_groups.push(base_group);

    // Add composite materials group
    let matindices = vec![0, 1, 2]; // Indices of Red, Green, Blue
    let mut comp_group = CompositeMaterials::new(2, 1, matindices);

    // Create 3 different composites with different mixing ratios
    comp_group
        .composites
        .push(Composite::new(vec![0.7, 0.3, 0.0])); // Mostly red + some green = Orange
    comp_group
        .composites
        .push(Composite::new(vec![0.0, 0.6, 0.4])); // Green + blue = Teal
    comp_group
        .composites
        .push(Composite::new(vec![0.5, 0.0, 0.5])); // Red + blue = Purple
    model.resources.composite_materials.push(comp_group);

    // Create a mesh with 3 triangles
    let mut mesh = Mesh::new();

    // Triangle 1 vertices
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));

    // Triangle 2 vertices
    mesh.vertices.push(Vertex::new(15.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(25.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(20.0, 10.0, 0.0));

    // Triangle 3 vertices
    mesh.vertices.push(Vertex::new(30.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(40.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(35.0, 10.0, 0.0));

    // Create triangles with different composites
    let mut tri1 = Triangle::new(0, 1, 2);
    tri1.pid = Some(2); // Use composite materials group
    tri1.p1 = Some(0); // First composite: Orange
    mesh.triangles.push(tri1);

    let mut tri2 = Triangle::new(3, 4, 5);
    tri2.pid = Some(2);
    tri2.p1 = Some(1); // Second composite: Teal
    mesh.triangles.push(tri2);

    let mut tri3 = Triangle::new(6, 7, 8);
    tri3.pid = Some(2);
    tri3.p1 = Some(2); // Third composite: Purple
    mesh.triangles.push(tri3);

    // Create object with the mesh
    let mut object = Object::new(1);
    object.mesh = Some(mesh);
    model.resources.objects.push(object);
    model.build.items.push(BuildItem::new(1));

    // Write to file
    let output_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "test_composites.3mf".to_string());

    let file = File::create(&output_path)?;
    model.to_writer(file)?;

    println!("Created {}", output_path);
    println!("This file contains 3 triangles with different composite materials:");
    println!("  Triangle 1: 70% Red + 30% Green = Orange");
    println!("  Triangle 2: 60% Green + 40% Blue = Teal");
    println!("  Triangle 3: 50% Red + 50% Blue = Purple");

    Ok(())
}
