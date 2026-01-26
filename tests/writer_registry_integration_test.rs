//! Integration tests for ExtensionRegistry.pre_write_all() in writer flow

use lib3mf::extension::{ExtensionHandler, ExtensionRegistry};
use lib3mf::{BuildItem, Extension, Mesh, Model, Object, Triangle, Vertex};
use std::io::Cursor;
use std::sync::{Arc, Mutex};

/// Test extension handler that tracks when pre_write is called
struct TestPreWriteHandler {
    extension_type: Extension,
    call_count: Arc<Mutex<usize>>,
}

impl ExtensionHandler for TestPreWriteHandler {
    fn extension_type(&self) -> Extension {
        self.extension_type
    }

    fn validate(&self, _model: &Model) -> lib3mf::Result<()> {
        Ok(())
    }

    fn pre_write(&self, _model: &mut Model) -> lib3mf::Result<()> {
        // Increment call counter when pre_write is called
        let mut count = self.call_count.lock().unwrap();
        *count += 1;
        Ok(())
    }
}

#[test]
fn test_to_writer_with_registry_calls_pre_write() {
    // Create a simple model
    let mut model = Model::new();

    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));

    let mut object = Object::new(1);
    object.mesh = Some(mesh);
    model.resources.objects.push(object);
    model.build.items.push(BuildItem::new(1));

    // Create a registry with a test handler
    let call_count = Arc::new(Mutex::new(0));
    let handler = Arc::new(TestPreWriteHandler {
        extension_type: Extension::Material,
        call_count: call_count.clone(),
    });

    let mut registry = ExtensionRegistry::new();
    registry.register(handler);

    // Verify pre_write hasn't been called yet
    assert_eq!(
        *call_count.lock().unwrap(),
        0,
        "pre_write should not be called before writing"
    );

    // Write the model with registry
    let buffer = Vec::new();
    let cursor = Cursor::new(buffer);
    let result = model.to_writer_with_registry(cursor, &registry);

    assert!(result.is_ok(), "Failed to write model with registry");

    // Verify pre_write was called exactly once
    assert_eq!(
        *call_count.lock().unwrap(),
        1,
        "pre_write should be called exactly once"
    );
}

#[test]
fn test_to_writer_without_registry_does_not_call_pre_write() {
    // Create a simple model
    let mut model = Model::new();

    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));

    let mut object = Object::new(1);
    object.mesh = Some(mesh);
    model.resources.objects.push(object);
    model.build.items.push(BuildItem::new(1));

    // Write the model without registry (backward compatibility test)
    let buffer = Vec::new();
    let cursor = Cursor::new(buffer);
    let result = model.to_writer(cursor);

    assert!(
        result.is_ok(),
        "to_writer should work without registry (backward compatibility)"
    );
}

#[test]
fn test_multiple_handlers_all_called() {
    // Create a simple model
    let mut model = Model::new();

    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));

    let mut object = Object::new(1);
    object.mesh = Some(mesh);
    model.resources.objects.push(object);
    model.build.items.push(BuildItem::new(1));

    // Create registry with multiple handlers
    let call_count_1 = Arc::new(Mutex::new(0));
    let call_count_2 = Arc::new(Mutex::new(0));
    let call_count_3 = Arc::new(Mutex::new(0));

    let handler_1 = Arc::new(TestPreWriteHandler {
        extension_type: Extension::Material,
        call_count: call_count_1.clone(),
    });

    let handler_2 = Arc::new(TestPreWriteHandler {
        extension_type: Extension::Production,
        call_count: call_count_2.clone(),
    });

    let handler_3 = Arc::new(TestPreWriteHandler {
        extension_type: Extension::BeamLattice,
        call_count: call_count_3.clone(),
    });

    let mut registry = ExtensionRegistry::new();
    registry.register(handler_1);
    registry.register(handler_2);
    registry.register(handler_3);

    // Write the model with registry
    let buffer = Vec::new();
    let cursor = Cursor::new(buffer);
    let result = model.to_writer_with_registry(cursor, &registry);

    assert!(result.is_ok(), "Failed to write model with registry");

    // Verify all handlers were called
    assert_eq!(
        *call_count_1.lock().unwrap(),
        1,
        "Handler 1 should be called"
    );
    assert_eq!(
        *call_count_2.lock().unwrap(),
        1,
        "Handler 2 should be called"
    );
    assert_eq!(
        *call_count_3.lock().unwrap(),
        1,
        "Handler 3 should be called"
    );
}

#[test]
fn test_write_to_file_with_registry() {
    // Create a simple model
    let mut model = Model::new();

    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));

    let mut object = Object::new(1);
    object.mesh = Some(mesh);
    model.resources.objects.push(object);
    model.build.items.push(BuildItem::new(1));

    // Create a registry with a test handler
    let call_count = Arc::new(Mutex::new(0));
    let handler = Arc::new(TestPreWriteHandler {
        extension_type: Extension::Material,
        call_count: call_count.clone(),
    });

    let mut registry = ExtensionRegistry::new();
    registry.register(handler);

    // Write to a temporary file
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("test_writer_registry.3mf");

    let result = model.write_to_file_with_registry(&temp_file, &registry);
    assert!(
        result.is_ok(),
        "Failed to write to file with registry: {:?}",
        result.err()
    );

    // Verify pre_write was called
    assert_eq!(
        *call_count.lock().unwrap(),
        1,
        "pre_write should be called when writing to file"
    );

    // Clean up
    std::fs::remove_file(&temp_file).ok();
}

#[test]
fn test_roundtrip_with_registry() {
    // Create a model with some data
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

    // Create registry with handler
    let call_count = Arc::new(Mutex::new(0));
    let handler = Arc::new(TestPreWriteHandler {
        extension_type: Extension::Material,
        call_count: call_count.clone(),
    });

    let mut registry = ExtensionRegistry::new();
    registry.register(handler);

    // Write with registry
    let buffer = Vec::new();
    let cursor = Cursor::new(buffer);
    let cursor = model
        .to_writer_with_registry(cursor, &registry)
        .expect("Failed to write model");

    // Verify pre_write was called
    assert_eq!(*call_count.lock().unwrap(), 1);

    // Read it back and verify data integrity
    let buffer = cursor.into_inner();
    let cursor = Cursor::new(buffer);
    let parsed_model = Model::from_reader(cursor).expect("Failed to parse written model");

    assert_eq!(parsed_model.unit, "millimeter");
    assert_eq!(parsed_model.resources.objects.len(), 1);
    assert_eq!(parsed_model.build.items.len(), 1);

    let parsed_obj = &parsed_model.resources.objects[0];
    let parsed_mesh = parsed_obj.mesh.as_ref().unwrap();
    assert_eq!(parsed_mesh.vertices.len(), 3);
    assert_eq!(parsed_mesh.triangles.len(), 1);
}
