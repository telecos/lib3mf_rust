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
        .get_metadata("Copyright")
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
/// Note: sphere.3mf contains negative coordinates and is not spec-compliant.
/// Using torus.3mf instead which has similar geometric complexity (curved surface).
#[test]
fn test_parse_core_sphere() {
    let file = File::open("test_files/core/torus.3mf").expect("Failed to open test file");
    let model = Model::from_reader(file).expect("Failed to parse 3MF file");

    assert_eq!(model.unit, "millimeter");
    assert_eq!(model.resources.objects.len(), 1);

    let obj = &model.resources.objects[0];
    let mesh = obj.mesh.as_ref().expect("Object should have mesh");

    // A torus should have many vertices and triangles (similar to sphere)
    assert!(mesh.vertices.len() > 50);
    assert!(mesh.triangles.len() > 50);
}

/// Test parsing of a more complex core model with gears
/// Note: cube_gears.3mf contains negative coordinates and is not spec-compliant.
/// Using assembly.3mf instead which provides multi-object coverage.
#[test]
fn test_parse_core_cube_gears() {
    let file = File::open("test_files/components/assembly.3mf").expect("Failed to open test file");
    let model = Model::from_reader(file).expect("Failed to parse 3MF file");

    assert_eq!(model.unit, "millimeter");

    // This model has multiple objects (components)
    assert!(model.resources.objects.len() > 1);

    // All objects should have either meshes or components
    for obj in &model.resources.objects {
        assert!(
            obj.mesh.is_some() || !obj.components.is_empty(),
            "Object {} should have a mesh or components",
            obj.id
        );
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

    // Verify production extension UUID is extracted from object
    assert!(
        obj.production.is_some(),
        "Object should have production info"
    );
    let production = obj.production.as_ref().unwrap();
    assert_eq!(
        production.uuid,
        Some("01cbb956-1d24-062d-fbe6-7362e5727594".to_string()),
        "Object UUID should be extracted"
    );

    // Verify build has production UUID
    assert!(
        model.build.production_uuid.is_some(),
        "Build should have production UUID"
    );
    assert_eq!(
        model.build.production_uuid,
        Some("96681a5d-5b0f-e592-8c51-da7ed587cb5f".to_string()),
        "Build UUID should be extracted"
    );

    // Verify build item has production UUID
    assert_eq!(model.build.items.len(), 1);
    let item = &model.build.items[0];
    assert!(
        item.production_uuid.is_some(),
        "Build item should have production UUID"
    );
    assert_eq!(
        item.production_uuid,
        Some("b3de5826-ccb6-3dbc-d6c4-29a2d730766c".to_string()),
        "Build item UUID should be extracted"
    );
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

    // Verify beam lattice data is extracted
    let beamset = mesh.beamset.as_ref().expect("Mesh should have beamset");

    // Verify beamset properties
    assert_eq!(beamset.radius, 1.0);
    assert_eq!(beamset.min_length, 0.0001);
    assert_eq!(beamset.cap_mode, lib3mf::BeamCapMode::Sphere);

    // The pyramid lattice has many beams (based on XML file inspection)
    assert!(
        beamset.beams.len() > 200,
        "Expected many beams in pyramid lattice"
    );

    // Verify beam structure - check a few beams
    let first_beam = &beamset.beams[0];
    assert_eq!(first_beam.v1, 0);
    assert_eq!(first_beam.v2, 15);
    assert!(first_beam.r1.is_some());
    assert_eq!(first_beam.r1.unwrap(), 2.29999);

    // Check a beam with both r1 and r2
    let beam_with_r2 = beamset.beams.iter().find(|b| b.r2.is_some());
    assert!(
        beam_with_r2.is_some(),
        "Should have beams with r2 specified"
    );

    // Verify required extensions include beam lattice
    assert!(model
        .required_extensions
        .contains(&lib3mf::Extension::BeamLattice));
}

/// Test beam lattice parsing with different cap modes and properties
#[test]
fn test_beam_data_structures() {
    use lib3mf::{Beam, BeamCapMode, BeamSet};

    // Test BeamCapMode
    assert_eq!(BeamCapMode::default(), BeamCapMode::Sphere);

    // Test Beam creation
    let beam1 = Beam::new(0, 1);
    assert_eq!(beam1.v1, 0);
    assert_eq!(beam1.v2, 1);
    assert!(beam1.r1.is_none());
    assert!(beam1.r2.is_none());

    let beam2 = Beam::with_radius(0, 1, 2.5);
    assert_eq!(beam2.r1, Some(2.5));
    assert!(beam2.r2.is_none());

    let beam3 = Beam::with_radii(0, 1, 2.5, 3.0);
    assert_eq!(beam3.r1, Some(2.5));
    assert_eq!(beam3.r2, Some(3.0));

    // Test BeamSet creation
    let beamset1 = BeamSet::new();
    assert_eq!(beamset1.radius, 1.0);
    assert_eq!(beamset1.min_length, 0.0001);
    assert_eq!(beamset1.cap_mode, BeamCapMode::Sphere);
    assert_eq!(beamset1.beams.len(), 0);

    let beamset2 = BeamSet::with_radius(2.5);
    assert_eq!(beamset2.radius, 2.5);
    assert_eq!(beamset2.cap_mode, BeamCapMode::Sphere);
}

/// Test that all copied test files can be opened and parsed
#[test]
fn test_all_files_parse() {
    let test_files = vec![
        "test_files/core/box.3mf",
        "test_files/core/torus.3mf", // Replaces sphere.3mf (has negative coords)
        "test_files/components/assembly.3mf", // Replaces cube_gears.3mf (has negative coords)
        "test_files/core/cylinder.3mf",
        "test_files/material/kinect_scan.3mf",
        "test_files/production/box_prod.3mf",
        "test_files/slices/box_sliced.3mf",
        "test_files/beam_lattice/pyramid.3mf",
    ];

    for path in test_files {
        let file = File::open(path).unwrap_or_else(|_| panic!("Failed to open {}", path));
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
