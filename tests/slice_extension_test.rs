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
    assert!(!model.resources.slice_stacks.is_empty(), 
        "Slice stacks should be extracted from the model");
    
    // Verify the first slice stack
    let slice_stack = &model.resources.slice_stacks[0];
    assert_eq!(slice_stack.id, 1, "SliceStack ID should be 1");
    assert_eq!(slice_stack.zbottom, 0.0, "SliceStack zbottom should be 0.0");
    
    // Verify slice references
    assert_eq!(slice_stack.slice_refs.len(), 1, 
        "SliceStack should have 1 slice reference");
    
    let slice_ref = &slice_stack.slice_refs[0];
    assert_eq!(slice_ref.slicestackid, 1, "SliceRef slicestackid should be 1");
    assert_eq!(slice_ref.slicepath, "/2D/5321f611-9309-4ded-aa3a-0a0eff6be004.model",
        "SliceRef slicepath should match");
}

#[test]
fn test_object_slicestackid_reference() {
    // Load the box_sliced.3mf test file
    let file = File::open("test_files/slices/box_sliced.3mf")
        .expect("Failed to open box_sliced.3mf test file");
    
    let model = Model::from_reader(file).expect("Failed to parse box_sliced.3mf");
    
    // Find the object with slicestackid
    let object_with_slice = model.resources.objects.iter()
        .find(|obj| obj.slicestackid.is_some())
        .expect("Should have at least one object with slicestackid");
    
    assert_eq!(object_with_slice.slicestackid, Some(1), 
        "Object should reference slicestack 1");
    assert_eq!(object_with_slice.id, 2, "Object ID should be 2");
}

#[test]
fn test_slice_data_structure() {
    // Load the box_sliced.3mf test file
    let file = File::open("test_files/slices/box_sliced.3mf")
        .expect("Failed to open box_sliced.3mf test file");
    
    let model = Model::from_reader(file).expect("Failed to parse box_sliced.3mf");
    
    // Note: The actual slices are in a separate file referenced by sliceref
    // This test just verifies the slice stack structure is available
    assert_eq!(model.resources.slice_stacks.len(), 1,
        "Should have exactly 1 slice stack");
}
