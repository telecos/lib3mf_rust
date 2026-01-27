//! Test Suite Browser UI
//!
//! Interactive console-based browser for navigating and selecting
//! test files from the 3MF Consortium GitHub repository.

#![forbid(unsafe_code)]

use crate::github_api::{GitHubClient, GitHubContent, TestCategory, TestType};
use std::io::{self, Write};
use std::path::PathBuf;

/// Browser state
pub struct BrowserState {
    current_path: String,
    current_items: Vec<GitHubContent>,
    breadcrumb: Vec<String>,
}

impl BrowserState {
    fn new() -> Self {
        Self {
            current_path: String::new(),
            current_items: Vec::new(),
            breadcrumb: vec!["root".to_string()],
        }
    }

    fn navigate_to(&mut self, path: String, name: String) {
        self.current_path = path;
        self.breadcrumb.push(name);
    }

    fn navigate_back(&mut self) {
        if self.breadcrumb.len() > 1 {
            self.breadcrumb.pop();
            // Reconstruct path from breadcrumb
            if self.breadcrumb.len() == 1 {
                self.current_path = String::new();
            } else {
                // This is a simplified version - in a real implementation,
                // you'd want to track the full path properly
                self.current_path = self.breadcrumb[1..].join("/");
            }
        }
    }

    fn current_location(&self) -> String {
        self.breadcrumb.join(" > ")
    }
}

/// Launch the test suite browser
pub fn launch_browser() -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!("  3MF Test Suite Browser");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("  Browsing: https://github.com/3MFConsortium/test_suites");
    println!();

    let mut client = GitHubClient::new()?;
    let mut state = BrowserState::new();

    loop {
        // Fetch current directory contents
        state.current_items = client.build_tree(&state.current_path)?;

        // Display current location
        print_location(&state);

        // Display items
        print_items(&state.current_items);

        // Display menu
        print_menu();

        // Get user input
        let choice = get_user_input()?;

        match choice.as_str() {
            "q" | "quit" | "exit" => {
                println!("Exiting browser...");
                return Ok(None);
            }
            "b" | "back" => {
                if state.breadcrumb.len() > 1 {
                    state.navigate_back();
                } else {
                    println!("Already at root directory.");
                }
            }
            "r" | "refresh" => {
                client.clear_cache();
                println!("Cache cleared. Refreshing...");
            }
            "h" | "help" => {
                print_help();
            }
            _ => {
                // Try to parse as a number for item selection
                if let Ok(index) = choice.parse::<usize>() {
                    if index > 0 && index <= state.current_items.len() {
                        let selected = &state.current_items[index - 1];

                        if selected.content_type == "dir" {
                            // Navigate into directory
                            state.navigate_to(selected.path.clone(), selected.name.clone());
                        } else if selected.content_type == "file" && selected.name.ends_with(".3mf")
                        {
                            // Download and return file path
                            println!();
                            println!("Downloading file...");
                            match client.download_file(selected) {
                                Ok(path) => {
                                    println!("✓ File ready: {}", path.display());
                                    return Ok(Some(path));
                                }
                                Err(e) => {
                                    eprintln!("✗ Error downloading file: {}", e);
                                    println!("Press Enter to continue...");
                                    let _ = get_user_input();
                                }
                            }
                        } else {
                            println!("Cannot open this file type.");
                        }
                    } else {
                        println!("Invalid selection. Please try again.");
                    }
                } else {
                    println!("Unknown command: {}. Type 'h' for help.", choice);
                }
            }
        }

        println!();
    }
}

/// Print current location
fn print_location(state: &BrowserState) {
    println!("┌─ Location ─────────────────────────────────────────────┐");
    println!("│ {:<54} │", state.current_location());
    println!("└────────────────────────────────────────────────────────┘");
    println!();
}

/// Print directory/file listing
fn print_items(items: &[GitHubContent]) {
    if items.is_empty() {
        println!("  (empty directory)");
        println!();
        return;
    }

    println!("┌─ Contents ─────────────────────────────────────────────┐");

    for (i, item) in items.iter().enumerate() {
        let index = i + 1;
        let test_type = TestType::from_filename(&item.name);
        let category = TestCategory::from_path(&item.path);

        let icon = if item.content_type == "dir" {
            "📁"
        } else if item.name.ends_with(".3mf") {
            test_type.symbol()
        } else {
            "📄"
        };

        let size_str = if let Some(size) = item.size {
            format_size(size)
        } else {
            String::new()
        };

        let display_name = if item.name.len() > 35 {
            format!("{}...", &item.name[..32])
        } else {
            item.name.clone()
        };

        // Show category for files
        if item.content_type == "file" && item.name.ends_with(".3mf") {
            println!(
                "│ {:2}. {} {:<35} {:<8} {:<6} │",
                index,
                icon,
                display_name,
                category.display_name(),
                size_str
            );
        } else {
            println!(
                "│ {:2}. {} {:<35} {:<15} │",
                index, icon, display_name, size_str
            );
        }
    }

    println!("└────────────────────────────────────────────────────────┘");
    println!();
}

/// Print menu options
fn print_menu() {
    println!("─────────────────────────────────────────────────────────");
    println!("  Enter number to select | [b]ack | [r]efresh | [q]uit | [h]elp");
    println!("─────────────────────────────────────────────────────────");
}

/// Print help information
fn print_help() {
    println!();
    println!("┌─ Help ─────────────────────────────────────────────────┐");
    println!("│                                                        │");
    println!("│  Navigation:                                           │");
    println!("│    • Enter a number (1, 2, 3...) to select an item     │");
    println!("│    • Directories: Navigate into the folder             │");
    println!("│    • 3MF Files: Download and open in viewer            │");
    println!("│    • Type 'b' or 'back' to go to parent directory      │");
    println!("│                                                         │");
    println!("│  Commands:                                             │");
    println!("│    • r, refresh - Clear cache and reload directory     │");
    println!("│    • q, quit    - Exit the browser                     │");
    println!("│    • h, help    - Show this help message               │");
    println!("│                                                         │");
    println!("│  Icons:                                                │");
    println!("│    • 📁 - Directory                                     │");
    println!("│    • ✓ - Valid/Positive test file                      │");
    println!("│    • ✗ - Invalid/Negative test file                    │");
    println!("│    • ? - Unknown test type                             │");
    println!("│                                                         │");
    println!("└────────────────────────────────────────────────────────┘");
    println!();
    println!("Press Enter to continue...");
    let _ = get_user_input();
}

/// Get user input from stdin
fn get_user_input() -> Result<String, Box<dyn std::error::Error>> {
    print!("> ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}

/// Format file size in human-readable format
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;

    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{} KB", bytes / KB)
    } else {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(100), "100 B");
        assert_eq!(format_size(1024), "1 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1536 * 1024), "1.5 MB");
    }
}
