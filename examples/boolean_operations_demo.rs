//! Boolean Operations Visualization Example
//!
//! This example demonstrates how to use the lib3mf viewer to visualize
//! boolean operations from 3MF files.
//!
//! Usage:
//! ```bash
//! cargo run --example boolean_operations_demo test_files/boolean_ops/simple_union.3mf
//! ```

use lib3mf::Model;
use std::env;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get file path from command line or use default
    let args: Vec<String> = env::args().collect();
    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        "test_files/boolean_ops/simple_union.3mf"
    };

    println!("═══════════════════════════════════════════════════════════");
    println!("  Boolean Operations Visualization Demo");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("Loading: {}", file_path);
    println!();

    // Load and parse the 3MF file
    let file = File::open(file_path)?;
    let model = Model::from_reader(file)?;

    println!("✓ Model loaded successfully!");
    println!();

    // Display basic model information
    println!("┌─ Model Information ────────────────────────────────────┐");
    println!("│ Objects:              {:<35} │", model.resources.objects.len());
    println!("│ Build Items:          {:<35} │", model.build.items.len());
    println!("└────────────────────────────────────────────────────────┘");
    println!();

    // Detect and display boolean operations
    let boolean_objects: Vec<_> = model
        .resources
        .objects
        .iter()
        .filter(|obj| obj.boolean_shape.is_some())
        .collect();

    if boolean_objects.is_empty() {
        println!("No boolean operations found in this model.");
        println!();
        println!("To visualize boolean operations:");
        println!("  1. Load a 3MF file with boolean operations");
        println!("  2. Use the viewer with the --ui flag");
        println!("  3. Press 'V' to cycle through visualization modes");
        return Ok(());
    }

    println!("┌─ Boolean Operations ───────────────────────────────────┐");
    println!("│ Total Operations:     {:<35} │", boolean_objects.len());
    println!("└────────────────────────────────────────────────────────┘");
    println!();

    // Display details of each boolean operation
    for (i, obj) in boolean_objects.iter().enumerate() {
        if let Some(ref shape) = obj.boolean_shape {
            println!("┌─ Operation {} ──────────────────────────────────────────┐", i + 1);
            println!("│ Object ID:            {:<35} │", obj.id);
            println!("│ Operation Type:       {:<35} │", shape.operation.as_str());
            println!("│ Base Object ID:       {:<35} │", shape.objectid);
            println!("│ Number of Operands:   {:<35} │", shape.operands.len());
            println!("│                                                        │");
            
            // List all operands
            for (j, operand) in shape.operands.iter().enumerate() {
                println!("│ Operand {}:            {:<35} │", j + 1, operand.objectid);
                if let Some(ref path) = operand.path {
                    println!("│   Path:               {:<35} │", path);
                }
            }
            
            println!("└────────────────────────────────────────────────────────┘");
            println!();
        }
    }

    // Display visualization instructions
    println!("═══════════════════════════════════════════════════════════");
    println!("  Interactive Visualization");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("To interactively visualize these boolean operations:");
    println!();
    println!("  Run the viewer in UI mode:");
    println!("  $ cargo run --release --bin lib3mf-viewer -- --ui {}", file_path);
    println!();
    println!("  Keyboard controls in the viewer:");
    println!("  • V - Cycle through visualization modes:");
    println!("    - Normal: Show all meshes normally");
    println!("    - Show Inputs: Color-code boolean operands");
    println!("      (Blue = base object, Red = operand)");
    println!("    - Highlight Operands: Show only boolean operands");
    println!("      (Bright blue = base, Orange = operands)");
    println!();
    println!("  • T - Cycle through background themes");
    println!("  • A - Toggle coordinate axes");
    println!("  • B - Toggle beam lattice (if present)");
    println!();
    println!("═══════════════════════════════════════════════════════════");

    Ok(())
}
