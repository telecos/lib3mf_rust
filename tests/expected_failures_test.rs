//! Tests for the expected failures infrastructure
//!
//! This test validates that the expected failures mechanism works correctly.

mod common;

use common::expected_failures::ExpectedFailuresManager;

#[test]
fn test_expected_failures_loading() {
    // Should be able to load the expected failures configuration
    let manager = ExpectedFailuresManager::load();
    
    // Verify the structure is valid (no panic during load)
    assert!(manager.is_expected_failure("suite9_core_ext", "P_XXX_2202_01.3mf", "positive"));
}

#[test]
fn test_expected_failure_for_p_xxx_2202_01() {
    let manager = ExpectedFailuresManager::load();
    
    // This specific file should be marked as an expected failure
    assert!(
        manager.is_expected_failure("suite9_core_ext", "P_XXX_2202_01.3mf", "positive"),
        "P_XXX_2202_01.3mf should be marked as an expected positive test failure"
    );
    
    // Should not be marked for negative tests
    assert!(
        !manager.is_expected_failure("suite9_core_ext", "P_XXX_2202_01.3mf", "negative"),
        "P_XXX_2202_01.3mf should not be marked as an expected negative test failure"
    );
}

#[test]
fn test_expected_failure_reason() {
    let manager = ExpectedFailuresManager::load();
    
    // Get the failure details
    let failure = manager.get_failure("suite9_core_ext", "P_XXX_2202_01.3mf");
    
    assert!(failure.is_some(), "Expected failure details should exist");
    
    let failure = failure.unwrap();
    assert_eq!(failure.file, "P_XXX_2202_01.3mf");
    assert_eq!(failure.suite, "suite9_core_ext");
    assert_eq!(failure.test_type, "positive");
    assert!(!failure.reason.is_empty(), "Reason should not be empty");
    assert!(
        failure.reason.contains("production"),
        "Reason should mention production extension"
    );
    assert!(
        failure.reason.contains("UUID"),
        "Reason should mention UUID attribute"
    );
}

#[test]
fn test_non_existent_expected_failure() {
    let manager = ExpectedFailuresManager::load();
    
    // A file that doesn't exist in expected failures
    assert!(
        !manager.is_expected_failure("suite1_core_slice_prod", "NonExistent.3mf", "positive"),
        "Non-existent file should not be marked as expected failure"
    );
}

#[test]
fn test_expected_failures_json_valid() {
    use std::fs;
    
    // Test that the JSON file is valid and can be parsed
    let content = fs::read_to_string("tests/expected_failures.json")
        .expect("Should be able to read expected_failures.json");
    
    let config: serde_json::Value = serde_json::from_str(&content)
        .expect("expected_failures.json should be valid JSON");
    
    // Verify structure
    assert!(config["expected_failures"].is_array(), "Should have expected_failures array");
    
    let failures = config["expected_failures"].as_array().unwrap();
    assert!(!failures.is_empty(), "Should have at least one expected failure");
    
    // Validate each entry has required fields
    for failure in failures {
        assert!(failure["file"].is_string(), "Each failure should have a file field");
        assert!(failure["suite"].is_string(), "Each failure should have a suite field");
        assert!(failure["test_type"].is_string(), "Each failure should have a test_type field");
        assert!(failure["reason"].is_string(), "Each failure should have a reason field");
        assert!(failure["date_added"].is_string(), "Each failure should have a date_added field");
    }
}

#[test]
fn test_expected_failures_cloneable() {
    let manager1 = ExpectedFailuresManager::load();
    let manager2 = manager1.clone();
    
    // Both should work identically
    let result1 = manager1.is_expected_failure("suite9_core_ext", "P_XXX_2202_01.3mf", "positive");
    let result2 = manager2.is_expected_failure("suite9_core_ext", "P_XXX_2202_01.3mf", "positive");
    
    assert_eq!(result1, result2, "Cloned manager should behave identically");
}
