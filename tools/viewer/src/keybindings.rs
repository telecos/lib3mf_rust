//! Centralized keybinding registry for the 3MF viewer
//!
//! This module provides a single source of truth for all keyboard shortcuts
//! in the viewer application, making it easy to:
//! - Generate help text automatically
//! - Prevent duplicate/conflicting bindings
//! - Maintain consistency across the application
//! - Add new bindings easily

#![forbid(unsafe_code)]

use kiss3d::event::{Key, Modifiers};

/// Categories for organizing keybindings in help display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    File,
    View,
    Camera,
    Selection,
    Slice,
    Theme,
    Settings,
    Animation,
    Help,
}

impl Category {
    /// Get the display name for this category
    pub fn name(&self) -> &'static str {
        match self {
            Category::File => "FILE",
            Category::View => "VIEW",
            Category::Camera => "CAMERA",
            Category::Selection => "SELECTION",
            Category::Slice => "SLICE",
            Category::Theme => "THEME",
            Category::Settings => "SETTINGS",
            Category::Animation => "ANIMATION",
            Category::Help => "HELP",
        }
    }
}

/// A keybinding definition
#[derive(Debug, Clone)]
pub struct KeyBinding {
    /// The key to press
    #[allow(dead_code)]
    pub key: Option<Key>,
    /// Required modifiers (Ctrl, Shift, etc.)
    #[allow(dead_code)]
    pub modifiers: Modifiers,
    /// Human-readable description
    pub description: &'static str,
    /// Category for organization
    pub category: Category,
    /// Display string for the key combination
    pub display_key: &'static str,
}

impl KeyBinding {
    /// Create a new keybinding
    pub fn new(
        key: Option<Key>,
        modifiers: Modifiers,
        display_key: &'static str,
        description: &'static str,
        category: Category,
    ) -> Self {
        Self {
            key,
            modifiers,
            description,
            category,
            display_key,
        }
    }
}

