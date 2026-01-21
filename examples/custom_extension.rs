//! Example demonstrating custom 3MF extension support
//!
//! This example shows how to:
//! - Register custom extensions
//! - Implement element handlers for custom elements
//! - Implement custom validation rules
//! - Parse 3MF files with custom extensions

use lib3mf::{
    CustomElementResult, CustomExtensionContext, Model, ParserConfig,
};
use std::collections::HashMap;
use std::io::{Cursor, Write};
use std::sync::{Arc, Mutex};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Custom data structure to store custom extension data
#[derive(Debug, Clone)]
struct CustomExtensionData {
    elements: Vec<String>,
    attributes: HashMap<String, String>,
}

impl CustomExtensionData {
    fn new() -> Self {
        Self {
            elements: Vec::new(),
            attributes: HashMap::new(),
        }
    }
}

/// Create a 3MF file with a custom extension
fn create_sample_3mf_with_custom_extension() -> Vec<u8> {
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

    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" 
       xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:custom="http://example.com/3mf-extensions/custom/2024/01"
       requiredextensions="custom">
  <resources>
    <object id="1" type="model">
      <mesh>
        <vertices>
          <vertex x="0" y="0" z="0"/>
          <vertex x="10" y="0" z="0"/>
          <vertex x="5" y="10" z="0"/>
          <vertex x="5" y="5" z="10"/>
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
</model>"#,
    )
    .unwrap();

    let cursor = zip.finish().unwrap();
    cursor.into_inner()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Custom 3MF Extension Example ===\n");

    // Create shared state to track custom extension data
    let custom_data = Arc::new(Mutex::new(CustomExtensionData::new()));
    let custom_data_clone = custom_data.clone();

    // Define the custom extension namespace
    let custom_namespace = "http://example.com/3mf-extensions/custom/2024/01";

    println!("1. Registering custom extension: {}\n", custom_namespace);

    // Create a parser configuration with a custom extension handler
    let config = ParserConfig::new()
        .with_custom_extension_handlers(
            custom_namespace,
            "CustomExtension",
            // Element handler - called when custom elements are encountered
            Arc::new(move |ctx: &CustomExtensionContext| -> Result<CustomElementResult, String> {
                println!("   Element handler called:");
                println!("     - Element: {}", ctx.element_name);
                println!("     - Namespace: {}", ctx.namespace);
                println!("     - Attributes: {:?}", ctx.attributes);

                let mut data = custom_data_clone.lock().unwrap();
                data.elements.push(ctx.element_name.clone());
                for (key, value) in &ctx.attributes {
                    data.attributes.insert(key.clone(), value.clone());
                }

                Ok(CustomElementResult::Handled)
            }),
            // Validation handler - called during model validation
            Arc::new(|model: &Model| -> Result<(), String> {
                println!("   Custom validation handler called:");
                println!("     - Model has {} objects", model.resources.objects.len());
                println!("     - Model has {} build items", model.build.items.len());

                // Example custom validation: ensure model has at least one object
                if model.resources.objects.is_empty() {
                    return Err("Custom validation: Model must have at least one object".to_string());
                }

                println!("     - Custom validation passed!\n");
                Ok(())
            }),
        );

    println!("2. Creating sample 3MF file with custom extension...\n");
    let data = create_sample_3mf_with_custom_extension();

    println!("3. Parsing 3MF file with custom extension support...\n");
    let cursor = Cursor::new(data);
    let model = Model::from_reader_with_config(cursor, config)?;

    println!("4. Parse successful!\n");
    println!("   Model information:");
    println!("     - Unit: {}", model.unit);
    println!("     - Objects: {}", model.resources.objects.len());
    println!("     - Build items: {}", model.build.items.len());
    println!("     - Required extensions: {:?}", model.required_extensions);
    println!("     - Required custom extensions: {:?}", model.required_custom_extensions);

    // Display collected custom extension data
    let data = custom_data.lock().unwrap();
    println!("\n5. Custom extension data collected:");
    println!("     - Elements encountered: {:?}", data.elements);
    println!("     - Attributes collected: {:?}", data.attributes);

    println!("\n=== Example completed successfully! ===");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_runs() {
        // This test ensures the example code compiles and runs
        assert!(main().is_ok());
    }
}
