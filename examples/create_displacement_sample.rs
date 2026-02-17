//! Create a 3MF file with displacement mapping for slicer testing
//!
//! This creates a simple box with displacement mapping applied to demonstrate
//! the displacement extension in 3MF files.

use lib3mf::{
    BuildItem, Disp2DCoords, Disp2DGroup, Displacement2D, DisplacementMesh, DisplacementTriangle,
    Extension, Model, NormVector, NormVectorGroup, Object, Vertex,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating displacement sample 3MF file...");

    let mut model = Model::new();
    
    // Declare displacement extension as required
    model.required_extensions.push(Extension::Displacement);

    // Create a simple box mesh with displacement (10mm x 10mm x 10mm centered at origin)
    let mut displacement_mesh = DisplacementMesh::new();

    // Box vertices
    // Bottom face (Z=0)
    displacement_mesh.vertices.push(Vertex::new(-5.0, -5.0, 0.0));  // 0
    displacement_mesh.vertices.push(Vertex::new(5.0, -5.0, 0.0));   // 1
    displacement_mesh.vertices.push(Vertex::new(5.0, 5.0, 0.0));    // 2
    displacement_mesh.vertices.push(Vertex::new(-5.0, 5.0, 0.0));   // 3
    // Top face (Z=10)
    displacement_mesh.vertices.push(Vertex::new(-5.0, -5.0, 10.0)); // 4
    displacement_mesh.vertices.push(Vertex::new(5.0, -5.0, 10.0));  // 5
    displacement_mesh.vertices.push(Vertex::new(5.0, 5.0, 10.0));   // 6
    displacement_mesh.vertices.push(Vertex::new(-5.0, 5.0, 10.0));  // 7

    // Add triangles with displacement coordinates
    // Bottom face (Z=0) - 2 triangles
    let mut tri = DisplacementTriangle::new(0, 2, 1);
    tri.did = Some(1);
    tri.d1 = Some(0);
    tri.d2 = Some(2);
    tri.d3 = Some(1);
    displacement_mesh.triangles.push(tri);

    let mut tri = DisplacementTriangle::new(0, 3, 2);
    tri.did = Some(1);
    tri.d1 = Some(0);
    tri.d2 = Some(3);
    tri.d3 = Some(2);
    displacement_mesh.triangles.push(tri);

    // Top face (Z=10) - 2 triangles
    let mut tri = DisplacementTriangle::new(4, 5, 6);
    tri.did = Some(1);
    tri.d1 = Some(4);
    tri.d2 = Some(5);
    tri.d3 = Some(6);
    displacement_mesh.triangles.push(tri);

    let mut tri = DisplacementTriangle::new(4, 6, 7);
    tri.did = Some(1);
    tri.d1 = Some(4);
    tri.d2 = Some(6);
    tri.d3 = Some(7);
    displacement_mesh.triangles.push(tri);

    // Side faces - 4 faces × 2 triangles = 8 triangles
    // Front face (Y=-5)
    let mut tri = DisplacementTriangle::new(0, 1, 5);
    tri.did = Some(1);
    tri.d1 = Some(0);
    tri.d2 = Some(1);
    tri.d3 = Some(5);
    displacement_mesh.triangles.push(tri);

    let mut tri = DisplacementTriangle::new(0, 5, 4);
    tri.did = Some(1);
    tri.d1 = Some(0);
    tri.d2 = Some(5);
    tri.d3 = Some(4);
    displacement_mesh.triangles.push(tri);

    // Right face (X=5)
    let mut tri = DisplacementTriangle::new(1, 2, 6);
    tri.did = Some(1);
    tri.d1 = Some(1);
    tri.d2 = Some(2);
    tri.d3 = Some(6);
    displacement_mesh.triangles.push(tri);

    let mut tri = DisplacementTriangle::new(1, 6, 5);
    tri.did = Some(1);
    tri.d1 = Some(1);
    tri.d2 = Some(6);
    tri.d3 = Some(5);
    displacement_mesh.triangles.push(tri);

    // Back face (Y=5)
    let mut tri = DisplacementTriangle::new(2, 3, 7);
    tri.did = Some(1);
    tri.d1 = Some(2);
    tri.d2 = Some(3);
    tri.d3 = Some(7);
    displacement_mesh.triangles.push(tri);

    let mut tri = DisplacementTriangle::new(2, 7, 6);
    tri.did = Some(1);
    tri.d1 = Some(2);
    tri.d2 = Some(7);
    tri.d3 = Some(6);
    displacement_mesh.triangles.push(tri);

    // Left face (X=-5)
    let mut tri = DisplacementTriangle::new(3, 0, 4);
    tri.did = Some(1);
    tri.d1 = Some(3);
    tri.d2 = Some(0);
    tri.d3 = Some(4);
    displacement_mesh.triangles.push(tri);

    let mut tri = DisplacementTriangle::new(3, 4, 7);
    tri.did = Some(1);
    tri.d1 = Some(3);
    tri.d2 = Some(4);
    tri.d3 = Some(7);
    displacement_mesh.triangles.push(tri);

    // Create object with displacement mesh
    let mut object = Object::new(1);
    object.displacement_mesh = Some(displacement_mesh);

    model.resources.objects.push(object);
    model.build.items.push(BuildItem::new(1));

    // Add displacement map (PNG texture)
    let displacement_map = Displacement2D::new(1, "/3D/Textures/displacement.png".to_string());
    model.resources.displacement_maps.push(displacement_map);

    // Add normal vector group (outward normals for each vertex)
    let mut norm_group = NormVectorGroup::new(1);
    // Bottom face vertices point down
    norm_group.vectors.push(NormVector::new(0.0, 0.0, -1.0)); // 0
    norm_group.vectors.push(NormVector::new(0.0, 0.0, -1.0)); // 1
    norm_group.vectors.push(NormVector::new(0.0, 0.0, -1.0)); // 2
    norm_group.vectors.push(NormVector::new(0.0, 0.0, -1.0)); // 3
    // Top face vertices point up
    norm_group.vectors.push(NormVector::new(0.0, 0.0, 1.0));  // 4
    norm_group.vectors.push(NormVector::new(0.0, 0.0, 1.0));  // 5
    norm_group.vectors.push(NormVector::new(0.0, 0.0, 1.0));  // 6
    norm_group.vectors.push(NormVector::new(0.0, 0.0, 1.0));  // 7
    model.resources.norm_vector_groups.push(norm_group);

    // Add displacement coordinate group
    let mut disp_group = Disp2DGroup::new(1, 1, 1, 1.5); // height=1.5mm displacement
    disp_group.offset = 0.0;
    
    // UV coordinates for each vertex (8 vertices = 8 coords)
    // Bottom vertices
    disp_group.coords.push(Disp2DCoords::new(0.0, 0.0, 0)); // vertex 0
    disp_group.coords.push(Disp2DCoords::new(1.0, 0.0, 1)); // vertex 1
    disp_group.coords.push(Disp2DCoords::new(1.0, 1.0, 2)); // vertex 2
    disp_group.coords.push(Disp2DCoords::new(0.0, 1.0, 3)); // vertex 3
    // Top vertices
    disp_group.coords.push(Disp2DCoords::new(0.0, 0.0, 4)); // vertex 4
    disp_group.coords.push(Disp2DCoords::new(1.0, 0.0, 5)); // vertex 5
    disp_group.coords.push(Disp2DCoords::new(1.0, 1.0, 6)); // vertex 6
    disp_group.coords.push(Disp2DCoords::new(0.0, 1.0, 7)); // vertex 7
    
    model.resources.disp2d_groups.push(disp_group);

    // Get output path from command line or use default
    let output_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "displacement_box.3mf".to_string());

    // Write the model
    model.write_to_file(&output_path)?;
    println!("Created: {}", output_path);

    println!("Sample displacement 3MF file created successfully!");
    println!("Note: You need to manually add the displacement texture PNG to the 3MF archive.");
    println!("The texture should be placed at /3D/Textures/displacement.png inside the ZIP.");
    
    Ok(())
}
