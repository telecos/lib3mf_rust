use lib3mf::{Model, ParserConfig, Extension};
use std::fs::File;

fn main() {
    let path = "test_suites/suite2_core_prod_matl/negative_test_cases/N_XPM_0601_02.3mf";
    
    // Use the same config as suite2_core_prod_matl
    let config = ParserConfig::new()
        .with_extension(Extension::Production)
        .with_extension(Extension::Material);
    
    let file = File::open(path).expect("Failed to open file");
    
    match Model::from_reader_with_config(file, config) {
        Ok(_model) => {
            println!("❌ UNEXPECTED: File parsed successfully but should have been rejected!");
            println!("This test case should be rejected because:");
            println!("  - Object 1 has triangles with per-vertex properties (p1/p2/p3)");
            println!("  - Object 1 does NOT have a default pid attribute");
            println!("  - Per 3MF spec, per-vertex properties require a pid context");
            std::process::exit(1);
        }
        Err(e) => {
            println!("✓ File rejected as expected");
            println!("  Error: {}", e);
            std::process::exit(0);
        }
    }
}
