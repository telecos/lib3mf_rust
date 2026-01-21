//! Tests for 3MF extension support and validation

use lib3mf::{Error, Extension, Model, ParserConfig};
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Create a minimal 3MF file with specified required extensions
fn create_test_3mf(required_extensions: &str) -> Vec<u8> {
    let buffer = Vec::new();
    let mut zip = ZipWriter::new(Cursor::new(buffer));

    // Add Content_Types.xml
    zip.start_file(
        "[Content_Types].xml",
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated),
    )
    .unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
</Types>"#,
    )
    .unwrap();

    // Add _rels/.rels
    zip.start_file(
        "_rels/.rels",
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated),
    )
    .unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Target="/3D/3dmodel.model" Id="rel0" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"#,
    )
    .unwrap();

    // Add 3D/3dmodel.model
    zip.start_file(
        "3D/3dmodel.model",
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated),
    )
    .unwrap();

    let model_xml = if required_extensions.is_empty() {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <resources>
    <object id="1" type="model">
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
    </object>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"#.to_string()
    } else {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" requiredextensions="{}">
  <resources>
    <object id="1" type="model">
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
    </object>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"#,
            required_extensions
        )
    };

    zip.write_all(model_xml.as_bytes()).unwrap();

    let cursor = zip.finish().unwrap();
    cursor.into_inner()
}

#[test]
fn test_parse_without_required_extensions() {
    // File with no requiredextensions attribute
    let data = create_test_3mf("");
    let cursor = Cursor::new(data);

    // Should parse successfully with any config
    let model = Model::from_reader(cursor).unwrap();
    assert!(model.required_extensions.is_empty());
}

#[test]
fn test_parse_with_material_extension() {
    // File requiring material extension
    let data = create_test_3mf("http://schemas.microsoft.com/3dmanufacturing/material/2015/02");
    let cursor = Cursor::new(data);

    // Should parse with default config (supports all extensions)
    let model = Model::from_reader(cursor).unwrap();
    assert_eq!(model.required_extensions.len(), 1);
    assert_eq!(model.required_extensions[0], Extension::Material);
}

#[test]
fn test_parse_with_multiple_extensions() {
    // File requiring multiple extensions
    let data = create_test_3mf(
        "http://schemas.microsoft.com/3dmanufacturing/material/2015/02 \
         http://schemas.microsoft.com/3dmanufacturing/production/2015/06",
    );
    let cursor = Cursor::new(data);

    let model = Model::from_reader(cursor).unwrap();
    assert_eq!(model.required_extensions.len(), 2);
    assert!(model.required_extensions.contains(&Extension::Material));
    assert!(model.required_extensions.contains(&Extension::Production));
}

#[test]
fn test_reject_unsupported_extension() {
    // File requiring material extension
    let data = create_test_3mf("http://schemas.microsoft.com/3dmanufacturing/material/2015/02");
    let cursor = Cursor::new(data);

    // Configure parser to only support core (not material)
    let config = ParserConfig::new(); // Only core

    // Should fail because material extension is required but not supported
    let result = Model::from_reader_with_config(cursor, config);
    assert!(result.is_err());

    match result.unwrap_err() {
        Error::UnsupportedExtension(msg) => {
            assert!(msg.contains("Material"));
            assert!(msg.contains("not supported"));
        }
        _ => panic!("Expected UnsupportedExtension error"),
    }
}

#[test]
fn test_accept_supported_extension() {
    // File requiring material extension
    let data = create_test_3mf("http://schemas.microsoft.com/3dmanufacturing/material/2015/02");
    let cursor = Cursor::new(data);

    // Configure parser to support material extension
    let config = ParserConfig::new().with_extension(Extension::Material);

    // Should succeed
    let model = Model::from_reader_with_config(cursor, config).unwrap();
    assert_eq!(model.required_extensions.len(), 1);
    assert_eq!(model.required_extensions[0], Extension::Material);
}

#[test]
fn test_reject_multiple_unsupported_extensions() {
    // File requiring production and slice extensions
    let data = create_test_3mf(
        "http://schemas.microsoft.com/3dmanufacturing/production/2015/06 \
         http://schemas.microsoft.com/3dmanufacturing/slice/2015/07",
    );
    let cursor = Cursor::new(data);

    // Only support production, not slice
    let config = ParserConfig::new().with_extension(Extension::Production);

    // Should fail because slice is required but not supported
    let result = Model::from_reader_with_config(cursor, config);
    assert!(result.is_err());

    match result.unwrap_err() {
        Error::UnsupportedExtension(msg) => {
            assert!(msg.contains("Slice"));
        }
        _ => panic!("Expected UnsupportedExtension error"),
    }
}

#[test]
fn test_parser_config_with_all_extensions() {
    let config = ParserConfig::with_all_extensions();

    // Should support all known extensions
    assert!(config.supports(&Extension::Core));
    assert!(config.supports(&Extension::Material));
    assert!(config.supports(&Extension::Production));
    assert!(config.supports(&Extension::Slice));
    assert!(config.supports(&Extension::BeamLattice));
    assert!(config.supports(&Extension::SecureContent));
    assert!(config.supports(&Extension::BooleanOperations));
    assert!(config.supports(&Extension::Displacement));
}

#[test]
fn test_parser_config_default() {
    let config = ParserConfig::new();

    // Should only support core by default
    assert!(config.supports(&Extension::Core));
    assert!(!config.supports(&Extension::Material));
}

#[test]
fn test_parser_config_builder() {
    let config = ParserConfig::new()
        .with_extension(Extension::Material)
        .with_extension(Extension::Production);

    assert!(config.supports(&Extension::Core));
    assert!(config.supports(&Extension::Material));
    assert!(config.supports(&Extension::Production));
    assert!(!config.supports(&Extension::Slice));
}

#[test]
fn test_extension_namespace_roundtrip() {
    let extensions = vec![
        Extension::Core,
        Extension::Material,
        Extension::Production,
        Extension::Slice,
        Extension::BeamLattice,
        Extension::SecureContent,
        Extension::BooleanOperations,
        Extension::Displacement,
    ];

    for ext in extensions {
        let namespace = ext.namespace();
        let parsed = Extension::from_namespace(namespace);
        assert_eq!(Some(ext), parsed, "Failed roundtrip for {:?}", ext);
    }
}

#[test]
fn test_unknown_extension_ignored() {
    // File with an unknown extension namespace
    let data = create_test_3mf(
        "http://example.com/unknown/extension \
         http://schemas.microsoft.com/3dmanufacturing/material/2015/02",
    );
    let cursor = Cursor::new(data);

    let model = Model::from_reader(cursor).unwrap();
    // Unknown extension should be ignored, only material should be parsed
    assert_eq!(model.required_extensions.len(), 1);
    assert_eq!(model.required_extensions[0], Extension::Material);
}

#[test]
fn test_backward_compatibility() {
    // The default from_reader should accept any extension for backward compatibility
    let data = create_test_3mf(
        "http://schemas.microsoft.com/3dmanufacturing/material/2015/02 \
         http://schemas.microsoft.com/3dmanufacturing/production/2015/06",
    );
    let cursor = Cursor::new(data);

    // Should succeed with default from_reader (no config specified)
    let model = Model::from_reader(cursor).unwrap();
    assert_eq!(model.required_extensions.len(), 2);
}
