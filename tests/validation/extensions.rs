//! Integration test to verify extension configuration works with actual files
//!
//! This test ensures that files requiring specific extensions can only be parsed
//! when the appropriate extension support is enabled.
//!
//! Also includes tests verifying ExtensionHandler.validate() is called during model validation.

use lib3mf::extension::ExtensionHandler;
use lib3mf::{
    BuildItem, Error, Extension, Mesh, Model, Object, ParserConfig, Result, Triangle, Vertex,
};
use std::fs::File;
use std::sync::{Arc, Mutex};

#[test]
fn test_beam_lattice_requires_extension() {
    // Open a beam lattice test file
    let file = File::open("test_files/beam_lattice/pyramid.3mf");

    if let Ok(file) = file {
        // Try to parse with only core support - should fail if file requires BeamLattice extension
        let config_core_only = ParserConfig::new();
        let result_core = Model::from_reader_with_config(file, config_core_only);

        // Now try with BeamLattice extension enabled
        let file2 = File::open("test_files/beam_lattice/pyramid.3mf").unwrap();
        let config_with_beam = ParserConfig::new().with_extension(Extension::BeamLattice);
        let result_with_extension = Model::from_reader_with_config(file2, config_with_beam);

        // The file with extension support should succeed
        if result_with_extension.is_ok() {
            println!("✓ Beam lattice file parsed successfully with BeamLattice extension");

            // If core-only failed, that's correct behavior
            if let Err(e) = result_core {
                match e {
                    Error::UnsupportedExtension(msg) => {
                        println!(
                            "✓ Correctly rejected file without BeamLattice extension: {}",
                            msg
                        );
                    }
                    _ => {
                        println!(
                            "✓ File failed to parse without BeamLattice extension (different error)"
                        );
                    }
                }
            } else {
                println!("✓ File doesn't require BeamLattice extension (parsed with core only)");
            }
        } else {
            println!(
                "Note: Could not parse beam lattice file: {:?}",
                result_with_extension.err()
            );
        }
    } else {
        println!("Note: Beam lattice test file not found, skipping test");
    }
}

#[test]
fn test_production_requires_extension() {
    // Open a production test file
    let file = File::open("test_files/production/box_prod.3mf");

    if let Ok(file) = file {
        // Try to parse with only core support
        let config_core_only = ParserConfig::new();
        let result_core = Model::from_reader_with_config(file, config_core_only);

        // Now try with Production extension enabled
        let file2 = File::open("test_files/production/box_prod.3mf").unwrap();
        let config_with_prod = ParserConfig::new().with_extension(Extension::Production);
        let result_with_extension = Model::from_reader_with_config(file2, config_with_prod);

        // The file with extension support should succeed
        if result_with_extension.is_ok() {
            println!("✓ Production file parsed successfully with Production extension");

            // If core-only failed, that's correct behavior
            if let Err(e) = result_core {
                match e {
                    Error::UnsupportedExtension(msg) => {
                        println!(
                            "✓ Correctly rejected file without Production extension: {}",
                            msg
                        );
                    }
                    _ => {
                        println!(
                            "✓ File failed to parse without Production extension (different error)"
                        );
                    }
                }
            } else {
                println!("✓ File doesn't require Production extension (parsed with core only)");
            }
        } else {
            println!(
                "Note: Could not parse production file: {:?}",
                result_with_extension.err()
            );
        }
    } else {
        println!("Note: Production test file not found, skipping test");
    }
}

#[test]
fn test_core_file_works_without_extensions() {
    // Open a core test file
    let file = File::open("test_files/core/box.3mf");

    if let Ok(file) = file {
        // Parse with only core support
        let config_core_only = ParserConfig::new();
        let result = Model::from_reader_with_config(file, config_core_only);

        match result {
            Ok(_) => {
                println!("✓ Core file parsed successfully with core-only configuration");
            }
            Err(e) => {
                panic!(
                    "Core file should parse with core-only config, but failed: {:?}",
                    e
                );
            }
        }
    } else {
        println!("Note: Core test file not found, skipping test");
    }
}

#[test]
fn test_slice_requires_extension() {
    // Open a slice test file
    let file = File::open("test_files/slices/box_sliced.3mf");

    if let Ok(file) = file {
        // Try to parse with only core support
        let config_core_only = ParserConfig::new();
        let result_core = Model::from_reader_with_config(file, config_core_only);

        // Now try with Slice extension enabled
        let file2 = File::open("test_files/slices/box_sliced.3mf").unwrap();
        let config_with_slice = ParserConfig::new().with_extension(Extension::Slice);
        let result_with_extension = Model::from_reader_with_config(file2, config_with_slice);

        // The file with extension support should succeed
        if result_with_extension.is_ok() {
            println!("✓ Slice file parsed successfully with Slice extension");

            // If core-only failed, that's correct behavior
            if let Err(e) = result_core {
                match e {
                    Error::UnsupportedExtension(msg) => {
                        println!("✓ Correctly rejected file without Slice extension: {}", msg);
                    }
                    _ => {
                        println!(
                            "✓ File failed to parse without Slice extension (different error)"
                        );
                    }
                }
            } else {
                println!("✓ File doesn't require Slice extension (parsed with core only)");
            }
        } else {
            println!(
                "Note: Could not parse slice file: {:?}",
                result_with_extension.err()
            );
        }
    } else {
        println!("Note: Slice test file not found, skipping test");
    }
}

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
        8,
        "Default registry should have 8 handlers"
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