/// Get all registered keybindings
pub fn get_keybindings() -> Vec<KeyBinding> {
    vec![
        // FILE category
        KeyBinding::new(
            None,
            Modifiers::empty(),
            "Drag & Drop",
            "Drag .3mf file onto window to load",
            Category::File,
        ),
        KeyBinding::new(
            Some(Key::O),
            Modifiers::Control,
            "Ctrl+O",
            "Open file",
            Category::File,
        ),
        KeyBinding::new(
            Some(Key::S),
            Modifiers::empty(),
            "S",
            "Save screenshot",
            Category::File,
        ),
        KeyBinding::new(
            Some(Key::T),
            Modifiers::Control,
            "Ctrl+T",
            "Browse test suites",
            Category::File,
        ),
        // VIEW category
        KeyBinding::new(
            Some(Key::A),
            Modifiers::empty(),
            "A",
            "Toggle axes",
            Category::View,
        ),
        KeyBinding::new(
            Some(Key::P),
            Modifiers::empty(),
            "P",
            "Toggle print bed",
            Category::View,
        ),
        KeyBinding::new(
            Some(Key::M),
            Modifiers::empty(),
            "M",
            "Toggle menu",
            Category::View,
        ),
        KeyBinding::new(
            Some(Key::I),
            Modifiers::empty(),
            "I",
            "Toggle model information",
            Category::View,
        ),
        KeyBinding::new(
            Some(Key::R),
            Modifiers::empty(),
            "R",
            "Toggle materials",
            Category::View,
        ),
        KeyBinding::new(
            Some(Key::B),
            Modifiers::empty(),
            "B",
            "Toggle beam lattice",
            Category::View,
        ),
        KeyBinding::new(
            Some(Key::V),
            Modifiers::empty(),
            "V",
            "Cycle boolean visualization",
            Category::View,
        ),
        KeyBinding::new(
            Some(Key::D),
            Modifiers::empty(),
            "D",
            "Toggle displacement",
            Category::View,
        ),
        // CAMERA category
        KeyBinding::new(
            Some(Key::Home),
            Modifiers::empty(),
            "Home",
            "Reset camera",
            Category::Camera,
        ),
        KeyBinding::new(
            None,
            Modifiers::empty(),
            "Mouse Left",
            "Rotate view",
            Category::Camera,
        ),
        KeyBinding::new(
            None,
            Modifiers::empty(),
            "Mouse Right",
            "Pan view",
            Category::Camera,
        ),
        KeyBinding::new(
            None,
            Modifiers::empty(),
            "Scroll",
            "Zoom in/out",
            Category::Camera,
        ),
        KeyBinding::new(
            Some(Key::PageUp),
            Modifiers::empty(),
            "+/PgUp",
            "Zoom in",
            Category::Camera,
        ),
        KeyBinding::new(
            Some(Key::PageDown),
            Modifiers::empty(),
            "-/PgDn",
            "Zoom out",
            Category::Camera,
        ),
        KeyBinding::new(
            None,
            Modifiers::empty(),
            "Arrow Keys",
            "Pan view",
            Category::Camera,
        ),
        // SELECTION category
        KeyBinding::new(
            None,
            Modifiers::empty(),
            "Left Click",
            "Select object",
            Category::Selection,
        ),
        KeyBinding::new(
            None,
            Modifiers::Control,
            "Ctrl+Click",
            "Multi-select object",
            Category::Selection,
        ),
        KeyBinding::new(
            Some(Key::Escape),
            Modifiers::empty(),
            "Escape",
            "Clear selection",
            Category::Selection,
        ),
        KeyBinding::new(
            Some(Key::F),
            Modifiers::empty(),
            "F",
            "Focus on selected / Fit all",
            Category::Selection,
        ),
        KeyBinding::new(
            Some(Key::G),
            Modifiers::empty(),
            "G",
            "Hide/show selected",
            Category::Selection,
        ),
        KeyBinding::new(
            Some(Key::Y),
            Modifiers::empty(),
            "Y",
            "Isolate selected",
            Category::Selection,
        ),
        // SLICE category
        KeyBinding::new(
            Some(Key::Z),
            Modifiers::empty(),
            "Z",
            "Toggle slice view",
            Category::Slice,
        ),
        KeyBinding::new(
            Some(Key::W),
            Modifiers::empty(),
            "W",
            "Toggle wireframe mode",
            Category::View,
        ),
        KeyBinding::new(
            Some(Key::W),
            Modifiers::Shift,
            "Shift+W",
            "Cycle render modes",
            Category::View,
        ),
        KeyBinding::new(
            Some(Key::Q),
            Modifiers::empty(),
            "Q",
            "Toggle slice preview window",
            Category::Slice,
        ),
        KeyBinding::new(
            Some(Key::Up),
            Modifiers::Shift,
            "Shift+↑",
            "Move slice up",
            Category::Slice,
        ),
        KeyBinding::new(
            Some(Key::Down),
            Modifiers::Shift,
            "Shift+↓",
            "Move slice down",
            Category::Slice,
        ),
        KeyBinding::new(
            Some(Key::L),
            Modifiers::empty(),
            "L",
            "Toggle slice plane",
            Category::Slice,
        ),
        KeyBinding::new(
            Some(Key::X),
            Modifiers::empty(),
            "X",
            "Toggle X-Ray mode",
            Category::View,
        ),
        KeyBinding::new(
            Some(Key::X),
            Modifiers::Shift,
            "Shift+X",
            "Export slice to PNG",
            Category::Slice,
        ),
        KeyBinding::new(
            Some(Key::K),
            Modifiers::empty(),
            "K",
            "Toggle 3D stack view",
            Category::Slice,
        ),
        KeyBinding::new(
            Some(Key::N),
            Modifiers::empty(),
            "N",
            "Toggle filled/outline mode",
            Category::Slice,
        ),
        // ANIMATION category (slice stack)
        KeyBinding::new(
            Some(Key::Space),
            Modifiers::empty(),
            "Space",
            "Play/pause animation",
            Category::Animation,
        ),
        KeyBinding::new(
            Some(Key::Home),
            Modifiers::empty(),
            "Home",
            "First slice",
            Category::Animation,
        ),
        KeyBinding::new(
            Some(Key::End),
            Modifiers::empty(),
            "End",
            "Last slice",
            Category::Animation,
        ),
        KeyBinding::new(
            Some(Key::RBracket),
            Modifiers::empty(),
            "]",
            "Increase speed",
            Category::Animation,
        ),
        KeyBinding::new(
            Some(Key::LBracket),
            Modifiers::empty(),
            "[",
            "Decrease speed",
            Category::Animation,
        ),
        // THEME category
        KeyBinding::new(
            Some(Key::T),
            Modifiers::empty(),
            "T",
            "Cycle themes",
            Category::Theme,
        ),
        // SETTINGS category
        KeyBinding::new(
            Some(Key::C),
            Modifiers::empty(),
            "C",
            "Configure print bed",
            Category::Settings,
        ),
        KeyBinding::new(
            Some(Key::U),
            Modifiers::empty(),
            "U",
            "Toggle ruler",
            Category::Settings,
        ),
        KeyBinding::new(
            Some(Key::J),
            Modifiers::empty(),
            "J",
            "Toggle scale bar",
            Category::Settings,
        ),
        // HELP category
        // Note: Both H and ? (Shift+/) are handled to show help, but we only need
        // one entry since they perform the same action and display_key shows both
        KeyBinding::new(
            Some(Key::H),
            Modifiers::empty(),
            "H or ?",
            "Show this help",
            Category::Help,
        ),
    ]
}

