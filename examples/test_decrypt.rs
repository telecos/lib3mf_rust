use std::fs::File;
use lib3mf::{Model, ParserConfig, Extension};

fn main() {
    let test_file = "test_suites/suite8_secure/positive_test_cases/P_EPX_2101_02.3mf";
    
    let config = ParserConfig::new()
        .with_extension(Extension::SecureContent)
        .with_extension(Extension::Production)
        .with_extension(Extension::Material)
        .with_extension(Extension::Slice);
    
    let file = File::open(test_file).unwrap();
    let result = Model::from_reader_with_config(file, config);
    
    match result {
        Ok(model) => {
            println!("Model loaded successfully!");
            if let Some(ref sc) = model.secure_content {
                println!("Secure content info:");
                println!("  Consumers: {}", sc.consumers.len());
                println!("  Resource data groups: {}", sc.resource_data_groups.len());
                
                for (i, group) in sc.resource_data_groups.iter().enumerate() {
                    println!("\nGroup {}:", i);
                    println!("  Key UUID: {}", group.key_uuid);
                    println!("  Access rights: {}", group.access_rights.len());
                    
                    for (j, ar) in group.access_rights.iter().enumerate() {
                        println!("\n  Access right {}:", j);
                        println!("    Consumer index: {}", ar.consumer_index);
                        println!("    Wrapping algorithm: {}", ar.kek_params.wrapping_algorithm);
                        println!("    Digest method: {:?}", ar.kek_params.digest_method);
                        println!("    MGF algorithm: {:?}", ar.kek_params.mgf_algorithm);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
