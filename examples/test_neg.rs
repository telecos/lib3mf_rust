use lib3mf::{Extension, Model, ParserConfig};
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
        "test_suites/suite8_secure/negative_test_cases/N_EPX_2601_01.3mf",
        "test_suites/suite8_secure/negative_test_cases/N_EPX_2602_01.3mf",
        "test_suites/suite8_secure/negative_test_cases/N_EPX_2603_01.3mf",
    ];

    for path in files {
        println!("\n=== Testing {} ===", path);
        match File::open(path) {
            Ok(file) => match Model::from_reader_with_config(file, config.clone()) {
                Ok(_model) => println!("  ✗ SUCCEEDED (expected to fail)"),
                Err(e) => println!("  ✓ FAILED as expected: {}", e),
            },
            Err(e) => println!("  Error opening file: {}", e),
        }
    }
}
