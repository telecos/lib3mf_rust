//! Example demonstrating thumbnail extraction from 3MF files
//!
//! This example shows how to:
//! - Check if a 3MF file contains a thumbnail
//! - Extract thumbnail metadata (path and content type)
//! - Read the thumbnail binary data and save it to a file

use lib3mf::Model;
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse a 3MF file
    let file = File::open("test_files/test_thumbnail.3mf")?;
    let model = Model::from_reader(file)?;

    // Check if model has a thumbnail
    if let Some(ref thumbnail) = model.thumbnail {
        println!("Thumbnail found!");
        println!("  Path: {}", thumbnail.path);
        println!("  Content Type: {}", thumbnail.content_type);

        // Read the thumbnail binary data
        let file = File::open("test_files/test_thumbnail.3mf")?;
        if let Some(thumbnail_data) = lib3mf::parser::read_thumbnail(file)? {
            println!("  Size: {} bytes", thumbnail_data.len());

            // Save thumbnail to a file
            let output_path = "extracted_thumbnail.png";
            let mut output_file = File::create(output_path)?;
            output_file.write_all(&thumbnail_data)?;
            println!("  Saved to: {}", output_path);
        }
    } else {
        println!("No thumbnail found in this 3MF file");
    }

    // Display basic model info
    println!("\nModel Information:");
    println!("  Unit: {}", model.unit);
    println!("  Objects: {}", model.resources.objects.len());
    println!("  Build items: {}", model.build.items.len());

    Ok(())
}
