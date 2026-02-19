//! Create a sample 3MF file with boolean operations for testing the slicer
//!
//! This example creates a 3MF file with:
//! - Two overlapping cube objects
//! - A boolean difference operation (cube A - cube B)
//! - Demonstrates boolean operation parsing and detection

use lib3mf::{
    BooleanOpType, BooleanRef, BooleanShape, BuildItem, Mesh, Model, Object, Triangle, Vertex,
};

fn create_cube_mesh(size: f64, offset_x: f64, offset_y: f64, offset_z: f64) -> Mesh {
    let half = size / 2.0;

    let vertices = vec![
        // Bottom face
        Vertex::new(-half + offset_x, -half + offset_y, -half + offset_z),
        Vertex::new(half + offset_x, -half + offset_y, -half + offset_z),
        Vertex::new(half + offset_x, half + offset_y, -half + offset_z),
        Vertex::new(-half + offset_x, half + offset_y, -half + offset_z),
        // Top face
        Vertex::new(-half + offset_x, -half + offset_y, half + offset_z),
        Vertex::new(half + offset_x, -half + offset_y, half + offset_z),
        Vertex::new(half + offset_x, half + offset_y, half + offset_z),
        Vertex::new(-half + offset_x, half + offset_y, half + offset_z),
    ];

    let triangles = vec![
        // Bottom face (z = -half, normal pointing down -Z)
        Triangle::new(0, 2, 1),
        Triangle::new(0, 3, 2),
        // Top face (z = +half, normal pointing up +Z)
        Triangle::new(4, 5, 6),
        Triangle::new(4, 6, 7),
        // Front face (y = -half, normal pointing -Y)
        Triangle::new(0, 1, 5),
        Triangle::new(0, 5, 4),
        // Back face (y = +half, normal pointing +Y)
        Triangle::new(2, 3, 7),
        Triangle::new(2, 7, 6),
        // Left face (x = -half, normal pointing -X)
        Triangle::new(0, 4, 7),
        Triangle::new(0, 7, 3),
        // Right face (x = +half, normal pointing +X)
        Triangle::new(1, 2, 6),
        Triangle::new(1, 6, 5),
    ];

    Mesh {
        vertices,
        triangles,
        beamset: None,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut model = Model::new();

    // Object 1: Base cube (20x20x20mm) centered at origin
    let mut cube_a = Object::new(1);
    cube_a.name = Some("Cube A (Base)".to_string());
    cube_a.mesh = Some(create_cube_mesh(20.0, 0.0, 0.0, 0.0));

    // Object 2: Smaller cube (12x12x12mm) offset to create an interesting difference
    let mut cube_b = Object::new(2);
    cube_b.name = Some("Cube B (Subtracted)".to_string());
    cube_b.mesh = Some(create_cube_mesh(12.0, 5.0, 5.0, 0.0));

    // Object 3: Boolean difference (A - B)
    let mut boolean_obj = Object::new(3);
    boolean_obj.name = Some("Boolean Difference Result".to_string());

    let mut bool_shape = BooleanShape::new(1, BooleanOpType::Difference);
    bool_shape.operands.push(BooleanRef::new(2));
    boolean_obj.boolean_shape = Some(bool_shape);

    // Add objects to model
    model.resources.objects.push(cube_a);
    model.resources.objects.push(cube_b);
    model.resources.objects.push(boolean_obj);

    // Add build item - references the boolean object
    let mut build_item = BuildItem::new(3);

    // Transform to move the result up for better viewing
    let build_transform = [
        1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 10.0, // Move up by 10mm
    ];
    build_item.transform = Some(build_transform);

    model.build.items.push(build_item);

    // Enable boolean operations extension
    model
        .required_extensions
        .push(lib3mf::Extension::BooleanOperations);

    // Write to file
    let output_path = "tools/slicer/samples/boolean/boolean_diff.3mf";

    // Create directory if it doesn't exist
    std::fs::create_dir_all("tools/slicer/samples/boolean")?;

    model.write_to_file(output_path)?;

    println!(
        "Created boolean operations sample 3MF file: {}",
        output_path
    );
    println!("  - Cube A: 20x20x20mm centered at origin");
    println!("  - Cube B: 12x12x12mm offset at (5,5,0)");
    println!("  - Boolean operation: DIFFERENCE (A - B)");
    println!("  - Build transform: moves result up 10mm");
    println!("  - Final Z range: approximately 0mm to 20mm");
    println!(
        "\nThe slicer applies the boolean difference at each slice layer using 2D polygon operations."
    );
    println!("Slices will show the L-shaped cross-section where Cube A overlaps with Cube B.");

    Ok(())
}
