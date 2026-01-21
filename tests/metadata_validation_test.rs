//! Tests for metadata validation according to 3MF Core Specification Chapter 4

use lib3mf::Model;
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Helper function to create a 3MF file with custom metadata
fn create_3mf_with_metadata(metadata_xml: &str) -> Vec<u8> {
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

    // Add 3D/3dmodel.model with custom metadata
    let model = format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  {}
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
</model>"##,
        metadata_xml
    );

    zip.start_file("3D/3dmodel.model", options).unwrap();
    zip.write_all(model.as_bytes()).unwrap();

    zip.finish().unwrap();
    buffer
}

#[test]
fn test_metadata_missing_name_attribute() {
    // Metadata without name attribute should fail
    let metadata = r#"<metadata>Invalid Metadata</metadata>"#;
    let data = create_3mf_with_metadata(metadata);
    let cursor = Cursor::new(data);

    let result = Model::from_reader(cursor);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Metadata element must have a 'name' attribute"));
}

#[test]
fn test_metadata_with_preserve_true() {
    // Metadata with preserve="1" should be parsed correctly
    let metadata = r#"<metadata name="Title" preserve="1">Test Model</metadata>"#;
    let data = create_3mf_with_metadata(metadata);
    let cursor = Cursor::new(data);

    let model = Model::from_reader(cursor).unwrap();
    assert_eq!(model.metadata.len(), 1);
    assert_eq!(model.metadata[0].name, "Title");
    assert_eq!(model.metadata[0].value, "Test Model");
    assert_eq!(model.metadata[0].preserve, Some(true));
}

#[test]
fn test_metadata_with_preserve_false() {
    // Metadata with preserve="0" should be parsed correctly
    let metadata = r#"<metadata name="Title" preserve="0">Test Model</metadata>"#;
    let data = create_3mf_with_metadata(metadata);
    let cursor = Cursor::new(data);

    let model = Model::from_reader(cursor).unwrap();
    assert_eq!(model.metadata.len(), 1);
    assert_eq!(model.metadata[0].name, "Title");
    assert_eq!(model.metadata[0].value, "Test Model");
    assert_eq!(model.metadata[0].preserve, Some(false));
}

#[test]
fn test_metadata_with_preserve_true_boolean() {
    // Metadata with preserve="true" should be parsed correctly
    let metadata = r#"<metadata name="Title" preserve="true">Test Model</metadata>"#;
    let data = create_3mf_with_metadata(metadata);
    let cursor = Cursor::new(data);

    let model = Model::from_reader(cursor).unwrap();
    assert_eq!(model.metadata.len(), 1);
    assert_eq!(model.metadata[0].preserve, Some(true));
}

#[test]
fn test_metadata_with_preserve_false_boolean() {
    // Metadata with preserve="false" should be parsed correctly
    let metadata = r#"<metadata name="Title" preserve="false">Test Model</metadata>"#;
    let data = create_3mf_with_metadata(metadata);
    let cursor = Cursor::new(data);

    let model = Model::from_reader(cursor).unwrap();
    assert_eq!(model.metadata.len(), 1);
    assert_eq!(model.metadata[0].preserve, Some(false));
}

#[test]
fn test_metadata_without_preserve() {
    // Metadata without preserve attribute should have None for preserve
    let metadata = r#"<metadata name="Title">Test Model</metadata>"#;
    let data = create_3mf_with_metadata(metadata);
    let cursor = Cursor::new(data);

    let model = Model::from_reader(cursor).unwrap();
    assert_eq!(model.metadata.len(), 1);
    assert_eq!(model.metadata[0].name, "Title");
    assert_eq!(model.metadata[0].value, "Test Model");
    assert_eq!(model.metadata[0].preserve, None);
}

#[test]
fn test_metadata_with_invalid_preserve() {
    // Metadata with invalid preserve value should fail
    let metadata = r#"<metadata name="Title" preserve="invalid">Test Model</metadata>"#;
    let data = create_3mf_with_metadata(metadata);
    let cursor = Cursor::new(data);

    let result = Model::from_reader(cursor);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid preserve attribute value"));
}

#[test]
fn test_metadata_duplicate_names() {
    // Duplicate metadata names should fail
    let metadata = r#"
  <metadata name="Title">First Title</metadata>
  <metadata name="Title">Second Title</metadata>
"#;
    let data = create_3mf_with_metadata(metadata);
    let cursor = Cursor::new(data);

    let result = Model::from_reader(cursor);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Duplicate metadata name 'Title'"));
}

#[test]
fn test_metadata_with_namespace() {
    // Metadata with namespaced name should work if namespace is declared
    let metadata = r#"
  <metadata name="xml:lang">en-US</metadata>
"#;
    let data = create_3mf_with_metadata(metadata);
    let cursor = Cursor::new(data);

    let model = Model::from_reader(cursor).unwrap();
    assert_eq!(model.metadata.len(), 1);
    assert_eq!(model.metadata[0].name, "xml:lang");
    assert_eq!(model.metadata[0].value, "en-US");
}

#[test]
fn test_multiple_metadata_entries() {
    // Multiple metadata entries should all be parsed
    let metadata = r#"
  <metadata name="Title">Test Model</metadata>
  <metadata name="Designer">John Doe</metadata>
  <metadata name="Description" preserve="1">A test model for validation</metadata>
  <metadata name="Copyright" preserve="0">Copyright 2024</metadata>
"#;
    let data = create_3mf_with_metadata(metadata);
    let cursor = Cursor::new(data);

    let model = Model::from_reader(cursor).unwrap();
    assert_eq!(model.metadata.len(), 4);

    // Check each metadata entry
    assert_eq!(model.metadata[0].name, "Title");
    assert_eq!(model.metadata[0].value, "Test Model");
    assert_eq!(model.metadata[0].preserve, None);

    assert_eq!(model.metadata[1].name, "Designer");
    assert_eq!(model.metadata[1].value, "John Doe");
    assert_eq!(model.metadata[1].preserve, None);

    assert_eq!(model.metadata[2].name, "Description");
    assert_eq!(model.metadata[2].value, "A test model for validation");
    assert_eq!(model.metadata[2].preserve, Some(true));

    assert_eq!(model.metadata[3].name, "Copyright");
    assert_eq!(model.metadata[3].value, "Copyright 2024");
    assert_eq!(model.metadata[3].preserve, Some(false));
}

#[test]
fn test_metadata_helper_methods() {
    // Test the get_metadata and has_metadata helper methods
    let metadata = r#"
  <metadata name="Title">Test Model</metadata>
  <metadata name="Designer">John Doe</metadata>
"#;
    let data = create_3mf_with_metadata(metadata);
    let cursor = Cursor::new(data);

    let model = Model::from_reader(cursor).unwrap();

    // Test get_metadata
    assert_eq!(model.get_metadata("Title"), Some("Test Model"));
    assert_eq!(model.get_metadata("Designer"), Some("John Doe"));
    assert_eq!(model.get_metadata("NonExistent"), None);

    // Test has_metadata
    assert!(model.has_metadata("Title"));
    assert!(model.has_metadata("Designer"));
    assert!(!model.has_metadata("NonExistent"));
}
