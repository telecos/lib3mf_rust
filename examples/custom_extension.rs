//! Example demonstrating custom 3MF extension handling
//!
//! This example shows how to register and handle custom/proprietary 3MF extensions.

use lib3mf::{CustomElementData, CustomExtension, ParserConfig};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Basic custom extension (no validation)
    println!("Example 1: Basic custom extension");
    println!("===================================\n");

    let basic_extension = CustomExtension::new(
        "http://example.com/myextension".to_string(),
        "MyExtension".to_string(),
    );

    let config = ParserConfig::new().with_custom_extension(basic_extension);

    println!("Registered custom extension: http://example.com/myextension");
    println!(
        "Number of custom extensions: {}",
        config.custom_extensions().count()
    );

    // Example 2: Custom extension with validation callback
    println!("\n\nExample 2: Custom extension with validation");
    println!("============================================\n");

    // Create a validation callback
    let callback = Arc::new(|element: &CustomElementData| -> lib3mf::Result<()> {
        println!(
            "Validating custom element: {} (namespace: {})",
            element.local_name, element.namespace
        );

        // Example validation logic
        match element.local_name.as_str() {
            "metadata" => {
                // Metadata elements are allowed
                println!("  ✓ Metadata element is valid");
                Ok(())
            }
            "property" => {
                // Property elements must have either 'value' attribute or text content
                if element.attributes.contains_key("value") || element.text_content.is_some() {
                    println!("  ✓ Property element is valid");
                    Ok(())
                } else {
                    Err(lib3mf::Error::InvalidXml(
                        "Property element must have 'value' attribute or text content".to_string(),
                    ))
                }
            }
            "annotation" => {
                // Annotations must have a 'type' attribute
                if element.attributes.contains_key("type") {
                    println!("  ✓ Annotation element is valid");
                    Ok(())
                } else {
                    Err(lib3mf::Error::InvalidXml(
                        "Annotation element must have 'type' attribute".to_string(),
                    ))
                }
            }
            _ => {
                // Reject unknown elements
                Err(lib3mf::Error::InvalidXml(format!(
                    "Unknown custom element: {}",
                    element.local_name
                )))
            }
        }
    });

    let validated_extension = CustomExtension::with_callback(
        "http://example.com/validated".to_string(),
        "ValidatedExtension".to_string(),
        callback,
    );

    let _config_with_validation = ParserConfig::new()
        .with_custom_extension(validated_extension);

    println!(
        "Registered custom extension with validation callback: http://example.com/validated"
    );

    // Example 3: Multiple custom extensions
    println!("\n\nExample 3: Multiple custom extensions");
    println!("======================================\n");

    let ext1 = CustomExtension::new(
        "http://company.com/internal/v1".to_string(),
        "InternalExtension".to_string(),
    );

    let ext2 = CustomExtension::new(
        "http://vendor.com/proprietary".to_string(),
        "VendorExtension".to_string(),
    );

    let multi_config = ParserConfig::new()
        .with_custom_extension(ext1)
        .with_custom_extension(ext2);

    println!(
        "Registered {} custom extensions:",
        multi_config.custom_extensions().count()
    );
    for ext in multi_config.custom_extensions() {
        println!("  - {} ({})", ext.name(), ext.namespace());
    }

    // Example 4: Accessing custom extension data
    println!("\n\nExample 4: Accessing custom extension data");
    println!("===========================================\n");

    println!("When parsing a 3MF file with custom extensions:");
    println!();
    println!("// After parsing");
    println!("let model = Model::from_reader_with_config(file, config)?;");
    println!();
    println!("// Access custom extension elements");
    println!(
        "if let Some(elements) = model.get_custom_elements(\"http://example.com/myextension\") {{"
    );
    println!("    for element in elements {{");
    println!("        println!(\"Element: {{}}\", element.local_name);");
    println!("        println!(\"Attributes: {{:?}}\", element.attributes);");
    println!("        if let Some(text) = &element.text_content {{");
    println!("            println!(\"Text content: {{}}\", text);");
    println!("        }}");
    println!("        println!(\"Children: {{}}\", element.children.len());");
    println!("    }}");
    println!("}}");

    // Example 5: Custom extension with both standard and custom extensions
    println!("\n\nExample 5: Combining standard and custom extensions");
    println!("====================================================\n");

    use lib3mf::Extension;

    let combined_config = ParserConfig::new()
        .with_extension(Extension::Material)
        .with_extension(Extension::Production)
        .with_custom_extension(CustomExtension::new(
            "http://example.com/custom".to_string(),
            "CustomExt".to_string(),
        ));

    println!("Standard extensions:");
    for ext in combined_config.supported_extensions() {
        println!("  - {} ({})", ext.name(), ext.namespace());
    }

    println!("\nCustom extensions:");
    for ext in combined_config.custom_extensions() {
        println!("  - {} ({})", ext.name(), ext.namespace());
    }

    println!("\n\nExample complete!");

    Ok(())
}
