//! Example demonstrating the pluggable Extension trait system
//!
//! This example shows how to create custom extension handlers that can be
//! registered and used for validation and processing.

use lib3mf::{Extension, ExtensionHandler, ExtensionRegistry, Model, Result};

/// Custom handler for the Material extension
struct MaterialExtensionHandler;

impl ExtensionHandler for MaterialExtensionHandler {
    fn extension_type(&self) -> Extension {
        Extension::Material
    }

    fn validate(&self, model: &Model) -> Result<()> {
        // Custom material validation logic
        println!("Validating Material extension...");

        // Example: Check if there are any base materials defined
        let has_base_materials = model
            .resources
            .base_material_groups
            .iter()
            .any(|group| !group.materials.is_empty());

        if has_base_materials {
            println!("  ✓ Found base materials");
        }

        // Example: Check for color groups
        let color_group_count = model.resources.color_groups.len();
        println!("  ✓ Found {} color groups", color_group_count);

        Ok(())
    }

    fn is_used_in_model(&self, model: &Model) -> bool {
        // Check if the model actually uses materials
        !model.resources.base_material_groups.is_empty()
            || !model.resources.color_groups.is_empty()
            || !model.resources.texture2d_groups.is_empty()
            || !model.resources.composite_materials.is_empty()
    }
}

/// Custom handler for the Production extension
struct ProductionExtensionHandler;

impl ExtensionHandler for ProductionExtensionHandler {
    fn extension_type(&self) -> Extension {
        Extension::Production
    }

    fn validate(&self, model: &Model) -> Result<()> {
        println!("Validating Production extension...");

        // Check for production paths
        let production_items = model
            .resources
            .objects
            .iter()
            .filter(|obj| obj.production.is_some())
            .count();

        println!(
            "  ✓ Found {} objects with production info",
            production_items
        );

        Ok(())
    }

    fn is_used_in_model(&self, model: &Model) -> bool {
        model
            .resources
            .objects
            .iter()
            .any(|obj| obj.production.is_some())
    }

    fn post_parse(&self, _model: &mut Model) -> Result<()> {
        println!("  Post-parse processing for Production extension");
        Ok(())
    }
}

/// Custom handler for the BeamLattice extension
struct BeamLatticeExtensionHandler;

impl ExtensionHandler for BeamLatticeExtensionHandler {
    fn extension_type(&self) -> Extension {
        Extension::BeamLattice
    }

    fn validate(&self, model: &Model) -> Result<()> {
        println!("Validating BeamLattice extension...");

        let beam_lattice_count = model
            .resources
            .objects
            .iter()
            .filter_map(|obj| obj.mesh.as_ref())
            .filter(|mesh| mesh.beamset.is_some())
            .count();

        println!(
            "  ✓ Found {} objects with beam lattices",
            beam_lattice_count
        );

        Ok(())
    }

    fn is_used_in_model(&self, model: &Model) -> bool {
        model
            .resources
            .objects
            .iter()
            .filter_map(|obj| obj.mesh.as_ref())
            .any(|mesh| mesh.beamset.is_some())
    }
}

fn main() -> Result<()> {
    println!("=== Extension Handler Example ===\n");

    // Create a simple model
    let model = Model::new();

    // Create an extension registry
    let mut registry = ExtensionRegistry::new();

    // Register extension handlers
    println!("Registering extension handlers...");
    registry.register(Box::new(MaterialExtensionHandler));
    registry.register(Box::new(ProductionExtensionHandler));
    registry.register(Box::new(BeamLatticeExtensionHandler));
    println!("  ✓ Registered {} handlers\n", registry.handlers().len());

    // Get specific handler
    if let Some(handler) = registry.get_handler(Extension::Material) {
        println!("Material extension handler:");
        println!("  Name: {}", handler.name());
        println!("  Namespace: {}", handler.namespace());
        println!();
    }

    // Validate all extensions
    println!("Running validation for all registered extensions...");
    registry.validate_all(&model)?;
    println!("\n✓ All validations passed!\n");

    // Demonstrate post-parse hook
    let mut model = model;
    println!("Running post-parse hooks...");
    registry.post_parse_all(&mut model)?;
    println!("\n✓ Post-parse processing complete!\n");

    // Example: Load and validate an actual 3MF file
    println!("=== Usage with actual 3MF files ===\n");
    println!("To validate a 3MF file with custom extensions:");
    println!("1. Create your extension handlers");
    println!("2. Register them in an ExtensionRegistry");
    println!("3. Parse the 3MF file");
    println!("4. Call registry.validate_all(&model)");
    println!("5. Use registry.post_parse_all(&mut model) if needed");

    Ok(())
}
