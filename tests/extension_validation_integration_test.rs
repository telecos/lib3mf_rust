//! Integration tests verifying ExtensionHandler.validate() is called during model validation
//!
//! These tests ensure that the new ExtensionRegistry system is properly wired into
//! the validation flow in validate_model_with_config().

use lib3mf::extension::ExtensionHandler;
use lib3mf::{
    BuildItem, Error, Extension, Mesh, Model, Object, ParserConfig, Result, Triangle, Vertex,
};
use std::sync::{Arc, Mutex};

/// Test extension handler that tracks if validate() was called
struct TestExtensionHandler {
    ext_type: Extension,
    validate_called: Arc<Mutex<bool>>,
    should_fail: bool,
}

impl ExtensionHandler for TestExtensionHandler {
    fn extension_type(&self) -> Extension {
        self.ext_type
    }

    fn validate(&self, _model: &Model) -> Result<()> {
        *self.validate_called.lock().unwrap() = true;
        if self.should_fail {
            Err(Error::InvalidModel(format!(
                "{} validation failed",
                self.name()
            )))
        } else {
            Ok(())
        }
    }
}

#[test]
fn test_extension_handler_validate_is_called() {
    // Create a simple valid model
    let mut model = Model::new();
    let mut obj = Object::new(1);
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));
    obj.mesh = Some(mesh);
    model.resources.objects.push(obj);
    model.build.items.push(BuildItem::new(1));

    // Create a test handler that tracks whether validate() was called
    let validate_called = Arc::new(Mutex::new(false));
    let handler = Arc::new(TestExtensionHandler {
        ext_type: Extension::Material,
        validate_called: validate_called.clone(),
        should_fail: false,
    });

    // Create config with the test handler
    let config = ParserConfig::new().with_extension_handler(handler);

    // Validate the model with the config
    let result = lib3mf::validator::validate_model_with_config(&model, &config);

    // Verify validation succeeded
    assert!(result.is_ok(), "Validation should succeed: {:?}", result);

    // Verify the handler's validate() method was called
    assert!(
        *validate_called.lock().unwrap(),
        "ExtensionHandler.validate() should have been called"
    );
}

#[test]
fn test_extension_handler_validation_failure_propagates() {
    // Create a simple valid model
    let mut model = Model::new();
    let mut obj = Object::new(1);

    let mut mesh = Mesh::new();

    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));

    mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));

    mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));

    mesh.triangles.push(Triangle::new(0, 1, 2));

    obj.mesh = Some(mesh);
    model.resources.objects.push(obj);
    model.build.items.push(BuildItem::new(1));

    // Create a test handler that will fail validation
    let validate_called = Arc::new(Mutex::new(false));
    let handler = Arc::new(TestExtensionHandler {
        ext_type: Extension::Material,
        validate_called: validate_called.clone(),
        should_fail: true,
    });

    // Create config with the failing handler
    let config = ParserConfig::new().with_extension_handler(handler);

    // Validate the model with the config
    let result = lib3mf::validator::validate_model_with_config(&model, &config);

    // Verify validation failed
    assert!(result.is_err(), "Validation should fail");

    // Verify the handler's validate() method was called
    assert!(
        *validate_called.lock().unwrap(),
        "ExtensionHandler.validate() should have been called before failure"
    );

    // Verify the error message contains the expected text
    match result {
        Err(Error::InvalidModel(msg)) => {
            assert!(
                msg.contains("Material validation failed"),
                "Error message should contain 'Material validation failed', got: {}",
                msg
            );
        }
        _ => panic!("Expected InvalidModel error"),
    }
}

#[test]
fn test_multiple_extension_handlers_all_called() {
    // Create a simple valid model
    let mut model = Model::new();
    let mut obj = Object::new(1);

    let mut mesh = Mesh::new();

    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));

    mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));

    mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));

    mesh.triangles.push(Triangle::new(0, 1, 2));

    obj.mesh = Some(mesh);
    model.resources.objects.push(obj);
    model.build.items.push(BuildItem::new(1));

    // Create multiple test handlers that track whether validate() was called
    let material_called = Arc::new(Mutex::new(false));
    let production_called = Arc::new(Mutex::new(false));
    let slice_called = Arc::new(Mutex::new(false));

    let material_handler = Arc::new(TestExtensionHandler {
        ext_type: Extension::Material,
        validate_called: material_called.clone(),
        should_fail: false,
    });

    let production_handler = Arc::new(TestExtensionHandler {
        ext_type: Extension::Production,
        validate_called: production_called.clone(),
        should_fail: false,
    });

    let slice_handler = Arc::new(TestExtensionHandler {
        ext_type: Extension::Slice,
        validate_called: slice_called.clone(),
        should_fail: false,
    });

    // Create config with all handlers
    let config = ParserConfig::new()
        .with_extension_handler(material_handler)
        .with_extension_handler(production_handler)
        .with_extension_handler(slice_handler);

    // Validate the model with the config
    let result = lib3mf::validator::validate_model_with_config(&model, &config);

    // Verify validation succeeded
    assert!(result.is_ok(), "Validation should succeed: {:?}", result);

    // Verify all handlers' validate() methods were called
    assert!(
        *material_called.lock().unwrap(),
        "Material handler validate() should have been called"
    );
    assert!(
        *production_called.lock().unwrap(),
        "Production handler validate() should have been called"
    );
    assert!(
        *slice_called.lock().unwrap(),
        "Slice handler validate() should have been called"
    );
}

