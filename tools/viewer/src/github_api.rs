//! GitHub API integration for browsing 3MF Consortium test suites
//!
//! This module provides functionality to browse and download test files
//! from the official 3MF Consortium test suites repository on GitHub.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const REPO_API: &str = "https://api.github.com/repos/3MFConsortium/test_suites/contents";
const USER_AGENT: &str = "lib3mf_rust_viewer/0.1.0";

/// Represents a file or directory in the GitHub repository
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitHubContent {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub content_type: String, // "file" or "dir"
    pub download_url: Option<String>,
    pub size: Option<u64>,
    #[serde(skip)]
    #[allow(dead_code)] // Reserved for future tree caching feature
    pub children: Option<Vec<GitHubContent>>, // For cached directory trees
}

/// Category of test file
#[derive(Debug, Clone, PartialEq)]
pub enum TestCategory {
    Core,
    Materials,
    Production,
    BeamLattice,
    Slice,
    SecureContent,
    Boolean,
    Displacement,
    Other(String),
}

impl TestCategory {
    /// Parse category from path
    pub fn from_path(path: &str) -> Self {
        let lower = path.to_lowercase();
        if lower.contains("core") {
            TestCategory::Core
        } else if lower.contains("material") {
            TestCategory::Materials
        } else if lower.contains("production") {
            TestCategory::Production
        } else if lower.contains("beam") || lower.contains("lattice") {
            TestCategory::BeamLattice
        } else if lower.contains("slice") {
            TestCategory::Slice
        } else if lower.contains("secure") {
            TestCategory::SecureContent
        } else if lower.contains("boolean") {
            TestCategory::Boolean
        } else if lower.contains("displacement") {
            TestCategory::Displacement
        } else {
            TestCategory::Other(path.to_string())
        }
    }

    /// Get display name for the category
    pub fn display_name(&self) -> &str {
        match self {
            TestCategory::Core => "Core",
            TestCategory::Materials => "Materials",
            TestCategory::Production => "Production",
            TestCategory::BeamLattice => "Beam Lattice",
            TestCategory::Slice => "Slice",
            TestCategory::SecureContent => "Secure Content",
            TestCategory::Boolean => "Boolean Operations",
            TestCategory::Displacement => "Displacement",
            TestCategory::Other(name) => name,
        }
    }
}

/// Test type classification
#[derive(Debug, Clone, PartialEq)]
pub enum TestType {
    Positive, // Valid 3MF file
    Negative, // Invalid 3MF file (should fail parsing)
    Unknown,
}

impl TestType {
    /// Determine test type from filename/path
    pub fn from_filename(filename: &str) -> Self {
        let lower = filename.to_lowercase();
        // Check for negative indicators first (before checking "valid" which might be part of "invalid")
        if lower.contains("fail") || lower.contains("invalid") || lower.contains("negative") {
            TestType::Negative
        } else if lower.contains("pass") || lower.contains("valid") || lower.contains("positive") {
            TestType::Positive
        } else {
            TestType::Unknown
        }
    }

    /// Get display symbol
    pub fn symbol(&self) -> &str {
        match self {
            TestType::Positive => "✓",
            TestType::Negative => "✗",
            TestType::Unknown => "?",
        }
    }
}

/// GitHub API client for test suites
pub struct GitHubClient {
    client: reqwest::blocking::Client,
    cache_dir: PathBuf,
    directory_cache: HashMap<String, Vec<GitHubContent>>,
}

impl GitHubClient {
    /// Create a new GitHub client
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create cache directory
        let cache_dir = dirs::cache_dir()
            .ok_or("Unable to determine cache directory")?
            .join("lib3mf_viewer")
            .join("github_cache");

        fs::create_dir_all(&cache_dir)?;

        let client = reqwest::blocking::Client::builder()
            .user_agent(USER_AGENT)
            .build()?;

