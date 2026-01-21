//! Tests for custom 3MF extension support

use lib3mf::{CustomElementResult, CustomExtensionContext, Error, Model, ParserConfig};
use std::io::{Cursor, Write};
use std::sync::{Arc, Mutex};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Create a minimal 3MF file with custom extension in requiredextensions
fn create_3mf_with_custom_extension(namespace: &str, prefix: &str) -> Vec<u8> {
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

    // Add 3D/3dmodel.model with custom extension
    zip.start_file(
        "3D/3dmodel.model",
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated),
    )
    .unwrap();

    let model_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" 
       xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:{prefix}="{namespace}"
       requiredextensions="{prefix}">
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
        prefix = prefix,
        namespace = namespace
    );

    zip.write_all(model_xml.as_bytes()).unwrap();

    let cursor = zip.finish().unwrap();
    cursor.into_inner()
}

#[test]
fn test_custom_extension_registration() {
    let config = ParserConfig::new()
        .with_custom_extension("http://example.com/myextension/2024/01", "MyExtension");

    assert!(config.has_custom_extension("http://example.com/myextension/2024/01"));
    assert!(!config.has_custom_extension("http://example.com/other/2024/01"));
}

#[test]
fn test_custom_extension_with_handler() {
    let called = Arc::new(Mutex::new(false));
    let called_clone = called.clone();

    let config = ParserConfig::new().with_custom_extension_handler(
        "http://example.com/myextension/2024/01",
        "MyExtension",
        Arc::new(move |_ctx: &CustomExtensionContext| {
            *called_clone.lock().unwrap() = true;
            Ok(CustomElementResult::Handled)
        }),
    );

    assert!(config.has_custom_extension("http://example.com/myextension/2024/01"));

    // Verify handler is registered
    let ext_info = config
        .get_custom_extension("http://example.com/myextension/2024/01")
        .unwrap();
    assert!(ext_info.element_handler.is_some());
    assert!(ext_info.validation_handler.is_none());
}

#[test]
fn test_custom_extension_with_both_handlers() {
    let config = ParserConfig::new().with_custom_extension_handlers(
        "http://example.com/myextension/2024/01",
        "MyExtension",
        Arc::new(|_ctx: &CustomExtensionContext| Ok(CustomElementResult::Handled)),
        Arc::new(|_model| Ok(())),
    );

    let ext_info = config
        .get_custom_extension("http://example.com/myextension/2024/01")
        .unwrap();
    assert!(ext_info.element_handler.is_some());
    assert!(ext_info.validation_handler.is_some());
}

#[test]
fn test_parse_with_registered_custom_extension() {
    let namespace = "http://example.com/myextension/2024/01";
    let data = create_3mf_with_custom_extension(namespace, "custom");

    let config = ParserConfig::new().with_custom_extension(namespace, "MyExtension");

    let cursor = Cursor::new(data);
    let result = Model::from_reader_with_config(cursor, config);

    // Should succeed because custom extension is registered
    assert!(result.is_ok());
    let model = result.unwrap();

    // Verify the custom extension is in required_custom_extensions
    assert_eq!(model.required_custom_extensions.len(), 1);
    assert_eq!(model.required_custom_extensions[0], namespace);
}

#[test]
fn test_parse_without_registered_custom_extension() {
    let namespace = "http://example.com/myextension/2024/01";
    let data = create_3mf_with_custom_extension(namespace, "custom");

    // Don't register the custom extension
    let config = ParserConfig::new();

    let cursor = Cursor::new(data);
    let result = Model::from_reader_with_config(cursor, config);

    // Should fail because custom extension is not registered
    assert!(result.is_err());
    match result {
        Err(Error::UnsupportedExtension(msg)) => {
            assert!(msg.contains("not registered"));
        }
        _ => panic!("Expected UnsupportedExtension error"),
    }
}

