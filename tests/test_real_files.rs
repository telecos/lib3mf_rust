//! Integration tests using real 3MF files from 3MF Consortium
//!
//! These tests validate parsing of actual 3MF files covering various extensions

use lib3mf::Model;
use std::fs::File;

/// Test parsing of a basic core specification 3MF file (box)
#[test]
fn test_parse_core_box() {
    let file = File::open("test_files/core/box.3mf").expect("Failed to open test file");
    let model = Model::from_reader(file).expect("Failed to parse 3MF file");

    // Verify basic model properties
    assert_eq!(model.unit, "millimeter");
    assert!(model
        .metadata
        .get("Copyright")
        .unwrap()
        .contains("3MF Consortium"));

    // Should have one object
    assert_eq!(model.resources.objects.len(), 1);

    let obj = &model.resources.objects[0];
    assert_eq!(obj.id, 1);

    // Verify mesh
    let mesh = obj.mesh.as_ref().expect("Object should have mesh");
    assert_eq!(mesh.vertices.len(), 8); // A box has 8 vertices
    assert_eq!(mesh.triangles.len(), 12); // A box has 12 triangles (2 per face)

    // Check a few vertices
    assert_eq!(mesh.vertices[0].x, 0.0);
    assert_eq!(mesh.vertices[0].y, 0.0);
    assert_eq!(mesh.vertices[0].z, 0.0);

    assert_eq!(mesh.vertices[6].x, 10.0);
    assert_eq!(mesh.vertices[6].y, 20.0);
    assert_eq!(mesh.vertices[6].z, 30.0);

    // Verify build items
    assert_eq!(model.build.items.len(), 1);
    assert_eq!(model.build.items[0].objectid, 1);
}

/// Test parsing of a core specification sphere
#[test]
fn test_parse_core_sphere() {
    let file = File::open("test_files/core/sphere.3mf").expect("Failed to open test file");
    let model = Model::from_reader(file).expect("Failed to parse 3MF file");

    assert_eq!(model.unit, "millimeter");
    assert_eq!(model.resources.objects.len(), 1);

    let obj = &model.resources.objects[0];
    let mesh = obj.mesh.as_ref().expect("Object should have mesh");

    // A sphere should have many vertices and triangles
    assert!(mesh.vertices.len() > 50);
    assert!(mesh.triangles.len() > 50);
}

/// Test parsing of a more complex core model with gears
#[test]
fn test_parse_core_cube_gears() {
    let file = File::open("test_files/core/cube_gears.3mf").expect("Failed to open test file");
    let model = Model::from_reader(file).expect("Failed to parse 3MF file");

    assert_eq!(model.unit, "millimeter");

    // This model has multiple objects
    assert!(model.resources.objects.len() > 1);

    // All objects should have meshes
    for obj in &model.resources.objects {
        assert!(obj.mesh.is_some(), "Object {} should have a mesh", obj.id);
    }
}

/// Test parsing of a material extension file with color groups
#[test]
fn test_parse_material_kinect_scan() {
    let file = File::open("test_files/material/kinect_scan.3mf").expect("Failed to open test file");
    let model = Model::from_reader(file).expect("Failed to parse 3MF file");

    assert_eq!(model.unit, "millimeter");
    
    // This file uses materials extension with color groups
    assert_eq!(model.resources.color_groups.len(), 1);
    
    // The color group should have many colors
    let colorgroup = &model.resources.color_groups[0];
    assert_eq!(colorgroup.id, 2);
    assert!(colorgroup.colors.len() > 1000); // Kinect scan has many colors
    
    // Check first color
    assert_eq!(colorgroup.colors[0], (0x4F, 0x47, 0x2F, 0xFF));
    
    assert_eq!(model.resources.objects.len(), 1);
    
    let obj = &model.resources.objects[0];
    let mesh = obj.mesh.as_ref().expect("Object should have mesh");
    
    // The kinect scan should have many vertices and triangles
    assert!(mesh.vertices.len() > 100);
    assert!(mesh.triangles.len() > 100);
    
    // Triangles should reference colors via pid
    let triangles_with_pid = mesh.triangles.iter().filter(|t| t.pid.is_some()).count();
    assert!(triangles_with_pid > 0);
}

