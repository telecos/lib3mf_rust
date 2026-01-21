//! Tests for 3MF writing/serialization functionality

use lib3mf::{
    BaseMaterial, BaseMaterialGroup, BuildItem, ColorGroup, Mesh, Model, Object,
    Triangle, Vertex,
};
use lib3mf::model::MetadataEntry;
use std::io::Cursor;

/// Test writing a minimal model
#[test]
fn test_write_minimal_model() {
    let mut model = Model::new();

    // Add a minimal object to satisfy validation
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));

    let mut object = Object::new(1);
    object.mesh = Some(mesh);
    model.resources.objects.push(object);
    model.build.items.push(BuildItem::new(1));

    let buffer = Vec::new();
    let cursor = Cursor::new(buffer);

    let result = model.to_writer(cursor);
    assert!(result.is_ok(), "Failed to write minimal model");

    let cursor = result.unwrap();
    let buffer = cursor.into_inner();

    // Verify the buffer is not empty
    assert!(!buffer.is_empty(), "Written buffer should not be empty");

    // Verify it's a valid ZIP file by trying to parse it
    let cursor = Cursor::new(buffer);
    let parsed = Model::from_reader(cursor);
    assert!(parsed.is_ok(), "Failed to parse written model");
}

/// Test round-trip: write then read
#[test]
fn test_roundtrip_basic() {
    let mut model = Model::new();
    model.unit = "millimeter".to_string();
    model.metadata.push(MetadataEntry::new(
        "Title".to_string(),
        "Test Model".to_string(),
    ));
    model.metadata.push(MetadataEntry::new(
        "Designer".to_string(),
        "lib3mf_rust".to_string(),
    ));

    // Add a simple object
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));

    let mut object = Object::new(1);
    object.mesh = Some(mesh);

    model.resources.objects.push(object);
    model.build.items.push(BuildItem::new(1));

    // Write the model
    let buffer = Vec::new();
    let cursor = Cursor::new(buffer);
    let cursor = model.to_writer(cursor).expect("Failed to write model");

    // Read it back
    let buffer = cursor.into_inner();
    let cursor = Cursor::new(buffer);
    let parsed_model = Model::from_reader(cursor).expect("Failed to parse written model");

    // Verify the data
    assert_eq!(parsed_model.unit, "millimeter");
    assert_eq!(parsed_model.metadata.len(), 2);
    assert_eq!(parsed_model.resources.objects.len(), 1);
    assert_eq!(parsed_model.build.items.len(), 1);

    let parsed_obj = &parsed_model.resources.objects[0];
    assert_eq!(parsed_obj.id, 1);

    let parsed_mesh = parsed_obj.mesh.as_ref().unwrap();
    assert_eq!(parsed_mesh.vertices.len(), 3);
    assert_eq!(parsed_mesh.triangles.len(), 1);

    // Verify vertex values
    assert_eq!(parsed_mesh.vertices[0].x, 0.0);
    assert_eq!(parsed_mesh.vertices[0].y, 0.0);
    assert_eq!(parsed_mesh.vertices[0].z, 0.0);

    assert_eq!(parsed_mesh.vertices[1].x, 10.0);
    assert_eq!(parsed_mesh.vertices[2].y, 10.0);

    // Verify triangle
    assert_eq!(parsed_mesh.triangles[0].v1, 0);
    assert_eq!(parsed_mesh.triangles[0].v2, 1);
    assert_eq!(parsed_mesh.triangles[0].v3, 2);
}

/// Test writing a tetrahedron mesh
#[test]
fn test_roundtrip_tetrahedron() {
    let mut model = Model::new();
    model.unit = "millimeter".to_string();

    // Create a tetrahedron
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(5.0, 8.66, 0.0));
    mesh.vertices.push(Vertex::new(5.0, 2.89, 8.16));

    mesh.triangles.push(Triangle::new(0, 1, 2));
    mesh.triangles.push(Triangle::new(0, 1, 3));
    mesh.triangles.push(Triangle::new(1, 2, 3));
    mesh.triangles.push(Triangle::new(2, 0, 3));

    let mut object = Object::new(1);
    object.mesh = Some(mesh);

    model.resources.objects.push(object);
    model.build.items.push(BuildItem::new(1));

    // Write and read back
    let buffer = Vec::new();
    let cursor = Cursor::new(buffer);
    let cursor = model.to_writer(cursor).expect("Failed to write model");

    let buffer = cursor.into_inner();
    let cursor = Cursor::new(buffer);
    let parsed_model = Model::from_reader(cursor).expect("Failed to parse written model");

    // Verify
    let parsed_mesh = parsed_model.resources.objects[0].mesh.as_ref().unwrap();
    assert_eq!(parsed_mesh.vertices.len(), 4);
    assert_eq!(parsed_mesh.triangles.len(), 4);
}

