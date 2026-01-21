//! Test to verify that conformance tests use appropriate extension configurations
//!
//! This test ensures that each conformance suite is configured with the correct
//! extensions based on the suite's purpose.

mod common;

use lib3mf::Extension;

#[test]
fn test_suite1_config() {
    let config = common::get_suite_config("suite1_core_slice_prod");
    assert!(config.supports(&Extension::Core));
    assert!(config.supports(&Extension::Production));
    assert!(config.supports(&Extension::Slice));
    assert!(!config.supports(&Extension::Material));
    assert!(!config.supports(&Extension::BeamLattice));
}

#[test]
fn test_suite2_config() {
    let config = common::get_suite_config("suite2_core_prod_matl");
    assert!(config.supports(&Extension::Core));
    assert!(config.supports(&Extension::Production));
    assert!(config.supports(&Extension::Material));
    assert!(!config.supports(&Extension::Slice));
}

#[test]
fn test_suite3_config() {
    let config = common::get_suite_config("suite3_core");
    assert!(config.supports(&Extension::Core));
    assert!(!config.supports(&Extension::Material));
    assert!(!config.supports(&Extension::Production));
    assert!(!config.supports(&Extension::Slice));
}

#[test]
fn test_suite4_config() {
    let config = common::get_suite_config("suite4_core_slice");
    assert!(config.supports(&Extension::Core));
    assert!(config.supports(&Extension::Slice));
    assert!(!config.supports(&Extension::Production));
}

#[test]
fn test_suite5_config() {
    let config = common::get_suite_config("suite5_core_prod");
    assert!(config.supports(&Extension::Core));
    assert!(config.supports(&Extension::Production));
    assert!(!config.supports(&Extension::Slice));
    assert!(!config.supports(&Extension::Material));
}

#[test]
fn test_suite6_config() {
    let config = common::get_suite_config("suite6_core_matl");
    assert!(config.supports(&Extension::Core));
    assert!(config.supports(&Extension::Material));
    assert!(!config.supports(&Extension::Production));
}

#[test]
fn test_suite7_config() {
    let config = common::get_suite_config("suite7_beam");
    assert!(config.supports(&Extension::Core));
    assert!(config.supports(&Extension::BeamLattice));
    assert!(!config.supports(&Extension::Material));
    assert!(!config.supports(&Extension::SecureContent));
}

#[test]
fn test_suite8_config() {
    let config = common::get_suite_config("suite8_secure");
    assert!(config.supports(&Extension::Core));
    assert!(config.supports(&Extension::SecureContent));
    assert!(!config.supports(&Extension::Material));
    assert!(!config.supports(&Extension::BeamLattice));
}

#[test]
fn test_suite9_config() {
    let config = common::get_suite_config("suite9_core_ext");
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
    let config = common::get_suite_config("suite10_boolean");
    assert!(config.supports(&Extension::Core));
    assert!(config.supports(&Extension::BooleanOperations));
    assert!(!config.supports(&Extension::Material));
    assert!(!config.supports(&Extension::Displacement));
}

#[test]
fn test_suite11_config() {
    let config = common::get_suite_config("suite11_Displacement");
    assert!(config.supports(&Extension::Core));
    assert!(config.supports(&Extension::Displacement));
    assert!(!config.supports(&Extension::Material));
    assert!(!config.supports(&Extension::BooleanOperations));
}
