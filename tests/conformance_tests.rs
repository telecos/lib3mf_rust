//! Conformance tests using the official 3MF Consortium test suites
//!
//! This test file validates the parser against all official test cases from:
//! https://github.com/3MFConsortium/test_suites
//!
//! The test suites cover:
//! - Suite 1: Core + Production + Slice
//! - Suite 2: Core + Production + Materials
//! - Suite 3: Core (basic)
//! - Suite 4: Core + Slice
//! - Suite 5: Core + Production
//! - Suite 6: Core + Materials
//! - Suite 7: Beam Lattice
//! - Suite 8: Secure Content
//! - Suite 9: Core Extensions
//! - Suite 10: Boolean Operations
//! - Suite 11: Displacement
//!
//! Positive tests should parse successfully.
//! Negative tests should fail to parse (return an error).
//!
//! Each suite runs with the appropriate extension support enabled to ensure
//! proper validation of extension-specific features.

use lib3mf::{Extension, Model, ParserConfig};
use std::fs::File;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Get parser configuration for a specific test suite
///
/// Each suite tests specific extensions, so we configure the parser
/// to support only the extensions relevant to that suite. This ensures
/// proper validation of extension support.
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

/// Get all .3mf files in a directory recursively
fn get_3mf_files<P: AsRef<Path>>(dir: P) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("3mf"))
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Test a positive test case - should parse successfully
fn test_positive_case(path: &Path, config: &ParserConfig) -> Result<(), String> {
    let file =
        File::open(path).map_err(|e| format!("Failed to open file {}: {}", path.display(), e))?;

    Model::from_reader_with_config(file, config.clone())
        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;

    Ok(())
}

/// Test a negative test case - should fail to parse
fn test_negative_case(path: &Path, config: &ParserConfig) -> Result<(), String> {
    let file =
        File::open(path).map_err(|e| format!("Failed to open file {}: {}", path.display(), e))?;

    match Model::from_reader_with_config(file, config.clone()) {
        Ok(_) => Err(format!(
            "Expected parsing to fail for {}, but it succeeded",
            path.display()
        )),
        Err(_) => Ok(()), // Expected to fail
    }
}

// Helper macro to generate test functions for each suite
macro_rules! suite_tests {
    ($suite_name:ident, $suite_dir:expr, $pos_dir:expr, $neg_dir:expr) => {
        mod $suite_name {
            use super::*;

            #[test]
            fn positive_tests() {
                let suite_path = format!("test_suites/{}/{}", $suite_dir, $pos_dir);
                if !Path::new(&suite_path).exists() {
                    eprintln!("Warning: {} not found, skipping", suite_path);
                    return;
                }

                let test_files = get_3mf_files(&suite_path);
                if test_files.is_empty() {
                    eprintln!("Warning: No .3mf files found in {}", suite_path);
                    return;
                }

                let config = get_suite_config($suite_dir);
                let mut passed = 0;
                let mut failed = Vec::new();

                for file in &test_files {
                    match test_positive_case(file, &config) {
                        Ok(_) => passed += 1,
                        Err(e) => failed.push(e),
                    }
                }

                println!(
                    "{}: {}/{} positive tests passed",
                    stringify!($suite_name),
                    passed,
                    test_files.len()
                );

                if !failed.is_empty() {
                    println!("\nFailed tests:");
                    for (i, err) in failed.iter().enumerate() {
                        println!("  {}. {}", i + 1, err);
                    }
                    panic!("\n{} positive test(s) failed", failed.len());
                }
            }

            #[test]
            fn negative_tests() {
                let suite_path = format!("test_suites/{}/{}", $suite_dir, $neg_dir);
                if !Path::new(&suite_path).exists() {
                    eprintln!("Warning: {} not found, skipping", suite_path);
                    return;
                }

                let test_files = get_3mf_files(&suite_path);
                if test_files.is_empty() {
                    eprintln!("Warning: No .3mf files found in {}", suite_path);
                    return;
                }

                let config = get_suite_config($suite_dir);
                let mut passed = 0;
                let mut failed = Vec::new();

                for file in &test_files {
                    match test_negative_case(file, &config) {
                        Ok(_) => passed += 1,
                        Err(e) => failed.push(e),
                    }
                }

                println!(
                    "{}: {}/{} negative tests passed",
                    stringify!($suite_name),
                    passed,
                    test_files.len()
                );

                if !failed.is_empty() {
                    println!("\nFailed tests:");
                    for (i, err) in failed.iter().enumerate() {
                        println!("  {}. {}", i + 1, err);
                    }
                    panic!("\n{} negative test(s) failed", failed.len());
                }
            }
        }
    };
}

