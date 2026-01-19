//! Example demonstrating 3MF extension support
//!
//! This example shows how to:
//! 1. Parse 3MF files and check their required extensions
//! 2. Configure the parser to accept only specific extensions
//! 3. Handle files that require unsupported extensions

use lib3mf::{Extension, Model, ParserConfig};
use std::env;
use std::fs::File;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <3mf-file> [extension-mode]", args[0]);
        eprintln!();
        eprintln!("extension-mode:");
        eprintln!("  all        - Accept all extensions (default)");
        eprintln!("  core-only  - Accept only core specification");
        eprintln!("  core-mat   - Accept core + materials extension");
        process::exit(1);
    }

    let filename = &args[1];
    let mode = args.get(2).map(|s| s.as_str()).unwrap_or("all");

    // Configure parser based on mode
    let config = match mode {
        "core-only" => {
            println!("Parser configured: Core only");
            ParserConfig::new()
        }
        "core-mat" => {
            println!("Parser configured: Core + Materials");
            ParserConfig::new().with_extension(Extension::Material)
        }
        "all" => {
            println!("Parser configured: All extensions");
            ParserConfig::with_all_extensions()
        }
        _ => {
            eprintln!("Unknown mode: {}", mode);
            process::exit(1);
        }
    };

    println!("Parsing: {}", filename);
    println!();

    // Open and parse the file
    let file = match File::open(filename) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening file: {}", e);
            process::exit(1);
        }
    };

    match Model::from_reader_with_config(file, config) {
        Ok(model) => {
            println!("✅ File parsed successfully!");
            println!();
            println!("Model Information:");
            println!("  Unit: {}", model.unit);
            println!("  Objects: {}", model.resources.objects.len());
            println!("  Build items: {}", model.build.items.len());
            println!();

            if model.required_extensions.is_empty() {
                println!("Required extensions: None (core only)");
            } else {
                println!("Required extensions:");
                for ext in &model.required_extensions {
                    println!("  - {} ({})", ext.name(), ext.namespace());
                }
            }
            println!();

            // Show materials if present
            if !model.resources.materials.is_empty() {
                println!("Materials: {}", model.resources.materials.len());
            }
            if !model.resources.color_groups.is_empty() {
                println!("Color groups: {}", model.resources.color_groups.len());
                for cg in &model.resources.color_groups {
                    println!("  Color group {}: {} colors", cg.id, cg.colors.len());
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Error parsing file: {}", e);
            eprintln!();
            eprintln!("This likely means the file requires an extension that");
            eprintln!("is not supported by the current parser configuration.");
            eprintln!();
            eprintln!("Try running with 'all' mode to accept all extensions:");
            eprintln!("  {} {} all", args[0], filename);
            process::exit(1);
        }
    }
}
