use lib3mf::{Extension, Model, ParserConfig};
use std::fs::File;

#[test]
fn test_cmyk_jpeg() {
    let config = ParserConfig::new()
        .with_extension(Extension::Production)
        .with_extension(Extension::Slice);

    let path = "test_suites/suite1_core_slice_prod/negative_test_cases/N_SPX_0419_01.3mf";
    let file = File::open(path).unwrap();
    let result = Model::from_reader_with_config(file, config);

    match result {
        Ok(_) => {
            println!("ERROR: File should have been rejected for CMYK JPEG");
            panic!("File incorrectly accepted");
        }
        Err(e) => {
            println!("Correctly rejected with error: {}", e);
            assert!(e.to_string().contains("CMYK") || e.to_string().contains("color space"));
        }
    }
}
