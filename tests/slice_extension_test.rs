//! Tests for Slice Extension support

use lib3mf::Model;
use std::fs::File;

#[test]
fn test_slice_extension_parsing() {
    // Load the box_sliced.3mf test file
    let file = File::open("test_files/slices/box_sliced.3mf")
        .expect("Failed to open box_sliced.3mf test file");

    let model = Model::from_reader(file).expect("Failed to parse box_sliced.3mf");

    // Verify slice stacks are extracted
    assert!(
        !model.resources.slice_stacks.is_empty(),
        "Slice stacks should be extracted from the model"
    );

    // Verify the first slice stack
    let slice_stack = &model.resources.slice_stacks[0];
    assert_eq!(slice_stack.id, 1, "SliceStack ID should be 1");
    assert_eq!(slice_stack.zbottom, 0.0, "SliceStack zbottom should be 0.0");

    // Verify slice references
    assert_eq!(
        slice_stack.slice_refs.len(),
        1,
        "SliceStack should have 1 slice reference"
    );

    let slice_ref = &slice_stack.slice_refs[0];
    assert_eq!(
        slice_ref.slicestackid, 1,
        "SliceRef slicestackid should be 1"
    );
    assert_eq!(
        slice_ref.slicepath, "/2D/5321f611-9309-4ded-aa3a-0a0eff6be004.model",
        "SliceRef slicepath should match"
    );
}

#[test]
fn test_object_slicestackid_reference() {
    // Load the box_sliced.3mf test file
    let file = File::open("test_files/slices/box_sliced.3mf")
        .expect("Failed to open box_sliced.3mf test file");

    let model = Model::from_reader(file).expect("Failed to parse box_sliced.3mf");

    // Find the object with slicestackid
    let object_with_slice = model
        .resources
        .objects
        .iter()
        .find(|obj| obj.slicestackid.is_some())
        .expect("Should have at least one object with slicestackid");

    assert_eq!(
        object_with_slice.slicestackid,
        Some(1),
        "Object should reference slicestack 1"
    );
    assert_eq!(object_with_slice.id, 2, "Object ID should be 2");
}

#[test]
fn test_slice_data_structure() {
    // Load the box_sliced.3mf test file
    let file = File::open("test_files/slices/box_sliced.3mf")
        .expect("Failed to open box_sliced.3mf test file");

    let model = Model::from_reader(file).expect("Failed to parse box_sliced.3mf");

    // Verify slice stack exists
    assert_eq!(
        model.resources.slice_stacks.len(),
        1,
        "Should have exactly 1 slice stack"
    );

    // Verify that slices are loaded from the external file
    let slice_stack = &model.resources.slice_stacks[0];
    assert!(
        !slice_stack.slices.is_empty(),
        "Slices should be loaded from external slice file"
    );

    // Verify first slice has data
    let first_slice = &slice_stack.slices[0];
    assert!(
        first_slice.ztop > 0.0,
        "First slice should have a ztop value"
    );
    assert!(
        !first_slice.vertices.is_empty(),
        "First slice should have vertices"
    );
    assert!(
        !first_slice.polygons.is_empty(),
        "First slice should have polygons"
    );

    // Verify polygon structure
    let first_polygon = &first_slice.polygons[0];
    assert!(
        !first_polygon.segments.is_empty(),
        "First polygon should have segments"
    );
}
