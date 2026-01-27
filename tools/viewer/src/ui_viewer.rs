//! Interactive 3D UI Viewer for 3MF files
//!
//! This module provides an interactive 3D viewer using kiss3d
//! for rendering 3MF models with mouse controls and real-time interaction.

#![forbid(unsafe_code)]

use kiss3d::event::{Action, Key, WindowEvent};
use kiss3d::light::Light;
use kiss3d::nalgebra::{Point3, Vector3}; // Use nalgebra from kiss3d
use kiss3d::ncollide3d::procedural::TriMesh;
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use lib3mf::Model;
use rfd::FileDialog;
use std::fs::File;
use std::path::PathBuf;

/// Color themes for the viewer background
#[derive(Debug, Clone, Copy, PartialEq)]
enum Theme {
    Dark,
    Light,
    Blue,
    White,
    Black,
    #[allow(dead_code)]
    Custom(f32, f32, f32),
}

impl Theme {
    /// Get the background color for this theme
    fn background_color(&self) -> (f32, f32, f32) {
        match self {
            Theme::Dark => (0.1, 0.1, 0.1),
            Theme::Light => (0.88, 0.88, 0.88),
            Theme::Blue => (0.04, 0.09, 0.16),
            Theme::White => (1.0, 1.0, 1.0),
            Theme::Black => (0.0, 0.0, 0.0),
            Theme::Custom(r, g, b) => (*r, *g, *b),
        }
    }

    /// Get the next theme in the cycle
    fn next(&self) -> Theme {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Blue,
            Theme::Blue => Theme::White,
            Theme::White => Theme::Black,
            Theme::Black => Theme::Dark,
            Theme::Custom(_, _, _) => Theme::Dark,
        }
    }

    /// Get the name of the theme for display
    fn name(&self) -> &'static str {
        match self {
            Theme::Dark => "Dark",
            Theme::Light => "Light",
            Theme::Blue => "Blue",
            Theme::White => "White",
            Theme::Black => "Black",
            Theme::Custom(_, _, _) => "Custom",
        }
    }
}

/// Viewer state that can optionally hold a loaded model
struct ViewerState {
    model: Option<Model>,
    file_path: Option<PathBuf>,
    mesh_nodes: Vec<SceneNode>,
    theme: Theme,
}

impl ViewerState {
    /// Create a new empty viewer state
    fn new_empty() -> Self {
        Self {
            model: None,
            file_path: None,
            mesh_nodes: Vec::new(),
            theme: Theme::Dark,
        }
    }

    /// Create a viewer state with a loaded model
    fn with_model(model: Model, file_path: PathBuf) -> Self {
        Self {
            model: Some(model),
            file_path: Some(file_path),
            mesh_nodes: Vec::new(),
            theme: Theme::Dark,
        }
    }

    /// Load a file into the viewer state
    fn load_file(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(&path)?;
        let model = Model::from_reader(file)?;
        self.model = Some(model);
        self.file_path = Some(path);
        Ok(())
    }

    /// Get window title based on current state
    fn window_title(&self) -> String {
        if let Some(ref path) = self.file_path {
            format!("3MF Viewer - {}", path.display())
        } else {
            "3MF Viewer - No file loaded".to_string()
        }
    }

    /// Cycle to next theme and apply it to the window
    fn cycle_theme(&mut self, window: &mut Window) {
        self.theme = self.theme.next();
        let bg_color = self.theme.background_color();
        window.set_background_color(bg_color.0, bg_color.1, bg_color.2);
        println!("Theme changed to: {}", self.theme.name());
    }
}

