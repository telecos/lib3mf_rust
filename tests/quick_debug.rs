use lib3mf::{Extension, Model, ParserConfig};
use std::fs::File;

#[test]
fn test_0409_files() {
    let suite3_file = "test_suites/suite3_core/negative_test_cases/N_XXX_0409_01.3mf";
    let disp_file = "test_suites/suite11_Displacement/Negative Tests/N_DPX_3316_01.3mf";
    
    // suite3 just uses core
    if let Ok(f3) = File::open(suite3_file) {
        let r3 = Model::from_reader_with_config(f3, ParserConfig::new());
        println!("suite3 N_XXX_0409_01: {}", r3.map(|_| "OK".to_string()).unwrap_or_else(|e| format!("ERR: {}", e)));
    }
    
    // suite11 uses displacement + boolean + production (same as get_suite_config)
    let disp_config = ParserConfig::new()
        .with_extension(Extension::Displacement)
        .with_extension(Extension::BooleanOperations)
        .with_extension(Extension::Production);
    
    if let Ok(disp) = File::open(disp_file) {
        let r11 = Model::from_reader_with_config(disp, disp_config);
        println!("suite11 N_DPX_3316_01: {}", r11.map(|_| "OK".to_string()).unwrap_or_else(|e| format!("ERR: {}", e)));
    }
}
