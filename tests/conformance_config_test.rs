//! Test to verify that conformance tests use appropriate extension configurations
//!
//! This test ensures that each conformance suite is configured with the correct
//! extensions based on the suite's purpose.

use lib3mf::{Extension, ParserConfig};

/// Get parser configuration for a specific test suite (duplicated from conformance tests)
fn get_suite_config(suite_name: &str) -> ParserConfig {
    match suite_name {
        // Suite 1: Core + Production + Slice
        "suite1_core_slice_prod" => ParserConfig::new()
            .with_extension(Extension::Production)
            .with_extension(Extension::Slice),

        // Suite 2: Core + Production + Materials
        "suite2_core_prod_matl" => ParserConfig::new()
            .with_extension(Extension::Production)
            .with_extension(Extension::Material),

        // Suite 3: Core only
        "suite3_core" => ParserConfig::new(),

        // Suite 4: Core + Slice
        "suite4_core_slice" => ParserConfig::new().with_extension(Extension::Slice),

        // Suite 5: Core + Production
        "suite5_core_prod" => ParserConfig::new().with_extension(Extension::Production),

        // Suite 6: Core + Materials
        "suite6_core_matl" => ParserConfig::new().with_extension(Extension::Material),

        // Suite 7: Beam Lattice
        "suite7_beam" => ParserConfig::new().with_extension(Extension::BeamLattice),

        // Suite 8: Secure Content
        "suite8_secure" => ParserConfig::new().with_extension(Extension::SecureContent),

        // Suite 9: Core Extensions - support all for compatibility
        "suite9_core_ext" => ParserConfig::with_all_extensions(),

        // Suite 10: Boolean Operations
        "suite10_boolean" => ParserConfig::new().with_extension(Extension::BooleanOperations),

        // Suite 11: Displacement
        "suite11_Displacement" => ParserConfig::new().with_extension(Extension::Displacement),

        // Default: support all extensions for unknown suites
        _ => ParserConfig::with_all_extensions(),
    }
}

#[test]
fn test_suite1_config() {
    let config = get_suite_config("suite1_core_slice_prod");
    assert!(config.supports(&Extension::Core));
    assert!(config.supports(&Extension::Production));
    assert!(config.supports(&Extension::Slice));
    assert!(!config.supports(&Extension::Material));
    assert!(!config.supports(&Extension::BeamLattice));
}

#[test]
fn test_suite2_config() {
    let config = get_suite_config("suite2_core_prod_matl");
    assert!(config.supports(&Extension::Core));
    assert!(config.supports(&Extension::Production));
    assert!(config.supports(&Extension::Material));
    assert!(!config.supports(&Extension::Slice));
}

#[test]
fn test_suite3_config() {
    let config = get_suite_config("suite3_core");
    assert!(config.supports(&Extension::Core));
    assert!(!config.supports(&Extension::Material));
    assert!(!config.supports(&Extension::Production));
    assert!(!config.supports(&Extension::Slice));
}

#[test]
fn test_suite4_config() {
    let config = get_suite_config("suite4_core_slice");
    assert!(config.supports(&Extension::Core));
    assert!(config.supports(&Extension::Slice));
    assert!(!config.supports(&Extension::Production));
}

#[test]
fn test_suite5_config() {
    let config = get_suite_config("suite5_core_prod");
    assert!(config.supports(&Extension::Core));
    assert!(config.supports(&Extension::Production));
    assert!(!config.supports(&Extension::Slice));
    assert!(!config.supports(&Extension::Material));
}

#[test]
fn test_suite6_config() {
    let config = get_suite_config("suite6_core_matl");
    assert!(config.supports(&Extension::Core));
    assert!(config.supports(&Extension::Material));
    assert!(!config.supports(&Extension::Production));
}

#[test]
fn test_suite7_config() {
    let config = get_suite_config("suite7_beam");
    assert!(config.supports(&Extension::Core));
    assert!(config.supports(&Extension::BeamLattice));
    assert!(!config.supports(&Extension::Material));
    assert!(!config.supports(&Extension::SecureContent));
}

#[test]
fn test_suite8_config() {
    let config = get_suite_config("suite8_secure");
    assert!(config.supports(&Extension::Core));
    assert!(config.supports(&Extension::SecureContent));
    assert!(!config.supports(&Extension::Material));
    assert!(!config.supports(&Extension::BeamLattice));
}

#[test]
fn test_suite9_config() {
    let config = get_suite_config("suite9_core_ext");
    // Suite 9 should support all extensions
    assert!(config.supports(&Extension::Core));
    assert!(config.supports(&Extension::Material));
    assert!(config.supports(&Extension::Production));
    assert!(config.supports(&Extension::Slice));
    assert!(config.supports(&Extension::BeamLattice));
    assert!(config.supports(&Extension::SecureContent));
    assert!(config.supports(&Extension::BooleanOperations));
    assert!(config.supports(&Extension::Displacement));
}

#[test]
fn test_suite10_config() {
    let config = get_suite_config("suite10_boolean");
    assert!(config.supports(&Extension::Core));
    assert!(config.supports(&Extension::BooleanOperations));
    assert!(!config.supports(&Extension::Material));
    assert!(!config.supports(&Extension::Displacement));
}

#[test]
fn test_suite11_config() {
    let config = get_suite_config("suite11_Displacement");
    assert!(config.supports(&Extension::Core));
    assert!(config.supports(&Extension::Displacement));
    assert!(!config.supports(&Extension::Material));
    assert!(!config.supports(&Extension::BooleanOperations));
}
