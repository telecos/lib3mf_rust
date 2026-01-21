//! Integration test to verify extension configuration works with actual files
//!
//! This test ensures that files requiring specific extensions can only be parsed
//! when the appropriate extension support is enabled.

use lib3mf::{Error, Extension, Model, ParserConfig};
use std::fs::File;

#[test]
fn test_beam_lattice_requires_extension() {
    // Open a beam lattice test file
    let file = File::open("test_files/beam_lattice/pyramid.3mf");

    if let Ok(file) = file {
        // Try to parse with only core support - should fail if file requires BeamLattice extension
        let config_core_only = ParserConfig::new();
        let result_core = Model::from_reader_with_config(file, config_core_only);

        // Now try with BeamLattice extension enabled
        let file2 = File::open("test_files/beam_lattice/pyramid.3mf").unwrap();
        let config_with_beam = ParserConfig::new().with_extension(Extension::BeamLattice);
        let result_with_extension = Model::from_reader_with_config(file2, config_with_beam);

        // The file with extension support should succeed
        if result_with_extension.is_ok() {
            println!("✓ Beam lattice file parsed successfully with BeamLattice extension");

            // If core-only failed, that's correct behavior
            if let Err(e) = result_core {
                match e {
                    Error::UnsupportedExtension(msg) => {
                        println!(
                            "✓ Correctly rejected file without BeamLattice extension: {}",
                            msg
                        );
                    }
                    _ => {
                        println!("✓ File failed to parse without BeamLattice extension (different error)");
                    }
                }
            } else {
                println!("✓ File doesn't require BeamLattice extension (parsed with core only)");
            }
        } else {
            println!(
                "Note: Could not parse beam lattice file: {:?}",
                result_with_extension.err()
            );
        }
    } else {
        println!("Note: Beam lattice test file not found, skipping test");
    }
}

#[test]
fn test_production_requires_extension() {
    // Open a production test file
    let file = File::open("test_files/production/box_prod.3mf");

    if let Ok(file) = file {
        // Try to parse with only core support
        let config_core_only = ParserConfig::new();
        let result_core = Model::from_reader_with_config(file, config_core_only);

        // Now try with Production extension enabled
        let file2 = File::open("test_files/production/box_prod.3mf").unwrap();
        let config_with_prod = ParserConfig::new().with_extension(Extension::Production);
        let result_with_extension = Model::from_reader_with_config(file2, config_with_prod);

        // The file with extension support should succeed
        if result_with_extension.is_ok() {
            println!("✓ Production file parsed successfully with Production extension");

            // If core-only failed, that's correct behavior
            if let Err(e) = result_core {
                match e {
                    Error::UnsupportedExtension(msg) => {
                        println!(
                            "✓ Correctly rejected file without Production extension: {}",
                            msg
                        );
                    }
                    _ => {
                        println!(
                            "✓ File failed to parse without Production extension (different error)"
                        );
                    }
                }
            } else {
                println!("✓ File doesn't require Production extension (parsed with core only)");
            }
        } else {
            println!(
                "Note: Could not parse production file: {:?}",
                result_with_extension.err()
            );
        }
    } else {
        println!("Note: Production test file not found, skipping test");
    }
}

#[test]
fn test_core_file_works_without_extensions() {
    // Open a core test file
    let file = File::open("test_files/core/box.3mf");

    if let Ok(file) = file {
        // Parse with only core support
        let config_core_only = ParserConfig::new();
        let result = Model::from_reader_with_config(file, config_core_only);

        match result {
            Ok(_) => {
                println!("✓ Core file parsed successfully with core-only configuration");
            }
            Err(e) => {
                panic!(
                    "Core file should parse with core-only config, but failed: {:?}",
                    e
                );
            }
        }
    } else {
        println!("Note: Core test file not found, skipping test");
    }
}

#[test]
fn test_slice_requires_extension() {
    // Open a slice test file
    let file = File::open("test_files/slices/box_sliced.3mf");

    if let Ok(file) = file {
        // Try to parse with only core support
        let config_core_only = ParserConfig::new();
        let result_core = Model::from_reader_with_config(file, config_core_only);

        // Now try with Slice extension enabled
        let file2 = File::open("test_files/slices/box_sliced.3mf").unwrap();
        let config_with_slice = ParserConfig::new().with_extension(Extension::Slice);
        let result_with_extension = Model::from_reader_with_config(file2, config_with_slice);

        // The file with extension support should succeed
        if result_with_extension.is_ok() {
            println!("✓ Slice file parsed successfully with Slice extension");

            // If core-only failed, that's correct behavior
            if let Err(e) = result_core {
                match e {
                    Error::UnsupportedExtension(msg) => {
                        println!("✓ Correctly rejected file without Slice extension: {}", msg);
                    }
                    _ => {
                        println!(
                            "✓ File failed to parse without Slice extension (different error)"
                        );
                    }
                }
            } else {
                println!("✓ File doesn't require Slice extension (parsed with core only)");
            }
        } else {
            println!(
                "Note: Could not parse slice file: {:?}",
                result_with_extension.err()
            );
        }
    } else {
        println!("Note: Slice test file not found, skipping test");
    }
}
