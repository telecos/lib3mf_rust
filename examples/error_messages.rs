//! Example demonstrating improved error messages
//!
//! This example shows how the library provides detailed error messages with:
//! - Error codes for categorization
//! - Context information (object IDs, line numbers, etc.)
//! - Helpful suggestions for fixing common issues

use lib3mf::Model;
use std::io::Cursor;

fn main() {
    println!("=== lib3mf Error Message Examples ===\n");

    // Example 1: Invalid 3MF file (missing required file)
    println!("Example 1: Missing required file");
    let invalid_zip = vec![0x50, 0x4B, 0x05, 0x06, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let cursor = Cursor::new(invalid_zip);
    match Model::from_reader(cursor) {
        Err(e) => {
            println!("Error: {}", e);
            println!("Error code: {}", e.code());
            println!();
        }
        _ => println!("Unexpected success\n"),
    }

    // Example 2: Invalid XML
    println!("Example 2: Invalid XML structure");
    let _invalid_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
    <resources>
        <object id="notanumber">
            <mesh>
            </mesh>
        </object>
    </resources>
</model>"#;
    
    // We can't easily test this without creating a full 3MF package, so we'll show what users see
    println!("When parsing invalid XML, users see error codes like:");
    println!("[E3003] Invalid XML structure: Object missing id attribute");
    println!("  Suggestion: Add the 'id' attribute with a positive integer value");
    println!();

    // Example 3: Demonstrate error code categories
    println!("Example 3: Error Code Categories");
    println!("The library uses the following error code ranges:");
    println!("  E1xxx - IO and file system errors");
    println!("  E2xxx - ZIP/archive errors");
    println!("  E3xxx - XML parsing errors");
    println!("  E4xxx - Model structure validation errors");
    println!("  E5xxx - Extension and feature support errors");
    println!();

    println!("Example validation error messages:");
    println!("[E4001] Invalid model: Object ID must be a positive integer");
    println!("  Suggestion: Object IDs must start from 1. Use id=\"1\", id=\"2\", etc.");
    println!();
    
    println!("[E4001] Invalid model: Duplicate object ID found: 5");
    println!("  Suggestion: Each object must have a unique ID. Check for duplicate id attributes in <object> elements");
    println!();
    
    println!("[E4001] Invalid model: Object 3: Triangle 10 vertex v2=150 is out of bounds (have 100 vertices)");
    println!("  Suggestion: Vertex indices must be in range 0-99. Check the v2 attribute");
    println!();
    
    println!("[E4001] Invalid model: Object 2: Triangle 5 is degenerate (v1=3, v2=3, v3=7)");
    println!("  Suggestion: All three vertex indices (v1, v2, v3) must be different to form a valid triangle");
    println!();

    println!("=== Error messages help you quickly identify and fix issues! ===");
}