/// Launch the interactive UI viewer
pub fn launch_ui_viewer(file_path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    // Create initial viewer state
    let mut state = if let Some(path) = file_path {
        println!("Loading: {}", path.display());
        let file = File::open(&path)?;
        let model = Model::from_reader(file)?;
        println!("✓ Model loaded successfully!");
        ViewerState::with_model(model, path)
    } else {
        println!("Starting viewer with empty scene...");
        println!("Press Ctrl+O to open a 3MF file");
        ViewerState::new_empty()
    };

    let mut window = Window::new(&state.window_title());
    window.set_light(Light::StickToCamera);

    // The ArcBall camera in kiss3d is controlled by mouse automatically
    // Just set a reasonable initial distance
    window.set_framerate_limit(Some(60));

    // Set initial background color based on theme
    let bg_color = state.theme.background_color();
    window.set_background_color(bg_color.0, bg_color.1, bg_color.2);

    // Create meshes from the model if one is loaded
    if state.model.is_some() {
        state.mesh_nodes = create_mesh_nodes(&mut window, state.model.as_ref().unwrap());
        print_model_info(state.model.as_ref().unwrap());
    } else {
        print_empty_scene_info();
    }

    print_controls();

    // Track axis visualization state (default: visible)
    let mut show_axes = true;

    // Calculate axis length based on model size (if model is loaded)
    let mut axis_length = 100.0; // Default length for empty scene
    if let Some(ref model) = state.model {
        let (min_bound, max_bound) = calculate_model_bounds(model);
        let size = Vector3::new(
            max_bound.0 - min_bound.0,
            max_bound.1 - min_bound.1,
            max_bound.2 - min_bound.2,
        );
        let max_size = size.x.max(size.y).max(size.z);
        axis_length = max_size * 0.5; // 50% of model size
    }

    // Main event loop
    while window.render() {
        // Handle window events
        for event in window.events().iter() {
            match event.value {
                WindowEvent::Key(Key::O, Action::Press, modifiers)
                    if modifiers.contains(kiss3d::event::Modifiers::Control) =>
                {
                    // Ctrl+O: Open file dialog
                    if let Some(path) = open_file_dialog() {
                        match state.load_file(path) {
                            Ok(()) => {
                                // Hide existing mesh nodes by setting them invisible
                                for node in &mut state.mesh_nodes {
                                    node.set_visible(false);
                                }
                                state.mesh_nodes.clear();

                                // Create new mesh nodes
                                if let Some(ref model) = state.model {
                                    state.mesh_nodes = create_mesh_nodes(&mut window, model);
                                    window.set_title(&state.window_title());
                                    println!("\n✓ File loaded successfully!");
                                    print_model_info(model);

                                    // Recalculate axis length based on new model
                                    let (min_bound, max_bound) = calculate_model_bounds(model);
                                    let size = Vector3::new(
                                        max_bound.0 - min_bound.0,
                                        max_bound.1 - min_bound.1,
                                        max_bound.2 - min_bound.2,
                                    );
                                    let max_size = size.x.max(size.y).max(size.z);
                                    axis_length = max_size * 0.5;
                                }
                            }
                            Err(e) => {
                                eprintln!("\n✗ Error loading file: {}", e);
                            }
                        }
                    }
                }
                WindowEvent::Key(Key::T, Action::Press, modifiers)
                    if modifiers.contains(kiss3d::event::Modifiers::Control) =>
                {
                    // Ctrl+T: Browse test suites
                    println!("\n");
                    println!("Opening test suite browser...");
                    println!("(The 3D viewer window will remain open in the background)");
                    println!();
                    
                    if let Ok(Some(path)) = crate::browser_ui::launch_browser() {
                        match state.load_file(path) {
                            Ok(()) => {
                                // Hide existing mesh nodes by setting them invisible
                                for node in &mut state.mesh_nodes {
                                    node.set_visible(false);
                                }
                                state.mesh_nodes.clear();

                                // Create new mesh nodes
                                if let Some(ref model) = state.model {
                                    state.mesh_nodes = create_mesh_nodes(&mut window, model);
                                    window.set_title(&state.window_title());
                                    println!("\n✓ File loaded successfully!");
                                    print_model_info(model);
                                }
                            }
                            Err(e) => {
                                eprintln!("\n✗ Error loading file: {}", e);
                            }
                        }
                    }
                }
                WindowEvent::Key(Key::T, Action::Press, _) => {
                    // T: Cycle through themes
                    state.cycle_theme(&mut window);
                }
                WindowEvent::Key(Key::B, Action::Press, _) => {
                    // B: Cycle through themes (alternative to T key)
                    state.cycle_theme(&mut window);
                }
                WindowEvent::Key(Key::A, Action::Release, _) => {
                    // A key: Toggle XYZ axes
                    show_axes = !show_axes;
                    println!("XYZ Axes: {}", if show_axes { "ON" } else { "OFF" });
                }
                _ => {}
            }
        }

        // Draw XYZ axes if visible
        if show_axes {
            draw_axes(&mut window, axis_length);
        }
    }

    Ok(())
}