/// Test writing with base materials
#[test]
fn test_roundtrip_with_materials() {
    let mut model = Model::new();
    model.unit = "millimeter".to_string();

    // Add base material group
    let mut material_group = BaseMaterialGroup::new(1);
    material_group.materials.push(BaseMaterial::new(
        "Red".to_string(),
        (255, 0, 0, 255), // RGBA
    ));
    material_group
        .materials
        .push(BaseMaterial::new("Blue".to_string(), (0, 0, 255, 255)));
    model.resources.base_material_groups.push(material_group);

    // Add object with material references
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));

    let mut triangle = Triangle::new(0, 1, 2);
    triangle.pid = Some(1);
    triangle.pindex = Some(0);
    mesh.triangles.push(triangle);

    let mut object = Object::new(1);
    object.mesh = Some(mesh);
    model.resources.objects.push(object);

    model.build.items.push(BuildItem::new(1));

    // Write and read back
    let buffer = Vec::new();
    let cursor = Cursor::new(buffer);
    let cursor = model.to_writer(cursor).expect("Failed to write model");

    let buffer = cursor.into_inner();
    let cursor = Cursor::new(buffer);
    let parsed_model = Model::from_reader(cursor).expect("Failed to parse written model");

    // Verify materials
    assert_eq!(parsed_model.resources.base_material_groups.len(), 1);
    let parsed_group = &parsed_model.resources.base_material_groups[0];
    assert_eq!(parsed_group.id, 1);
    assert_eq!(parsed_group.materials.len(), 2);
    assert_eq!(parsed_group.materials[0].name, "Red");
    assert_eq!(parsed_group.materials[0].displaycolor, (255, 0, 0, 255));
    assert_eq!(parsed_group.materials[1].name, "Blue");
    assert_eq!(parsed_group.materials[1].displaycolor, (0, 0, 255, 255));

    // Verify triangle material reference
    let parsed_triangle = &parsed_model.resources.objects[0]
        .mesh
        .as_ref()
        .unwrap()
        .triangles[0];
    assert_eq!(parsed_triangle.pid, Some(1));
    assert_eq!(parsed_triangle.pindex, Some(0));
}

/// Test writing with color groups
#[test]
fn test_roundtrip_with_colors() {
    let mut model = Model::new();

    // Add color group
    let mut color_group = ColorGroup::new(1);
    color_group.colors.push((255, 0, 0, 255)); // Red
    color_group.colors.push((0, 255, 0, 255)); // Green
    color_group.colors.push((0, 0, 255, 255)); // Blue
    model.resources.color_groups.push(color_group);

    // Add a simple object
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));

    let mut object = Object::new(1);
    object.mesh = Some(mesh);
    model.resources.objects.push(object);
    model.build.items.push(BuildItem::new(1));

    // Write and read back
    let buffer = Vec::new();
    let cursor = Cursor::new(buffer);
    let cursor = model.to_writer(cursor).expect("Failed to write model");

    let buffer = cursor.into_inner();
    let cursor = Cursor::new(buffer);
    let parsed_model = Model::from_reader(cursor).expect("Failed to parse written model");

    // Verify colors
    assert_eq!(parsed_model.resources.color_groups.len(), 1);
    let parsed_colors = &parsed_model.resources.color_groups[0];
    assert_eq!(parsed_colors.id, 1);
    assert_eq!(parsed_colors.colors.len(), 3);
    assert_eq!(parsed_colors.colors[0], (255, 0, 0, 255));
    assert_eq!(parsed_colors.colors[1], (0, 255, 0, 255));
    assert_eq!(parsed_colors.colors[2], (0, 0, 255, 255));
}

/// Test writing with multiple objects
#[test]
fn test_roundtrip_multiple_objects() {
    let mut model = Model::new();

    // Create first object
    let mut mesh1 = Mesh::new();
    mesh1.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh1.vertices.push(Vertex::new(10.0, 0.0, 0.0));
    mesh1.vertices.push(Vertex::new(5.0, 10.0, 0.0));
    mesh1.triangles.push(Triangle::new(0, 1, 2));

    let mut object1 = Object::new(1);
    object1.name = Some("Triangle".to_string());
    object1.mesh = Some(mesh1);

    // Create second object
    let mut mesh2 = Mesh::new();
    mesh2.vertices.push(Vertex::new(20.0, 0.0, 0.0));
    mesh2.vertices.push(Vertex::new(30.0, 0.0, 0.0));
    mesh2.vertices.push(Vertex::new(25.0, 10.0, 0.0));
    mesh2.vertices.push(Vertex::new(25.0, 5.0, 10.0));
    mesh2.triangles.push(Triangle::new(0, 1, 2));
    mesh2.triangles.push(Triangle::new(0, 1, 3));

    let mut object2 = Object::new(2);
    object2.name = Some("Tetrahedron".to_string());
    object2.mesh = Some(mesh2);

    model.resources.objects.push(object1);
    model.resources.objects.push(object2);

    model.build.items.push(BuildItem::new(1));
    model.build.items.push(BuildItem::new(2));

    // Write and read back
    let buffer = Vec::new();
    let cursor = Cursor::new(buffer);
    let cursor = model.to_writer(cursor).expect("Failed to write model");

    let buffer = cursor.into_inner();
    let cursor = Cursor::new(buffer);
    let parsed_model = Model::from_reader(cursor).expect("Failed to parse written model");

    // Verify
    assert_eq!(parsed_model.resources.objects.len(), 2);
    assert_eq!(parsed_model.build.items.len(), 2);

    let obj1 = &parsed_model.resources.objects[0];
    assert_eq!(obj1.id, 1);
    assert_eq!(obj1.name, Some("Triangle".to_string()));
    assert_eq!(obj1.mesh.as_ref().unwrap().vertices.len(), 3);
    assert_eq!(obj1.mesh.as_ref().unwrap().triangles.len(), 1);

    let obj2 = &parsed_model.resources.objects[1];
    assert_eq!(obj2.id, 2);
    assert_eq!(obj2.name, Some("Tetrahedron".to_string()));
    assert_eq!(obj2.mesh.as_ref().unwrap().vertices.len(), 4);
    assert_eq!(obj2.mesh.as_ref().unwrap().triangles.len(), 2);
}

