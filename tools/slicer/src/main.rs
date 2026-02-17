//! lib3mf-slicer: A tool for slicing 3MF files into 2D images
//!
//! This tool takes a 3MF file and a JSON configuration file, and generates
//! slice images at specified intervals within a printable box volume.

#![forbid(unsafe_code)]

mod color;
mod config;
mod displacement;
mod renderer;
mod slicer;

use clap::Parser;
use config::SlicerConfig;
use slicer::Slicer;
use std::path::PathBuf;

/// Command-line arguments for lib3mf-slicer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the 3MF file to slice
    #[arg(value_name = "INPUT_FILE")]
    input_file: PathBuf,

    /// Path to the JSON configuration file
    #[arg(value_name = "CONFIG_FILE")]
    config_file: PathBuf,

    /// Output directory for slice images (default: ./slices)
    #[arg(short, long, value_name = "OUTPUT_DIR", default_value = "slices")]
    output: PathBuf,

    /// Show detailed model information
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("=== lib3mf-slicer ===\n");

    // Load configuration
    println!("Loading configuration from: {}", args.config_file.display());
    let config_path = args
        .config_file
        .to_str()
        .ok_or("Invalid UTF-8 in config file path")?;
    let config = SlicerConfig::from_file(config_path)?;
    println!("Configuration loaded successfully.\n");

    // Create slicer
    let slicer = Slicer::new(config);

    // Load 3MF model
    println!("Loading 3MF file: {}", args.input_file.display());
    let model = slicer.load_model(&args.input_file)?;
    println!("Model loaded successfully.\n");

    // Show model information if verbose
    if args.verbose {
        slicer.print_model_info(&model);
    }

    // Slice the model
    println!("Output directory: {}\n", args.output.display());
    let output_files = slicer.slice_model(&model, &args.input_file, &args.output)?;

    println!("\n=== Slicing Summary ===");
    println!("  Total slices: {}", output_files.len());
    println!("  Output location: {}", args.output.display());
    println!("\nDone!");

    Ok(())
}
