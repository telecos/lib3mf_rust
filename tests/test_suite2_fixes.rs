//! Test for suite2 positive test case fixes

use lib3mf::{Extension, Model, ParserConfig};
use std::fs::File;

#[test]
fn test_p_xpm_0505_01() {
    let config = ParserConfig::new()
        .with_extension(Extension::Production)
        .with_extension(Extension::Material);

    let file = File::open("test_suites/suite2_core_prod_matl/positive_test_cases/P_XPM_0505_01.3mf")
        .expect("Test file should exist");

    let result = Model::from_reader_with_config(file, config);
    assert!(
        result.is_ok(),
        "P_XPM_0505_01.3mf should parse successfully"
    );
}

#[test]
fn test_p_xpm_0504_03() {
    let config = ParserConfig::new()
        .with_extension(Extension::Production)
        .with_extension(Extension::Material);

    let file = File::open("test_suites/suite2_core_prod_matl/positive_test_cases/P_XPM_0504_03.3mf")
        .expect("Test file should exist");

    let result = Model::from_reader_with_config(file, config);
    assert!(
        result.is_ok(),
        "P_XPM_0504_03.3mf should parse successfully"
    );
}

#[test]
fn test_p_xpm_0337_06() {
    let config = ParserConfig::new()
        .with_extension(Extension::Production)
        .with_extension(Extension::Material);

    let file = File::open("test_suites/suite2_core_prod_matl/positive_test_cases/P_XPM_0337_06.3mf")
        .expect("Test file should exist");

    let result = Model::from_reader_with_config(file, config);
    assert!(
        result.is_ok(),
        "P_XPM_0337_06.3mf should parse successfully (external reference)"
    );
}
