use lib3mf::{Extension, Model, ParserConfig};
use std::fs::File;

#[test]
fn test_1605_path() {
    let config = ParserConfig::new()
        .with_extension(Extension::Production)
        .with_extension(Extension::Slice);

    let file =
        File::open("test_suites/suite1_core_slice_prod/negative_test_cases/N_SPX_1605_01.3mf")
            .unwrap();
    let result = Model::from_reader_with_config(file, config);

    match result {
        Ok(_) => println!("File parsed successfully"),
        Err(e) => println!("Error: {}", e),
    }
}
