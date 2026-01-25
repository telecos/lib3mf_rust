/// Demonstration of proper validation for multiproperties test cases 0604_01 and 0604_03
///
/// This example verifies that the parser correctly rejects invalid multiproperties configurations
/// according to the 3MF Material Extension specification.

use lib3mf::{Extension, Model, ParserConfig};
use std::fs::File;

fn main() {
    println!("=== Verifying Multiproperties Validation ===\n");

    let config = ParserConfig::new()
        .with_extension(Extension::Material)
        .with_extension(Extension::Production);

    // Test 0604_01: Multiple colorgroups in multiproperties
    println!("Test 0604_01: Multiple colorgroups in multiproperties pids");
    println!("File: N_XPM_0604_01.3mf");
    println!("Expected: REJECT (pids list MUST NOT contain more than one colorgroup)\n");

    let file =
        File::open("test_suites/suite2_core_prod_matl/negative_test_cases/N_XPM_0604_01.3mf")
            .expect("Test file not found");

    match Model::from_reader_with_config(file, config.clone()) {
        Ok(_) => println!("❌ FAILED: File was accepted but should have been rejected\n"),
        Err(e) => {
            println!("✅ PASSED: File was correctly rejected");
            println!("Error: {}\n", e);
        }
    }

    // Test 0604_03: Basematerials at wrong position in multiproperties
    println!("---\n");
    println!("Test 0604_03: Basematerials at layer 1 of multiproperties");
    println!("File: N_XPM_0604_03.3mf");
    println!("Expected: REJECT (basematerials MUST be at layer 0 if included)\n");

    let file =
        File::open("test_suites/suite2_core_prod_matl/negative_test_cases/N_XPM_0604_03.3mf")
            .expect("Test file not found");

    match Model::from_reader_with_config(file, config) {
        Ok(_) => println!("❌ FAILED: File was accepted but should have been rejected\n"),
        Err(e) => {
            println!("✅ PASSED: File was correctly rejected");
            println!("Error: {}\n", e);
        }
    }

    println!("=== Validation Complete ===");
}
