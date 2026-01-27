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

// Constants for beam lattice rendering
const BEAM_COLOR: (f32, f32, f32) = (1.0, 0.6, 0.0); // Orange color for beams
const GEOMETRY_SEGMENTS: u32 = 8; // Number of segments for cylinder/sphere meshes
const IDENTITY_SCALE: Vector3<f32> = Vector3::new(1.0, 1.0, 1.0); // Identity scale for meshes

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

/// Print area configuration for build volume visualization
#[derive(Debug, Clone)]
struct PrintArea {
    width: f32,   // X dimension
    depth: f32,   // Y dimension
    height: f32,  // Z dimension
    unit: String, // "mm", "inch", etc.
    visible: bool,
}

impl PrintArea {
    /// Create a new print area with default settings
    fn new() -> Self {
        Self {
            width: 200.0,
            depth: 200.0,
            height: 200.0,
            unit: "mm".to_string(),
            visible: true,
        }
    }

    /// Toggle visibility of the print area
    fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }
}

/// Viewer state that can optionally hold a loaded model
struct ViewerState {
    model: Option<Model>,
    file_path: Option<PathBuf>,
    mesh_nodes: Vec<SceneNode>,
    beam_nodes: Vec<SceneNode>,
    show_beams: bool,
    theme: Theme,
    print_area: PrintArea,
    show_menu: bool,
}

impl ViewerState {
    /// Create a new empty viewer state
    fn new_empty() -> Self {
        Self {
            model: None,
            file_path: None,
            mesh_nodes: Vec::new(),
            beam_nodes: Vec::new(),
            show_beams: true,
            theme: Theme::Dark,
            print_area: PrintArea::new(),
            show_menu: false,
        }
    }