        Ok(Self {
            client,
            cache_dir,
            directory_cache: HashMap::new(),
        })
    }

    /// List contents of a directory in the test suites repository
    pub fn list_directory(
        &mut self,
        path: &str,
    ) -> Result<Vec<GitHubContent>, Box<dyn std::error::Error>> {
        // Check cache first
        if let Some(cached) = self.directory_cache.get(path) {
            return Ok(cached.clone());
        }

        // Build API URL
        let url = if path.is_empty() {
            REPO_API.to_string()
        } else {
            format!("{}/{}", REPO_API, path)
        };

        println!("Fetching from GitHub: {}", url);

        // Make API request
        let response = self.client.get(&url).send()?;

        // Check for rate limiting
        if response.status() == 403 {
            return Err("GitHub API rate limit exceeded. Please try again later.".into());
        }

        response.error_for_status_ref()?;

        let contents: Vec<GitHubContent> = response.json()?;

        // Cache the results
        self.directory_cache
            .insert(path.to_string(), contents.clone());

        Ok(contents)
    }

    /// Download a file from the test suites repository
    pub fn download_file(
        &self,
        item: &GitHubContent,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let download_url = item
            .download_url
            .as_ref()
            .ok_or("No download URL available")?;

        println!(
            "Downloading: {} ({} bytes)",
            item.name,
            item.size.unwrap_or(0)
        );

        // Download the file
        let response = self.client.get(download_url).send()?;
        response.error_for_status_ref()?;
        let bytes = response.bytes()?;

        // Save to temp directory with unique name based on path
        let file_name = format!("{}_{}", item.path.replace('/', "_"), item.name);
        let temp_path = self.cache_dir.join(&file_name);

        fs::write(&temp_path, bytes)?;

        println!("✓ Downloaded to: {}", temp_path.display());

        Ok(temp_path)
    }

    /// Get the cache directory path
    #[allow(dead_code)] // May be useful for debugging/inspection
    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    /// Clear the directory cache (useful for refreshing)
    pub fn clear_cache(&mut self) {
        self.directory_cache.clear();
    }

    /// Build a complete directory tree (recursive)
    pub fn build_tree(
        &mut self,
        path: &str,
    ) -> Result<Vec<GitHubContent>, Box<dyn std::error::Error>> {
        let mut contents = self.list_directory(path)?;

        // Sort: directories first, then files
        contents.sort_by(|a, b| match (&a.content_type[..], &b.content_type[..]) {
            ("dir", "file") => std::cmp::Ordering::Less,
            ("file", "dir") => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        Ok(contents)
    }
}

impl Default for GitHubClient {
    fn default() -> Self {
        Self::new().expect("Failed to create GitHub client")
    }
}

/// Filter test files by category
#[allow(dead_code)] // Reserved for future filtering feature
pub fn filter_by_category(items: &[GitHubContent], category: &TestCategory) -> Vec<GitHubContent> {
    items
        .iter()
        .filter(|item| {
            let item_category = TestCategory::from_path(&item.path);
            &item_category == category
        })
        .cloned()
        .collect()
}

/// Search for files matching a query string
#[allow(dead_code)] // Reserved for future search feature
pub fn search_files(items: &[GitHubContent], query: &str) -> Vec<GitHubContent> {
    let query_lower = query.to_lowercase();
    items
        .iter()
        .filter(|item| {
            item.name.to_lowercase().contains(&query_lower)
                || item.path.to_lowercase().contains(&query_lower)
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_parsing() {
        assert_eq!(TestCategory::from_path("core/box.3mf"), TestCategory::Core);
        assert_eq!(
            TestCategory::from_path("materials/color.3mf"),
            TestCategory::Materials
        );
        assert_eq!(
            TestCategory::from_path("production/uuid.3mf"),
            TestCategory::Production
        );
    }

    #[test]
    fn test_type_detection() {
        assert_eq!(TestType::from_filename("valid_box.3mf"), TestType::Positive);
        assert_eq!(
            TestType::from_filename("invalid_mesh.3mf"),
            TestType::Negative
        );
        assert_eq!(TestType::from_filename("box.3mf"), TestType::Unknown);
    }
}
