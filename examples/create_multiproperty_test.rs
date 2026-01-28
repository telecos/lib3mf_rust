//! Create a test 3MF file with multiproperties for viewer testing

use lib3mf::*;
use std::fs::File;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Create a model with multi-properties
    let mut model = Model::new();
    model.unit = "millimeter".to_string();

    // Add base material group with Red, Green, Blue materials
    let mut base_group = BaseMaterialGroup::new(1);
    base_group.materials.push(BaseMaterial::new(
        "Red".to_string(),
        (255, 0, 0, 255),
    ));
    base_group.materials.push(BaseMaterial::new(
        "Green".to_string(),
        (0, 255, 0, 255),
    ));
    base_group.materials.push(BaseMaterial::new(
        "Blue".to_string(),
        (0, 0, 255, 255),
    ));
    model.resources.base_material_groups.push(base_group);

    // Add color group with Yellow and Cyan
    let mut color_group = ColorGroup::new(2);
    color_group.colors.push((255, 255, 0, 255)); // Yellow
    color_group.colors.push((0, 255, 255, 255)); // Cyan
    model.resources.color_groups.push(color_group);

    // Add multi-properties group that blends base materials and colors
    let mut multi = MultiProperties::new(3, vec![1, 2]); // Base group 1, Color group 2
    multi.blendmethods.push(BlendMethod::Mix);
    
    // Create 3 different multiproperties
    multi.multis.push(Multi::new(vec![0, 0])); // Red + Yellow
    multi.multis.push(Multi::new(vec![1, 1])); // Green + Cyan
    multi.multis.push(Multi::new(vec![2, 0])); // Blue + Yellow
    model.resources.multi_properties.push(multi);

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
    
    // Create triangles with different multiproperties
    let mut tri1 = Triangle::new(0, 1, 2);
    tri1.pid = Some(3); // Use multi-properties group
    tri1.p1 = Some(0);  // First multi: Red + Yellow
    mesh.triangles.push(tri1);
    
    let mut tri2 = Triangle::new(3, 4, 5);
    tri2.pid = Some(3);
    tri2.p1 = Some(1); // Second multi: Green + Cyan
    mesh.triangles.push(tri2);
    
    let mut tri3 = Triangle::new(6, 7, 8);
    tri3.pid = Some(3);
    tri3.p1 = Some(2); // Third multi: Blue + Yellow
    mesh.triangles.push(tri3);

    // Create object with the mesh
    let mut object = Object::new(1);
    object.mesh = Some(mesh);
    model.resources.objects.push(object);
    model.build.items.push(BuildItem::new(1));

    // Write to file
    let output_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "test_multiproperties.3mf".to_string());
    
    let file = File::create(&output_path)?;
    model.to_writer(file)?;
    
    println!("Created {}", output_path);
    println!("This file contains 3 triangles with different multiproperty blends:");
    println!("  Triangle 1: Red + Yellow blend");
    println!("  Triangle 2: Green + Cyan blend");
    println!("  Triangle 3: Blue + Yellow blend");
    
    Ok(())
}