/// Open a file dialog to select a 3MF file
fn open_file_dialog() -> Option<PathBuf> {
    FileDialog::new()
        .add_filter("3MF Files", &["3mf"])
        .add_filter("All Files", &["*"])
        .set_title("Open 3MF File")
        .pick_file()
}

/// Print controls information
fn print_controls() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  Interactive 3D Viewer Controls");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("  🖱️  Left Mouse + Drag  : Rotate view");
    println!("  🖱️  Right Mouse + Drag : Pan view");
    println!("  🖱️  Scroll Wheel       : Zoom in/out");
    println!("  ⌨️  Arrow Keys         : Pan view");
    println!("  ⌨️  A Key              : Toggle XYZ axes");
    println!("  ⌨️  Ctrl+O             : Open file");
    println!("  ⌨️  T or B             : Cycle themes");
    println!("  ⌨️  Ctrl+T             : Browse test suites");
    println!("  ⌨️  ESC / Close Window : Exit viewer");
    println!();
    println!("═══════════════════════════════════════════════════════════");
}

/// Print empty scene information
fn print_empty_scene_info() {
    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!("  No file loaded");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("  Press Ctrl+O to open a 3MF file");
    println!("  Press Ctrl+T to browse test suites from GitHub");
    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!();
}

/// Print model information
fn print_model_info(model: &Model) {
    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!("  Model Information:");
    println!("  - Objects: {}", model.resources.objects.len());
    println!("  - Triangles: {}", count_triangles(model));
    println!("  - Vertices: {}", count_vertices(model));
    println!("  - Unit: {}", model.unit);
    println!();
    println!("═══════════════════════════════════════════════════════════");
}

/// Create mesh scene nodes from the 3MF model
fn create_mesh_nodes(window: &mut Window, model: &Model) -> Vec<SceneNode> {
    let mut nodes = Vec::new();

    for item in &model.build.items {
        if let Some(obj) = model
            .resources
            .objects
            .iter()
            .find(|o| o.id == item.objectid)
        {
            if let Some(ref mesh_data) = obj.mesh {
                // Convert vertices to nalgebra Point3
                let vertices: Vec<Point3<f32>> = mesh_data
                    .vertices
                    .iter()
                    .map(|v| Point3::new(v.x as f32, v.y as f32, v.z as f32))
                    .collect();

                // Convert triangles to face indices (Point3<u32> for TriMesh)
                let faces: Vec<Point3<u32>> = mesh_data
                    .triangles
                    .iter()
                    .filter(|t| {
                        t.v1 < vertices.len() && t.v2 < vertices.len() && t.v3 < vertices.len()
                    })
                    .map(|t| Point3::new(t.v1 as u32, t.v2 as u32, t.v3 as u32))
                    .collect();

                // Create TriMesh
                let tri_mesh = TriMesh::new(
                    vertices,
                    None, // No normals, will be computed
                    None, // No UVs
                    Some(kiss3d::ncollide3d::procedural::IndexBuffer::Unified(faces)),
                );

                // Get object color
                let color = get_object_color(model, obj);

                // Create mesh and add to scene
                let scale = Vector3::new(1.0, 1.0, 1.0);
                let mut mesh_node = window.add_trimesh(tri_mesh, scale);
                mesh_node.set_color(color.0, color.1, color.2);

                nodes.push(mesh_node);
            }
        }
    }

    nodes
}

/// Calculate the bounding box of all meshes in the model
#[allow(dead_code)]
fn calculate_model_bounds(model: &Model) -> ((f32, f32, f32), (f32, f32, f32)) {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut min_z = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    let mut max_z = f32::MIN;

    for item in &model.build.items {
        if let Some(obj) = model
            .resources
            .objects
            .iter()
            .find(|o| o.id == item.objectid)
        {
            if let Some(ref mesh) = obj.mesh {
                for v in &mesh.vertices {
                    min_x = min_x.min(v.x as f32);
                    min_y = min_y.min(v.y as f32);
                    min_z = min_z.min(v.z as f32);
                    max_x = max_x.max(v.x as f32);
                    max_y = max_y.max(v.y as f32);
                    max_z = max_z.max(v.z as f32);
                }
            }
        }
    }

    // Provide default bounds if no meshes found
    if min_x == f32::MAX {
        return ((0.0, 0.0, 0.0), (1.0, 1.0, 1.0));
    }

    ((min_x, min_y, min_z), (max_x, max_y, max_z))
}

