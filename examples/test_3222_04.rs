use lib3mf::{Model, ParserConfig, Extension};
use std::fs::File;

fn main() {
    let config = ParserConfig::new()
        .with_extension(Extension::Displacement)
        .with_extension(Extension::BooleanOperations)
        .with_extension(Extension::Production)
        .with_extension(Extension::Material)
        .with_custom_extension(
            "http://schemas.3mf.io/3dmanufacturing/displacement/2023/10",
            "Displacement 2023/10",
        );
    
    let file = File::open("test_suites/suite11_Displacement/Positive Tests/P_DPX_3222_04_material.3mf").unwrap();
    match Model::from_reader_with_config(file, config.clone()) {
        Ok(_) => println!("SUCCESS: File parsed successfully"),
        Err(e) => println!("ERROR: {}", e),
    }
}
