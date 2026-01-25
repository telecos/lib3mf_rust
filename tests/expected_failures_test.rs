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
    assert_eq!(failure.test_case_id, "2202_01");
    assert!(failure.suites.contains(&"suite9_core_ext".to_string()));
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

    let config: serde_json::Value =
        serde_json::from_str(&content).expect("expected_failures.json should be valid JSON");

    // Verify structure
    assert!(
        config["expected_failures"].is_array(),
        "Should have expected_failures array"
    );

    let failures = config["expected_failures"].as_array().unwrap();
    assert!(
        !failures.is_empty(),
        "Should have at least one expected failure"
    );

    // Validate each entry has required fields
    for failure in failures {
        // New format requires test_case_id and suites
        let has_test_case_id = failure
            .get("test_case_id")
            .and_then(|v| v.as_str())
            .map(|s| !s.is_empty())
            .unwrap_or(false);

        if has_test_case_id {
            assert!(
                failure["suites"].is_array(),
                "Each failure with test_case_id should have a suites array"
            );
            let suites_array = failure["suites"]
                .as_array()
                .expect("suites should be an array if is_array() returned true");
            assert!(!suites_array.is_empty(), "suites array should not be empty");
        } else {
            // Old format uses file and suite fields
            assert!(
                failure["file"].is_string(),
                "Each old-format failure should have a file field"
            );
            assert!(
                failure["suite"].is_string(),
                "Each old-format failure should have a suite field"
            );
        }

        assert!(
            failure["test_type"].is_string(),
            "Each failure should have a test_type field"
        );
        assert!(
            failure["reason"].is_string(),
            "Each failure should have a reason field"
        );
        assert!(
            failure["date_added"].is_string(),
            "Each failure should have a date_added field"
        );
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

#[test]
fn test_expected_failure_for_p_spx_0313_01() {
    let manager = ExpectedFailuresManager::load();

    // This specific file should be marked as an expected failure
    assert!(
        manager.is_expected_failure("suite1_core_slice_prod", "P_SPX_0313_01.3mf", "positive"),
        "P_SPX_0313_01.3mf should be marked as an expected positive test failure"
    );

    // Verify the reason is documented
    let reason = manager.get_reason("suite1_core_slice_prod", "P_SPX_0313_01.3mf");
    assert!(
        reason.is_some(),
        "P_SPX_0313_01.3mf should have a documented reason"
    );

    let reason_text = reason.unwrap();
    assert!(
        reason_text.contains("content type"),
        "Reason should mention content type issue"
    );
    assert!(
        reason_text.contains("PNG"),
        "Reason should mention PNG extension"
    );
}

#[test]
fn test_expected_failure_for_p_sxx_0313_01() {
    let manager = ExpectedFailuresManager::load();

    // This specific file should be marked as an expected failure in suite4
    assert!(
        manager.is_expected_failure("suite4_core_slice", "P_SXX_0313_01.3mf", "positive"),
        "P_SXX_0313_01.3mf should be marked as an expected positive test failure"
    );

    // Verify the reason is documented
    let reason = manager.get_reason("suite4_core_slice", "P_SXX_0313_01.3mf");
    assert!(
        reason.is_some(),
        "P_SXX_0313_01.3mf should have a documented reason"
    );

    let reason_text = reason.unwrap();
    assert!(
        reason_text.contains("content type"),
        "Reason should mention content type issue"
    );
    assert!(
        reason_text.contains("PNG"),
        "Reason should mention PNG extension"
    );
    assert!(
        reason_text.contains("E2004"),
        "Reason should mention error code E2004"
    );
}

#[test]
fn test_multi_suite_test_case() {
    let manager = ExpectedFailuresManager::load();

    // Test case 0421_01 should be an expected failure in suites 2, 3, 5, and 6
    assert!(
        manager.is_expected_failure("suite2_core_prod_matl", "N_XPM_0421_01.3mf", "negative"),
        "N_XPM_0421_01.3mf should be marked as expected failure in suite2"
    );

    assert!(
        manager.is_expected_failure("suite3_core", "N_XXX_0421_01.3mf", "negative"),
        "N_XXX_0421_01.3mf should be marked as expected failure in suite3"
    );

    assert!(
        manager.is_expected_failure("suite5_core_prod", "N_XPX_0421_01.3mf", "negative"),
        "N_XPX_0421_01.3mf should be marked as expected failure in suite5"
    );

    assert!(
        manager.is_expected_failure("suite6_core_matl", "N_XXM_0421_01.3mf", "negative"),
        "N_XXM_0421_01.3mf should be marked as expected failure in suite6"
    );

    // All should have the same reason
    let reason_suite2 = manager.get_reason("suite2_core_prod_matl", "N_XPM_0421_01.3mf");
    let reason_suite3 = manager.get_reason("suite3_core", "N_XXX_0421_01.3mf");
    let reason_suite5 = manager.get_reason("suite5_core_prod", "N_XPX_0421_01.3mf");
    let reason_suite6 = manager.get_reason("suite6_core_matl", "N_XXM_0421_01.3mf");

    assert!(reason_suite2.is_some(), "suite2 should have a reason");
    assert!(reason_suite3.is_some(), "suite3 should have a reason");
    assert!(reason_suite5.is_some(), "suite5 should have a reason");
    assert!(reason_suite6.is_some(), "suite6 should have a reason");
    
    let reason2 = reason_suite2.unwrap();
    let reason3 = reason_suite3.unwrap();
    let reason5 = reason_suite5.unwrap();
    let reason6 = reason_suite6.unwrap();
    
    assert_eq!(reason2, reason3, "suite2 and suite3 should have the same reason");
    assert_eq!(reason2, reason5, "suite2 and suite5 should have the same reason");
    assert_eq!(reason2, reason6, "suite2 and suite6 should have the same reason");
}

#[test]
fn test_multi_suite_test_case_0326_03() {
    let manager = ExpectedFailuresManager::load();

    // Test case 0326_03 appears in 4 different suites
    let test_cases = vec![
        ("suite2_core_prod_matl", "P_XPM_0326_03.3mf"),
        ("suite3_core", "P_XXX_0326_03.3mf"),
        ("suite5_core_prod", "P_XPX_0326_03.3mf"),
        ("suite6_core_matl", "P_XXM_0326_03.3mf"),
    ];

    for (suite, filename) in test_cases {
        assert!(
            manager.is_expected_failure(suite, filename, "positive"),
            "{} should be marked as expected failure in {}",
            filename,
            suite
        );

        let reason = manager.get_reason(suite, filename);
        assert!(
            reason.is_some(),
            "{} should have a reason in {}",
            filename,
            suite
        );

        let reason_text = reason.unwrap();
        assert!(
            reason_text.contains("zero determinant"),
            "Reason should mention zero determinant for {}",
            filename
        );
    }
}

#[test]
fn test_test_case_id_extraction() {
    // Test the internal test case ID extraction logic indirectly
    let manager = ExpectedFailuresManager::load();

    // These should all match because they have the same test case ID
    assert!(
        manager.is_expected_failure("suite2_core_prod_matl", "N_XPM_0421_01.3mf", "negative"),
        "Should match N_XPM_0421_01.3mf"
    );
    assert!(
        manager.is_expected_failure("suite3_core", "N_XXX_0421_01.3mf", "negative"),
        "Should match N_XXX_0421_01.3mf"
    );
    assert!(
        manager.is_expected_failure("suite5_core_prod", "N_XPX_0421_01.3mf", "negative"),
        "Should match N_XPX_0421_01.3mf"
    );
    assert!(
        manager.is_expected_failure("suite6_core_matl", "N_XXM_0421_01.3mf", "negative"),
        "Should match N_XXM_0421_01.3mf"
    );

    // These should not match (wrong suite)
    assert!(
        !manager.is_expected_failure("suite1_core_slice_prod", "N_XPM_0421_01.3mf", "negative"),
        "Should not match in wrong suite"
    );

    // Test with different prefix lengths shouldn't break
    // (even though these files don't exist in our config)
    assert!(
        !manager.is_expected_failure("suite3_core", "N_EXTRA_LONG_0999_99.3mf", "negative"),
        "Should handle longer prefixes gracefully"
    );
}
