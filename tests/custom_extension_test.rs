use lib3mf::{CustomElementData, CustomExtension, Error, Model, ParserConfig};
use std::io::Cursor;
use std::sync::Arc;
use std::sync::Mutex;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;

// Test constants
const TEST_CUSTOM_NAMESPACE: &str = "http://example.com/custom";

/// Create a test 3MF file with custom extension elements
fn create_test_3mf_with_custom_extension(custom_namespace: &str) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut zip = ZipWriter::new(Cursor::new(&mut buffer));

    // Add [Content_Types].xml
    let content_types = r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
</Types>"#;
    zip.start_file(
        "[Content_Types].xml",
        SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
    )
    .unwrap();
    std::io::Write::write_all(&mut zip, content_types.as_bytes()).unwrap();

    // Add _rels/.rels
    let rels = r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel" Target="/3D/3dmodel.model" Id="rel0"/>
</Relationships>"#;
    zip.start_file(
        "_rels/.rels",
        SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
    )
    .unwrap();
    std::io::Write::write_all(&mut zip, rels.as_bytes()).unwrap();

    // Add 3D/3dmodel.model with custom extension
    let model_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" 
       xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:custom="{}">
  <metadata name="Title">Test Model</metadata>
  <resources>
    <object id="1">
      <mesh>
        <vertices>
          <vertex x="0" y="0" z="0"/>
          <vertex x="1" y="0" z="0"/>
          <vertex x="0" y="1" z="0"/>
        </vertices>
        <triangles>
          <triangle v1="0" v2="1" v3="2"/>
        </triangles>
      </mesh>
      <custom:metadata>
        <custom:property name="test" value="123"/>
        <custom:property name="foo">bar</custom:property>
      </custom:metadata>
    </object>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"#,
        custom_namespace
    );

    zip.start_file(
        "3D/3dmodel.model",
        SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
    )
    .unwrap();
    std::io::Write::write_all(&mut zip, model_xml.as_bytes()).unwrap();

    zip.finish().unwrap();
    buffer.to_vec()
}

#[test]
fn test_custom_extension_basic() {
    // Create a custom extension without callback
    let custom_ext = CustomExtension::new(
        TEST_CUSTOM_NAMESPACE.to_string(),
        "CustomExtension".to_string(),
    );

    let config = ParserConfig::new().with_custom_extension(custom_ext);

    let data = create_test_3mf_with_custom_extension(TEST_CUSTOM_NAMESPACE);
    let cursor = Cursor::new(data);

    let model = Model::from_reader_with_config(cursor, config).unwrap();

    // Check that custom elements were parsed
    let custom_elements = model
        .get_custom_elements(TEST_CUSTOM_NAMESPACE)
        .expect("Custom elements should be present");

    assert!(!custom_elements.is_empty(), "Should have custom elements");

    // Find the metadata element
    let metadata = custom_elements
        .iter()
        .find(|e| e.local_name == "metadata")
        .expect("Should have metadata element");

    assert_eq!(metadata.namespace, TEST_CUSTOM_NAMESPACE);
    assert_eq!(metadata.children.len(), 2, "Should have 2 child properties");
}

#[test]
fn test_custom_extension_with_callback() {
    // Track callback invocations
    let callback_count = Arc::new(Mutex::new(0));
    let callback_count_clone = Arc::clone(&callback_count);

    // Create a custom extension with validation callback
    let callback = Arc::new(move |element: &CustomElementData| {
        let mut count = callback_count_clone.lock().unwrap();
        *count += 1;

        // Validate the element
        if element.local_name == "metadata" {
            Ok(())
        } else if element.local_name == "property" {
            // Validate property has either 'value' attribute or text content
            if element.attributes.contains_key("value") || element.text_content.is_some() {
                Ok(())
            } else {
                Err(Error::InvalidXml(
                    "Property must have value attribute or text content".to_string(),
                ))
            }
        } else {
            Err(Error::InvalidXml(format!(
                "Unknown custom element: {}",
                element.local_name
            )))
        }
    });

    let custom_ext = CustomExtension::with_callback(
        TEST_CUSTOM_NAMESPACE.to_string(),
        "CustomExtension".to_string(),
        callback,
    );

    let config = ParserConfig::new().with_custom_extension(custom_ext);

    let data = create_test_3mf_with_custom_extension(TEST_CUSTOM_NAMESPACE);
    let cursor = Cursor::new(data);

    let model = Model::from_reader_with_config(cursor, config).unwrap();

    // Verify callback was invoked
    let count = callback_count.lock().unwrap();
    assert!(*count > 0, "Callback should have been invoked");

    // Check that custom elements were parsed and stored
    let custom_elements = model
        .get_custom_elements(TEST_CUSTOM_NAMESPACE)
        .expect("Custom elements should be present");

    assert!(!custom_elements.is_empty(), "Should have custom elements");
}

