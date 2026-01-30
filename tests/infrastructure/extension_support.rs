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

    // Check if Production extension is required
    let needs_production = required_extensions.contains("production/2015/06");

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
</model>"#
            .to_string()
    } else if needs_production {
        // When Production extension is required, we need p:UUID on build and items
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" requiredextensions="{}">
  <resources>
    <object id="1" type="model" p:UUID="f47ac10b-58cc-4372-a567-0e02b2c3d479">
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
  <build p:UUID="a47ac10b-58cc-4372-a567-0e02b2c3d479">
    <item objectid="1" p:UUID="b47ac10b-58cc-4372-a567-0e02b2c3d479"/>
  </build>
</model>"#,
            required_extensions
        )
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

    // With default from_reader, unknown extensions are tracked but require registration
    // Use with_all_extensions which only supports known extensions
    let config = ParserConfig::with_all_extensions()
        .with_custom_extension("http://example.com/unknown/extension", "Unknown");

    let model = Model::from_reader_with_config(cursor, config).unwrap();

    // Material should be in required_extensions (known extension)
    assert_eq!(model.required_extensions.len(), 1);
    assert_eq!(model.required_extensions[0], Extension::Material);

    // Unknown extension should be in required_custom_extensions
    assert_eq!(model.required_custom_extensions.len(), 1);
    assert_eq!(
        model.required_custom_extensions[0],
        "http://example.com/unknown/extension"
    );
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

#[test]
fn test_displacement_extension_parsing() {
    // Create a 3MF file with displacement extension resources
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
  <Default Extension="png" ContentType="image/png"/>
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

    // Add 3D/3dmodel.model with displacement resources
    zip.start_file(
        "3D/3dmodel.model",
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated),
    )
    .unwrap();

    let model_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" 
       xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:d="http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07"
       requiredextensions="d">
  <resources>
    <d:displacement2d id="1" path="/3D/Textures/disp.png" channel="R" tilestyleu="wrap" tilestylev="mirror" filter="linear"/>
    <d:normvectorgroup id="2">
      <d:normvector x="0.0" y="0.0" z="1.0"/>
      <d:normvector x="0.577" y="0.577" z="0.577"/>
      <d:normvector x="1.0" y="0.0" z="0.0"/>
    </d:normvectorgroup>
    <d:disp2dgroup id="3" dispid="1" nid="2" height="5.0" offset="0.5">
      <d:disp2dcoord u="0.0" v="0.0" n="0" f="1.0"/>
      <d:disp2dcoord u="1.0" v="0.0" n="1" f="0.8"/>
      <d:disp2dcoord u="0.5" v="1.0" n="2"/>
    </d:disp2dgroup>
    <object id="10" type="model">
      <mesh>
        <vertices>
          <vertex x="0" y="0" z="0"/>
          <vertex x="10" y="0" z="0"/>
          <vertex x="5" y="10" z="0"/>
        </vertices>
        <triangles>
          <triangle v1="0" v2="1" v3="2"/>
        </triangles>
      </mesh>
    </object>
  </resources>
  <build>
    <item objectid="10"/>
  </build>
</model>"#;

    zip.write_all(model_xml.as_bytes()).unwrap();

    let result = zip.finish().unwrap();
    let data = result.into_inner();

    // Parse the 3MF file
    let cursor = Cursor::new(data);
    let model = Model::from_reader(cursor).unwrap();

    // Verify displacement extension is recognized
    assert_eq!(model.required_extensions.len(), 1);
    assert_eq!(model.required_extensions[0], Extension::Displacement);

    // Verify displacement2d resource was parsed
    assert_eq!(model.resources.displacement_maps.len(), 1);
    let disp = &model.resources.displacement_maps[0];
    assert_eq!(disp.id, 1);
    assert_eq!(disp.path, "/3D/Textures/disp.png");
    assert_eq!(disp.channel, lib3mf::Channel::R);
    assert_eq!(disp.tilestyleu, lib3mf::TileStyle::Wrap);
    assert_eq!(disp.tilestylev, lib3mf::TileStyle::Mirror);
    assert_eq!(disp.filter, lib3mf::FilterMode::Linear);

    // Verify normvectorgroup was parsed
    assert_eq!(model.resources.norm_vector_groups.len(), 1);
    let nvgroup = &model.resources.norm_vector_groups[0];
    assert_eq!(nvgroup.id, 2);
    assert_eq!(nvgroup.vectors.len(), 3);
    assert_eq!(nvgroup.vectors[0].x, 0.0);
    assert_eq!(nvgroup.vectors[0].y, 0.0);
    assert_eq!(nvgroup.vectors[0].z, 1.0);

    // Verify disp2dgroup was parsed
    assert_eq!(model.resources.disp2d_groups.len(), 1);
    let d2dgroup = &model.resources.disp2d_groups[0];
    assert_eq!(d2dgroup.id, 3);
    assert_eq!(d2dgroup.dispid, 1);
    assert_eq!(d2dgroup.nid, 2);
    assert_eq!(d2dgroup.height, 5.0);
    assert_eq!(d2dgroup.offset, 0.5);
    assert_eq!(d2dgroup.coords.len(), 3);

    // Verify first coordinate
    assert_eq!(d2dgroup.coords[0].u, 0.0);
    assert_eq!(d2dgroup.coords[0].v, 0.0);
    assert_eq!(d2dgroup.coords[0].n, 0);
    assert_eq!(d2dgroup.coords[0].f, 1.0);

    // Verify third coordinate (tests default f value)
    assert_eq!(d2dgroup.coords[2].u, 0.5);
    assert_eq!(d2dgroup.coords[2].v, 1.0);
    assert_eq!(d2dgroup.coords[2].n, 2);
    assert_eq!(d2dgroup.coords[2].f, 1.0); // Default value
}
