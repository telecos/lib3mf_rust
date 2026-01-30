//! Test for displacement rendering functionality
//!
//! This test verifies that the displacement data structures are correctly
//! exported and can be used programmatically.

use lib3mf::{
    Disp2DCoords, Disp2DGroup, Displacement2D, DisplacementMesh, DisplacementTriangle, Model,
    NormVector, NormVectorGroup, Object, Vertex,
};

#[test]
fn test_displacement_types_exported() {
    // This test verifies that DisplacementMesh and DisplacementTriangle
    // are properly exported from the lib3mf crate

    // Create a displacement mesh
    let mut disp_mesh = DisplacementMesh::new();

    // Add vertices
    disp_mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    disp_mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
    disp_mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));

    // Add a displacement triangle
    let mut triangle = DisplacementTriangle::new(0, 1, 2);
    triangle.did = Some(1);
    triangle.d1 = Some(0);
    triangle.d2 = Some(1);
    triangle.d3 = Some(2);

    disp_mesh.triangles.push(triangle);

    // Create an object with the displacement mesh
    let mut obj = Object::new(1);
    obj.displacement_mesh = Some(disp_mesh);

    // Create a model and add resources
    let mut model = Model::new();

    // Add displacement map
    model.resources.displacement_maps.push(Displacement2D::new(
        1,
        "/3D/Textures/displacement.png".to_string(),
    ));

    // Add normal vector group
    let mut norm_group = NormVectorGroup::new(2);
    norm_group.vectors.push(NormVector::new(0.0, 0.0, 1.0));
    model.resources.norm_vector_groups.push(norm_group);

    // Add displacement coordinate group
    let mut disp_group = Disp2DGroup::new(1, 1, 2, 1.0);
    disp_group.coords.push(Disp2DCoords::new(0.0, 0.0, 0));
    disp_group.coords.push(Disp2DCoords::new(1.0, 0.0, 0));
    disp_group.coords.push(Disp2DCoords::new(0.5, 1.0, 0));
    model.resources.disp2d_groups.push(disp_group);

    // Add object to model
    model.resources.objects.push(obj);

    // Verify the structure
    assert_eq!(model.resources.objects.len(), 1);
    assert!(model.resources.objects[0].displacement_mesh.is_some());

    let disp_mesh = model.resources.objects[0]
        .displacement_mesh
        .as_ref()
        .unwrap();
    assert_eq!(disp_mesh.vertices.len(), 3);
    assert_eq!(disp_mesh.triangles.len(), 1);

    let triangle = &disp_mesh.triangles[0];
    assert_eq!(triangle.did, Some(1));
    assert_eq!(triangle.d1, Some(0));
    assert_eq!(triangle.d2, Some(1));
    assert_eq!(triangle.d3, Some(2));

    assert_eq!(model.resources.displacement_maps.len(), 1);
    assert_eq!(model.resources.norm_vector_groups.len(), 1);
    assert_eq!(model.resources.disp2d_groups.len(), 1);
}

#[test]
fn test_displacement_mesh_default() {
    // Test that DisplacementMesh has a proper default/new implementation
    let mesh = DisplacementMesh::new();
    assert_eq!(mesh.vertices.len(), 0);
    assert_eq!(mesh.triangles.len(), 0);
}

#[test]
fn test_displacement_triangle_new() {
    // Test DisplacementTriangle creation
    let triangle = DisplacementTriangle::new(0, 1, 2);
    assert_eq!(triangle.v1, 0);
    assert_eq!(triangle.v2, 1);
    assert_eq!(triangle.v3, 2);
    assert_eq!(triangle.did, None);
    assert_eq!(triangle.d1, None);
    assert_eq!(triangle.d2, None);
    assert_eq!(triangle.d3, None);
}