#[test]
fn test_multiple_custom_extensions() {
    let config = ParserConfig::new()
        .with_custom_extension("http://example.com/ext1/2024/01", "Ext1")
        .with_custom_extension("http://example.com/ext2/2024/01", "Ext2")
        .with_custom_extension("http://example.com/ext3/2024/01", "Ext3");

    assert_eq!(config.custom_extensions().len(), 3);
    assert!(config.has_custom_extension("http://example.com/ext1/2024/01"));
    assert!(config.has_custom_extension("http://example.com/ext2/2024/01"));
    assert!(config.has_custom_extension("http://example.com/ext3/2024/01"));
}

#[test]
fn test_custom_extension_info() {
    let config = ParserConfig::new()
        .with_custom_extension("http://example.com/myextension/2024/01", "MyExtension");

    let ext_info = config
        .get_custom_extension("http://example.com/myextension/2024/01")
        .unwrap();
    assert_eq!(ext_info.namespace, "http://example.com/myextension/2024/01");
    assert_eq!(ext_info.name, "MyExtension");
    assert!(ext_info.element_handler.is_none());
    assert!(ext_info.validation_handler.is_none());
}

#[test]
fn test_custom_extension_context() {
    let mut attrs = std::collections::HashMap::new();
    attrs.insert("attr1".to_string(), "value1".to_string());
    attrs.insert("attr2".to_string(), "value2".to_string());

    let context = CustomExtensionContext {
        element_name: "customElement".to_string(),
        namespace: "http://example.com/myextension/2024/01".to_string(),
        attributes: attrs,
    };

    assert_eq!(context.element_name, "customElement");
    assert_eq!(context.namespace, "http://example.com/myextension/2024/01");
    assert_eq!(context.attributes.get("attr1").unwrap(), "value1");
    assert_eq!(context.attributes.get("attr2").unwrap(), "value2");
}

#[test]
fn test_custom_extension_handler_result() {
    // Test Handled result
    let handler = Arc::new(
        |_ctx: &CustomExtensionContext| -> Result<CustomElementResult, String> {
            Ok(CustomElementResult::Handled)
        },
    );

    let ctx = CustomExtensionContext {
        element_name: "test".to_string(),
        namespace: "http://example.com/test".to_string(),
        attributes: std::collections::HashMap::new(),
    };

    let result = handler(&ctx);
    assert!(result.is_ok());
    match result.unwrap() {
        CustomElementResult::Handled => {}
        _ => panic!("Expected Handled result"),
    }

    // Test NotHandled result
    let handler2 = Arc::new(
        |_ctx: &CustomExtensionContext| -> Result<CustomElementResult, String> {
            Ok(CustomElementResult::NotHandled)
        },
    );

    let result2 = handler2(&ctx);
    assert!(result2.is_ok());
    match result2.unwrap() {
        CustomElementResult::NotHandled => {}
        _ => panic!("Expected NotHandled result"),
    }
}

#[test]
fn test_custom_validation_handler() {
    let validation_called = Arc::new(Mutex::new(false));
    let validation_called_clone = validation_called.clone();

    let _handler = Arc::new(move |_model: &Model| -> Result<(), String> {
        *validation_called_clone.lock().unwrap() = true;
        Ok(())
    });

    // Verify the handler can be called
    let model = Model::new();
    let result = _handler(&model);
    assert!(result.is_ok());
    assert!(*validation_called.lock().unwrap());
}

#[test]
fn test_custom_validation_handler_error() {
    let handler = Arc::new(|_model: &Model| -> Result<(), String> {
        Err("Custom validation failed".to_string())
    });

    let model = Model::new();
    let result = handler(&model);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Custom validation failed");
}

#[test]
fn test_parser_config_debug() {
    let config = ParserConfig::new()
        .with_custom_extension("http://example.com/myextension/2024/01", "MyExtension");

    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("ParserConfig"));
    assert!(debug_str.contains("custom_extensions_count"));
}
