//! Example demonstrating the use of ExtensionRegistry with writer
//!
//! This example shows how to use the new `to_writer_with_registry()` method
//! to wire extension pre_write hooks into the writing pipeline.

use lib3mf::extension::{ExtensionHandler, ExtensionRegistry};
use lib3mf::{
    create_default_registry, BuildItem, Extension, Mesh, Model, Object, Result, Triangle, Vertex,
};
use std::fs::File;

/// Custom extension handler that demonstrates the pre_write hook
struct CustomPreWriteHandler;

impl ExtensionHandler for CustomPreWriteHandler {
    fn extension_type(&self) -> Extension {
        Extension::Material
    }

    fn validate(&self, _model: &Model) -> Result<()> {
        println!("  [validate] Validating model...");
        Ok(())
    }

    fn pre_write(&self, model: &mut Model) -> Result<()> {
        println!("  [pre_write] Preparing model for writing...");

        // Example: Add metadata if not present
        if !model.has_metadata("ProcessedBy") {
            println!("    Adding 'ProcessedBy' metadata");
            model.metadata.push(lib3mf::model::MetadataEntry::new(
                "ProcessedBy".to_string(),
                "lib3mf_rust with ExtensionRegistry".to_string(),
            ));
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    println!("=== ExtensionRegistry Writer Integration Example ===\n");

    // Get temporary directory for portable file paths
    let temp_dir = std::env::temp_dir();

    // Create a simple 3D model
    println!("1. Creating a simple 3MF model...");
    let mut model = Model::new();
    model.unit = "millimeter".to_string();

    // Add a simple triangle mesh
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));

    let mut object = Object::new(1);
    object.mesh = Some(mesh);
    model.resources.objects.push(object);
    model.build.items.push(BuildItem::new(1));

    println!("   ✓ Model created with 1 object (triangle)\n");

    // Method 1: Using default registry
    println!("2. Writing with default registry (all standard extensions)...");
    let registry = create_default_registry();

    let file_path = temp_dir.join("example_with_default_registry.3mf");
    let file = File::create(&file_path)?;
    model.clone().to_writer_with_registry(file, &registry)?;
    println!("   ✓ Written to {:?}\n", file_path);

    // Method 2: Using custom registry with custom handler
    println!("3. Writing with custom registry...");
    let mut custom_registry = ExtensionRegistry::new();
    custom_registry.register(std::sync::Arc::new(CustomPreWriteHandler));

    let file_path = temp_dir.join("example_with_custom_registry.3mf");
    let file = File::create(&file_path)?;
    model
        .clone()
        .to_writer_with_registry(file, &custom_registry)?;
    println!("   ✓ Written to {:?}\n", file_path);

    // Method 3: Traditional method (no registry, for backward compatibility)
    println!("4. Writing without registry (backward compatibility)...");
    let file_path = temp_dir.join("example_without_registry.3mf");
    let file = File::create(&file_path)?;
    model.clone().to_writer(file)?;
    println!("   ✓ Written to {:?}\n", file_path);

    // Method 4: Using write_to_file_with_registry
    println!("5. Using write_to_file_with_registry convenience method...");
    let file_path = temp_dir.join("example_convenience.3mf");
    model
        .clone()
        .write_to_file_with_registry(&file_path, &custom_registry)?;
    println!("   ✓ Written to {:?}\n", file_path);

    // Verify the files can be read back
    println!("6. Verifying written files can be read back...");
    let model1 = Model::from_reader(File::open(
        temp_dir.join("example_with_default_registry.3mf"),
    )?)?;
    let model2 = Model::from_reader(File::open(
        temp_dir.join("example_with_custom_registry.3mf"),
    )?)?;
    let model3 = Model::from_reader(File::open(temp_dir.join("example_without_registry.3mf"))?)?;
    let model4 = Model::from_reader(File::open(temp_dir.join("example_convenience.3mf"))?)?;

    println!("   ✓ All files successfully read back");
    println!("   - Model 1: {} objects", model1.resources.objects.len());
    println!(
        "   - Model 2: {} objects, {} metadata entries (with ProcessedBy)",
        model2.resources.objects.len(),
        model2.metadata.len()
    );
    println!("   - Model 3: {} objects", model3.resources.objects.len());
    println!(
        "   - Model 4: {} objects, {} metadata entries",
        model4.resources.objects.len(),
        model4.metadata.len()
    );

    println!("\n=== Summary ===");
    println!("✓ ExtensionRegistry.pre_write_all() is now wired into the writer flow");
    println!("✓ Backward compatibility maintained with to_writer()");
    println!("✓ New API: to_writer_with_registry() and write_to_file_with_registry()");
    println!("✓ Extension handlers can prepare/transform data before writing");

    // Clean up
    std::fs::remove_file(temp_dir.join("example_with_default_registry.3mf")).ok();
    std::fs::remove_file(temp_dir.join("example_with_custom_registry.3mf")).ok();
    std::fs::remove_file(temp_dir.join("example_without_registry.3mf")).ok();
    std::fs::remove_file(temp_dir.join("example_convenience.3mf")).ok();

    Ok(())
}