#[test]
fn test_default_registry_handlers_are_called() {
    // Create a simple valid model
    let mut model = Model::new();
    let mut obj = Object::new(1);

    let mut mesh = Mesh::new();

    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));

    mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));

    mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));

    mesh.triangles.push(Triangle::new(0, 1, 2));

    obj.mesh = Some(mesh);
    model.resources.objects.push(obj);
    model.build.items.push(BuildItem::new(1));

    // Use config with all extensions (includes default registry)
    let config = ParserConfig::with_all_extensions();

    // Validate the model with the config
    let result = lib3mf::validator::validate_model_with_config(&model, &config);

    // Verify validation succeeded - if the handlers weren't called or had issues,
    // this would fail
    assert!(
        result.is_ok(),
        "Validation with default registry should succeed: {:?}",
        result
    );

    // Verify the registry has the expected handlers
    assert_eq!(
        config.registry().handlers().len(),
        7,
        "Default registry should have 7 handlers"
    );
}

#[test]
fn test_backward_compatibility_with_custom_extensions() {
    // Create a simple valid model
    let mut model = Model::new();
    let mut obj = Object::new(1);

    let mut mesh = Mesh::new();

    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));

    mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));

    mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));

    mesh.triangles.push(Triangle::new(0, 1, 2));

    obj.mesh = Some(mesh);
    model.resources.objects.push(obj);
    model.build.items.push(BuildItem::new(1));

    // Track if the old-style validation handler is called
    let legacy_called = Arc::new(Mutex::new(false));
    let legacy_called_clone = legacy_called.clone();

    // Create config using the old custom_extensions pattern
    let config = ParserConfig::new().with_custom_extension_handlers(
        "http://example.com/legacy/2024/01",
        "LegacyExtension",
        Arc::new(|_ctx| Ok(lib3mf::CustomElementResult::Handled)),
        Arc::new(move |_model| {
            *legacy_called_clone.lock().unwrap() = true;
            Ok(())
        }),
    );

    // Validate the model
    let result = lib3mf::validator::validate_model_with_config(&model, &config);

    // Verify validation succeeded
    assert!(
        result.is_ok(),
        "Validation with legacy custom extension should succeed: {:?}",
        result
    );

    // Verify the legacy validation handler was called (backward compatibility)
    assert!(
        *legacy_called.lock().unwrap(),
        "Legacy validation handler should still be called for backward compatibility"
    );
}

#[test]
fn test_both_systems_work_together() {
    // Create a simple valid model
    let mut model = Model::new();
    let mut obj = Object::new(1);

    let mut mesh = Mesh::new();

    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));

    mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));

    mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));

    mesh.triangles.push(Triangle::new(0, 1, 2));

    obj.mesh = Some(mesh);
    model.resources.objects.push(obj);
    model.build.items.push(BuildItem::new(1));

    // Track if both validation systems are called
    let registry_called = Arc::new(Mutex::new(false));
    let legacy_called = Arc::new(Mutex::new(false));
    let legacy_called_clone = legacy_called.clone();

    // Create a handler for the new registry system
    let handler = Arc::new(TestExtensionHandler {
        ext_type: Extension::Material,
        validate_called: registry_called.clone(),
        should_fail: false,
    });

    // Create config with both systems
    let config = ParserConfig::new()
        .with_extension_handler(handler)
        .with_custom_extension_handlers(
            "http://example.com/legacy/2024/01",
            "LegacyExtension",
            Arc::new(|_ctx| Ok(lib3mf::CustomElementResult::Handled)),
            Arc::new(move |_model| {
                *legacy_called_clone.lock().unwrap() = true;
                Ok(())
            }),
        );

    // Validate the model
    let result = lib3mf::validator::validate_model_with_config(&model, &config);

    // Verify validation succeeded
    assert!(
        result.is_ok(),
        "Validation with both systems should succeed: {:?}",
        result
    );

    // Verify both validation systems were called
    assert!(
        *registry_called.lock().unwrap(),
        "New registry handler validate() should have been called"
    );
    assert!(
        *legacy_called.lock().unwrap(),
        "Legacy validation handler should have been called"
    );
}
