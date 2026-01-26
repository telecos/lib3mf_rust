//! Integration test for post_parse hooks in the parsing flow
//!
//! This test verifies that ExtensionRegistry.post_parse_all() is correctly
//! called during the parsing process.

use lib3mf::{
    Extension, ExtensionHandler, ExtensionRegistry, Model, ParserConfig, Result as Lib3mfResult,
};
use std::io::{Cursor, Write};
use std::sync::{Arc, Mutex};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Test extension handler that tracks whether post_parse was called
struct TestExtensionHandler {
    post_parse_called: Arc<Mutex<bool>>,
}

impl TestExtensionHandler {
    fn new() -> (Self, Arc<Mutex<bool>>) {
        let flag = Arc::new(Mutex::new(false));
        (
            TestExtensionHandler {
                post_parse_called: flag.clone(),
            },
            flag,
        )
    }
}

impl ExtensionHandler for TestExtensionHandler {
    fn extension_type(&self) -> Extension {
        Extension::Material
    }

    fn validate(&self, _model: &Model) -> Lib3mfResult<()> {
        // No validation needed for this test
        Ok(())
    }

    fn post_parse(&self, _model: &mut Model) -> Lib3mfResult<()> {
        // Mark that post_parse was called
        *self.post_parse_called.lock().unwrap() = true;
        Ok(())
    }

    fn is_used_in_model(&self, _model: &Model) -> bool {
        // Always return true so post_parse is called
        true
    }
}

/// Create a minimal valid 3MF file for testing
fn create_test_3mf() -> Vec<u8> {
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    let mut zip = ZipWriter::new(cursor);

    let options = SimpleFileOptions::default();

    // Add [Content_Types].xml
    let content_types = r##"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
</Types>"##;

    zip.start_file("[Content_Types].xml", options).unwrap();
    zip.write_all(content_types.as_bytes()).unwrap();

    // Add _rels/.rels
    let rels = r##"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Target="/3D/3dmodel.model" Id="rel0" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"##;

    zip.start_file("_rels/.rels", options).unwrap();
    zip.write_all(rels.as_bytes()).unwrap();

    // Add 3D/3dmodel.model
    let model = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <metadata name="Title">Test Model</metadata>
  <resources>
    <object id="1" type="model">
      <mesh>
        <vertices>
          <vertex x="0.0" y="0.0" z="0.0"/>
          <vertex x="10.0" y="0.0" z="0.0"/>
          <vertex x="5.0" y="10.0" z="0.0"/>
          <vertex x="5.0" y="5.0" z="10.0"/>
        </vertices>
        <triangles>
          <triangle v1="0" v2="1" v3="2"/>
          <triangle v1="0" v2="1" v3="3"/>
          <triangle v1="1" v2="2" v3="3"/>
          <triangle v1="2" v2="0" v3="3"/>
        </triangles>
      </mesh>
    </object>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"##;

    zip.start_file("3D/3dmodel.model", options).unwrap();
    zip.write_all(model.as_bytes()).unwrap();

    zip.finish().unwrap();
    buffer
}

#[test]
fn test_post_parse_called_during_parsing() {
    // Create test handler that tracks post_parse calls
    let (handler, post_parse_called) = TestExtensionHandler::new();

    // Create a registry and register the test handler
    let mut registry = ExtensionRegistry::new();
    registry.register(Arc::new(handler));

    // Create parser config with the registry
    let mut config = ParserConfig::new().with_extension(Extension::Material);
    *config.registry_mut() = registry;

    // Parse a 3MF file
    let data = create_test_3mf();
    let cursor = Cursor::new(data);
    let result = lib3mf::parser::parse_3mf_with_config(cursor, config);

    // Verify parsing succeeded
    assert!(
        result.is_ok(),
        "Parsing should succeed: {:?}",
        result.err()
    );

    // Verify that post_parse was called
    assert!(
        *post_parse_called.lock().unwrap(),
        "post_parse should have been called during parsing"
    );
}

#[test]
fn test_post_parse_not_called_when_extension_not_used() {
    // Create test handler that tracks post_parse calls
    struct UnusedExtensionHandler {
        post_parse_called: Arc<Mutex<bool>>,
    }

    impl ExtensionHandler for UnusedExtensionHandler {
        fn extension_type(&self) -> Extension {
            Extension::Material
        }

        fn validate(&self, _model: &Model) -> Lib3mfResult<()> {
            Ok(())
        }

        fn post_parse(&self, _model: &mut Model) -> Lib3mfResult<()> {
            *self.post_parse_called.lock().unwrap() = true;
            Ok(())
        }

        fn is_used_in_model(&self, _model: &Model) -> bool {
            // Return false to simulate extension not being used
            false
        }
    }

    let post_parse_called = Arc::new(Mutex::new(false));
    let handler = UnusedExtensionHandler {
        post_parse_called: post_parse_called.clone(),
    };

    // Create a registry and register the test handler
    let mut registry = ExtensionRegistry::new();
    registry.register(Arc::new(handler));

    // Create parser config with the registry
    let mut config = ParserConfig::new().with_extension(Extension::Material);
    *config.registry_mut() = registry;

    // Parse a 3MF file
    let data = create_test_3mf();
    let cursor = Cursor::new(data);
    let result = lib3mf::parser::parse_3mf_with_config(cursor, config);

    // Verify parsing succeeded
    assert!(
        result.is_ok(),
        "Parsing should succeed: {:?}",
        result.err()
    );

    // Verify that post_parse was NOT called (extension not used)
    assert!(
        !*post_parse_called.lock().unwrap(),
        "post_parse should NOT have been called when extension is not used"
    );
}

#[test]
fn test_post_parse_error_propagates() {
    // Create test handler that returns an error from post_parse
    struct ErrorExtensionHandler;

    impl ExtensionHandler for ErrorExtensionHandler {
        fn extension_type(&self) -> Extension {
            Extension::Material
        }

        fn validate(&self, _model: &Model) -> Lib3mfResult<()> {
            Ok(())
        }

        fn post_parse(&self, _model: &mut Model) -> Lib3mfResult<()> {
            // Return an error to test error propagation
            Err(lib3mf::Error::invalid_xml_element(
                "test",
                "Test error from post_parse",
            ))
        }

        fn is_used_in_model(&self, _model: &Model) -> bool {
            true
        }
    }

    // Create a registry and register the error handler
    let mut registry = ExtensionRegistry::new();
    registry.register(Arc::new(ErrorExtensionHandler));

    // Create parser config with the registry
    let mut config = ParserConfig::new().with_extension(Extension::Material);
    *config.registry_mut() = registry;

    // Parse a 3MF file
    let data = create_test_3mf();
    let cursor = Cursor::new(data);
    let result = lib3mf::parser::parse_3mf_with_config(cursor, config);

    // Verify parsing failed with the expected error
    assert!(result.is_err(), "Parsing should fail due to post_parse error");
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Test error from post_parse"),
        "Error message should contain the post_parse error"
    );
}

#[test]
fn test_post_parse_with_default_registry() {
    // Test that default registry with all standard handlers works
    let config = ParserConfig::with_all_extensions();

    // Parse a 3MF file
    let data = create_test_3mf();
    let cursor = Cursor::new(data);
    let result = lib3mf::parser::parse_3mf_with_config(cursor, config);

    // Verify parsing succeeded - all standard handlers have default post_parse that does nothing
    assert!(
        result.is_ok(),
        "Parsing should succeed with default registry: {:?}",
        result.err()
    );
}
