//! Example program to demonstrate the help display
//! 
//! This shows the organized keyboard controls without launching the UI.
//! 
//! NOTE: This example duplicates the help display logic rather than importing
//! from the keybindings module because examples in Rust binary crates cannot
//! easily import internal modules. This is a standalone demonstration of the
//! help output format. The actual implementation is in src/keybindings.rs.

#![forbid(unsafe_code)]

fn main() {
    println!();
    print_help();
    println!();
}

fn print_help() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                    3MF Viewer - Controls                      ║");
    println!("╠══════════════════════════════════════════════════════════════╣");

    print_section("FILE", &[
        ("Ctrl+O", "Open file"),
        ("S", "Save screenshot"),
        ("Escape", "Exit"),
        ("Ctrl+T", "Browse test suites"),
    ]);

    print_section("VIEW", &[
        ("A", "Toggle axes"),
        ("P", "Toggle print bed"),
        ("M", "Toggle menu"),
        ("R", "Toggle materials"),
        ("B", "Toggle beam lattice"),
        ("V", "Cycle boolean visualization"),
        ("D", "Toggle displacement"),
    ]);

    print_section("CAMERA", &[
        ("F", "Fit model to view"),
        ("Home", "Reset camera"),
        ("Mouse Left", "Rotate view"),
        ("Mouse Right", "Pan view"),
        ("Scroll", "Zoom in/out"),
        ("+/PgUp", "Zoom in"),
        ("-/PgDn", "Zoom out"),
        ("Arrow Keys", "Pan view"),
    ]);

    print_section("SLICE", &[
        ("Z", "Toggle slice view"),
        ("W", "Toggle slice preview window"),
        ("Shift+↑", "Move slice up"),
        ("Shift+↓", "Move slice down"),
        ("L", "Toggle slice plane"),
        ("X", "Export slice to PNG"),
        ("K", "Toggle 3D stack view"),
        ("N", "Toggle filled/outline mode"),
    ]);

    print_section("ANIMATION", &[
        ("Space", "Play/pause animation"),
        ("Home", "First slice"),
        ("End", "Last slice"),
        ("]", "Increase speed"),
        ("[", "Decrease speed"),
    ]);

    print_section("THEME", &[
        ("T", "Cycle themes"),
    ]);

    print_section("SETTINGS", &[
        ("C", "Configure print bed"),
    ]);

    print_section("HELP", &[
        ("H or ?", "Show this help"),
    ]);

    println!("╚══════════════════════════════════════════════════════════════╝");
}

fn print_section(title: &str, items: &[(&str, &str)]) {
    println!("║  {:60}║", title);
    for (key, desc) in items {
        println!("║    {:12} {:45}║", key, desc);
    }
    println!("║{:62}║", "");
}
