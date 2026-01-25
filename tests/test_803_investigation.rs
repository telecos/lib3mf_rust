use lib3mf::{Model, ParserConfig};
use std::fs::File;

#[test]
fn test_803_01_investigate() {
    let config = ParserConfig::new()
        .with_extension(lib3mf::Extension::Production)
        .with_extension(lib3mf::Extension::Material);
    
    let file = File::open("test_suites/suite2_core_prod_matl/negative_test_cases/N_XPM_0803_01.3mf").unwrap();
    match Model::from_reader_with_config(file, config) {
        Ok(model) => {
            eprintln!("✗ File parsed successfully (should have failed!)");
            eprintln!("Model has {} objects", model.resources.objects.len());
            for obj in &model.resources.objects {
                eprintln!("  Object {}: {} components", obj.id, obj.components.len());
                for (i, comp) in obj.components.iter().enumerate() {
                    eprintln!("    Component {}: objectid={}, path={:?}, production={:?}", 
                              i, comp.objectid, comp.path, comp.production);
                }
            }
            panic!("File should have failed validation");
        },
        Err(e) => {
            eprintln!("✓ File failed to parse as expected: {}", e);
        }
    }
}
