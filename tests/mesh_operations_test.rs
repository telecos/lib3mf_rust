//! Integration test for mesh operations
//!
//! This test demonstrates the new mesh operation capabilities using parry3d:
//! - Volume computation for validating mesh integrity
//! - Bounding box calculation for spatial queries
//! - Affine transformations for build volume validation
//!
//! These capabilities help address the requirements in tests N_XXX_0418/0420/0421

use lib3mf::{mesh_ops, BuildItem, Mesh, Model, Object, Triangle, Vertex};

#[test]
fn test_mesh_volume_computation() {
    // Create a simple 10x20x30 box
    let mut mesh = Mesh::new();

    // Vertices of a box
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0)); // 0
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0)); // 1
    mesh.vertices.push(Vertex::new(10.0, 20.0, 0.0)); // 2
    mesh.vertices.push(Vertex::new(0.0, 20.0, 0.0)); // 3
    mesh.vertices.push(Vertex::new(0.0, 0.0, 30.0)); // 4
    mesh.vertices.push(Vertex::new(10.0, 0.0, 30.0)); // 5
    mesh.vertices.push(Vertex::new(10.0, 20.0, 30.0)); // 6
    mesh.vertices.push(Vertex::new(0.0, 20.0, 30.0)); // 7

    // Triangles with correct winding order
    mesh.triangles.push(Triangle::new(3, 2, 1));
    mesh.triangles.push(Triangle::new(1, 0, 3));
    mesh.triangles.push(Triangle::new(4, 5, 6));
    mesh.triangles.push(Triangle::new(6, 7, 4));
    mesh.triangles.push(Triangle::new(0, 1, 5));
    mesh.triangles.push(Triangle::new(5, 4, 0));
    mesh.triangles.push(Triangle::new(1, 2, 6));
    mesh.triangles.push(Triangle::new(6, 5, 1));
    mesh.triangles.push(Triangle::new(2, 3, 7));
    mesh.triangles.push(Triangle::new(7, 6, 2));
    mesh.triangles.push(Triangle::new(3, 0, 4));
    mesh.triangles.push(Triangle::new(4, 7, 3));

    // Test signed volume (should be positive for correct winding)
    let signed_volume = mesh_ops::compute_mesh_signed_volume(&mesh).unwrap();
    assert!(
        signed_volume > 0.0,
        "Correctly oriented mesh should have positive signed volume"
    );

    // Expected volume: 10 * 20 * 30 = 6000
    assert!(
        (signed_volume - 6000.0).abs() < 1.0,
        "Volume should be approximately 6000, got {}",
        signed_volume
    );

    // Test unsigned volume
    let volume = mesh_ops::compute_mesh_volume(&mesh).unwrap();
    assert!(
        (volume - 6000.0).abs() < 1.0,
        "Volume should be approximately 6000, got {}",
        volume
    );
}

#[test]
fn test_bounding_box_computation() {
    let mut mesh = Mesh::new();

    // Create a triangle in 3D space
    mesh.vertices.push(Vertex::new(-5.0, -10.0, 0.0));
    mesh.vertices.push(Vertex::new(15.0, 5.0, 20.0));
    mesh.vertices.push(Vertex::new(3.0, 25.0, 8.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));

    let (min, max) = mesh_ops::compute_mesh_aabb(&mesh).unwrap();

    // Check bounds
    assert_eq!(min, (-5.0, -10.0, 0.0));
    assert_eq!(max, (15.0, 25.0, 20.0));

    // Bounding box dimensions
    let width = max.0 - min.0;
    let height = max.1 - min.1;
    let depth = max.2 - min.2;

    assert_eq!(width, 20.0);
    assert_eq!(height, 35.0);
    assert_eq!(depth, 20.0);
}

