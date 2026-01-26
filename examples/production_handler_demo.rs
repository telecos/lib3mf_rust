//! Demonstration of the ProductionExtensionHandler
//!
//! This example shows how to use the ProductionExtensionHandler to validate
//! production extension data in 3MF files.

use lib3mf::extensions::ProductionExtensionHandler;
use lib3mf::{Extension, ExtensionHandler, ExtensionRegistry, Model};
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ProductionExtensionHandler Demo ===\n");

    // Create the production handler
    let handler = ProductionExtensionHandler;

    // Display handler properties
    println!("Handler Properties:");
    println!("  Extension Type: {:?}", handler.extension_type());
    println!("  Name: {}", handler.name());
    println!("  Namespace: {}\n", handler.namespace());

    // Test 1: Validate an empty model
    println!("Test 1: Empty Model");
    let empty_model = Model::new();
    println!(
        "  Is used in model: {}",
        handler.is_used_in_model(&empty_model)
    );
    println!("  Validation: {:?}\n", handler.validate(&empty_model));

    // Test 2: Load a real 3MF file with production data
    println!("Test 2: Real 3MF file with production extension");
    let test_file = "test_files/production/box_prod.3mf";

    match File::open(test_file) {
        Ok(file) => {
            match Model::from_reader(file) {
                Ok(model) => {
                    println!("  ✓ File loaded successfully");
                    println!("  Model has {} objects", model.resources.objects.len());

                    // Check if production extension is used
                    let is_used = handler.is_used_in_model(&model);
                    println!("  Production extension used: {}", is_used);

                    // Count production data
                    let objects_with_prod = model
                        .resources
                        .objects
                        .iter()
                        .filter(|obj| obj.production.is_some())
                        .count();
                    println!("  Objects with production info: {}", objects_with_prod);

                    // Validate production data
                    match handler.validate(&model) {
                        Ok(()) => println!("  ✓ Validation passed"),
                        Err(e) => println!("  ✗ Validation failed: {}", e),
                    }

                    // Display production data details
                    for obj in &model.resources.objects {
                        if let Some(ref prod) = obj.production {
                            println!("\n  Object {} production info:", obj.id);
                            if let Some(ref uuid) = prod.uuid {
                                println!("    UUID: {}", uuid);
                            }
                            if let Some(ref path) = prod.path {
                                println!("    Path: {}", path);
                            }
                        }
                    }
                }
                Err(e) => println!("  ✗ Failed to parse file: {}", e),
            }
        }
        Err(_) => {
            println!("  ⓘ Test file not found (this is OK for CI environments)");
        }
    }

    // Test 3: Using with ExtensionRegistry
    println!("\n\nTest 3: Using with ExtensionRegistry");
    let mut registry = ExtensionRegistry::new();
    registry.register(Box::new(ProductionExtensionHandler));
    println!("  ✓ Handler registered");

    let model = Model::new();
    match registry.validate_all(&model) {
        Ok(()) => println!("  ✓ Registry validation passed"),
        Err(e) => println!("  ✗ Registry validation failed: {}", e),
    }

    // Test 4: Get handler from registry
    println!("\nTest 4: Retrieve handler from registry");
    if let Some(retrieved_handler) = registry.get_handler(Extension::Production) {
        println!("  ✓ Handler retrieved successfully");
        println!("  Handler name: {}", retrieved_handler.name());
    } else {
        println!("  ✗ Failed to retrieve handler");
    }

    println!("\n✓ All demonstrations complete!");
    Ok(())
}