// Generate tests for all suites
suite_tests!(
    suite1_core_slice_prod,
    "suite1_core_slice_prod",
    "positive_test_cases",
    "negative_test_cases"
);
suite_tests!(
    suite2_core_prod_matl,
    "suite2_core_prod_matl",
    "positive_test_cases",
    "negative_test_cases"
);
suite_tests!(
    suite3_core,
    "suite3_core",
    "positive_test_cases",
    "negative_test_cases"
);
suite_tests!(
    suite4_core_slice,
    "suite4_core_slice",
    "positive_test_cases",
    "negative_test_cases"
);
suite_tests!(
    suite5_core_prod,
    "suite5_core_prod",
    "positive_test_cases",
    "negative_test_cases"
);
suite_tests!(
    suite6_core_matl,
    "suite6_core_matl",
    "positive_test_cases",
    "negative_test_cases"
);
suite_tests!(
    suite7_beam,
    "suite7_beam",
    "positive_test_cases",
    "negative_test_cases"
);
suite_tests!(
    suite8_secure,
    "suite8_secure",
    "positive_test_cases",
    "negative_test_cases"
);
suite_tests!(
    suite9_core_ext,
    "suite9_core_ext",
    "Positive Tests",
    "Negative Tests"
);
suite_tests!(
    suite10_boolean,
    "suite10_boolean",
    "Positive Tests",
    "Negative Tests"
);
suite_tests!(
    suite11_displacement,
    "suite11_Displacement",
    "Positive Tests",
    "Negative Tests"
);

/// Summary test that reports overall conformance statistics
#[test]
#[ignore] // Run with: cargo test --test conformance_tests summary -- --ignored --nocapture
fn summary() {
    println!("\n=== 3MF Conformance Test Suite Summary ===\n");

    let suites = vec![
        (
            "suite1_core_slice_prod",
            "positive_test_cases",
            "negative_test_cases",
        ),
        (
            "suite2_core_prod_matl",
            "positive_test_cases",
            "negative_test_cases",
        ),
        ("suite3_core", "positive_test_cases", "negative_test_cases"),
        (
            "suite4_core_slice",
            "positive_test_cases",
            "negative_test_cases",
        ),
        (
            "suite5_core_prod",
            "positive_test_cases",
            "negative_test_cases",
        ),
        (
            "suite6_core_matl",
            "positive_test_cases",
            "negative_test_cases",
        ),
        ("suite7_beam", "positive_test_cases", "negative_test_cases"),
        (
            "suite8_secure",
            "positive_test_cases",
            "negative_test_cases",
        ),
        ("suite9_core_ext", "Positive Tests", "Negative Tests"),
        ("suite10_boolean", "Positive Tests", "Negative Tests"),
        ("suite11_Displacement", "Positive Tests", "Negative Tests"),
    ];

    let mut total_positive = 0;
    let mut total_negative = 0;
    let mut total_positive_passed = 0;
    let mut total_negative_passed = 0;

    for (suite, pos_dir, neg_dir) in suites {
        let pos_path = format!("test_suites/{}/{}", suite, pos_dir);
        let neg_path = format!("test_suites/{}/{}", suite, neg_dir);

        let pos_files = if Path::new(&pos_path).exists() {
            get_3mf_files(&pos_path)
        } else {
            Vec::new()
        };

        let neg_files = if Path::new(&neg_path).exists() {
            get_3mf_files(&neg_path)
        } else {
            Vec::new()
        };

        let config = get_suite_config(suite);
        let mut pos_passed = 0;
        let mut neg_passed = 0;

        for file in &pos_files {
            if test_positive_case(file, &config).is_ok() {
                pos_passed += 1;
            }
        }

        for file in &neg_files {
            if test_negative_case(file, &config).is_ok() {
                neg_passed += 1;
            }
        }

        total_positive += pos_files.len();
        total_negative += neg_files.len();
        total_positive_passed += pos_passed;
        total_negative_passed += neg_passed;

        println!(
            "{:25} Positive: {:3}/{:3}  Negative: {:3}/{:3}",
            suite,
            pos_passed,
            pos_files.len(),
            neg_passed,
            neg_files.len()
        );
    }

    println!("\n{:=<60}", "");
    println!(
        "{:25} Positive: {:3}/{:3}  Negative: {:3}/{:3}",
        "TOTAL", total_positive_passed, total_positive, total_negative_passed, total_negative
    );
    println!("{:=<60}\n", "");

    let total_tests = total_positive + total_negative;
    let total_passed = total_positive_passed + total_negative_passed;
    let pass_rate = if total_tests > 0 {
        (total_passed as f64 / total_tests as f64) * 100.0
    } else {
        0.0
    };

    println!(
        "Overall conformance: {:.1}% ({}/{})",
        pass_rate, total_passed, total_tests
    );
}
