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
    /// The filename of the test file (e.g., "P_XXX_2202_01.3mf")
    pub file: String,

    /// The suite this file belongs to (e.g., "suite9_core_ext")
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
            let key = (failure.suite.clone(), failure.file.clone());
            failures.insert(key, failure);
        }

        Self { failures }
    }

    /// Check if a test file is expected to fail
    pub fn is_expected_failure(&self, suite: &str, filename: &str, test_type: &str) -> bool {
        if let Some(failure) = self
            .failures
            .get(&(suite.to_string(), filename.to_string()))
        {
            failure.test_type == test_type
        } else {
            false
        }
    }

    /// Get the expected failure details for a test file
    #[allow(dead_code)]
    pub fn get_failure(&self, suite: &str, filename: &str) -> Option<&ExpectedFailure> {
        self.failures
            .get(&(suite.to_string(), filename.to_string()))
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