#[test]
fn test_custom_extension_validation_failure() {
    // Create a callback that always rejects
    let callback = Arc::new(|element: &CustomElementData| {
        Err(Error::InvalidXml(format!(
            "Rejecting element: {}",
            element.local_name
        )))
    });

    let custom_ext = CustomExtension::with_callback(
        TEST_CUSTOM_NAMESPACE.to_string(),
        "CustomExtension".to_string(),
        callback,
    );

    let config = ParserConfig::new().with_custom_extension(custom_ext);

    let data = create_test_3mf_with_custom_extension(TEST_CUSTOM_NAMESPACE);
    let cursor = Cursor::new(data);

    // Should fail because callback rejects all elements
    let result = Model::from_reader_with_config(cursor, config);
    assert!(result.is_err(), "Should fail validation");
}

#[test]
fn test_multiple_custom_extensions() {
    let custom_ext1 = CustomExtension::new(
        "http://example.com/ext1".to_string(),
        "Extension1".to_string(),
    );

    let custom_ext2 = CustomExtension::new(
        "http://example.com/ext2".to_string(),
        "Extension2".to_string(),
    );

    let config = ParserConfig::new()
        .with_custom_extension(custom_ext1)
        .with_custom_extension(custom_ext2);

    // Verify both extensions are registered
    assert!(config.has_custom_extension("http://example.com/ext1"));
    assert!(config.has_custom_extension("http://example.com/ext2"));
    assert!(!config.has_custom_extension("http://example.com/ext3"));
}

#[test]
fn test_custom_extension_element_attributes() {
    let custom_ext = CustomExtension::new(
        TEST_CUSTOM_NAMESPACE.to_string(),
        "CustomExtension".to_string(),
    );

    let config = ParserConfig::new().with_custom_extension(custom_ext);

    let data = create_test_3mf_with_custom_extension(TEST_CUSTOM_NAMESPACE);
    let cursor = Cursor::new(data);

    let model = Model::from_reader_with_config(cursor, config).unwrap();

    let custom_elements = model
        .get_custom_elements(TEST_CUSTOM_NAMESPACE)
        .expect("Custom elements should be present");

    // Find a property element with attributes
    let property_with_attr = custom_elements
        .iter()
        .find(|e| e.local_name == "metadata")
        .and_then(|metadata| {
            metadata
                .children
                .iter()
                .find(|child| child.attributes.contains_key("name"))
        })
        .expect("Should find property with name attribute");

    assert_eq!(
        property_with_attr.attributes.get("name"),
        Some(&"test".to_string())
    );
    assert_eq!(
        property_with_attr.attributes.get("value"),
        Some(&"123".to_string())
    );
}

#[test]
fn test_custom_extension_element_text_content() {
    let custom_ext = CustomExtension::new(
        TEST_CUSTOM_NAMESPACE.to_string(),
        "CustomExtension".to_string(),
    );

    let config = ParserConfig::new().with_custom_extension(custom_ext);

    let data = create_test_3mf_with_custom_extension(TEST_CUSTOM_NAMESPACE);
    let cursor = Cursor::new(data);

    let model = Model::from_reader_with_config(cursor, config).unwrap();

    let custom_elements = model
        .get_custom_elements(TEST_CUSTOM_NAMESPACE)
        .expect("Custom elements should be present");

    // Find the property element with text content
    let property_with_text = custom_elements
        .iter()
        .find(|e| e.local_name == "metadata")
        .and_then(|metadata| {
            metadata
                .children
                .iter()
                .find(|child| child.text_content.is_some())
        })
        .expect("Should find property with text content");

    assert_eq!(
        property_with_text.text_content,
        Some("bar".to_string())
    );
}

#[test]
fn test_unregistered_custom_extension_ignored() {
    // Don't register the custom extension
    let config = ParserConfig::new();

    let data = create_test_3mf_with_custom_extension(TEST_CUSTOM_NAMESPACE);
    let cursor = Cursor::new(data);

    // Should still parse successfully, but custom elements are ignored
    let model = Model::from_reader_with_config(cursor, config).unwrap();

    // Custom elements should not be present
    assert!(model.get_custom_elements(TEST_CUSTOM_NAMESPACE).is_none());
}