    /// Create a viewer state with a loaded model
    fn with_model(model: Model, file_path: PathBuf) -> Self {
        Self {
            model: Some(model),
            file_path: Some(file_path),
            mesh_nodes: Vec::new(),
            beam_nodes: Vec::new(),
            show_beams: true,
            theme: Theme::Dark,
            print_area: PrintArea::new(),
            show_menu: false,
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
        state.beam_nodes = create_beam_lattice_nodes(&mut window, state.model.as_ref().unwrap());
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

                                // Hide existing beam nodes
                                for node in &mut state.beam_nodes {
                                    node.set_visible(false);
                                }
                                state.beam_nodes.clear();

                                // Create new mesh and beam nodes
                                if let Some(ref model) = state.model {
                                    state.mesh_nodes = create_mesh_nodes(&mut window, model);
                                    state.beam_nodes = create_beam_lattice_nodes(&mut window, model);
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

                                // Hide existing beam nodes
                                for node in &mut state.beam_nodes {
                                    node.set_visible(false);
                                }
                                state.beam_nodes.clear();

                                // Create new mesh and beam nodes
                                if let Some(ref model) = state.model {
                                    state.mesh_nodes = create_mesh_nodes(&mut window, model);
                                    state.beam_nodes = create_beam_lattice_nodes(&mut window, model);
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
                WindowEvent::Key(Key::B, Action::Press, _) => {
                    // B: Toggle beam lattice visibility
                    state.show_beams = !state.show_beams;
                    for node in &mut state.beam_nodes {
                        node.set_visible(state.show_beams);
                    }
                    println!(
                        "\nBeam lattice: {}",
                        if state.show_beams { "visible" } else { "hidden" }
                    );
                }
                WindowEvent::Key(Key::T, Action::Press, _) => {
                    // T: Cycle through themes
                    state.cycle_theme(&mut window);
                }
                WindowEvent::Key(Key::A, Action::Release, _) => {
                    // A key: Toggle XYZ axes
                    show_axes = !show_axes;
                    println!("XYZ Axes: {}", if show_axes { "ON" } else { "OFF" });
                }
                WindowEvent::Key(Key::S, Action::Release, _) => {
                    // S key: Capture screenshot
                    if let Err(e) = capture_screenshot(&window) {
                        eprintln!("\n✗ Error capturing screenshot: {}", e);
                    }
                }
                WindowEvent::Key(Key::M, Action::Release, _) => {
                    // M key: Toggle menu display
                    state.show_menu = !state.show_menu;
                    if state.show_menu {
                        print_menu(&state);
                    } else {
                        println!("Menu hidden");
                    }
                }
                WindowEvent::Key(Key::P, Action::Release, _) => {
                    // P key: Toggle print area visibility
                    state.print_area.toggle_visibility();
                    println!(
                        "Print Area: {}",
                        if state.print_area.visible { "ON" } else { "OFF" }
                    );
                }
                WindowEvent::Key(Key::C, Action::Release, _) => {
                    // C key: Configure print area
                    println!("\n═══════════════════════════════════════════════════════════");
                    println!("  Configure Print Area");
                    println!("═══════════════════════════════════════════════════════════");
                    println!();
                    println!("Current settings:");
                    println!("  Width (X):  {} {}", state.print_area.width, state.print_area.unit);
                    println!("  Depth (Y):  {} {}", state.print_area.depth, state.print_area.unit);
                    println!("  Height (Z): {} {}", state.print_area.height, state.print_area.unit);
                    println!();
                    println!("To change settings, use the console:");
                    println!("  - Enter new dimensions when prompted");
                    println!("  - Press Enter to keep current value");
                    println!();
                    println!("═══════════════════════════════════════════════════════════");
                    
                    // Simple console-based configuration
                    if let Ok(new_config) = configure_print_area(&state.print_area) {
                        state.print_area = new_config;
                        println!("\n✓ Print area updated successfully!");
                        println!("  Width (X):  {} {}", state.print_area.width, state.print_area.unit);
                        println!("  Depth (Y):  {} {}", state.print_area.depth, state.print_area.unit);
                        println!("  Height (Z): {} {}", state.print_area.height, state.print_area.unit);
                    }
                }
                _ => {}
            }
        }

        // Draw XYZ axes if visible
        if show_axes {
            draw_axes(&mut window, axis_length);
        }

        // Draw print area if visible
        if state.print_area.visible {
            draw_print_area(&mut window, &state.print_area);
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

/// Generate a timestamped filename for screenshots
fn generate_screenshot_filename() -> String {
    let now = chrono::Local::now();
    format!("screenshot_{}.png", now.format("%Y-%m-%d_%H%M%S"))
}

/// Capture screenshot of the current window view
fn capture_screenshot(window: &Window) -> Result<(), Box<dyn std::error::Error>> {
    let filename = generate_screenshot_filename();
    
    // Capture the current frame
    let img = window.snap_image();
    
    // Save as PNG
    img.save(&filename)?;
    
    println!("\n✓ Screenshot saved: {}", filename);
    
    Ok(())
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
    println!("  ⌨️  M Key              : Toggle menu");
    println!("  ⌨️  P Key              : Toggle print area");
    println!("  ⌨️  C Key              : Configure print area");
    println!("  ⌨️  Ctrl+O             : Open file");
    println!("  ⌨️  T                  : Cycle themes");
    println!("  ⌨️  Ctrl+T             : Browse test suites");
    println!("  ⌨️  B                  : Toggle beam lattice");
    println!("  ⌨️  S                  : Capture screenshot");
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
    let beam_count = count_beams(model);
    
    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!("  Model Information:");
    println!("  - Objects: {}", model.resources.objects.len());
    println!("  - Triangles: {}", count_triangles(model));
    println!("  - Vertices: {}", count_vertices(model));
    println!("  - Unit: {}", model.unit);
    if beam_count > 0 {
        println!("  - Beam Lattice: {} beams", beam_count);
    }
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

/// Count total beams in the model
fn count_beams(model: &Model) -> usize {
    let mut total = 0;
    for item in &model.build.items {
        if let Some(obj) = model
            .resources
            .objects
            .iter()
            .find(|o| o.id == item.objectid)
        {
            if let Some(ref mesh) = obj.mesh {
                if let Some(ref beamset) = mesh.beamset {
                    total += beamset.beams.len();
                }
            }
        }
    }
    total
}

/// Create a cylinder mesh between two points with specified radii
///
/// Creates a tapered cylinder (cone if r1 != r2) connecting p1 and p2.
/// The cylinder is generated with the specified number of segments around the circumference.
fn create_cylinder_mesh(
    p1: Point3<f32>,
    p2: Point3<f32>,
    r1: f32,
    r2: f32,
    segments: u32,
) -> TriMesh<f32> {
    let mut vertices = Vec::new();
    let mut faces = Vec::new();

    // Calculate cylinder axis and length
    let axis = p2 - p1;
    let length = axis.norm();
    
    if length < 1e-6 {
        // Degenerate cylinder, return empty mesh
        return TriMesh::new(vertices, None, None, Some(kiss3d::ncollide3d::procedural::IndexBuffer::Unified(faces)));
    }

    let axis_normalized = axis.normalize();

    // Find perpendicular vectors for circle generation
    let up = if axis_normalized.y.abs() < 0.9 {
        Vector3::new(0.0, 1.0, 0.0)
    } else {
        Vector3::new(1.0, 0.0, 0.0)
    };
    let right = axis_normalized.cross(&up).normalize();
    let forward = axis_normalized.cross(&right).normalize();

    // Generate vertices for both circles
    for i in 0..segments {
        let angle = 2.0 * std::f32::consts::PI * (i as f32) / (segments as f32);
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // Bottom circle (at p1)
        let offset1 = right * (cos_a * r1) + forward * (sin_a * r1);
        vertices.push(p1 + offset1);

        // Top circle (at p2)
        let offset2 = right * (cos_a * r2) + forward * (sin_a * r2);
        vertices.push(p2 + offset2);
    }

    // Generate faces connecting the two circles
    for i in 0..segments {
        let next_i = (i + 1) % segments;
        
        let b1 = i * 2;
        let t1 = i * 2 + 1;
        let b2 = next_i * 2;
        let t2 = next_i * 2 + 1;

        // Two triangles per quad
        faces.push(Point3::new(b1, t1, b2));
        faces.push(Point3::new(b2, t1, t2));
    }

    // Add end caps if radii are non-zero
    let base_vertex_count = vertices.len();
    
    // Bottom cap (at p1)
    if r1 > 1e-6 {
        vertices.push(p1); // Center vertex
        let center_idx = base_vertex_count as u32;
        for i in 0..segments {
            let next_i = (i + 1) % segments;
            let v1 = i * 2;
            let v2 = next_i * 2;
            faces.push(Point3::new(center_idx, v2, v1));
        }
    }

    // Top cap (at p2)
    if r2 > 1e-6 {
        let cap_vertex_count = vertices.len();
        vertices.push(p2); // Center vertex
        let center_idx = cap_vertex_count as u32;
        for i in 0..segments {
            let next_i = (i + 1) % segments;
            let v1 = i * 2 + 1;
            let v2 = next_i * 2 + 1;
            faces.push(Point3::new(center_idx, v1, v2));
        }
    }

    TriMesh::new(
        vertices,
        None, // No normals, will be computed
        None, // No UVs
        Some(kiss3d::ncollide3d::procedural::IndexBuffer::Unified(faces)),
    )
}

/// Create a sphere mesh at a given center point with specified radius
fn create_sphere_mesh(center: Point3<f32>, radius: f32, segments: u32) -> TriMesh<f32> {
    let mut vertices = Vec::new();
    let mut faces = Vec::new();

    // Add top vertex
    vertices.push(center + Vector3::new(0.0, 0.0, radius));

    // Generate rings of vertices
    let rings = segments / 2;
    for ring in 1..rings {
        let phi = std::f32::consts::PI * (ring as f32) / (rings as f32);
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();

        for seg in 0..segments {
            let theta = 2.0 * std::f32::consts::PI * (seg as f32) / (segments as f32);
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            let x = sin_phi * cos_theta * radius;
            let y = sin_phi * sin_theta * radius;
            let z = cos_phi * radius;

            vertices.push(center + Vector3::new(x, y, z));
        }
    }

    // Add bottom vertex
    vertices.push(center + Vector3::new(0.0, 0.0, -radius));

    // Generate faces for top cap
    for seg in 0..segments {
        let next_seg = (seg + 1) % segments;
        faces.push(Point3::new(
            0,
            seg + 1,
            next_seg + 1,
        ));
    }

    // Generate faces for middle rings
    for ring in 0..(rings - 2) {
        let ring_start = 1 + ring * segments;
        let next_ring_start = ring_start + segments;

        for seg in 0..segments {
            let next_seg = (seg + 1) % segments;

            let v1 = ring_start + seg;
            let v2 = ring_start + next_seg;
            let v3 = next_ring_start + seg;
            let v4 = next_ring_start + next_seg;

            faces.push(Point3::new(v1, v3, v2));
            faces.push(Point3::new(v2, v3, v4));
        }
    }

    // Generate faces for bottom cap
    let last_ring_start = 1 + (rings - 2) * segments;
    let bottom_vertex = (vertices.len() - 1) as u32;
    for seg in 0..segments {
        let next_seg = (seg + 1) % segments;
        faces.push(Point3::new(
            bottom_vertex,
            last_ring_start + next_seg,
            last_ring_start + seg,
        ));
    }

    TriMesh::new(
        vertices,
        None, // No normals, will be computed
        None, // No UVs
        Some(kiss3d::ncollide3d::procedural::IndexBuffer::Unified(faces)),
    )
}

/// Create beam lattice nodes from beamsets in the model
fn create_beam_lattice_nodes(window: &mut Window, model: &Model) -> Vec<SceneNode> {
    let mut nodes = Vec::new();

    for item in &model.build.items {
        if let Some(obj) = model
            .resources
            .objects
            .iter()
            .find(|o| o.id == item.objectid)
        {
            if let Some(ref mesh_data) = obj.mesh {
                if let Some(ref beamset) = mesh_data.beamset {
                    // Generate beam cylinders
                    for beam in &beamset.beams {
                        // Get vertex positions
                        if beam.v1 >= mesh_data.vertices.len() || beam.v2 >= mesh_data.vertices.len() {
                            continue; // Skip invalid beam
                        }

                        let v1 = &mesh_data.vertices[beam.v1];
                        let v2 = &mesh_data.vertices[beam.v2];

                        let p1 = Point3::new(v1.x as f32, v1.y as f32, v1.z as f32);
                        let p2 = Point3::new(v2.x as f32, v2.y as f32, v2.z as f32);

                        // Get beam radii (use beam radius or beamset default)
                        let r1 = beam.r1.unwrap_or(beamset.radius) as f32;
                        let r2 = beam.r2.map(|r| r as f32).unwrap_or(r1);

                        // Create cylinder mesh for the beam
                        let cylinder = create_cylinder_mesh(p1, p2, r1, r2, GEOMETRY_SEGMENTS);
                        let mut mesh_node = window.add_trimesh(cylinder, IDENTITY_SCALE);
                        
                        // Set beam color
                        mesh_node.set_color(BEAM_COLOR.0, BEAM_COLOR.1, BEAM_COLOR.2);
                        
                        nodes.push(mesh_node);
                    }

                    // Add spherical joints at highly connected vertices
                    // (only for sphere cap mode)
                    if beamset.cap_mode == lib3mf::BeamCapMode::Sphere {
                        use std::collections::HashMap;
                        let mut vertex_connections: HashMap<usize, usize> = HashMap::new();
                        
                        for beam in &beamset.beams {
                            *vertex_connections.entry(beam.v1).or_insert(0) += 1;
                            *vertex_connections.entry(beam.v2).or_insert(0) += 1;
                        }

                        // Add spheres at vertices with multiple connections
                        for (vertex_idx, connection_count) in vertex_connections.iter() {
                            if *connection_count >= 2 && *vertex_idx < mesh_data.vertices.len() {
                                let v = &mesh_data.vertices[*vertex_idx];
                                let center = Point3::new(v.x as f32, v.y as f32, v.z as f32);
                                
                                // Use the maximum radius of beams connected to this vertex
                                let max_radius = beamset.beams.iter()
                                    .filter(|b| b.v1 == *vertex_idx || b.v2 == *vertex_idx)
                                    .map(|b| {
                                        if b.v1 == *vertex_idx {
                                            b.r1.unwrap_or(beamset.radius)
                                        } else {
                                            b.r2.unwrap_or(b.r1.unwrap_or(beamset.radius))
                                        }
                                    })
                                    .fold(beamset.radius, f64::max) as f32;

                                let sphere = create_sphere_mesh(center, max_radius, GEOMETRY_SEGMENTS);
                                let mut sphere_node = window.add_trimesh(sphere, IDENTITY_SCALE);
                                sphere_node.set_color(BEAM_COLOR.0, BEAM_COLOR.1, BEAM_COLOR.2);
                                
                                nodes.push(sphere_node);
                            }
                        }
                    }
                }
            }
        }
    }

    nodes
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

/// Draw print area as a wireframe box (12 lines)
fn draw_print_area(window: &mut Window, area: &PrintArea) {
    // Calculate half dimensions for centering at origin
    let half_width = area.width / 2.0;
    let half_depth = area.depth / 2.0;

    // Define 8 corners of the box
    let corners = [
        Point3::new(-half_width, -half_depth, 0.0),           // 0: bottom front left
        Point3::new(half_width, -half_depth, 0.0),            // 1: bottom front right
        Point3::new(half_width, half_depth, 0.0),             // 2: bottom back right
        Point3::new(-half_width, half_depth, 0.0),            // 3: bottom back left
        Point3::new(-half_width, -half_depth, area.height),   // 4: top front left
        Point3::new(half_width, -half_depth, area.height),    // 5: top front right
        Point3::new(half_width, half_depth, area.height),     // 6: top back right
        Point3::new(-half_width, half_depth, area.height),    // 7: top back left
    ];

    // Color for print area - light blue/gray
    let color = Point3::new(0.5, 0.7, 0.9);

    // Draw bottom face (4 lines)
    window.draw_line(&corners[0], &corners[1], &color);
    window.draw_line(&corners[1], &corners[2], &color);
    window.draw_line(&corners[2], &corners[3], &color);
    window.draw_line(&corners[3], &corners[0], &color);

    // Draw top face (4 lines)
    window.draw_line(&corners[4], &corners[5], &color);
    window.draw_line(&corners[5], &corners[6], &color);
    window.draw_line(&corners[6], &corners[7], &color);
    window.draw_line(&corners[7], &corners[4], &color);

    // Draw vertical edges (4 lines)
    window.draw_line(&corners[0], &corners[4], &color);
    window.draw_line(&corners[1], &corners[5], &color);
    window.draw_line(&corners[2], &corners[6], &color);
    window.draw_line(&corners[3], &corners[7], &color);
}

/// Print the menu with current settings
fn print_menu(state: &ViewerState) {
    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!("  Menu - Current Settings");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("  Theme:           {}", state.theme.name());
    println!("  Print Area:      {}", if state.print_area.visible { "ON" } else { "OFF" });
    println!("    Width (X):     {} {}", state.print_area.width, state.print_area.unit);
    println!("    Depth (Y):     {} {}", state.print_area.depth, state.print_area.unit);
    println!("    Height (Z):    {} {}", state.print_area.height, state.print_area.unit);
    if let Some(ref path) = state.file_path {
        println!("  File:            {}", path.file_name().unwrap_or_default().to_string_lossy());
    }
    println!();
    println!("  Press M to hide menu");
    println!("  Press C to configure print area");
    println!("═══════════════════════════════════════════════════════════");
    println!();
}

/// Configure print area dimensions via console input
fn configure_print_area(current: &PrintArea) -> Result<PrintArea, Box<dyn std::error::Error>> {
    use std::io::{self, Write};

    let mut new_area = current.clone();

    // Helper function to read a line
    fn read_line() -> Result<String, io::Error> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }

    // Helper function to read and validate a positive dimension
    fn read_dimension(prompt: &str, current_value: f32, unit: &str) -> Result<f32, io::Error> {
        print!("{} in {} [{}]: ", prompt, unit, current_value);
        io::stdout().flush()?;
        let input = read_line()?;
        if !input.is_empty() {
            if let Ok(val) = input.parse::<f32>() {
                if val > 0.0 {
                    return Ok(val);
                }
            }
        }
        Ok(current_value)
    }

    // Configure width
    new_area.width = read_dimension("Enter width (X)", current.width, &current.unit)?;

    // Configure depth
    new_area.depth = read_dimension("Enter depth (Y)", current.depth, &current.unit)?;

    // Configure height
    new_area.height = read_dimension("Enter height (Z)", current.height, &current.unit)?;

    // Configure unit with validation
    print!("Enter unit (mm/inch/cm) [{}]: ", current.unit);
    io::stdout().flush()?;
    let input = read_line()?;
    if !input.is_empty() {
        // Validate unit against allowed values
        let normalized = input.to_lowercase();
        match normalized.as_str() {
            "mm" | "millimeter" | "millimeters" => new_area.unit = "mm".to_string(),
            "cm" | "centimeter" | "centimeters" => new_area.unit = "cm".to_string(),
            "inch" | "inches" | "in" => new_area.unit = "inch".to_string(),
            "m" | "meter" | "meters" => new_area.unit = "m".to_string(),
            _ => {
                println!("Warning: Unknown unit '{}', keeping '{}'", input, current.unit);
            }
        }
    }

    Ok(new_area)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_area_new() {
        let area = PrintArea::new();
        assert_eq!(area.width, 200.0);
        assert_eq!(area.depth, 200.0);
        assert_eq!(area.height, 200.0);
        assert_eq!(area.unit, "mm");
        assert!(area.visible);
    }

    #[test]
    fn test_print_area_toggle_visibility() {
        let mut area = PrintArea::new();
        assert!(area.visible);
        
        area.toggle_visibility();
        assert!(!area.visible);
        
        area.toggle_visibility();
        assert!(area.visible);
    }

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
