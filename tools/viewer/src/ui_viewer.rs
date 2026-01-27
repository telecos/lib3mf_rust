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

/// Viewer state that can optionally hold a loaded model
struct ViewerState {
    model: Option<Model>,
    file_path: Option<PathBuf>,
    mesh_nodes: Vec<SceneNode>,
}

impl ViewerState {
    /// Create a new empty viewer state
    fn new_empty() -> Self {
        Self {
            model: None,
            file_path: None,
            mesh_nodes: Vec::new(),
        }
    }

    /// Create a viewer state with a loaded model
    fn with_model(model: Model, file_path: PathBuf) -> Self {
        Self {
            model: Some(model),
            file_path: Some(file_path),
            mesh_nodes: Vec::new(),
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
    window.set_framerate_limit(Some(60));

    // Create meshes from the model if one is loaded
    if state.model.is_some() {
        state.mesh_nodes = create_mesh_nodes(&mut window, state.model.as_ref().unwrap());
        print_model_info(state.model.as_ref().unwrap());
    } else {
        print_empty_scene_info();
    }

    print_controls();

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
                                }
                            }
                            Err(e) => {
                                eprintln!("\n✗ Error loading file: {}", e);
                            }
                        }
                    }
                }
                _ => {}
            }
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
    println!("  ⌨️  Ctrl+O             : Open file");
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
    println!();
}

/// Launch the interactive UI viewer
#[allow(dead_code)]
pub fn launch_ui_viewer_legacy(
    _model: Model,
    file_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    launch_ui_viewer(Some(file_path))
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
