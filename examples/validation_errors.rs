//! Example: Validation and error handling
//!
//! This example demonstrates how to:
//! 1. Handle various types of parsing errors
//! 2. Validate file structure and content
//! 3. Provide meaningful error messages
//! 4. Check for common issues in 3MF files
//!
//! This is useful for building robust applications that can gracefully handle
//! invalid or malformed 3MF files.

use lib3mf::{Model, ParserConfig};
use std::env;
use std::fs::File;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <3mf-file> [strict|permissive]", args[0]);
        eprintln!();
        eprintln!("Validates a 3MF file and demonstrates error handling");
        eprintln!();
        eprintln!("Modes:");
        eprintln!("  strict      - Only accept core specification (no extensions)");
        eprintln!("  permissive  - Accept all extensions (default)");
        process::exit(1);
    }

    let filename = &args[1];
    let mode = args.get(2).map(|s| s.as_str()).unwrap_or("permissive");

    println!("=== 3MF File Validation ===");
    println!("File: {}", filename);
    println!("Mode: {}", mode);
    println!();

    // Configure parser based on strictness
    let config = match mode {
        "strict" => {
            println!("Using STRICT validation (core only)");
            ParserConfig::new()
        }
        "permissive" => {
            println!("Using PERMISSIVE validation (all extensions)");
            ParserConfig::with_all_extensions()
        }
        _ => {
            eprintln!("Unknown mode: {}", mode);
            process::exit(1);
        }
    };
    println!();

    // Step 1: Try to open the file
    println!("Step 1: Opening file...");
    let file = match File::open(filename) {
        Ok(f) => {
            println!("  ✓ File opened successfully");
            f
        }
        Err(e) => {
            eprintln!("  ✗ ERROR: Failed to open file");
            eprintln!("    Reason: {}", e);
            eprintln!();
            eprintln!("Possible causes:");
            eprintln!("  - File does not exist");
            eprintln!("  - Insufficient permissions");
            eprintln!("  - File is locked by another process");
            process::exit(1);
        }
    };
    println!();

    // Step 2: Parse the 3MF file
    println!("Step 2: Parsing 3MF structure...");
    let model = match Model::from_reader_with_config(file, config) {
        Ok(m) => {
            println!("  ✓ File parsed successfully!");
            m
        }
        Err(e) => {
            eprintln!("  ✗ ERROR: Failed to parse 3MF file");
            eprintln!("    Error type: {}", e);
            eprintln!();

            // Provide specific guidance based on error type
            let error_str = format!("{}", e);
            
            if error_str.contains("ZIP") {
                eprintln!("This appears to be a ZIP-related error.");
                eprintln!("Possible causes:");
                eprintln!("  - File is not a valid ZIP archive");
                eprintln!("  - File is corrupted");
                eprintln!("  - File is not actually a 3MF file");
            } else if error_str.contains("XML") {
                eprintln!("This appears to be an XML parsing error.");
                eprintln!("Possible causes:");
                eprintln!("  - Invalid XML syntax in model file");
                eprintln!("  - Unexpected XML structure");
                eprintln!("  - Missing required XML elements");
            } else if error_str.contains("extension") {
                eprintln!("This appears to be an extension compatibility error.");
                eprintln!("The file requires an extension that is not supported.");
                eprintln!();
                eprintln!("Try running with 'permissive' mode:");
                eprintln!("  {} {} permissive", args[0], filename);
            } else if error_str.contains("Missing required file") {
                eprintln!("This appears to be a structure error.");
                eprintln!("The 3MF archive is missing required files.");
                eprintln!("Valid 3MF files must contain:");
                eprintln!("  - [Content_Types].xml");
                eprintln!("  - _rels/.rels");
                eprintln!("  - 3D/3dmodel.model");
            } else if error_str.contains("Invalid") {
                eprintln!("The file structure or content is invalid.");
                eprintln!("Possible causes:");
                eprintln!("  - Malformed vertex or triangle data");
                eprintln!("  - Invalid attribute values");
                eprintln!("  - Non-compliant with 3MF specification");
            }

            process::exit(1);
        }
    };
    println!();

    // Step 3: Validate model content
    println!("Step 3: Validating model content...");
    
    let mut warnings: Vec<String> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    // Check for required extensions
    if !model.required_extensions.is_empty() {
        println!("  Required extensions:");
        for ext in &model.required_extensions {
            println!("    - {} ({})", ext.name(), ext.namespace());
        }
    } else {
        println!("  ✓ No extensions required (core only)");
    }

    // Check for objects
    if model.resources.objects.is_empty() {
        errors.push("No objects found in model".to_string());
    } else {
        println!("  ✓ Found {} objects", model.resources.objects.len());
    }

    // Check for build items
    if model.build.items.is_empty() {
        warnings.push("No build items specified - nothing will be manufactured".to_string());
    } else {
        println!("  ✓ Found {} build items", model.build.items.len());
    }

    // Validate each object
    for obj in &model.resources.objects {
        if let Some(ref mesh) = obj.mesh {
            // Check for vertices
            if mesh.vertices.is_empty() {
                errors.push(format!("Object {} has no vertices", obj.id));
            }

            // Check for triangles
            if mesh.triangles.is_empty() {
                warnings.push(format!("Object {} has no triangles", obj.id));
            }

            // Check for degenerate triangles
            for (i, tri) in mesh.triangles.iter().enumerate() {
                if tri.v1 == tri.v2 || tri.v2 == tri.v3 || tri.v1 == tri.v3 {
                    warnings.push(format!(
                        "Object {} triangle {} is degenerate (repeated vertices)",
                        obj.id, i
                    ));
                }

                // Check vertex indices are in range
                if tri.v1 >= mesh.vertices.len()
                    || tri.v2 >= mesh.vertices.len()
                    || tri.v3 >= mesh.vertices.len()
                {
                    errors.push(format!(
                        "Object {} triangle {} has out-of-range vertex index",
                        obj.id, i
                    ));
                }
            }
        } else {
            warnings.push(format!("Object {} has no mesh data", obj.id));
        }
    }

    // Validate build items reference valid objects
    for (i, item) in model.build.items.iter().enumerate() {
        let obj_exists = model
            .resources
            .objects
            .iter()
            .any(|o| o.id == item.objectid);
        if !obj_exists {
            errors.push(format!(
                "Build item {} references non-existent object ID {}",
                i, item.objectid
            ));
        }
    }

    println!();

    // Report validation results
    if !errors.is_empty() {
        println!("ERRORS found ({}):", errors.len());
        for error in &errors {
            println!("  ✗ {}", error);
        }
        println!();
    }

    if !warnings.is_empty() {
        println!("WARNINGS ({}):", warnings.len());
        for warning in &warnings {
            println!("  ⚠ {}", warning);
        }
        println!();
    }

    // Final summary
    if errors.is_empty() && warnings.is_empty() {
        println!("=== VALIDATION PASSED ===");
        println!("✓ File is valid with no errors or warnings");
    } else if errors.is_empty() {
        println!("=== VALIDATION PASSED WITH WARNINGS ===");
        println!("✓ File is valid but has {} warning(s)", warnings.len());
    } else {
        println!("=== VALIDATION FAILED ===");
        println!("✗ File has {} error(s)", errors.len());
        process::exit(1);
    }

    // Print summary statistics
    println!();
    println!("Model Summary:");
    println!("  Unit: {}", model.unit);
    println!("  Objects: {}", model.resources.objects.len());
    
    let total_vertices: usize = model
        .resources
        .objects
        .iter()
        .filter_map(|o| o.mesh.as_ref())
        .map(|m| m.vertices.len())
        .sum();
    let total_triangles: usize = model
        .resources
        .objects
        .iter()
        .filter_map(|o| o.mesh.as_ref())
        .map(|m| m.triangles.len())
        .sum();

    println!("  Total vertices: {}", total_vertices);
    println!("  Total triangles: {}", total_triangles);
    println!("  Materials: {}", model.resources.materials.len());
    println!("  Color groups: {}", model.resources.color_groups.len());
    println!("  Build items: {}", model.build.items.len());
    println!("  Metadata entries: {}", model.metadata.len());
}