/// Test writing with transformation matrix
#[test]
fn test_roundtrip_with_transform() {
    let mut model = Model::new();

    // Create object
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));

    let mut object = Object::new(1);
    object.mesh = Some(mesh);
    model.resources.objects.push(object);

    // Add build item with transformation matrix
    let mut item = BuildItem::new(1);
    // Identity matrix with translation (10, 20, 30)
    item.transform = Some([
        1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 10.0, 20.0, 30.0,
    ]);
    model.build.items.push(item);

    // Write and read back
    let buffer = Vec::new();
    let cursor = Cursor::new(buffer);
    let cursor = model.to_writer(cursor).expect("Failed to write model");

    let buffer = cursor.into_inner();
    let cursor = Cursor::new(buffer);
    let parsed_model = Model::from_reader(cursor).expect("Failed to parse written model");

    // Verify transformation
    let parsed_item = &parsed_model.build.items[0];
    assert!(parsed_item.transform.is_some());
    let transform = parsed_item.transform.unwrap();
    assert_eq!(transform[0], 1.0);
    assert_eq!(transform[9], 10.0);
    assert_eq!(transform[10], 20.0);
    assert_eq!(transform[11], 30.0);
}

/// Test writing with metadata preservation flag
#[test]
fn test_roundtrip_metadata_preserve() {
    let mut model = Model::new();

    model.metadata.push(MetadataEntry::new_with_preserve(
        "Title".to_string(),
        "Test".to_string(),
        true,
    ));
    model.metadata.push(MetadataEntry::new_with_preserve(
        "Description".to_string(),
        "Test Description".to_string(),
        false,
    ));

    let mut object = Object::new(1);
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));
    object.mesh = Some(mesh);
    model.resources.objects.push(object);
    model.build.items.push(BuildItem::new(1));

    // Write and read back
    let buffer = Vec::new();
    let cursor = Cursor::new(buffer);
    let cursor = model.to_writer(cursor).expect("Failed to write model");

    let buffer = cursor.into_inner();
    let cursor = Cursor::new(buffer);
    let parsed_model = Model::from_reader(cursor).expect("Failed to parse written model");

    // Verify metadata
    assert_eq!(parsed_model.metadata.len(), 2);
    assert_eq!(parsed_model.metadata[0].name, "Title");
    assert_eq!(parsed_model.metadata[0].preserve, Some(true));
    assert_eq!(parsed_model.metadata[1].name, "Description");
    assert_eq!(parsed_model.metadata[1].preserve, Some(false));
}

/// Test writing to a file
#[test]
fn test_write_to_file() {
    let mut model = Model::new();
    model.unit = "millimeter".to_string();

    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));

    let mut object = Object::new(1);
    object.mesh = Some(mesh);
    model.resources.objects.push(object);
    model.build.items.push(BuildItem::new(1));

    // Write to temp file
    let temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
    let temp_path = temp_file.path();

    model
        .write_to_file(temp_path)
        .expect("Failed to write to file");

    // Read it back
    let file = std::fs::File::open(temp_path).expect("Failed to open written file");
    let parsed_model = Model::from_reader(file).expect("Failed to parse written file");

    // Verify
    assert_eq!(parsed_model.unit, "millimeter");
    assert_eq!(parsed_model.resources.objects.len(), 1);
}

/// Test that model with minimal content can be written and read
#[test]
fn test_minimal_content_roundtrip() {
    let mut model = Model::new();

    // Add minimal object to satisfy validation
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));

    let mut object = Object::new(1);
    object.mesh = Some(mesh);
    model.resources.objects.push(object);
    model.build.items.push(BuildItem::new(1));

    let buffer = Vec::new();
    let cursor = Cursor::new(buffer);
    let cursor = model.to_writer(cursor).expect("Failed to write model");

    let buffer = cursor.into_inner();
    let cursor = Cursor::new(buffer);
    let parsed_model = Model::from_reader(cursor).expect("Failed to parse model");

    // Verify basic structure
    assert_eq!(parsed_model.unit, "millimeter");
    assert_eq!(parsed_model.resources.objects.len(), 1);
    assert_eq!(parsed_model.build.items.len(), 1);
}
