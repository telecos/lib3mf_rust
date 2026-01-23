use lib3mf::{Extension, Model, ParserConfig};
use std::fs::File;

fn main() {
    let config = ParserConfig::new()
        .with_extension(Extension::SecureContent)
        .with_extension(Extension::Production)
        .with_extension(Extension::Material)
        .with_extension(Extension::Slice)
        .with_custom_extension(
            "http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/04",
            "SecureContent 2019/04",
        );

    let files = vec![
        ("N_EPX_2602_01.3mf", true),  // Should fail
        ("N_EPX_2602_02.3mf", true),  // Should fail
        ("N_EPX_2602_03.3mf", true),  // Should fail  
        ("N_EPX_2602_04.3mf", true),  // Should fail
    ];

    for (filename, should_fail) in files {
        let path = format!("test_suites/suite8_secure/negative_test_cases/{}", filename);
        println!("\n=== Testing {} ===", filename);
        match File::open(&path) {
            Ok(file) => match Model::from_reader_with_config(file, config.clone()) {
                Ok(model) => {
                    if should_fail {
                        println!("  ✗ SUCCEEDED (expected to fail)");
                        if let Some(ref sc) = model.secure_content {
                            println!("    Consumers: {}", sc.consumers.len());
                            println!("    Resource groups: {}", sc.resource_data_groups.len());
                            for (i, group) in sc.resource_data_groups.iter().enumerate() {
                                println!("    Group {}: {} access rights", i, group.access_rights.len());
                            }
                        }
                    } else {
                        println!("  ✓ SUCCEEDED as expected");
                    }
                }
                Err(e) => {
                    if should_fail {
                        println!("  ✓ FAILED as expected: {}", e);
                    } else {
                        println!("  ✗ FAILED (expected to succeed): {}", e);
                    }
                }
            },
            Err(e) => println!("  Error opening file: {}", e),
        }
    }
}
