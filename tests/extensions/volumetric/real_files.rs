//! Integration tests using real 3MF files with Volumetric extension
//!
//! These tests validate parsing of actual 3MF files containing volumetric data

use lib3mf::{Extension, Model};
use std::fs::File;

/// Test that volumetric 3MF files can be opened and extension is recognized
#[test]
fn test_volumetric_files_exist_and_open() {
    // Test simple volumetric file exists and can be opened
    let file = File::open("test_files/volumetric/simple_volumetric.3mf")
        .expect("Failed to open simple_volumetric.3mf test file");
    let model = Model::from_reader(file).expect("Failed to parse 3MF file");

    // Verify the extension is declared (parser recognizes it)
    assert!(
        model.required_extensions.contains(&Extension::Volumetric),
        "Volumetric extension should be recognized"
    );

    // Test properties file exists and can be opened
    let file2 = File::open("test_files/volumetric/volumetric_with_properties.3mf")
        .expect("Failed to open volumetric_with_properties.3mf test file");
    let model2 = Model::from_reader(file2).expect("Failed to parse 3MF file");

    // Verify the extension is declared
    assert!(
        model2.required_extensions.contains(&Extension::Volumetric),
        "Volumetric extension should be recognized in properties file"
    );
}

/// Test parsing a simple volumetric 3MF file with voxel grid
///
/// Note: This test is currently ignored because volumetric parsing is not yet implemented.
/// The test validates the structure that will be expected once parsing is added.
#[test]
#[ignore = "Volumetric parsing not yet implemented"]
fn test_parse_simple_volumetric() {
    let file = File::open("test_files/volumetric/simple_volumetric.3mf")
        .expect("Failed to open test file");
    let model = Model::from_reader(file).expect("Failed to parse 3MF file");

    // Verify the model has volumetric extension declared
    assert!(
        model.required_extensions.contains(&Extension::Volumetric),
        "Model should require Volumetric extension"
    );

    // Verify volumetric data is present
    assert_eq!(
        model.resources.volumetric_data.len(),
        1,
        "Should have one volumetric data resource"
    );

    let vol_data = &model.resources.volumetric_data[0];
    assert_eq!(vol_data.id, 1, "Volumetric data should have ID 1");

    // Verify boundary
    assert!(
        vol_data.boundary.is_some(),
        "Volumetric data should have boundary"
    );
    let boundary = vol_data.boundary.as_ref().unwrap();
    assert_eq!(boundary.min, (0.0, 0.0, 0.0));
    assert_eq!(boundary.max, (10.0, 10.0, 10.0));

    // Verify voxel grid
    assert!(
        vol_data.voxels.is_some(),
        "Volumetric data should have voxel grid"
    );
    let voxels = vol_data.voxels.as_ref().unwrap();
    assert_eq!(voxels.dimensions, (10, 10, 10));
    assert_eq!(voxels.spacing, Some((1.0, 1.0, 1.0)));

    // Verify individual voxels
    assert_eq!(voxels.voxels.len(), 3, "Should have 3 voxels");
    assert_eq!(voxels.voxels[0].position, (5, 5, 5));
    assert_eq!(voxels.voxels[1].position, (6, 5, 5));
    assert_eq!(voxels.voxels[2].position, (5, 6, 5));

    // Verify model also has a regular mesh object
    assert_eq!(
        model.resources.objects.len(),
        1,
        "Should have one regular object"
    );
    let obj = &model.resources.objects[0];
    assert_eq!(obj.id, 2);
    assert!(obj.mesh.is_some(), "Object should have a mesh");
}

/// Test parsing volumetric 3MF file with property groups
///
/// Note: This test is currently ignored because volumetric parsing is not yet implemented.
/// The test validates the structure that will be expected once parsing is added.
#[test]
#[ignore = "Volumetric parsing not yet implemented"]
fn test_parse_volumetric_with_properties() {
    let file = File::open("test_files/volumetric/volumetric_with_properties.3mf")
        .expect("Failed to open test file");
    let model = Model::from_reader(file).expect("Failed to parse 3MF file");

    // Verify volumetric extension is declared
    assert!(
        model.required_extensions.contains(&Extension::Volumetric),
        "Model should require Volumetric extension"
    );

    // Verify property groups
    assert_eq!(
        model.resources.volumetric_property_groups.len(),
        1,
        "Should have one volumetric property group"
    );
    let prop_group = &model.resources.volumetric_property_groups[0];
    assert_eq!(prop_group.id, 1);
    assert_eq!(
        prop_group.properties.len(),
        1,
        "Property group should have 1 property"
    );
    assert_eq!(prop_group.properties[0].index, 0);
    assert_eq!(prop_group.properties[0].value, "solid");

    // Verify volumetric data
    assert_eq!(
        model.resources.volumetric_data.len(),
        1,
        "Should have one volumetric data resource"
    );
    let vol_data = &model.resources.volumetric_data[0];
    assert_eq!(vol_data.id, 2);

    // Verify voxel grid
    let voxels = vol_data.voxels.as_ref().expect("Should have voxel grid");
    assert_eq!(voxels.dimensions, (5, 5, 5));
    assert_eq!(voxels.voxels.len(), 1, "Should have 1 voxel");

    // Verify voxel references property
    let voxel = &voxels.voxels[0];
    assert_eq!(voxel.position, (2, 2, 2));
    assert_eq!(
        voxel.property_id,
        Some(1),
        "Voxel should reference property group ID 1"
    );
}

/// Test that volumetric models can be validated
#[test]
fn test_volumetric_validation() {
    let file = File::open("test_files/volumetric/simple_volumetric.3mf")
        .expect("Failed to open test file");
    let model = Model::from_reader(file).expect("Failed to parse 3MF file");

    // Run volumetric validation
    let result = lib3mf::validator::validate_volumetric_extension(&model);
    assert!(
        result.is_ok(),
        "Volumetric validation should pass: {:?}",
        result.err()
    );
}

/// Test round-trip: parse and write volumetric data
///
/// Note: This test is currently ignored because volumetric parsing/writing is not yet implemented.
/// The test validates the structure that will be expected once parsing/writing is added.
#[test]
#[ignore = "Volumetric parsing/writing not yet implemented"]
fn test_volumetric_roundtrip() {
    use std::io::Cursor;

    // Parse the file
    let file = File::open("test_files/volumetric/simple_volumetric.3mf")
        .expect("Failed to open test file");
    let model = Model::from_reader(file).expect("Failed to parse 3MF file");

    // Write to memory using Cursor which implements Seek
    let buffer = Vec::new();
    let mut cursor = Cursor::new(buffer);
    model
        .to_writer(&mut cursor)
        .expect("Failed to write 3MF file");

    // Parse again
    cursor.set_position(0);
    let model2 = Model::from_reader(cursor).expect("Failed to parse written 3MF file");

    // Verify volumetric data is preserved
    assert_eq!(
        model2.resources.volumetric_data.len(),
        1,
        "Volumetric data should be preserved"
    );
    assert!(
        model2.required_extensions.contains(&Extension::Volumetric),
        "Extension should be preserved"
    );
}
