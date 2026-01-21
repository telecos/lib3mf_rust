use lib3mf::{Model, ParserConfig, Extension};
use std::fs::File;

fn main() {
    let config = ParserConfig::new()
        .with_extension(Extension::SecureContent)
        .with_extension(Extension::Production)
        .with_custom_extension(
            "http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/04",
            "SecureContent 2019/04",
        );
    
    let files = vec![
        "test_suites/suite8_secure/positive_test_cases/P_EPX_2101_03.3mf",
        "test_suites/suite8_secure/positive_test_cases/P_EPX_2101_02.3mf",
        "test_suites/suite8_secure/positive_test_cases/P_EPX_2111_02.3mf",
    ];
    
    for path in files {
        println!("\n=== Testing {} ===", path);
        match File::open(path) {
            Ok(file) => {
                match Model::from_reader_with_config(file, config.clone()) {
                    Ok(model) => {
                        println!("✓ SUCCESS");
                        println!("  Objects: {}", model.resources.objects.len());
                    }
                    Err(e) => println!("✗ FAILED: {}", e),
                }
            }
            Err(e) => println!("Error opening file: {}", e),
        }
    }
}