#[test]
fn test_transformed_bounding_box() {
    let mut mesh = Mesh::new();

    // Simple 10x10x10 cube at origin - need at least 2 triangles to define the volume
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0)); // 0
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0)); // 1
    mesh.vertices.push(Vertex::new(10.0, 10.0, 0.0)); // 2
    mesh.vertices.push(Vertex::new(0.0, 10.0, 0.0)); // 3
    mesh.vertices.push(Vertex::new(0.0, 0.0, 10.0)); // 4
    mesh.vertices.push(Vertex::new(10.0, 0.0, 10.0)); // 5
    mesh.vertices.push(Vertex::new(10.0, 10.0, 10.0)); // 6
    mesh.vertices.push(Vertex::new(0.0, 10.0, 10.0)); // 7
    // Add at least two triangles to define the bounding box properly
    mesh.triangles.push(Triangle::new(0, 1, 2));
    mesh.triangles.push(Triangle::new(4, 5, 6));

    // Identity transform (no change)
    let identity = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    let (min, max) = mesh_ops::compute_transformed_aabb(&mesh, Some(&identity)).unwrap();
    assert_eq!(min, (0.0, 0.0, 0.0));
    assert_eq!(max, (10.0, 10.0, 10.0));

    // Translation by (100, 200, 300)
    let translation = [
        1.0, 0.0, 0.0, 100.0, 0.0, 1.0, 0.0, 200.0, 0.0, 0.0, 1.0, 300.0,
    ];
    let (min, max) = mesh_ops::compute_transformed_aabb(&mesh, Some(&translation)).unwrap();
    assert_eq!(min, (100.0, 200.0, 300.0));
    assert_eq!(max, (110.0, 210.0, 310.0));

    // Scale by 2x in all dimensions
    let scale = [2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0];
    let (min, max) = mesh_ops::compute_transformed_aabb(&mesh, Some(&scale)).unwrap();
    assert_eq!(min, (0.0, 0.0, 0.0));
    assert_eq!(max, (20.0, 20.0, 20.0));
}

#[test]
fn test_build_volume_computation() {
    let mut model = Model::new();

    // Create first object: 10x10x10 cube at origin
    let mut mesh1 = Mesh::new();
    mesh1.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh1.vertices.push(Vertex::new(10.0, 10.0, 10.0));
    mesh1.triangles.push(Triangle::new(0, 1, 0));

    let mut object1 = Object::new(1);
    object1.mesh = Some(mesh1);
    model.resources.objects.push(object1);

    // Create second object: 5x5x5 cube
    let mut mesh2 = Mesh::new();
    mesh2.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh2.vertices.push(Vertex::new(5.0, 5.0, 5.0));
    mesh2.triangles.push(Triangle::new(0, 1, 0));

    let mut object2 = Object::new(2);
    object2.mesh = Some(mesh2);
    model.resources.objects.push(object2);

    // Add first build item at origin
    model.build.items.push(BuildItem::new(1));

    // Add second build item translated by (20, 20, 20)
    let mut item2 = BuildItem::new(2);
    item2.transform = Some([
        1.0, 0.0, 0.0, 20.0, 0.0, 1.0, 0.0, 20.0, 0.0, 0.0, 1.0, 20.0,
    ]);
    model.build.items.push(item2);

    // Compute overall build volume
    let build_volume = mesh_ops::compute_build_volume(&model).unwrap();

    // First object spans (0,0,0) to (10,10,10)
    // Second object spans (20,20,20) to (25,25,25)
    // Overall should span (0,0,0) to (25,25,25)
    assert_eq!(build_volume.0, (0.0, 0.0, 0.0));
    assert_eq!(build_volume.1, (25.0, 25.0, 25.0));

    println!(
        "Build volume: min={:?}, max={:?}",
        build_volume.0, build_volume.1
    );
}

