//! Individual conformance tests - each .3mf file is a separate test
//!
//! This test binary uses libtest-mimic to dynamically generate individual
//! test cases for each .3mf file in the test suites. Each file shows up
//! as a separate test in the test output.
//!
//! Run with: cargo test --test conformance_individual
//! Run specific suite: cargo test --test conformance_individual suite3
//!
//! Each suite runs with the appropriate extension support enabled to ensure
//! proper validation of extension-specific features.

#[path = "../common/mod.rs"]
mod common;

use lib3mf::{Model, ParserConfig};
use libtest_mimic::{Arguments, Failed, Trial};
use std::fs::File;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Get all .3mf files in a directory recursively, sorted by name
fn get_test_files(suite: &str, test_dir: &str) -> Vec<PathBuf> {
    let path = format!("test_suites/{}/{}", suite, test_dir);
    if !std::path::Path::new(&path).exists() {
        return Vec::new();
    }

    let mut files: Vec<PathBuf> = WalkDir::new(&path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("3mf"))
        .map(|e| e.path().to_path_buf())
        .collect();

    files.sort();
    files
}

/// Test that a positive test case parses successfully
fn test_positive_file(
    path: PathBuf,
    config: ParserConfig,
    suite: &str,
    expected_failures: &common::ExpectedFailuresManager,
) -> Result<(), Failed> {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let file = File::open(&path).map_err(|e| format!("Failed to open file: {}", e))?;

    match Model::from_reader_with_config(file, config) {
        Ok(_) => {
            if expected_failures.is_expected_failure(suite, filename, "positive") {
                Err(format!(
                    "File was expected to fail but succeeded. Reason: {}",
                    expected_failures
                        .get_reason(suite, filename)
                        .unwrap_or_else(|| "No reason provided".to_string())
                )
                .into())
            } else {
                Ok(())
            }
        }
        Err(e) => {
            if expected_failures.is_expected_failure(suite, filename, "positive") {
                // This is an expected failure, so we report it as success
                Ok(())
            } else {
                Err(format!("Failed to parse: {}", e).into())
            }
        }
    }
}

/// Test that a negative test case fails to parse
fn test_negative_file(
    path: PathBuf,
    config: ParserConfig,
    suite: &str,
    expected_failures: &common::ExpectedFailuresManager,
) -> Result<(), Failed> {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let file = File::open(&path).map_err(|e| format!("Failed to open file: {}", e))?;

    match Model::from_reader_with_config(file, config) {
        Ok(_) => {
            if expected_failures.is_expected_failure(suite, filename, "negative") {
                // This is an expected failure (expected to succeed when it should fail)
                Ok(())
            } else {
                Err("Expected parsing to fail, but it succeeded".into())
            }
        }
        Err(_) => {
            if expected_failures.is_expected_failure(suite, filename, "negative") {
                Err(format!(
                    "File was expected to succeed but failed. Reason: {}",
                    expected_failures
                        .get_reason(suite, filename)
                        .unwrap_or_else(|| "No reason provided".to_string())
                )
                .into())
            } else {
                Ok(()) // Expected to fail
            }
        }
    }
}

/// Create test trials for a suite
fn create_suite_tests(
    suite_name: &str,
    suite_dir: &str,
    positive_dir: &str,
    negative_dir: &str,
) -> Vec<Trial> {
    let mut trials = Vec::new();
    let config = common::get_suite_config(suite_dir);
    let expected_failures = common::ExpectedFailuresManager::load();

    // Positive tests
    let positive_files = get_test_files(suite_dir, positive_dir);
    for path in positive_files {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let test_name = format!("{}::positive::{}", suite_name, file_name);
        let config_clone = config.clone();
        let suite_dir_clone = suite_dir.to_string();
        let expected_failures_clone = expected_failures.clone();

        trials.push(Trial::test(test_name, move || {
            test_positive_file(
                path.clone(),
                config_clone,
                &suite_dir_clone,
                &expected_failures_clone,
            )
        }));
    }

    // Negative tests
    let negative_files = get_test_files(suite_dir, negative_dir);
    for path in negative_files {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let test_name = format!("{}::negative::{}", suite_name, file_name);
        let config_clone = config.clone();
        let suite_dir_clone = suite_dir.to_string();
        let expected_failures_clone = expected_failures.clone();

        trials.push(Trial::test(test_name, move || {
            test_negative_file(
                path.clone(),
                config_clone,
                &suite_dir_clone,
                &expected_failures_clone,
            )
        }));
    }

    trials
}

fn main() {
    let args = Arguments::from_args();

    let mut tests = Vec::new();

    // Suite 1: Core + Production + Slice
    tests.extend(create_suite_tests(
        "suite1",
        "suite1_core_slice_prod",
        "positive_test_cases",
        "negative_test_cases",
    ));

    // Suite 2: Core + Production + Materials
    tests.extend(create_suite_tests(
        "suite2",
        "suite2_core_prod_matl",
        "positive_test_cases",
        "negative_test_cases",
    ));

    // Suite 3: Core (Basic)
    tests.extend(create_suite_tests(
        "suite3",
        "suite3_core",
        "positive_test_cases",
        "negative_test_cases",
    ));

    // Suite 4: Core + Slice
    tests.extend(create_suite_tests(
        "suite4",
        "suite4_core_slice",
        "positive_test_cases",
        "negative_test_cases",
    ));

    // Suite 5: Core + Production
    tests.extend(create_suite_tests(
        "suite5",
        "suite5_core_prod",
        "positive_test_cases",
        "negative_test_cases",
    ));

    // Suite 6: Core + Materials
    tests.extend(create_suite_tests(
        "suite6",
        "suite6_core_matl",
        "positive_test_cases",
        "negative_test_cases",
    ));

    // Suite 7: Beam Lattice
    tests.extend(create_suite_tests(
        "suite7",
        "suite7_beam",
        "positive_test_cases",
        "negative_test_cases",
    ));

    // Suite 8: Secure Content
    tests.extend(create_suite_tests(
        "suite8",
        "suite8_secure",
        "positive_test_cases",
        "negative_test_cases",
    ));

    // Suite 9: Core Extensions
    tests.extend(create_suite_tests(
        "suite9",
        "suite9_core_ext",
        "Positive Tests",
        "Negative Tests",
    ));

    // Suite 10: Boolean Operations
    tests.extend(create_suite_tests(
        "suite10",
        "suite10_boolean",
        "Positive Tests",
        "Negative Tests",
    ));

    // Suite 11: Displacement
    tests.extend(create_suite_tests(
        "suite11",
        "suite11_Displacement",
        "Positive Tests",
        "Negative Tests",
    ));

    // Run all tests
    libtest_mimic::run(&args, tests).exit();
}