/// Get color for an object (from materials or default)
fn get_object_color(model: &Model, obj: &lib3mf::Object) -> (f32, f32, f32) {
    // Check if object has a default material
    if let Some(pid) = obj.pid {
        // Try to find in base materials
        if let Some(mat) = model.resources.materials.iter().find(|m| m.id == pid) {
            if let Some((r, g, b, _)) = mat.color {
                return (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
            }
        }
        // Try to find in color groups (use first color)
        if let Some(cg) = model.resources.color_groups.iter().find(|c| c.id == pid) {
            if !cg.colors.is_empty() {
                let (r, g, b, _) = cg.colors[0];
                return (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
            }
        }
    }

    // Default color: nice blue-gray
    (100.0 / 255.0, 150.0 / 255.0, 200.0 / 255.0)
}

/// Count total triangles in the model
fn count_triangles(model: &Model) -> usize {
    let mut total = 0;
    for item in &model.build.items {
        if let Some(obj) = model
            .resources
            .objects
            .iter()
            .find(|o| o.id == item.objectid)
        {
            if let Some(ref mesh) = obj.mesh {
                total += mesh.triangles.len();
            }
        }
    }
    total
}

/// Count total vertices in the model
fn count_vertices(model: &Model) -> usize {
    let mut total = 0;
    for item in &model.build.items {
        if let Some(obj) = model
            .resources
            .objects
            .iter()
            .find(|o| o.id == item.objectid)
        {
            if let Some(ref mesh) = obj.mesh {
                total += mesh.vertices.len();
            }
        }
    }
    total
}

/// Draw XYZ coordinate axes
/// X axis = Red, Y axis = Green, Z axis = Blue
fn draw_axes(window: &mut Window, length: f32) {
    let origin = Point3::origin();

    // X axis - Red
    window.draw_line(
        &origin,
        &Point3::new(length, 0.0, 0.0),
        &Point3::new(1.0, 0.0, 0.0), // Red color
    );

    // Y axis - Green
    window.draw_line(
        &origin,
        &Point3::new(0.0, length, 0.0),
        &Point3::new(0.0, 1.0, 0.0), // Green color
    );

    // Z axis - Blue
    window.draw_line(
        &origin,
        &Point3::new(0.0, 0.0, length),
        &Point3::new(0.0, 0.0, 1.0), // Blue color
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_cycling() {
        let theme = Theme::Dark;
        assert_eq!(theme.next(), Theme::Light);
        
        let theme = theme.next();
        assert_eq!(theme, Theme::Light);
        assert_eq!(theme.next(), Theme::Blue);
        
        let theme = theme.next();
        assert_eq!(theme, Theme::Blue);
        assert_eq!(theme.next(), Theme::White);
        
        let theme = theme.next();
        assert_eq!(theme, Theme::White);
        assert_eq!(theme.next(), Theme::Black);
        
        let theme = theme.next();
        assert_eq!(theme, Theme::Black);
        assert_eq!(theme.next(), Theme::Dark);
    }

    #[test]
    fn test_theme_background_colors() {
        assert_eq!(Theme::Dark.background_color(), (0.1, 0.1, 0.1));
        assert_eq!(Theme::Light.background_color(), (0.88, 0.88, 0.88));
        assert_eq!(Theme::Blue.background_color(), (0.04, 0.09, 0.16));
        assert_eq!(Theme::White.background_color(), (1.0, 1.0, 1.0));
        assert_eq!(Theme::Black.background_color(), (0.0, 0.0, 0.0));
        
        let custom = Theme::Custom(0.5, 0.6, 0.7);
        assert_eq!(custom.background_color(), (0.5, 0.6, 0.7));
    }

    #[test]
    fn test_theme_names() {
        assert_eq!(Theme::Dark.name(), "Dark");
        assert_eq!(Theme::Light.name(), "Light");
        assert_eq!(Theme::Blue.name(), "Blue");
        assert_eq!(Theme::White.name(), "White");
        assert_eq!(Theme::Black.name(), "Black");

        let custom = Theme::Custom(0.5, 0.6, 0.7);
        assert_eq!(custom.name(), "Custom");
    }
}
