use lib3mf::{Extension, Model, ParserConfig};
use std::fs::File;

#[test]
fn test_individual_failures() {
    let config = ParserConfig::new()
        .with_extension(Extension::Production)
        .with_extension(Extension::Slice);

    let files = vec![
        (
            "N_SPX_1605_01",
            "test_suites/suite1_core_slice_prod/negative_test_cases/N_SPX_1605_01.3mf",
        ),
        (
            "N_SPX_0417_01",
            "test_suites/suite1_core_slice_prod/negative_test_cases/N_SPX_0417_01.3mf",
        ),
        (
            "N_SPX_0415_01",
            "test_suites/suite1_core_slice_prod/negative_test_cases/N_SPX_0415_01.3mf",
        ),
        (
            "N_SPX_1607_01",
            "test_suites/suite1_core_slice_prod/negative_test_cases/N_SPX_1607_01.3mf",
        ),
        (
            "N_SPX_1609_01",
            "test_suites/suite1_core_slice_prod/negative_test_cases/N_SPX_1609_01.3mf",
        ),
        (
            "N_SPX_1606_01",
            "test_suites/suite1_core_slice_prod/negative_test_cases/N_SPX_1606_01.3mf",
        ),
        (
            "N_SPX_1610_01",
            "test_suites/suite1_core_slice_prod/negative_test_cases/N_SPX_1610_01.3mf",
        ),
        (
            "N_SPX_1608_01",
            "test_suites/suite1_core_slice_prod/negative_test_cases/N_SPX_1608_01.3mf",
        ),
        (
            "N_SPX_0419_01",
            "test_suites/suite1_core_slice_prod/negative_test_cases/N_SPX_0419_01.3mf",
        ),
        (
            "N_SPX_1609_02",
            "test_suites/suite1_core_slice_prod/negative_test_cases/N_SPX_1609_02.3mf",
        ),
    ];

    for (name, path) in files {
        println!("\n=== Testing {} ===", name);
        match File::open(path) {
            Ok(file) => match Model::from_reader_with_config(file, config.clone()) {
                Ok(_) => println!("✗ INCORRECTLY SUCCEEDED (should have failed)"),
                Err(e) => println!("✓ Correctly failed: {}", e),
            },
            Err(e) => println!("Error opening file: {}", e),
        }
    }
}
