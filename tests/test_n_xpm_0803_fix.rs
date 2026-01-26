/// Test to verify N_XPM_0803_01 fix
///
/// This test verifies that the parser correctly rejects 3MF files where
/// non-root model files contain components with p:path attributes.
///
/// Per 3MF Production Extension spec Chapter 2:
/// "Non-root model file components MUST only reference objects in the same model file."
use lib3mf::{Model, ParserConfig};
use std::fs::File;

#[test]
#[cfg_attr(not(feature = "ci"), ignore)]
fn test_n_xpm_0803_01_rejects_component_chain() {
    // This is a negative test file that should fail validation
    // It creates a component reference chain:
    //   root (3dmodel.model) -> gabe.model -> midway.model
    //
    // The violation is that gabe.model (non-root) has a component with p:path,
    // which is not allowed per the 3MF Production Extension specification.

    let config = ParserConfig::new()
        .with_extension(lib3mf::Extension::Production)
        .with_extension(lib3mf::Extension::Material);

    let file =
        File::open("test_suites/suite2_core_prod_matl/negative_test_cases/N_XPM_0803_01.3mf")
            .expect("Test file should exist");

    let result = Model::from_reader_with_config(file, config);

    // Verify the file fails validation
    assert!(result.is_err(), "N_XPM_0803_01 should fail validation");

    let error = result.unwrap_err();
    let error_msg = error.to_string();

    // Verify it's the right kind of error with a helpful message
    assert!(
        error_msg.contains("gabe.model") && error_msg.contains("p:path"),
        "Error should mention the non-root file (gabe.model) and p:path. Got: {}",
        error_msg
    );

    assert!(
        error_msg.contains("Non-root model") || error_msg.contains("same model file"),
        "Error should explain the spec requirement. Got: {}",
        error_msg
    );
}

#[test]
#[cfg_attr(not(feature = "ci"), ignore)]
fn test_n_spx_0803_01_also_rejects_chain() {
    // Suite1 version of the same test with slice extension
    let config = ParserConfig::new()
        .with_extension(lib3mf::Extension::Production)
        .with_extension(lib3mf::Extension::Slice);

    let file =
        File::open("test_suites/suite1_core_slice_prod/negative_test_cases/N_SPX_0803_01.3mf")
            .expect("Test file should exist");

    let result = Model::from_reader_with_config(file, config);

    // Should fail for the same reason
    assert!(result.is_err(), "N_SPX_0803_01 should fail validation");
}

#[test]
#[cfg_attr(not(feature = "ci"), ignore)]
fn test_n_xpx_0803_01_also_rejects_chain() {
    // Suite5 version of the same test
    let config = ParserConfig::new().with_extension(lib3mf::Extension::Production);

    let file = File::open("test_suites/suite5_core_prod/negative_test_cases/N_XPX_0803_01.3mf")
        .expect("Test file should exist");

    let result = Model::from_reader_with_config(file, config);

    // Should fail for the same reason
    assert!(result.is_err(), "N_XPX_0803_01 should fail validation");
}