/// Test parsing of production extension file
#[test]
fn test_parse_production_box() {
    let file = File::open("test_files/production/box_prod.3mf").expect("Failed to open test file");
    let model = Model::from_reader(file).expect("Failed to parse 3MF file");

    assert_eq!(model.unit, "millimeter");
    assert_eq!(model.resources.objects.len(), 1);

    let obj = &model.resources.objects[0];
    assert_eq!(obj.name, Some("box".to_string()));

    let mesh = obj.mesh.as_ref().expect("Object should have mesh");
    assert_eq!(mesh.vertices.len(), 8);
    assert_eq!(mesh.triangles.len(), 12);

    // Production extension adds UUID attributes which we may not parse yet
    // But the file should still parse successfully
}

/// Test parsing of slice extension file
#[test]
fn test_parse_slice_box() {
    let file = File::open("test_files/slices/box_sliced.3mf").expect("Failed to open test file");
    let model = Model::from_reader(file).expect("Failed to parse 3MF file");

    assert_eq!(model.unit, "millimeter");
    assert_eq!(model.resources.objects.len(), 1);

    let obj = &model.resources.objects[0];
    assert_eq!(obj.name, Some("box".to_string()));

    let mesh = obj.mesh.as_ref().expect("Object should have mesh");
    assert_eq!(mesh.vertices.len(), 8);
    assert_eq!(mesh.triangles.len(), 12);

    // Verify that the build item has a transformation
    assert_eq!(model.build.items.len(), 1);
    assert!(model.build.items[0].transform.is_some());

    let transform = model.build.items[0].transform.unwrap();
    // Check identity rotation part (first 9 values should be identity matrix)
    assert_eq!(transform[0], 1.0);
    assert_eq!(transform[4], 1.0);
    assert_eq!(transform[8], 1.0);
}

/// Test parsing of beam lattice extension file
#[test]
fn test_parse_beam_lattice_pyramid() {
    let file = File::open("test_files/beam_lattice/pyramid.3mf").expect("Failed to open test file");
    let model = Model::from_reader(file).expect("Failed to parse 3MF file");

    assert_eq!(model.unit, "millimeter");
    assert_eq!(model.resources.objects.len(), 1);

    let obj = &model.resources.objects[0];
    assert_eq!(obj.name, Some("Pyramid Lattice".to_string()));

    let mesh = obj.mesh.as_ref().expect("Object should have mesh");
    
    // Beam lattice uses vertices but may have empty triangles array
    assert!(!mesh.vertices.is_empty());
    
    // The pyramid lattice has many vertices for the beam structure
    // Actual count may vary but should be substantial for a lattice
    assert!(mesh.vertices.len() > 100);
}

/// Test that all copied test files can be opened and parsed
#[test]
fn test_all_files_parse() {
    let test_files = vec![
        "test_files/core/box.3mf",
        "test_files/core/sphere.3mf",
        "test_files/core/cube_gears.3mf",
        "test_files/core/cylinder.3mf",
        "test_files/core/torus.3mf",
        "test_files/material/kinect_scan.3mf",
        "test_files/production/box_prod.3mf",
        "test_files/slices/box_sliced.3mf",
        "test_files/beam_lattice/pyramid.3mf",
    ];

    for path in test_files {
        let file = File::open(path).expect(&format!("Failed to open {}", path));
        let result = Model::from_reader(file);
        assert!(
            result.is_ok(),
            "Failed to parse {}: {:?}",
            path,
            result.err()
        );
    }
}

/// Test parsing of cylinder (additional core test)
#[test]
fn test_parse_core_cylinder() {
    let file = File::open("test_files/core/cylinder.3mf").expect("Failed to open test file");
    let model = Model::from_reader(file).expect("Failed to parse 3MF file");

    assert_eq!(model.unit, "millimeter");
    assert_eq!(model.resources.objects.len(), 1);

    let obj = &model.resources.objects[0];
    let mesh = obj.mesh.as_ref().expect("Object should have mesh");

    // A cylinder has many triangles
    assert!(mesh.vertices.len() > 20);
    assert!(mesh.triangles.len() > 20);
}

/// Test parsing of torus (additional core test)
#[test]
fn test_parse_core_torus() {
    let file = File::open("test_files/core/torus.3mf").expect("Failed to open test file");
    let model = Model::from_reader(file).expect("Failed to parse 3MF file");

    assert_eq!(model.unit, "millimeter");
    assert_eq!(model.resources.objects.len(), 1);

    let obj = &model.resources.objects[0];
    let mesh = obj.mesh.as_ref().expect("Object should have mesh");

    // A torus has many vertices and triangles
    assert!(mesh.vertices.len() > 100);
    assert!(mesh.triangles.len() > 100);
}
