//! Example demonstrating the create_default_registry() function
//!
//! This example shows how to use the convenience functions to create
//! a registry with all standard extension handlers.

use lib3mf::{create_default_registry, Extension, Model, Result};

fn main() -> Result<()> {
    println!("=== Extension Registry Factory Example ===\n");

    // Create a simple model
    let mut model = Model::new();

    // Create a registry with all standard handlers using the factory function
    println!("Creating default registry with all standard handlers...");
    let registry = create_default_registry();

    // Show registered handlers
    println!("  ✓ Registered {} handlers:", registry.handlers().len());
    for ext in &[
        Extension::Material,
        Extension::Production,
        Extension::BeamLattice,
        Extension::Slice,
        Extension::BooleanOperations,
        Extension::Displacement,
        Extension::SecureContent,
    ] {
        if let Some(handler) = registry.get_handler(*ext) {
            println!("    - {}: {}", handler.name(), handler.namespace());
        }
    }
    println!();

    // Validate the model with all handlers
    println!("Validating model with all registered extensions...");
    registry.validate_all(&model)?;
    println!("  ✓ All validations passed!\n");

    // Run post-parse hooks
    println!("Running post-parse hooks...");
    registry.post_parse_all(&mut model)?;
    println!("  ✓ Post-parse processing complete!\n");

    // Run pre-write hooks
    println!("Running pre-write hooks...");
    registry.pre_write_all(&mut model)?;
    println!("  ✓ Pre-write processing complete!\n");

    println!("=== Benefits of create_default_registry() ===");
    println!("✓ Automatically includes all standard 3MF extension handlers");
    println!("✓ No need to manually register each handler");
    println!("✓ Ensures consistent extension support across your application");
    println!("✓ Easy to update when new extensions are added to the library");

    Ok(())
}
