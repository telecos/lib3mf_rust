//! Expected failures configuration and handling
//!
//! This module provides functionality to document and handle expected test failures
//! in the conformance test suite. Some files in the official 3MF test suite are
//! known to be incorrect or have issues that cannot be resolved on our side.
//!
//! Expected failures are documented in tests/expected_failures.json with details
//! about why each file is expected to fail.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Configuration for an expected test failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedFailure {
    /// The test case ID (e.g., "0420_01", "2202_01")
    /// This identifies the test case across all suites
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub test_case_id: String,

    /// The filename of the test file (e.g., "P_XXX_2202_01.3mf")
    /// For backward compatibility with old format, but will be deprecated
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub file: String,

    /// List of suites this test case appears in (e.g., ["suite9_core_ext", "suite3_core"])
    /// If empty, falls back to single suite field for backward compatibility
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suites: Vec<String>,

    /// The suite this file belongs to (e.g., "suite9_core_ext")
    /// Deprecated in favor of suites array, kept for backward compatibility
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub suite: String,

    /// Whether this is a "positive" or "negative" test
    pub test_type: String,

    /// Detailed explanation of why this test is expected to fail
    pub reason: String,

    /// Optional URL to issue tracker or documentation
    #[serde(default)]
    pub issue_url: String,

    /// Date when this expected failure was added (YYYY-MM-DD)
    pub date_added: String,

    /// Optional expected error type (e.g., "OutsidePositiveOctant", "InvalidFormat")
    /// When specified, the test will validate that the actual error matches this type
    #[serde(default)]
    pub expected_error_type: Option<String>,
}

/// Container for all expected failures
#[derive(Debug, Serialize, Deserialize)]
pub struct ExpectedFailuresConfig {
    pub expected_failures: Vec<ExpectedFailure>,
}

/// Manager for expected failures
#[derive(Clone)]
pub struct ExpectedFailuresManager {
    /// Map from (suite, filename) to ExpectedFailure
    failures: HashMap<(String, String), ExpectedFailure>,
}

impl ExpectedFailuresManager {
    /// Load expected failures from the configuration file
    pub fn load() -> Self {
        let config_path = "tests/expected_failures.json";

        if !Path::new(config_path).exists() {
            // No expected failures file, return empty manager
            return Self {
                failures: HashMap::new(),
            };
        }

        let content = fs::read_to_string(config_path).unwrap_or_else(|e| {
            panic!(
                "Failed to read expected_failures.json at {}: {}",
                Path::new(config_path)
                    .canonicalize()
                    .unwrap_or_else(|_| Path::new(config_path).to_path_buf())
                    .display(),
                e
            )
        });

        let config: ExpectedFailuresConfig = serde_json::from_str(&content).unwrap_or_else(|e| {
            panic!(
                "Failed to parse expected_failures.json at {} (current dir: {:?}): {}. Content: {}",
                Path::new(config_path)
                    .canonicalize()
                    .unwrap_or_else(|_| Path::new(config_path).to_path_buf())
                    .display(),
                std::env::current_dir().ok(),
                e,
                if content.len() > 200 {
                    format!("{}... ({} bytes total)", &content[..200], content.len())
                } else {
                    content
                }
            )
        });

        let mut failures = HashMap::new();
        for failure in config.expected_failures {
            // New format: test_case_id + suites array
            if !failure.test_case_id.is_empty() && !failure.suites.is_empty() {
                // For each suite, we need to match by test case ID pattern
                // Store the failure under a special key that includes the test case ID
                for suite in &failure.suites {
                    // Create a key with the test_case_id pattern
                    // We'll match this during lookup
                    let key = (suite.clone(), format!("*{}*", failure.test_case_id));
                    failures.insert(key, failure.clone());
                }
            }
            // Old format: single suite and file  
            else if !failure.suite.is_empty() && !failure.file.is_empty() {
                let key = (failure.suite.clone(), failure.file.clone());
                failures.insert(key, failure);
            }
        }

        Self { failures }
    }

    /// Extract test case ID from filename
    /// E.g., "P_XXX_0420_01.3mf" -> "0420_01"
    fn extract_test_case_id(filename: &str) -> Option<String> {
        // Pattern: [P/N]_[PREFIX]_[test_case_id].3mf
        // We want to extract the test_case_id part
        let without_ext = filename.strip_suffix(".3mf")?;
        let parts: Vec<&str> = without_ext.split('_').collect();
        
        // Expected format: P/N _ PREFIX _ NNNN _ NN
        // e.g., P_XXX_0420_01 -> parts = ["P", "XXX", "0420", "01"]
        if parts.len() >= 4 {
            // Join last two parts for test case ID
            Some(format!("{}_{}", parts[parts.len() - 2], parts[parts.len() - 1]))
        } else {
            None
        }
    }

    /// Check if a test file is expected to fail
    pub fn is_expected_failure(&self, suite: &str, filename: &str, test_type: &str) -> bool {
        // First try exact match (old format)
        if let Some(failure) = self
            .failures
            .get(&(suite.to_string(), filename.to_string()))
        {
            return failure.test_type == test_type;
        }
        
        // Try pattern match by test case ID (new format)
        if let Some(test_case_id) = Self::extract_test_case_id(filename) {
            let pattern_key = (suite.to_string(), format!("*{}*", test_case_id));
            if let Some(failure) = self.failures.get(&pattern_key) {
                return failure.test_type == test_type;
            }
        }
        
        false
    }

    /// Get the expected failure details for a test file
    #[allow(dead_code)]
    pub fn get_failure(&self, suite: &str, filename: &str) -> Option<&ExpectedFailure> {
        // First try exact match (old format)
        if let Some(failure) = self
            .failures
            .get(&(suite.to_string(), filename.to_string()))
        {
            return Some(failure);
        }
        
        // Try pattern match by test case ID (new format)
        if let Some(test_case_id) = Self::extract_test_case_id(filename) {
            let pattern_key = (suite.to_string(), format!("*{}*", test_case_id));
            return self.failures.get(&pattern_key);
        }
        
        None
    }

    /// Get the reason for an expected failure
    #[allow(dead_code)]
    pub fn get_reason(&self, suite: &str, filename: &str) -> Option<String> {
        self.get_failure(suite, filename).map(|f| f.reason.clone())
    }

    /// Get the expected error type for an expected failure
    #[allow(dead_code)]
    pub fn get_expected_error_type(&self, suite: &str, filename: &str) -> Option<String> {
        self.get_failure(suite, filename)
            .and_then(|f| f.expected_error_type.clone())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_load_expected_failures() {
        use super::*;
        let manager = ExpectedFailuresManager::load();

        // Should be able to load without panicking
        // The actual content depends on the configuration file
        let _ = manager.failures.len();
    }

    #[test]
    fn test_expected_failure_check() {
        use super::*;
        let manager = ExpectedFailuresManager::load();

        // Check a known expected failure if it exists
        // This test will pass even if there are no expected failures
        let is_expected =
            manager.is_expected_failure("suite9_core_ext", "P_XXX_2202_01.3mf", "positive");

        // We don't assert the result since it depends on the config
        // Just verify the method works
        let _ = is_expected;
    }
}
