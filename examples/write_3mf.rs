//! Example demonstrating how to create and write 3MF files
//!
//! This example shows:
//! 1. Creating a new 3MF model from scratch
//! 2. Adding geometry (vertices and triangles)
//! 3. Adding materials and colors
//! 4. Adding metadata
//! 5. Writing the model to a file

use lib3mf::{
    BaseMaterial, BaseMaterialGroup, BuildItem, Mesh, MetadataEntry, Model, Object, Triangle,
    Vertex,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating a 3MF file with materials and colors...\n");

    // Create a new model
    let mut model = Model::new();
    model.unit = "millimeter".to_string();

    // Add metadata
    model.metadata.push(MetadataEntry::new(
        "Title".to_string(),
        "Colored Cube".to_string(),
    ));
    model.metadata.push(MetadataEntry::new(
        "Designer".to_string(),
        "lib3mf_rust".to_string(),
    ));
    model.metadata.push(MetadataEntry::new(
        "Description".to_string(),
        "A simple cube with colored faces created programmatically".to_string(),
    ));

    // Create a base material group with colors
    let mut material_group = BaseMaterialGroup::new(1);
    material_group.materials.push(BaseMaterial::new(
        "Red".to_string(),
        (255, 0, 0, 255), // RGBA
    ));
    material_group.materials.push(BaseMaterial::new(
        "Green".to_string(),
        (0, 255, 0, 255),
    ));
    material_group.materials.push(BaseMaterial::new(
        "Blue".to_string(),
        (0, 0, 255, 255),
    ));
    material_group.materials.push(BaseMaterial::new(
        "Yellow".to_string(),
        (255, 255, 0, 255),
    ));
    material_group.materials.push(BaseMaterial::new(
        "Cyan".to_string(),
        (0, 255, 255, 255),
    ));
    material_group.materials.push(BaseMaterial::new(
        "Magenta".to_string(),
        (255, 0, 255, 255),
    ));
    model.resources.base_material_groups.push(material_group);

    // Create a cube mesh (8 vertices, 12 triangles)
    let mut mesh = Mesh::new();

    // Add vertices (corners of a 10mm cube)
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0)); // 0: bottom-front-left
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0)); // 1: bottom-front-right
    mesh.vertices.push(Vertex::new(10.0, 10.0, 0.0)); // 2: bottom-back-right
    mesh.vertices.push(Vertex::new(0.0, 10.0, 0.0)); // 3: bottom-back-left
    mesh.vertices.push(Vertex::new(0.0, 0.0, 10.0)); // 4: top-front-left
    mesh.vertices.push(Vertex::new(10.0, 0.0, 10.0)); // 5: top-front-right
    mesh.vertices.push(Vertex::new(10.0, 10.0, 10.0)); // 6: top-back-right
    mesh.vertices.push(Vertex::new(0.0, 10.0, 10.0)); // 7: top-back-left

    // Bottom face (Red)
    let mut tri = Triangle::new(0, 2, 1);
    tri.pid = Some(1);
    tri.pindex = Some(0); // Red
    mesh.triangles.push(tri);

    let mut tri = Triangle::new(0, 3, 2);
    tri.pid = Some(1);
    tri.pindex = Some(0); // Red
    mesh.triangles.push(tri);

    // Top face (Green)
    let mut tri = Triangle::new(4, 5, 6);
    tri.pid = Some(1);
    tri.pindex = Some(1); // Green
    mesh.triangles.push(tri);

    let mut tri = Triangle::new(4, 6, 7);
    tri.pid = Some(1);
    tri.pindex = Some(1); // Green
    mesh.triangles.push(tri);

    // Front face (Blue)
    let mut tri = Triangle::new(0, 1, 5);
    tri.pid = Some(1);
    tri.pindex = Some(2); // Blue
    mesh.triangles.push(tri);

    let mut tri = Triangle::new(0, 5, 4);
    tri.pid = Some(1);
    tri.pindex = Some(2); // Blue
    mesh.triangles.push(tri);

    // Back face (Yellow)
    let mut tri = Triangle::new(2, 3, 7);
    tri.pid = Some(1);
    tri.pindex = Some(3); // Yellow
    mesh.triangles.push(tri);

    let mut tri = Triangle::new(2, 7, 6);
    tri.pid = Some(1);
    tri.pindex = Some(3); // Yellow
    mesh.triangles.push(tri);

    // Left face (Cyan)
    let mut tri = Triangle::new(3, 0, 4);
    tri.pid = Some(1);
    tri.pindex = Some(4); // Cyan
    mesh.triangles.push(tri);

    let mut tri = Triangle::new(3, 4, 7);
    tri.pid = Some(1);
    tri.pindex = Some(4); // Cyan
    mesh.triangles.push(tri);

    // Right face (Magenta)
    let mut tri = Triangle::new(1, 2, 6);
    tri.pid = Some(1);
    tri.pindex = Some(5); // Magenta
    mesh.triangles.push(tri);

    let mut tri = Triangle::new(1, 6, 5);
    tri.pid = Some(1);
    tri.pindex = Some(5); // Magenta
    mesh.triangles.push(tri);

    // Create object with the mesh
    let mut object = Object::new(1);
    object.name = Some("Colored Cube".to_string());
    object.mesh = Some(mesh);

    // Add object to resources
    model.resources.objects.push(object);

    // Add to build
    model.build.items.push(BuildItem::new(1));

    println!("Model created with:");
    println!("  - {} vertices", 8);
    println!("  - {} triangles", 12);
    println!("  - {} materials", 6);
    println!();

    // Write to file
    let output_path = "colored_cube.3mf";
    println!("Writing to {}...", output_path);
    model.write_to_file(output_path)?;

    println!("✓ Successfully wrote 3MF file!");
    println!();

    // Verify by reading it back
    println!("Verifying by reading the file back...");
    let file = std::fs::File::open(output_path)?;
    let verified_model = Model::from_reader(file)?;

    println!("✓ Verification successful!");
    println!("  - Unit: {}", verified_model.unit);
    println!("  - Objects: {}", verified_model.resources.objects.len());
    println!(
        "  - Materials: {}",
        verified_model.resources.base_material_groups.len()
    );
    println!("  - Metadata entries: {}", verified_model.metadata.len());

    if let Some(obj) = verified_model.resources.objects.first() {
        if let Some(ref mesh) = obj.mesh {
            println!(
                "  - Mesh: {} vertices, {} triangles",
                mesh.vertices.len(),
                mesh.triangles.len()
            );
        }
    }

    println!();
    println!("Done! You can now open {} in a 3MF viewer.", output_path);

    Ok(())
}