/// Print the help display with organized categories
pub fn print_help() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    3MF Viewer - Controls                      ║");
    println!("╠══════════════════════════════════════════════════════════════╣");

    let bindings = get_keybindings();

    // Group bindings by category
    let categories = [
        Category::File,
        Category::View,
        Category::Camera,
        Category::Selection,
        Category::Slice,
        Category::Animation,
        Category::Theme,
        Category::Settings,
        Category::Help,
    ];

    for category in &categories {
        let category_bindings: Vec<&KeyBinding> = bindings
            .iter()
            .filter(|b| b.category == *category)
            .collect();

        if !category_bindings.is_empty() {
            print_section(category.name(), &category_bindings);
        }
    }

    println!("╚══════════════════════════════════════════════════════════════╝");
}

/// Print a section of keybindings
fn print_section(title: &str, bindings: &[&KeyBinding]) {
    println!("║  {:60}║", title);
    for binding in bindings {
        println!(
            "║    {:12} {:45}║",
            binding.display_key, binding.description
        );
    }
    println!("║{:62}║", "");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_duplicate_keys() {
        let bindings = get_keybindings();

        // Check for duplicate key+modifier combinations (excluding None keys like mouse)
        let mut seen = std::collections::HashSet::new();
        for binding in &bindings {
            if let Some(key) = binding.key {
                let combo = (format!("{:?}", key), format!("{:?}", binding.modifiers));
                // Note: Some keys like Home, Escape, and F are used in different contexts
                // This is acceptable as the context determines which handler is active
                if !seen.insert(combo.clone())
                    && key != Key::Home
                    && key != Key::Escape
                    && key != Key::F
                {
                    panic!("Duplicate keybinding found: {:?}", combo);
                }
            }
        }
    }

    #[test]
    fn test_all_categories_have_bindings() {
        let bindings = get_keybindings();
        let categories = [
            Category::File,
            Category::View,
            Category::Camera,
            Category::Selection,
            Category::Slice,
            Category::Animation,
            Category::Theme,
            Category::Settings,
            Category::Help,
        ];

        for category in &categories {
            let count = bindings.iter().filter(|b| b.category == *category).count();
            assert!(count > 0, "Category {:?} has no bindings", category);
        }
    }

    #[test]
    fn test_help_prints_without_panic() {
        // Just ensure the help function doesn't panic
        print_help();
    }
}