#[test]
fn test_detect_inverted_mesh() {
    // Create a cube with inverted winding order
    let mut mesh = Mesh::new();

    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0)); // 0
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0)); // 1
    mesh.vertices.push(Vertex::new(10.0, 10.0, 0.0)); // 2
    mesh.vertices.push(Vertex::new(0.0, 10.0, 0.0)); // 3
    mesh.vertices.push(Vertex::new(0.0, 0.0, 10.0)); // 4
    mesh.vertices.push(Vertex::new(10.0, 0.0, 10.0)); // 5
    mesh.vertices.push(Vertex::new(10.0, 10.0, 10.0)); // 6
    mesh.vertices.push(Vertex::new(0.0, 10.0, 10.0)); // 7

    // All triangles with INVERTED winding (clockwise instead of counter-clockwise)
    mesh.triangles.push(Triangle::new(1, 2, 3)); // inverted
    mesh.triangles.push(Triangle::new(3, 0, 1)); // inverted
    mesh.triangles.push(Triangle::new(6, 5, 4)); // inverted
    mesh.triangles.push(Triangle::new(4, 7, 6)); // inverted
    mesh.triangles.push(Triangle::new(5, 1, 0)); // inverted
    mesh.triangles.push(Triangle::new(0, 4, 5)); // inverted
    mesh.triangles.push(Triangle::new(6, 2, 1)); // inverted
    mesh.triangles.push(Triangle::new(1, 5, 6)); // inverted
    mesh.triangles.push(Triangle::new(7, 3, 2)); // inverted
    mesh.triangles.push(Triangle::new(2, 6, 7)); // inverted
    mesh.triangles.push(Triangle::new(4, 0, 3)); // inverted
    mesh.triangles.push(Triangle::new(3, 7, 4)); // inverted

    // Signed volume should be negative
    let signed_volume = mesh_ops::compute_mesh_signed_volume(&mesh).unwrap();
    assert!(
        signed_volume < 0.0,
        "Inverted mesh should have negative signed volume, got {}",
        signed_volume
    );

    // Unsigned volume should still be positive (absolute value)
    let volume = mesh_ops::compute_mesh_volume(&mesh).unwrap();
    assert!(
        volume > 0.0,
        "Unsigned volume should be positive, got {}",
        volume
    );
}

#[test]
fn test_build_volume_validation_integration() {
    let mut model = Model::new();

    // Create a mesh
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(10.0, 10.0, 10.0));
    mesh.triangles.push(Triangle::new(0, 1, 0));

    let mut object = Object::new(1);
    object.mesh = Some(mesh);
    model.resources.objects.push(object);

    // Test 1: Build item with transform that places mesh in valid space (positive coords)
    let mut item_valid = BuildItem::new(1);
    item_valid.transform = Some([
        1.0, 0.0, 0.0, 5.0, // Translation by (5, 0, 0)
        0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
    ]);

    let (min, max) = mesh_ops::compute_transformed_aabb(
        &model.resources.objects[0].mesh.as_ref().unwrap(),
        item_valid.transform.as_ref(),
    )
    .unwrap();

    // Mesh should be at (5, 0, 0) to (15, 10, 10)
    assert!(max.0 > 0.0 && max.1 > 0.0 && max.2 > 0.0);
    println!("Valid transform: min={:?}, max={:?}", min, max);

    // Test 2: Build item with transform that places mesh entirely in negative space
    // This would be caught by N_XPX_0421 validation
    let mut item_negative = BuildItem::new(1);
    item_negative.transform = Some([
        1.0, 0.0, 0.0, -50.0, // Translation by (-50, -50, -50)
        0.0, 1.0, 0.0, -50.0, 0.0, 0.0, 1.0, -50.0,
    ]);

    let (min, max) = mesh_ops::compute_transformed_aabb(
        &model.resources.objects[0].mesh.as_ref().unwrap(),
        item_negative.transform.as_ref(),
    )
    .unwrap();

    // All coordinates should be negative
    assert!(max.0 < 0.0 && max.1 < 0.0 && max.2 < 0.0);
    println!("Negative transform: min={:?}, max={:?}", min, max);
}
