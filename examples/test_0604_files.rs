use lib3mf::{Extension, Model, ParserConfig};
use std::fs::File;

fn main() {
    println!("Testing N_XPM_0604_01.3mf (two colorgroup references in multiproperties)...");
    let config = ParserConfig::new()
        .with_extension(Extension::Material)
        .with_extension(Extension::Production);
    
    let file = File::open("test_suites/suite2_core_prod_matl/negative_test_cases/N_XPM_0604_01.3mf").unwrap();
    match Model::from_reader_with_config(file, config.clone()) {
        Ok(_) => println!("  ❌ FAILED: File should have been rejected but was accepted"),
        Err(e) => println!("  ✓ PASSED: File was rejected with error:\n    {}", e),
    }
    
    println!("\nTesting N_XPM_0604_03.3mf (basematerials as layer 2 of multiproperties)...");
    let file = File::open("test_suites/suite2_core_prod_matl/negative_test_cases/N_XPM_0604_03.3mf").unwrap();
    match Model::from_reader_with_config(file, config) {
        Ok(_) => println!("  ❌ FAILED: File should have been rejected but was accepted"),
        Err(e) => println!("  ✓ PASSED: File was rejected with error:\n    {}", e),
    }
}
