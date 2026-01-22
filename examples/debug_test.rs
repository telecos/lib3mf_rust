use lib3mf::{Extension, Model, ParserConfig};
use std::fs::File;

fn main() {
    let config = ParserConfig::new()
        .with_extension(Extension::BooleanOperations)
        .with_extension(Extension::Production)
        .with_extension(Extension::BeamLattice)
        .with_extension(Extension::Material);

    let files = vec![
        "test_suites/suite10_boolean/Positive Tests/P_OPX_3011_01.3mf",
        "test_suites/suite10_boolean/Positive Tests/P_OPX_3011_02.3mf",
        "test_suites/suite10_boolean/Positive Tests/P_OPX_3011_03.3mf",
    ];

    for path in files {
        println!("\n=== Testing {} ===", path);
        match File::open(path) {
            Ok(file) => match Model::from_reader_with_config(file, config.clone()) {
                Ok(model) => {
                    println!("✓ SUCCESS");
                    println!("  Objects: {}", model.resources.objects.len());
                    for obj in &model.resources.objects {
                        println!("  Object {}: {} components", obj.id, obj.components.len());
                        for (i, comp) in obj.components.iter().enumerate() {
                            println!("    Component {}: objid={}, path={:?}, production={:?}", 
                                i, comp.objectid, comp.path, comp.production);
                        }
                    }
                }
                Err(e) => println!("✗ FAILED: {}", e),
            },
            Err(e) => println!("Error opening file: {}", e),
        }
    }
}
