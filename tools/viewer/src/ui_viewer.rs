//! Interactive 3D UI Viewer for 3MF files
//!
//! This module provides an interactive 3D viewer using kiss3d
//! for rendering 3MF models with mouse controls and real-time interaction.

#![forbid(unsafe_code)]

use crate::keybindings;
use crate::menu_ui::{MenuAction, MenuBar};
use crate::slice_window::{LineSegment2D, Point2D, SliceConfig, SlicePreviewWindow};
use image::{GenericImageView, Rgb, RgbImage};
use kiss3d::camera::ArcBall;
use kiss3d::event::{Action, Key, WindowEvent};
use kiss3d::light::Light;
use kiss3d::nalgebra::{Point2, Point3, Vector3}; // Use nalgebra from kiss3d
use kiss3d::ncollide3d::procedural::TriMesh;
use kiss3d::scene::SceneNode;
use kiss3d::window::Window;
use lib3mf::Model;
use rfd::FileDialog;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::rc::Rc;

// Constants for beam lattice rendering
const BEAM_COLOR: (f32, f32, f32) = (1.0, 0.6, 0.0); // Orange color for beams
const GEOMETRY_SEGMENTS: u32 = 8; // Number of segments for cylinder/sphere meshes
const IDENTITY_SCALE: Vector3<f32> = Vector3::new(1.0, 1.0, 1.0); // Identity scale for meshes
const CIRCLE_APPROXIMATION_SEGMENTS: u32 = 16; // Number of segments for circle approximations in slices

// Constants for camera controls
const CAMERA_DISTANCE_MULTIPLIER: f32 = 1.5; // Factor for comfortable viewing distance
const ZOOM_STEP: f32 = 0.9; // Zoom in multiplier (0.9 = 10% closer)
const PAN_STEP: f32 = 0.05; // Pan amount as percentage of camera distance

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

/// Boolean operation visualization mode
#[derive(Debug, Clone, Copy, PartialEq)]
enum BooleanMode {
    /// Show all meshes normally
    Normal,
    /// Show boolean operation inputs separately with different colors
    ShowInputs,
    /// Hide non-boolean objects and highlight boolean operands
    HighlightOperands,
}

impl BooleanMode {
    /// Get the next mode in the cycle
    fn next(&self) -> BooleanMode {
        match self {
            BooleanMode::Normal => BooleanMode::ShowInputs,
            BooleanMode::ShowInputs => BooleanMode::HighlightOperands,
            BooleanMode::HighlightOperands => BooleanMode::Normal,
        }
    }

    /// Get the name of the mode for display
    fn name(&self) -> &'static str {
        match self {
            BooleanMode::Normal => "Normal",
            BooleanMode::ShowInputs => "Show Inputs",
            BooleanMode::HighlightOperands => "Highlight Operands",
        }
    }
}

/// Length unit for print bed dimensions
#[derive(Debug, Clone, Copy, PartialEq)]
enum LengthUnit {
    Millimeters,
    Inches,
}

impl LengthUnit {
    /// Get the unit as a string
    fn as_str(&self) -> &'static str {
        match self {
            LengthUnit::Millimeters => "mm",
            LengthUnit::Inches => "inch",
        }
    }
    
    /// Convert from a string to LengthUnit
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "mm" | "millimeter" | "millimeters" => Some(LengthUnit::Millimeters),
            "inch" | "inches" | "in" => Some(LengthUnit::Inches),
            _ => None,
        }
    }
    
    /// Convert a value from this unit to millimeters
    fn to_mm(self, value: f32) -> f32 {
        match self {
            LengthUnit::Millimeters => value,
            LengthUnit::Inches => value * 25.4,
        }
    }
    
    /// Convert a value from millimeters to this unit
    #[allow(clippy::wrong_self_convention)]
    fn from_mm(self, value: f32) -> f32 {
        match self {
            LengthUnit::Millimeters => value,
            LengthUnit::Inches => value / 25.4,
        }
    }
}

/// Origin mode for the print bed
#[derive(Debug, Clone, Copy, PartialEq)]
enum Origin {
    /// Origin at (0, 0, 0) corner
    Corner,
    /// Origin at center of bed, Z=0
    CenterBottom,
}

impl Origin {
    /// Get the origin mode as a string
    fn as_str(&self) -> &'static str {
        match self {
            Origin::Corner => "Corner",
            Origin::CenterBottom => "Center",
        }
    }
}

/// Print area configuration for build volume visualization
#[derive(Debug, Clone)]
struct PrintArea {
    width: f32,       // X dimension
    depth: f32,       // Y dimension
    height: f32,      // Z dimension
    unit: LengthUnit,
    origin: Origin,
    visible: bool,
    show_ruler: bool,
    show_scale_bar: bool,
}

impl PrintArea {
    /// Create a new print area with default settings
    fn new() -> Self {
        Self {
            width: 200.0,
            depth: 200.0,
            height: 200.0,
            unit: LengthUnit::Millimeters,
            origin: Origin::Corner,
            visible: true,
            show_ruler: false,
            show_scale_bar: false,
        }
    }

    /// Toggle visibility of the print area
    fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }
    
    /// Toggle visibility of the ruler
    fn toggle_ruler(&mut self) {
        self.show_ruler = !self.show_ruler;
    }
    
    /// Toggle visibility of the scale bar
    fn toggle_scale_bar(&mut self) {
        self.show_scale_bar = !self.show_scale_bar;
    }
    
    /// Get width in millimeters (for internal calculations)
    fn width_mm(&self) -> f32 {
        self.unit.to_mm(self.width)
    }
    
    /// Get depth in millimeters (for internal calculations)
    fn depth_mm(&self) -> f32 {
        self.unit.to_mm(self.depth)
    }
    
    /// Get height in millimeters (for internal calculations)
    fn height_mm(&self) -> f32 {
        self.unit.to_mm(self.height)
    }
}

/// Printer preset configuration
struct PrinterPreset {
    name: &'static str,
    width: f32,
    depth: f32,
    height: f32,
}

/// Get list of printer presets
fn printer_presets() -> Vec<PrinterPreset> {
    vec![
        PrinterPreset {
            name: "Prusa MK3S+",
            width: 250.0,
            depth: 210.0,
            height: 210.0,
        },
        PrinterPreset {
            name: "Ender 3",
            width: 220.0,
            depth: 220.0,
            height: 250.0,
        },
        PrinterPreset {
            name: "Bambu Lab X1",
            width: 256.0,
            depth: 256.0,
            height: 256.0,
        },
        PrinterPreset {
            name: "Creality CR-10",
            width: 300.0,
            depth: 300.0,
            height: 400.0,
        },
        PrinterPreset {
            name: "Custom",
            width: 200.0,
            depth: 200.0,
            height: 200.0,
        },
    ]
}

/// Slice view state
#[derive(Debug, Clone)]
struct SliceView {
    /// Current Z height for the slice plane
    z_height: f32,
    /// Minimum Z bound of the model
    min_z: f32,
    /// Maximum Z bound of the model
    max_z: f32,
    /// Whether the slice view is visible
    visible: bool,
    /// Whether to show the slice plane in 3D view
    show_plane: bool,
    /// Computed contour line segments at current Z height (using slice_window types)
    contours: Vec<LineSegment2D>,
    /// Slice stack mode
    use_slice_stack: bool,
    /// Current slice index when using slice stack
    current_slice_index: usize,
    /// Total number of slices in the stack
    total_slices: usize,
    /// Animation state
    animation_playing: bool,
    /// Animation speed (slices per second)
    animation_speed: f32,
    /// Animation time accumulator
    animation_time: f32,
    /// Loop animation
    animation_loop: bool,
    /// Spread factor for 3D visualization (1.0 = actual positions, >1.0 = spread apart)
    spread_factor: f32,
    /// Show all slices in 3D stack
    show_stack_3d: bool,
    /// Render mode: true = filled, false = outline
    filled_mode: bool,
    /// Show grid in slice preview window
    show_grid: bool,
}

impl SliceView {
    /// Create a new slice view with default settings
    fn new() -> Self {
        Self {
            z_height: 0.0,
            min_z: 0.0,
            max_z: 100.0,
            visible: false,
            show_plane: true,
            contours: Vec::new(),
            use_slice_stack: false,
            current_slice_index: 0,
            total_slices: 0,
            animation_playing: false,
            animation_speed: 2.0, // 2 slices per second
            animation_time: 0.0,
            animation_loop: true,
            spread_factor: 1.0,
            show_stack_3d: false,
            filled_mode: false,
            show_grid: true,
        }
    }

    /// Initialize slice view with model bounds
    fn initialize_from_model(&mut self, model: &Model) {
        let (min_bound, max_bound) = calculate_model_bounds(model);
        self.min_z = min_bound.2;
        self.max_z = max_bound.2;
        // Start at middle of the model
        self.z_height = (self.min_z + self.max_z) / 2.0;
        
        // Check if model has slice stacks
        if !model.resources.slice_stacks.is_empty() {
            self.total_slices = model.resources.slice_stacks[0].slices.len();
            if self.total_slices > 0 {
                self.current_slice_index = 0;
            }
        }
    }

    /// Move the slice plane up or down
    fn adjust_z(&mut self, delta: f32) {
        self.z_height = (self.z_height + delta).clamp(self.min_z, self.max_z);
    }

    /// Toggle visibility of the slice view
    fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    /// Toggle visibility of the slice plane in 3D view
    fn toggle_plane(&mut self) {
        self.show_plane = !self.show_plane;
    }
    
    /// Navigate to next slice in stack
    fn next_slice(&mut self) {
        if self.total_slices > 0 {
            self.current_slice_index = (self.current_slice_index + 1).min(self.total_slices - 1);
        }
    }
    
    /// Navigate to previous slice in stack
    fn prev_slice(&mut self) {
        if self.current_slice_index > 0 {
            self.current_slice_index -= 1;
        }
    }
    
    /// Jump to first slice
    fn first_slice(&mut self) {
        self.current_slice_index = 0;
    }
    
    /// Jump to last slice
    fn last_slice(&mut self) {
        if self.total_slices > 0 {
            self.current_slice_index = self.total_slices - 1;
        }
    }
    
    /// Toggle animation
    fn toggle_animation(&mut self) {
        self.animation_playing = !self.animation_playing;
        self.animation_time = 0.0;
    }
    
    /// Update animation state
    fn update_animation(&mut self, delta_time: f32) {
        if self.animation_playing && self.total_slices > 0 {
            self.animation_time += delta_time;
            let slices_elapsed = (self.animation_time * self.animation_speed) as usize;
            
            if slices_elapsed > 0 {
                // Preserve fractional part for more accurate timing
                self.animation_time -= slices_elapsed as f32 / self.animation_speed;
                self.current_slice_index += slices_elapsed;
                
                if self.current_slice_index >= self.total_slices {
                    if self.animation_loop {
                        self.current_slice_index %= self.total_slices;
                    } else {
                        self.current_slice_index = self.total_slices - 1;
                        self.animation_playing = false;
                    }
                }
            }
        }
    }
    
    /// Increase animation speed
    fn increase_speed(&mut self) {
        self.animation_speed = (self.animation_speed * 1.5).min(20.0);
    }
    
    /// Decrease animation speed
    fn decrease_speed(&mut self) {
        self.animation_speed = (self.animation_speed / 1.5).max(0.1);
    }
    
    /// Increase spread factor
    fn increase_spread(&mut self) {
        self.spread_factor = (self.spread_factor + 0.2).min(5.0);
    }
    
    /// Decrease spread factor
    fn decrease_spread(&mut self) {
        self.spread_factor = (self.spread_factor - 0.2).max(1.0);
    }
    
    /// Toggle 3D stack visualization
    fn toggle_stack_3d(&mut self) {
        self.show_stack_3d = !self.show_stack_3d;
    }
    
    /// Toggle filled/outline mode
    fn toggle_filled_mode(&mut self) {
        self.filled_mode = !self.filled_mode;
    }
    
    /// Toggle slice stack mode
    fn toggle_slice_stack_mode(&mut self) {
        self.use_slice_stack = !self.use_slice_stack;
    }
}

/// Model information panel state
struct ModelInfoPanel {
    visible: bool,
}

impl ModelInfoPanel {
    /// Create a new model info panel (hidden by default)
    fn new() -> Self {
        Self { visible: false }
    }

    /// Toggle panel visibility
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
    boolean_mode: BooleanMode,
    print_area: PrintArea,
    slice_view: SliceView,
    show_displacement: bool,
    show_materials: bool,
    show_drop_zone: bool,
    drop_file_valid: bool,
    hovered_file_path: Option<String>,
    slice_preview_window: Option<SlicePreviewWindow>,
    info_panel: ModelInfoPanel,
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
            boolean_mode: BooleanMode::Normal,
            print_area: PrintArea::new(),
            slice_view: SliceView::new(),
            show_displacement: false,
            show_materials: true,
            show_drop_zone: false,
            drop_file_valid: false,
            hovered_file_path: None,
            slice_preview_window: None,
            info_panel: ModelInfoPanel::new(),
        }
    }

    /// Create a viewer state with a loaded model
    fn with_model(model: Model, file_path: PathBuf) -> Self {
        let mut slice_view = SliceView::new();
        slice_view.initialize_from_model(&model);
        
        Self {
            model: Some(model),
            file_path: Some(file_path),
            mesh_nodes: Vec::new(),
            beam_nodes: Vec::new(),
            show_beams: true,
            theme: Theme::Dark,
            boolean_mode: BooleanMode::Normal,
            print_area: PrintArea::new(),
            slice_view,
            show_displacement: false,
            show_materials: true,
            show_drop_zone: false,
            drop_file_valid: false,
            hovered_file_path: None,
            slice_preview_window: None,
            info_panel: ModelInfoPanel::new(),
        }
    }

    /// Load a file into the viewer state
    fn load_file(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(&path)?;
        let model = Model::from_reader(file)?;
        
        // Reinitialize slice view for new model
        self.slice_view.initialize_from_model(&model);
        
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

    /// Toggle slice preview window
    fn toggle_slice_preview_window(&mut self) {
        if self.slice_preview_window.is_some() {
            // Close existing window
            self.slice_preview_window = None;
            println!("Slice preview window closed");
        } else {
            // Create new window
            match SlicePreviewWindow::new() {
                Ok(mut window) => {
                    // Initialize with current model bounds if available
                    if let Some(ref model) = self.model {
                        let (min_bound, max_bound) = calculate_model_bounds(model);
                        window.set_model_bounds((min_bound.0, min_bound.1), (max_bound.0, max_bound.1));
                        
                        // Sync initial configuration
                        self.sync_slice_preview_window(&mut window);
                    }
                    self.slice_preview_window = Some(window);
                    println!("Slice preview window opened");
                    println!("  Controls: Up/Down arrows, PageUp/PageDown to adjust Z");
                    println!("  Press 'G' to toggle grid, 'F' to toggle filled mode");
                }
                Err(e) => {
                    eprintln!("Failed to create slice preview window: {}", e);
                }
            }
        }
    }

    /// Sync slice preview window with current slice view state
    fn sync_slice_preview_window(&mut self, window: &mut SlicePreviewWindow) {
        // Simply clone the contours vector (already correct type from slice_window)
        let contours = self.slice_view.contours.clone();

        let config = SliceConfig {
            z_height: self.slice_view.z_height,
            min_z: self.slice_view.min_z,
            max_z: self.slice_view.max_z,
            filled_mode: self.slice_view.filled_mode,
            show_grid: self.slice_view.show_grid,
            contours,
        };

        window.update_config(config);
    }

    /// Update slice preview window (call in main loop)
    fn update_slice_preview_window(&mut self) -> bool {
        // Extract the window temporarily to avoid borrow checker issues
        let mut window = match self.slice_preview_window.take() {
            Some(w) => w,
            None => return true,
        };
        
        // Check if Z height changed in the preview window
        let preview_z = window.get_z_height();
        if (preview_z - self.slice_view.z_height).abs() > 0.001 {
            // Update main slice view to match preview window
            self.slice_view.z_height = preview_z;
            
            // Recompute contours
            if let Some(ref model) = self.model {
                self.slice_view.contours = compute_slice_contours(model, self.slice_view.z_height);
            }
        }
        
        // Sync grid state from window to view state
        self.slice_view.show_grid = window.get_show_grid();
        
        // Sync current state to preview window
        self.sync_slice_preview_window(&mut window);
        
        // Update the preview window
        let should_continue = window.update();
        
        // Put window back
        self.slice_preview_window = Some(window);
        
        should_continue
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

    // Create a custom ArcBall camera for full control
    let mut camera = create_camera_for_model(state.model.as_ref());
    window.set_framerate_limit(Some(60));

    // Set initial background color based on theme
    let bg_color = state.theme.background_color();
    window.set_background_color(bg_color.0, bg_color.1, bg_color.2);

    // Create meshes from the model if one is loaded
    if state.model.is_some() {
        state.mesh_nodes = create_mesh_nodes_with_materials(
            &mut window,
            state.model.as_ref().unwrap(),
            state.show_materials,
            state.boolean_mode,
            state.show_displacement,
            state.file_path.as_ref(),
        );
        state.beam_nodes = create_beam_lattice_nodes(&mut window, state.model.as_ref().unwrap());
        print_model_info(state.model.as_ref().unwrap());
    } else {
        print_empty_scene_info();
    }

    print_controls();

    // Track axis visualization state (default: visible)
    let mut show_axes = true;

    // Initialize menu bar
    let mut menu_bar = MenuBar::new();

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
    while window.render_with_camera(&mut camera) {
        // Update menu bar dimensions
        let size = window.size();
        menu_bar.update_dimensions(size.x, size.y);
        
        // Handle window events
        for event in window.events().iter() {
            // Let menu bar handle event first
            if let Some(action) = menu_bar.handle_event(&event.value) {
                handle_menu_action(
                    action,
                    &mut window,
                    &mut state,
                    &mut camera,
                    &mut show_axes,
                    &mut axis_length,
                    &mut menu_bar,
                );
            }
            
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
                                    state.mesh_nodes = create_mesh_nodes_with_displacement(
                                        &mut window,
                                        model,
                                        state.boolean_mode,
                                        state.show_displacement,
                                        state.file_path.as_ref(),
                                    );
                                    state.beam_nodes =
                                        create_beam_lattice_nodes(&mut window, model);
                                    window.set_title(&state.window_title());
                                    println!("\n✓ File loaded successfully!");
                                    print_model_info(model);

                                    // Reset camera to fit new model
                                    camera = create_camera_for_model(state.model.as_ref());

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
                                    state.mesh_nodes = create_mesh_nodes_with_displacement(
                                        &mut window,
                                        model,
                                        state.boolean_mode,
                                        state.show_displacement,
                                        state.file_path.as_ref(),
                                    );
                                    state.beam_nodes =
                                        create_beam_lattice_nodes(&mut window, model);
                                    window.set_title(&state.window_title());
                                    println!("\n✓ File loaded successfully!");
                                    print_model_info(model);
                                    
                                    // Reset camera to fit new model
                                    camera = create_camera_for_model(state.model.as_ref());
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
                        if state.show_beams {
                            "visible"
                        } else {
                            "hidden"
                        }
                    );
                }
                WindowEvent::Key(Key::V, Action::Press, _) => {
                    // V: Cycle boolean operation visualization mode
                    state.boolean_mode = state.boolean_mode.next();
                    println!("\nBoolean mode: {}", state.boolean_mode.name());

                    // Recreate mesh nodes with new coloring
                    if let Some(ref model) = state.model {
                        // Hide existing mesh nodes
                        for node in &mut state.mesh_nodes {
                            node.set_visible(false);
                        }
                        state.mesh_nodes.clear();

                        // Create new mesh nodes with boolean-aware coloring
                        state.mesh_nodes = create_mesh_nodes_with_materials(
                            &mut window,
                            model,
                            state.show_materials,
                            state.boolean_mode,
                            state.show_displacement,
                            state.file_path.as_ref(),
                        );

                        // Print boolean operation information if in special mode
                        if state.boolean_mode != BooleanMode::Normal {
                            print_boolean_info(model);
                        }
                    }
                }
                WindowEvent::Key(Key::T, Action::Press, _) => {
                    // T: Cycle through themes
                    state.cycle_theme(&mut window);
                }
                WindowEvent::Key(Key::A, Action::Release, _) => {
                    // A key: Toggle XYZ axes
                    show_axes = !show_axes;
                    menu_bar.set_checked(MenuAction::ToggleAxes, show_axes);
                    println!("XYZ Axes: {}", if show_axes { "ON" } else { "OFF" });
                }
                WindowEvent::Key(Key::I, Action::Release, _) => {
                    // I key: Toggle model information panel
                    state.info_panel.toggle_visibility();
                    menu_bar.set_checked(MenuAction::ToggleModelInfo, state.info_panel.visible);
                    println!(
                        "Model Information Panel: {}",
                        if state.info_panel.visible { "ON" } else { "OFF" }
                    );
                }
                WindowEvent::Key(Key::S, Action::Release, _) => {
                    // S key: Toggle slice stack mode OR capture screenshot
                    // Priority: Slice stack mode when slice view is visible and has slice stacks
                    if state.slice_view.visible && state.model.is_some() {
                        if let Some(ref model) = state.model {
                            if !model.resources.slice_stacks.is_empty() {
                                // Toggle slice stack mode
                                state.slice_view.toggle_slice_stack_mode();
                                println!(
                                    "\nSlice Stack Mode: {}",
                                    if state.slice_view.use_slice_stack {
                                        "ON (use Up/Down to navigate)"
                                    } else {
                                        "OFF (use Shift+Up/Down for Z height)"
                                    }
                                );
                            } else {
                                // No slice stacks - capture screenshot
                                if let Err(e) = capture_screenshot(&window) {
                                    eprintln!("\n✗ Error capturing screenshot: {}", e);
                                }
                            }
                        }
                    } else {
                        // S key: Capture screenshot (when slice view not visible or no model)
                        if let Err(e) = capture_screenshot(&window) {
                            eprintln!("\n✗ Error capturing screenshot: {}", e);
                        }
                    }
                }
                WindowEvent::Key(Key::M, Action::Release, _) => {
                    // M key: Toggle menu bar visibility
                    menu_bar.toggle_visibility();
                    println!(
                        "Menu Bar: {}",
                        if menu_bar.visible { "ON" } else { "OFF" }
                    );
                }
                WindowEvent::Key(Key::R, Action::Release, _) => {
                    // R key: Toggle material rendering
                    state.show_materials = !state.show_materials;
                    println!(
                        "\nMaterial Rendering: {}",
                        if state.show_materials { "ON" } else { "OFF (default gray)" }
                    );
                    
                    // Recreate mesh nodes with new coloring
                    if let Some(ref model) = state.model {
                        // Remove existing mesh nodes
                        for node in &mut state.mesh_nodes {
                            window.remove_node(node);
                        }
                        state.mesh_nodes.clear();
                        
                        // Create new mesh nodes with appropriate coloring
                        state.mesh_nodes = create_mesh_nodes_with_materials(
                            &mut window,
                            model,
                            state.show_materials,
                            state.boolean_mode,
                            state.show_displacement,
                            state.file_path.as_ref(),
                        );
                    }
                }
                WindowEvent::Key(Key::P, Action::Release, _) => {
                    // P key: Toggle print area visibility
                    state.print_area.toggle_visibility();
                    println!(
                        "Print Area: {}",
                        if state.print_area.visible {
                            "ON"
                        } else {
                            "OFF"
                        }
                    );
                }
                WindowEvent::Key(Key::C, Action::Release, _) => {
                    // C key: Configure print area
                    println!("\n═══════════════════════════════════════════════════════════");
                    println!("  Configure Print Area");
                    println!("═══════════════════════════════════════════════════════════");
                    println!();
                    println!("Current settings:");
                    println!(
                        "  Width (X):  {} {}",
                        state.print_area.width, state.print_area.unit.as_str()
                    );
                    println!(
                        "  Depth (Y):  {} {}",
                        state.print_area.depth, state.print_area.unit.as_str()
                    );
                    println!(
                        "  Height (Z): {} {}",
                        state.print_area.height, state.print_area.unit.as_str()
                    );
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
                        println!(
                            "  Width (X):  {} {}",
                            state.print_area.width, state.print_area.unit.as_str()
                        );
                        println!(
                            "  Depth (Y):  {} {}",
                            state.print_area.depth, state.print_area.unit.as_str()
                        );
                        println!(
                            "  Height (Z): {} {}",
                            state.print_area.height, state.print_area.unit.as_str()
                        );
                    }
                }
                WindowEvent::Key(Key::U, Action::Release, _) => {
                    // U key: Toggle ruler visibility
                    state.print_area.toggle_ruler();
                    println!(
                        "Ruler: {}",
                        if state.print_area.show_ruler {
                            "ON"
                        } else {
                            "OFF"
                        }
                    );
                }
                WindowEvent::Key(Key::J, Action::Release, _) => {
                    // J key: Toggle scale bar visibility
                    state.print_area.toggle_scale_bar();
                    println!(
                        "Scale Bar: {}",
                        if state.print_area.show_scale_bar {
                            "ON"
                        } else {
                            "OFF"
                        }
                    );
                }
                WindowEvent::Key(Key::D, Action::Press, _) => {
                    // D key: Toggle displacement visualization
                    if let Some(ref model) = state.model {
                        if has_displacement_data(model) {
                            state.show_displacement = !state.show_displacement;
                            println!(
                                "\nDisplacement Visualization: {}",
                                if state.show_displacement {
                                    "ON"
                                } else {
                                    "OFF"
                                }
                            );
                            
                            // Print displacement info when enabling
                            if state.show_displacement {
                                let (maps, norm_groups, disp_groups) = count_displacement_resources(model);
                                let disp_objects = count_displacement_objects(model);
                                println!("  Displacement Maps: {}", maps);
                                println!("  Normal Vector Groups: {}", norm_groups);
                                println!("  Displacement Groups: {}", disp_groups);
                                println!("  Objects with Displacement: {}", disp_objects);
                            }
                            
                            // Recreate mesh nodes with displacement highlighting
                            state.mesh_nodes.clear();
                            
                            state.mesh_nodes = create_mesh_nodes_with_materials(
                                &mut window,
                                model,
                                state.show_materials,
                                state.boolean_mode,
                                state.show_displacement,
                                state.file_path.as_ref(),
                            );
                        } else {
                            println!("\nNo displacement data in this model");
                        }
                    }
                }
                WindowEvent::Key(Key::Z, Action::Release, _) => {
                    // Z key: Toggle slice view
                    if state.model.is_some() {
                        state.slice_view.toggle_visibility();
                        if state.slice_view.visible {
                            // Compute contours when enabling
                            if let Some(ref model) = state.model {
                                // Check for slice stacks
                                if !model.resources.slice_stacks.is_empty() {
                                    // Enable slice stack mode automatically
                                    state.slice_view.use_slice_stack = true;
                                    
                                    let stack = &model.resources.slice_stacks[0];
                                    println!("\n✓ Slice Stack Detected!");
                                    println!("  Total Slices: {}", stack.slices.len());
                                    println!("  Z Bottom: {:.3} {}", stack.zbottom, model.unit);
                                    if !stack.slices.is_empty() {
                                        let z_range = stack.slices.last().unwrap().ztop - stack.zbottom;
                                        let avg_spacing = if stack.slices.len() > 1 {
                                            z_range / (stack.slices.len() - 1) as f64
                                        } else {
                                            0.0
                                        };
                                        println!("  Z Top: {:.3} {}", stack.slices.last().unwrap().ztop, model.unit);
                                        println!("  Z Range: {:.3} {}", z_range, model.unit);
                                        println!("  Average Layer Height: {:.3} {}", avg_spacing, model.unit);
                                    }
                                    println!("\nSlice Stack Mode: ON");
                                    println!("  Current Slice: {} / {}", state.slice_view.current_slice_index + 1, state.slice_view.total_slices);
                                    println!("\nControls:");
                                    println!("  Up/Down arrows     - Navigate slices");
                                    println!("  Home/End           - Jump to first/last slice");
                                    println!("  Space              - Play/pause animation");
                                    println!("  [ / ]              - Adjust animation speed");
                                    println!("  Shift+Up/Down      - Adjust spread factor (in 3D mode)");
                                    println!("  S                  - Toggle slice stack mode");
                                    println!("  K                  - Toggle 3D stack visualization");
                                    println!("  N                  - Toggle filled/outline mode");
                                    println!("  L                  - Toggle slice plane visibility");
                                    println!("  X                  - Export slice to PNG");
                                } else {
                                    state.slice_view.contours = compute_slice_contours(model, state.slice_view.z_height);
                                    println!(
                                        "\nSlice View: ON at Z = {:.2} {} ({} segments)",
                                        state.slice_view.z_height,
                                        model.unit,
                                        state.slice_view.contours.len()
                                    );
                                    println!("  Z range: {:.2} to {:.2} {}", 
                                        state.slice_view.min_z, 
                                        state.slice_view.max_z,
                                        model.unit
                                    );
                                    println!("  Use Up/Down arrows (with Shift) to adjust Z height");
                                    println!("  Use X to export slice to PNG");
                                    println!("  Use L to toggle slice plane visibility");
                                }
                            }
                        } else {
                            println!("\nSlice View: OFF");
                        }
                    }
                }
                WindowEvent::Key(Key::Up, Action::Press, modifiers) => {
                    if state.slice_view.visible && state.model.is_some() {
                        if modifiers.contains(kiss3d::event::Modifiers::Shift) {
                            // Shift+Up: Adjust spread factor or Z height
                            if state.slice_view.use_slice_stack && state.slice_view.show_stack_3d {
                                state.slice_view.increase_spread();
                                println!("Spread factor: {:.1}x", state.slice_view.spread_factor);
                            } else {
                                // Increase Z height
                                let delta = (state.slice_view.max_z - state.slice_view.min_z) * 0.02; // 2% of range
                                state.slice_view.adjust_z(delta);
                                
                                // Recompute contours
                                if let Some(ref model) = state.model {
                                    state.slice_view.contours = compute_slice_contours(model, state.slice_view.z_height);
                                    println!(
                                        "Slice Z: {:.2} {} ({} segments)",
                                        state.slice_view.z_height,
                                        model.unit,
                                        state.slice_view.contours.len()
                                    );
                                }
                            }
                        } else {
                            // Up without modifier: Navigate to next slice in stack
                            if state.slice_view.use_slice_stack && state.slice_view.total_slices > 0 {
                                state.slice_view.next_slice();
                                if let Some(ref model) = state.model {
                                    if !model.resources.slice_stacks.is_empty() {
                                        let stack = &model.resources.slice_stacks[0];
                                        if state.slice_view.current_slice_index < stack.slices.len() {
                                            let slice = &stack.slices[state.slice_view.current_slice_index];
                                            println!(
                                                "Slice {} / {} - Z: {:.3} {} ({} vertices, {} polygons)",
                                                state.slice_view.current_slice_index + 1,
                                                state.slice_view.total_slices,
                                                slice.ztop,
                                                model.unit,
                                                slice.vertices.len(),
                                                slice.polygons.len()
                                            );
                                        }
                                    }
                                }
                            } else {
                                // Pan camera up when slice view not in stack mode
                                pan_camera(&mut camera, 0.0, 0.0, 1.0);
                            }
                        }
                    } else if !modifiers.contains(kiss3d::event::Modifiers::Shift) {
                        // Pan camera up when slice view not visible
                        pan_camera(&mut camera, 0.0, 0.0, 1.0);
                    }
                }
                WindowEvent::Key(Key::Down, Action::Press, modifiers) => {
                    if state.slice_view.visible && state.model.is_some() {
                        if modifiers.contains(kiss3d::event::Modifiers::Shift) {
                            // Shift+Down: Adjust spread factor or Z height
                            if state.slice_view.use_slice_stack && state.slice_view.show_stack_3d {
                                state.slice_view.decrease_spread();
                                println!("Spread factor: {:.1}x", state.slice_view.spread_factor);
                            } else {
                                // Decrease Z height
                                let delta = -(state.slice_view.max_z - state.slice_view.min_z) * 0.02; // 2% of range
                                state.slice_view.adjust_z(delta);
                                
                                // Recompute contours
                                if let Some(ref model) = state.model {
                                    state.slice_view.contours = compute_slice_contours(model, state.slice_view.z_height);
                                    println!(
                                        "Slice Z: {:.2} {} ({} segments)",
                                        state.slice_view.z_height,
                                        model.unit,
                                        state.slice_view.contours.len()
                                    );
                                }
                            }
                        } else {
                            // Down without modifier: Navigate to previous slice in stack
                            if state.slice_view.use_slice_stack && state.slice_view.total_slices > 0 {
                                state.slice_view.prev_slice();
                                if let Some(ref model) = state.model {
                                    if !model.resources.slice_stacks.is_empty() {
                                        let stack = &model.resources.slice_stacks[0];
                                        if state.slice_view.current_slice_index < stack.slices.len() {
                                            let slice = &stack.slices[state.slice_view.current_slice_index];
                                            println!(
                                                "Slice {} / {} - Z: {:.3} {} ({} vertices, {} polygons)",
                                                state.slice_view.current_slice_index + 1,
                                                state.slice_view.total_slices,
                                                slice.ztop,
                                                model.unit,
                                                slice.vertices.len(),
                                                slice.polygons.len()
                                            );
                                        }
                                    }
                                }
                            } else {
                                // Pan camera down when slice view not in stack mode
                                pan_camera(&mut camera, 0.0, 0.0, -1.0);
                            }
                        }
                    } else if !modifiers.contains(kiss3d::event::Modifiers::Shift) {
                        // Pan camera down when slice view not visible
                        pan_camera(&mut camera, 0.0, 0.0, -1.0);
                    }
                }
                WindowEvent::Key(Key::L, Action::Release, _) => {
                    // L key: Toggle slice plane visibility
                    if state.slice_view.visible {
                        state.slice_view.toggle_plane();
                        println!(
                            "Slice Plane: {}",
                            if state.slice_view.show_plane {
                                "ON"
                            } else {
                                "OFF"
                            }
                        );
                    }
                }
                WindowEvent::Key(Key::X, Action::Release, _) => {
                    // X key: Export slice to PNG
                    if state.slice_view.visible && state.model.is_some() {
                        if let Some(ref model) = state.model {
                            let bounds = calculate_model_bounds(model);
                            if let Err(e) = export_slice_to_png(&state.slice_view, bounds, &model.unit) {
                                eprintln!("\n✗ Error exporting slice: {}", e);
                            }
                        }
                    } else if !state.slice_view.visible {
                        println!("\nSlice view is not enabled. Press Z to enable it first.");
                    }
                }
                WindowEvent::Key(Key::K, Action::Release, _) => {
                    // K key: Toggle 3D stack visualization
                    if state.slice_view.visible && state.slice_view.use_slice_stack {
                        state.slice_view.toggle_stack_3d();
                        println!(
                            "\n3D Stack Visualization: {}",
                            if state.slice_view.show_stack_3d {
                                "ON"
                            } else {
                                "OFF"
                            }
                        );
                        if state.slice_view.show_stack_3d {
                            println!("  Use Shift+Up/Down to adjust spread factor");
                            println!("  Current spread: {:.1}x", state.slice_view.spread_factor);
                        }
                    }
                }
                WindowEvent::Key(Key::Space, Action::Release, _) => {
                    // Space: Play/pause slice animation
                    if state.slice_view.visible && state.slice_view.use_slice_stack {
                        state.slice_view.toggle_animation();
                        println!(
                            "\nSlice Animation: {}",
                            if state.slice_view.animation_playing {
                                "PLAYING"
                            } else {
                                "PAUSED"
                            }
                        );
                        if state.slice_view.animation_playing {
                            println!("  Speed: {:.1} slices/sec", state.slice_view.animation_speed);
                            println!("  Loop: {}", if state.slice_view.animation_loop { "ON" } else { "OFF" });
                        }
                    }
                }
                WindowEvent::Key(Key::Home, Action::Release, _) if state.slice_view.visible && state.slice_view.use_slice_stack => {
                    // Home key: Jump to first slice (when slice stack mode active)
                    state.slice_view.first_slice();
                    if let Some(ref model) = state.model {
                        if !model.resources.slice_stacks.is_empty() {
                            let stack = &model.resources.slice_stacks[0];
                            if !stack.slices.is_empty() {
                                let slice = &stack.slices[0];
                                println!(
                                    "First Slice - Z: {:.3} {} ({} vertices, {} polygons)",
                                    slice.ztop,
                                    model.unit,
                                    slice.vertices.len(),
                                    slice.polygons.len()
                                );
                            }
                        }
                    }
                }
                WindowEvent::Key(Key::End, Action::Release, _) if state.slice_view.visible && state.slice_view.use_slice_stack => {
                    // End key: Jump to last slice (when slice stack mode active)
                    state.slice_view.last_slice();
                    if let Some(ref model) = state.model {
                        if !model.resources.slice_stacks.is_empty() {
                            let stack = &model.resources.slice_stacks[0];
                            if state.slice_view.current_slice_index < stack.slices.len() {
                                let slice = &stack.slices[state.slice_view.current_slice_index];
                                println!(
                                    "Last Slice - Z: {:.3} {} ({} vertices, {} polygons)",
                                    slice.ztop,
                                    model.unit,
                                    slice.vertices.len(),
                                    slice.polygons.len()
                                );
                            }
                        }
                    }
                }
                WindowEvent::Key(Key::RBracket, Action::Release, _) => {
                    // ]: Increase animation speed
                    if state.slice_view.visible && state.slice_view.use_slice_stack {
                        state.slice_view.increase_speed();
                        println!("Animation speed: {:.1} slices/sec", state.slice_view.animation_speed);
                    }
                }
                WindowEvent::Key(Key::LBracket, Action::Release, _) => {
                    // [: Decrease animation speed
                    if state.slice_view.visible && state.slice_view.use_slice_stack {
                        state.slice_view.decrease_speed();
                        println!("Animation speed: {:.1} slices/sec", state.slice_view.animation_speed);
                    }
                }
                WindowEvent::Key(Key::N, Action::Release, _) => {
                    // N key: Toggle filled/outline mode for slices
                    if state.slice_view.visible && state.slice_view.use_slice_stack {
                        state.slice_view.toggle_filled_mode();
                        println!(
                            "\nSlice Render Mode: {}",
                            if state.slice_view.filled_mode {
                                "FILLED"
                            } else {
                                "OUTLINE"
                            }
                        );
                    }
                }
                WindowEvent::Key(Key::Equals, Action::Press, _) 
                    | WindowEvent::Key(Key::Add, Action::Press, _) 
                    | WindowEvent::Key(Key::PageUp, Action::Press, _) => {
                    // + key or PageUp: Zoom in
                    zoom_camera(&mut camera, ZOOM_STEP);
                    println!("Zoom in (distance: {:.1})", camera.dist());
                }
                WindowEvent::Key(Key::Minus, Action::Press, _) 
                    | WindowEvent::Key(Key::Subtract, Action::Press, _)
                    | WindowEvent::Key(Key::PageDown, Action::Press, _) => {
                    // - key or PageDown: Zoom out
                    zoom_camera(&mut camera, 1.0 / ZOOM_STEP);
                    println!("Zoom out (distance: {:.1})", camera.dist());
                }
                WindowEvent::Key(Key::F, Action::Release, _) => {
                    // F key: Fit model to view
                    if let Some(ref model) = state.model {
                        fit_camera_to_model(&mut camera, model);
                        println!("Camera fit to model");
                    }
                }
                WindowEvent::Key(Key::Home, Action::Release, _) => {
                    // Home key: Reset camera to default position
                    camera = create_camera_for_model(state.model.as_ref());
                    println!("Camera reset to default view");
                }
                WindowEvent::Key(Key::Left, Action::Press, _) => {
                    // Left arrow: Pan view left
                    pan_camera(&mut camera, -1.0, 0.0, 0.0);
                }
                WindowEvent::Key(Key::Right, Action::Press, _) => {
                    // Right arrow: Pan view right
                    pan_camera(&mut camera, 1.0, 0.0, 0.0);
                }
                WindowEvent::Key(Key::W, Action::Release, _) => {
                    // W key: Toggle slice preview window
                    state.toggle_slice_preview_window();
                }
                WindowEvent::Key(Key::H, Action::Release, _) => {
                    // H key: Show help
                    println!();
                    keybindings::print_help();
                    println!();
                }
                WindowEvent::Key(Key::Slash, Action::Release, modifiers)
                    if modifiers.contains(kiss3d::event::Modifiers::Shift) =>
                {
                    // ? key (Shift+/): Show help
                    println!();
                    keybindings::print_help();
                    println!();
                }
                WindowEvent::HoveredFile(ref path_str) => {
                    // File is being dragged over the window
                    state.show_drop_zone = true;
                    state.hovered_file_path = Some(path_str.clone());
                    // Check if it's a .3mf file
                    state.drop_file_valid = path_str.to_lowercase().ends_with(".3mf");
                }
                WindowEvent::HoveredFileCancelled => {
                    // File drag was cancelled
                    state.show_drop_zone = false;
                    state.drop_file_valid = false;
                    state.hovered_file_path = None;
                }
                WindowEvent::DroppedFile(ref path_str) => {
                    // File was dropped onto the window
                    state.show_drop_zone = false;
                    state.drop_file_valid = false;
                    state.hovered_file_path = None;
                    
                    let path = PathBuf::from(path_str);
                    
                    // Check if it's a .3mf file
                    if path.extension().is_some_and(|ext| ext == "3mf") {
                        println!("\nLoading dropped file: {}", path.display());
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
                                    state.mesh_nodes = create_mesh_nodes_with_displacement(
                                        &mut window,
                                        model,
                                        state.boolean_mode,
                                        state.show_displacement,
                                        state.file_path.as_ref(),
                                    );
                                    state.beam_nodes =
                                        create_beam_lattice_nodes(&mut window, model);
                                    window.set_title(&state.window_title());
                                    println!("\n✓ File loaded successfully!");
                                    print_model_info(model);

                                    // Reset camera to fit new model
                                    camera = create_camera_for_model(state.model.as_ref());

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
                    } else {
                        eprintln!("\n✗ Only .3mf files are supported");
                        eprintln!("   Dropped file: {}", path.display());
                    }
                }
                _ => {}
            }
        }

        // Update slice preview window if active
        if !state.update_slice_preview_window() {
            // Window was closed, remove it
            state.slice_preview_window = None;
        }

        // Draw XYZ axes if visible
        if show_axes {
            draw_axes(&mut window, axis_length);
        }

        // Draw print area if visible
        if state.print_area.visible {
            draw_print_area(&mut window, &state.print_area);
            
            // Draw ruler if enabled
            if state.print_area.show_ruler {
                draw_ruler(&mut window, &state.print_area);
            }
            
            // Draw scale bar if enabled
            if state.print_area.show_scale_bar {
                draw_scale_bar(&mut window, &state.print_area);
            }
        }

        // Draw slice view if visible
        if state.slice_view.visible && state.model.is_some() {
            if let Some(ref model) = state.model {
                let bounds = calculate_model_bounds(model);
                
                // Update animation if playing
                if state.slice_view.animation_playing {
                    let delta_time = 1.0 / 60.0; // Assume ~60 FPS
                    state.slice_view.update_animation(delta_time);
                }
                
                if state.slice_view.use_slice_stack && !model.resources.slice_stacks.is_empty() {
                    // Render slice stack data
                    if state.slice_view.show_stack_3d {
                        // Render all slices in 3D
                        draw_slice_stack_3d(&mut window, &state.slice_view, &model.resources.slice_stacks[0]);
                    } else {
                        // Render current slice only
                        if state.slice_view.current_slice_index < model.resources.slice_stacks[0].slices.len() {
                            draw_slice_stack_single(&mut window, &state.slice_view, &model.resources.slice_stacks[0]);
                        }
                    }
                } else {
                    // Original slice view mode (computed from mesh)
                    // Draw slice plane if enabled
                    if state.slice_view.show_plane {
                        draw_slice_plane(&mut window, &state.slice_view, bounds);
                    }
                    
                    // Draw slice contours
                    draw_slice_contours(&mut window, &state.slice_view);
                }
            }
        }
        
        // Draw drop zone overlay if file is being dragged
        if state.show_drop_zone {
            draw_drop_zone_overlay(&mut window, state.drop_file_valid, &state.hovered_file_path);
        }
        // Draw model information panel if visible
        render_model_info_panel(&mut window, &state);
        
        // Draw menu bar (rendered last so it's on top)
        menu_bar.render(&mut window);
    }

    Ok(())
}

/// Handle menu action triggered by the menu bar
#[allow(clippy::too_many_arguments)]
fn handle_menu_action(
    action: MenuAction,
    window: &mut Window,
    state: &mut ViewerState,
    camera: &mut ArcBall,
    show_axes: &mut bool,
    axis_length: &mut f32,
    menu_bar: &mut MenuBar,
) {
    match action {
        MenuAction::Open => {
            if let Some(path) = open_file_dialog() {
                match state.load_file(path) {
                    Ok(()) => {
                        // Hide existing nodes
                        for node in &mut state.mesh_nodes {
                            node.set_visible(false);
                        }
                        state.mesh_nodes.clear();
                        for node in &mut state.beam_nodes {
                            node.set_visible(false);
                        }
                        state.beam_nodes.clear();

                        // Create new nodes
                        if let Some(ref model) = state.model {
                            state.mesh_nodes = create_mesh_nodes_with_displacement(
                                window,
                                model,
                                state.boolean_mode,
                                state.show_displacement,
                                state.file_path.as_ref(),
                            );
                            state.beam_nodes = create_beam_lattice_nodes(window, model);
                            window.set_title(&state.window_title());
                            println!("\n✓ File loaded successfully!");
                            print_model_info(model);

                            // Reset camera
                            *camera = create_camera_for_model(state.model.as_ref());
                            
                            // Update axis length
                            let (min_bound, max_bound) = calculate_model_bounds(model);
                            let size = Vector3::new(
                                max_bound.0 - min_bound.0,
                                max_bound.1 - min_bound.1,
                                max_bound.2 - min_bound.2,
                            );
                            let max_size = size.x.max(size.y).max(size.z);
                            *axis_length = max_size * 0.5;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error loading file: {}", e);
                    }
                }
            }
        }
        MenuAction::BrowseTests => {
            println!("\nBrowse test suites feature requires restart with --browse-tests flag");
        }
        MenuAction::ExportScreenshot => {
            if let Err(e) = capture_screenshot(window) {
                eprintln!("Error capturing screenshot: {}", e);
            }
        }
        MenuAction::Exit => {
            window.close();
        }
        MenuAction::ToggleAxes => {
            *show_axes = !*show_axes;
            menu_bar.set_checked(MenuAction::ToggleAxes, *show_axes);
            println!("Axes: {}", if *show_axes { "ON" } else { "OFF" });
        }
        MenuAction::TogglePrintBed => {
            state.print_area.toggle_visibility();
            menu_bar.set_checked(MenuAction::TogglePrintBed, state.print_area.visible);
            println!("Print Bed: {}", if state.print_area.visible { "ON" } else { "OFF" });
        }
        MenuAction::ToggleGrid => {
            println!("Grid toggle not yet implemented");
        }
        MenuAction::ToggleModelInfo => {
            state.info_panel.toggle_visibility();
            menu_bar.set_checked(MenuAction::ToggleModelInfo, state.info_panel.visible);
            println!(
                "Model Information Panel: {}",
                if state.info_panel.visible { "ON" } else { "OFF" }
            );
        }
        MenuAction::ToggleRulers => {
            println!("Rulers toggle not yet implemented");
        }
        MenuAction::ResetCamera => {
            *camera = create_camera_for_model(state.model.as_ref());
            println!("Camera reset to default position");
        }
        MenuAction::FitToModel => {
            if let Some(ref model) = state.model {
                *camera = create_camera_for_model(Some(model));
                println!("Camera fitted to model");
            }
        }
        MenuAction::TopView | MenuAction::FrontView | MenuAction::SideView => {
            println!("View presets not yet implemented");
        }
        MenuAction::ThemeLight => {
            state.theme = Theme::Light;
            let bg = state.theme.background_color();
            window.set_background_color(bg.0, bg.1, bg.2);
            println!("Theme: Light");
        }
        MenuAction::ThemeDark => {
            state.theme = Theme::Dark;
            let bg = state.theme.background_color();
            window.set_background_color(bg.0, bg.1, bg.2);
            println!("Theme: Dark");
        }
        MenuAction::ThemeCustom => {
            println!("Custom theme not yet implemented");
        }
        MenuAction::PrintBedSettings => {
            println!("Print bed settings - use 'C' key to configure");
        }
        MenuAction::Preferences => {
            println!("Preferences dialog not yet implemented");
        }
        MenuAction::ToggleMaterials => {
            println!("Materials are always enabled");
        }
        MenuAction::ToggleBeamLattice => {
            state.show_beams = !state.show_beams;
            for node in &mut state.beam_nodes {
                node.set_visible(state.show_beams);
            }
            menu_bar.set_checked(MenuAction::ToggleBeamLattice, state.show_beams);
            println!("Beam Lattice: {}", if state.show_beams { "ON" } else { "OFF" });
        }
        MenuAction::ToggleSliceStack => {
            state.slice_view.toggle_visibility();
            menu_bar.set_checked(MenuAction::ToggleSliceStack, state.slice_view.visible);
            println!("Slice View: {}", if state.slice_view.visible { "ON" } else { "OFF" });
        }
        MenuAction::ToggleDisplacement => {
            state.show_displacement = !state.show_displacement;
            menu_bar.set_checked(MenuAction::ToggleDisplacement, state.show_displacement);
            
            // Rebuild mesh nodes with/without displacement
            for node in &mut state.mesh_nodes {
                node.set_visible(false);
            }
            state.mesh_nodes.clear();
            
            if let Some(ref model) = state.model {
                state.mesh_nodes = create_mesh_nodes_with_displacement(
                    window,
                    model,
                    state.boolean_mode,
                    state.show_displacement,
                    state.file_path.as_ref(),
                );
            }
            
            println!("Displacement: {}", if state.show_displacement { "ON" } else { "OFF" });
        }
        MenuAction::ToggleBooleanOps => {
            state.boolean_mode = state.boolean_mode.next();
            println!("Boolean mode: {}", state.boolean_mode.name());
            
            // Rebuild mesh nodes with new boolean mode
            for node in &mut state.mesh_nodes {
                node.set_visible(false);
            }
            state.mesh_nodes.clear();
            
            if let Some(ref model) = state.model {
                state.mesh_nodes = create_mesh_nodes_with_displacement(
                    window,
                    model,
                    state.boolean_mode,
                    state.show_displacement,
                    state.file_path.as_ref(),
                );
            }
        }
        MenuAction::KeyboardShortcuts => {
            print_controls();
        }
        MenuAction::About => {
            println!("\n═══════════════════════════════════════════════════════════");
            println!("  3MF Viewer");
            println!("  Version: 0.1.0");
            println!("  A viewer for 3D Manufacturing Format files");
            println!("  Built with lib3mf_rust");
            println!("═══════════════════════════════════════════════════════════\n");
        }
        MenuAction::OpenRecent | MenuAction::None => {
            // Not implemented
        }
    }
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
    keybindings::print_help();
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
    let boolean_count = count_boolean_operations(model);

    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!("  Model Information:");
    println!("  - Objects: {}", model.resources.objects.len());
    println!("  - Triangles: {}", count_triangles(model));
    println!("  - Vertices: {}", count_vertices(model));
    println!("  - Unit: {}", model.unit);
    
    // Display material information
    if !model.resources.materials.is_empty() 
        || !model.resources.color_groups.is_empty()
        || !model.resources.base_material_groups.is_empty() {
        println!("  - Materials:");
        if !model.resources.materials.is_empty() {
            println!("      Base Materials: {}", model.resources.materials.len());
        }
        if !model.resources.color_groups.is_empty() {
            println!("      Color Groups: {}", model.resources.color_groups.len());
        }
        if !model.resources.base_material_groups.is_empty() {
            println!("      Base Material Groups: {}", model.resources.base_material_groups.len());
        }
    }
    
    if beam_count > 0 {
        println!("  - Beam Lattice: {} beams", beam_count);
    }
    if boolean_count > 0 {
        println!("  - Boolean Operations: {} operations", boolean_count);
    }
    
    // Display displacement information if present
    if has_displacement_data(model) {
        let (maps, norm_groups, disp_groups) = count_displacement_resources(model);
        let disp_objects = count_displacement_objects(model);
        println!("  - Displacement:");
        if maps > 0 {
            println!("      Maps: {}", maps);
        }
        if norm_groups > 0 {
            println!("      Normal Vector Groups: {}", norm_groups);
        }
        if disp_groups > 0 {
            println!("      Displacement Groups: {}", disp_groups);
        }
        if disp_objects > 0 {
            println!("      Objects with Displacement: {}", disp_objects);
        }
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

/// Calculate the center and optimal distance for viewing the model
fn calculate_camera_params(model: &Model) -> (Point3<f32>, f32) {
    let (min_bound, max_bound) = calculate_model_bounds(model);
    
    // Calculate the center point of the model
    let center = Point3::new(
        (min_bound.0 + max_bound.0) / 2.0,
        (min_bound.1 + max_bound.1) / 2.0,
        (min_bound.2 + max_bound.2) / 2.0,
    );
    
    // Calculate the diagonal distance to determine camera distance
    let size = Vector3::new(
        max_bound.0 - min_bound.0,
        max_bound.1 - min_bound.1,
        max_bound.2 - min_bound.2,
    );
    let diagonal = size.magnitude();
    
    // Return center and optimal viewing distance
    (center, diagonal * CAMERA_DISTANCE_MULTIPLIER)
}

/// Create a camera positioned to view the model
fn create_camera_for_model(model: Option<&Model>) -> ArcBall {
    if let Some(model) = model {
        let (center, distance) = calculate_camera_params(model);
        
        // Camera eye position (looking from top-right-front)
        let eye = Point3::new(
            center.x + distance * 0.5,
            center.y + distance * 0.5,
            center.z + distance * 0.7,
        );
        
        ArcBall::new(eye, center)
    } else {
        // Default camera for empty scene
        let eye = Point3::new(100.0, 100.0, 100.0);
        let at = Point3::origin();
        ArcBall::new(eye, at)
    }
}

/// Fit the camera to show the entire model
fn fit_camera_to_model(camera: &mut ArcBall, model: &Model) {
    let (center, distance) = calculate_camera_params(model);
    
    // Set camera to look at center with appropriate distance
    camera.set_at(center);
    camera.set_dist(distance);
}

/// Zoom the camera by a given factor
fn zoom_camera(camera: &mut ArcBall, factor: f32) {
    let current_dist = camera.dist();
    camera.set_dist(current_dist * factor);
}

/// Pan the camera in a given direction
fn pan_camera(camera: &mut ArcBall, delta_x: f32, delta_y: f32, delta_z: f32) {
    let at = camera.at();
    let dist = camera.dist();
    let pan_amount = dist * PAN_STEP;
    camera.set_at(Point3::new(
        at.x + delta_x * pan_amount,
        at.y + delta_y * pan_amount,
        at.z + delta_z * pan_amount,
    ));
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
        // Try to find in base material groups (use first material)
        if let Some(bmg) = model.resources.base_material_groups.iter().find(|bg| bg.id == pid) {
            if !bmg.materials.is_empty() {
                let (r, g, b, _) = bmg.materials[0].displaycolor;
                return (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
            }
        }
    }

    // Default color: nice blue-gray
    (100.0 / 255.0, 150.0 / 255.0, 200.0 / 255.0)
}

/// Load textures from a 3MF package file
/// Returns a HashMap mapping texture IDs to loaded texture data
fn load_textures_from_package(
    file_path: &PathBuf,
    model: &Model,
) -> HashMap<usize, Rc<image::DynamicImage>> {
    let mut textures = HashMap::new();
    
    // Open the 3MF file as a ZIP archive
    let file = match File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open 3MF file for texture loading: {}", e);
            return textures;
        }
    };
    
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Failed to open 3MF as ZIP archive: {}", e);
            return textures;
        }
    };
    
    // Load each texture2d resource
    for texture2d in &model.resources.texture2d_resources {
        // Normalize path (remove leading slash if present)
        let normalized_path = texture2d.path.trim_start_matches('/');
        
        // Try both path variants and read the file
        let image_data = {
            let mut buffer = Vec::new();
            let mut found = false;
            
            // Try normalized path first
            if let Ok(mut file) = archive.by_name(normalized_path) {
                if file.read_to_end(&mut buffer).is_ok() {
                    found = true;
                }
            }
            
            // Try original path if normalized didn't work
            if !found {
                buffer.clear();
                if let Ok(mut file) = archive.by_name(&texture2d.path) {
                    if file.read_to_end(&mut buffer).is_ok() {
                        found = true;
                    }
                }
            }
            
            if !found {
                eprintln!(
                    "Texture file '{}' not found in 3MF package",
                    texture2d.path
                );
                continue;
            }
            
            buffer
        };
        
        // Load the image from memory
        match image::load_from_memory(&image_data) {
            Ok(img) => {
                println!(
                    "  ✓ Loaded texture ID {}: {} ({}x{})",
                    texture2d.id,
                    texture2d.path,
                    img.width(),
                    img.height()
                );
                textures.insert(texture2d.id, Rc::new(img));
            }
            Err(e) => {
                eprintln!(
                    "Failed to decode texture image '{}': {}",
                    texture2d.path, e
                );
            }
        }
    }
    
    if !textures.is_empty() {
        println!("  Loaded {} texture(s) from 3MF package", textures.len());
    }
    
    textures
}

/// Resolve composite material color by blending base materials
fn resolve_composite_color(
    model: &Model,
    comp: &lib3mf::CompositeMaterials,
    composite_idx: usize,
) -> (f32, f32, f32) {
    // Get the composite definition
    let composite = &comp.composites[composite_idx];
    
    // Find the base material group this composite references
    if let Some(base_group) = model.resources.base_material_groups.iter().find(|bg| bg.id == comp.matid) {
        // Blend colors according to the composite values
        let mut r_total = 0.0_f32;
        let mut g_total = 0.0_f32;
        let mut b_total = 0.0_f32;
        
        // Iterate up to the minimum of values and matindices length to prevent out-of-bounds
        let blend_count = composite.values.len().min(comp.matindices.len());
        
        for i in 0..blend_count {
            let value = composite.values[i];
            let mat_idx = comp.matindices[i];
            
            if mat_idx < base_group.materials.len() {
                let (r, g, b, _) = base_group.materials[mat_idx].displaycolor;
                r_total += (r as f32 / 255.0) * value;
                g_total += (g as f32 / 255.0) * value;
                b_total += (b as f32 / 255.0) * value;
            }
        }
        
        // Normalize by sum of values to handle cases where values don't sum to 1.0
        // This ensures proper color intensity regardless of value distribution
        let sum: f32 = composite.values.iter().take(blend_count).sum();
        if sum > 0.0 {
            r_total /= sum;
            g_total /= sum;
            b_total /= sum;
        }
        
        return (r_total, g_total, b_total);
    }
    
    // Fallback to purple to indicate composite material that couldn't be resolved
    (0.8, 0.0, 0.8)
}

/// Resolve multi-property color by blending multiple property groups
fn resolve_multiproperty_color(
    model: &Model,
    multi: &lib3mf::MultiProperties,
    multi_idx: usize,
) -> (f32, f32, f32) {
    // Get the multi element
    let multi_elem = &multi.multis[multi_idx];
    
    // Get colors from each referenced property group
    let mut colors = Vec::new();
    
    for (i, &pid) in multi.pids.iter().enumerate() {
        if i < multi_elem.pindices.len() {
            let pindex = multi_elem.pindices[i];
            
            // Try to get color from each property group type
            if let Some(cg) = model.resources.color_groups.iter().find(|c| c.id == pid) {
                if pindex < cg.colors.len() {
                    let (r, g, b, _) = cg.colors[pindex];
                    colors.push((r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0));
                }
            } else if let Some(bmg) = model.resources.base_material_groups.iter().find(|bg| bg.id == pid) {
                if pindex < bmg.materials.len() {
                    let (r, g, b, _) = bmg.materials[pindex].displaycolor;
                    colors.push((r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0));
                }
            }
        }
    }
    
    if colors.is_empty() {
        // Fallback to magenta to indicate multi-property that couldn't be resolved
        return (1.0, 0.0, 1.0);
    }
    
    // Blend colors based on blend methods
    // For simplicity, we average the colors (Mix blend method)
    // A more sophisticated implementation would handle Multiply and other blend modes
    let mut r_total = 0.0_f32;
    let mut g_total = 0.0_f32;
    let mut b_total = 0.0_f32;
    
    for (r, g, b) in &colors {
        r_total += r;
        g_total += g;
        b_total += b;
    }
    
    let count = colors.len() as f32;
    (r_total / count, g_total / count, b_total / count)
}

/// Get color for a specific triangle based on material properties
fn get_triangle_color(
    model: &Model,
    obj: &lib3mf::Object,
    triangle: &lib3mf::Triangle,
) -> (f32, f32, f32) {
    // First check triangle-level material
    if let Some(pid) = triangle.pid {
        // Determine which index to use for the color
        let pindex = triangle.pindex.or(triangle.p1);
        
        // Try to find in base materials (single color material)
        if let Some(mat) = model.resources.materials.iter().find(|m| m.id == pid) {
            if let Some((r, g, b, _)) = mat.color {
                return (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
            }
        }
        
        // Try to find in color groups
        if let Some(cg) = model.resources.color_groups.iter().find(|c| c.id == pid) {
            if let Some(idx) = pindex {
                if idx < cg.colors.len() {
                    let (r, g, b, _) = cg.colors[idx];
                    return (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
                }
            } else if !cg.colors.is_empty() {
                // No specific index, use first color
                let (r, g, b, _) = cg.colors[0];
                return (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
            }
        }
        
        // Try to find in base material groups
        if let Some(bmg) = model.resources.base_material_groups.iter().find(|bg| bg.id == pid) {
            if let Some(idx) = pindex {
                if idx < bmg.materials.len() {
                    let (r, g, b, _) = bmg.materials[idx].displaycolor;
                    return (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
                }
            } else if !bmg.materials.is_empty() {
                // No specific index, use first material
                let (r, g, b, _) = bmg.materials[0].displaycolor;
                return (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
            }
        }
        
        // Try to find in composite materials
        if let Some(comp) = model.resources.composite_materials.iter().find(|c| c.id == pid) {
            if let Some(idx) = pindex {
                if idx < comp.composites.len() {
                    return resolve_composite_color(model, comp, idx);
                }
            } else if !comp.composites.is_empty() {
                // Use first composite
                return resolve_composite_color(model, comp, 0);
            }
        }
        
        // Try to find in multi-properties
        if let Some(multi) = model.resources.multi_properties.iter().find(|m| m.id == pid) {
            if let Some(idx) = pindex {
                if idx < multi.multis.len() {
                    return resolve_multiproperty_color(model, multi, idx);
                }
            } else if !multi.multis.is_empty() {
                // Use first multi
                return resolve_multiproperty_color(model, multi, 0);
            }
        }
    }
    
    // Fall back to object-level color
    get_object_color(model, obj)
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
        return TriMesh::new(
            vertices,
            None,
            None,
            Some(kiss3d::ncollide3d::procedural::IndexBuffer::Unified(faces)),
        );
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
        faces.push(Point3::new(0, seg + 1, next_seg + 1));
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
                        if beam.v1 >= mesh_data.vertices.len()
                            || beam.v2 >= mesh_data.vertices.len()
                        {
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

                    // Render explicit balls from the balls extension
                    // These are explicitly defined in the 3MF file
                    for ball in &beamset.balls {
                        if ball.vindex < mesh_data.vertices.len() {
                            let v = &mesh_data.vertices[ball.vindex];
                            let center = Point3::new(v.x as f32, v.y as f32, v.z as f32);
                            
                            // Use ball's radius, or default from ball_radius, or beamset radius
                            let radius = ball.radius
                                .or(beamset.ball_radius)
                                .unwrap_or(beamset.radius) as f32;
                            
                            let sphere = create_sphere_mesh(center, radius, GEOMETRY_SEGMENTS);
                            let mut sphere_node = window.add_trimesh(sphere, IDENTITY_SCALE);
                            sphere_node.set_color(BEAM_COLOR.0, BEAM_COLOR.1, BEAM_COLOR.2);
                            
                            nodes.push(sphere_node);
                        }
                    }

                    // Add inferred spherical joints at highly connected vertices
                    // (only for sphere cap mode and when explicit balls are not defined)
                    if beamset.cap_mode == lib3mf::BeamCapMode::Sphere && beamset.balls.is_empty() {
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
                                let max_radius = beamset
                                    .beams
                                    .iter()
                                    .filter(|b| b.v1 == *vertex_idx || b.v2 == *vertex_idx)
                                    .map(|b| {
                                        if b.v1 == *vertex_idx {
                                            b.r1.unwrap_or(beamset.radius)
                                        } else {
                                            b.r2.unwrap_or(b.r1.unwrap_or(beamset.radius))
                                        }
                                    })
                                    .fold(beamset.radius, f64::max)
                                    as f32;

                                let sphere =
                                    create_sphere_mesh(center, max_radius, GEOMETRY_SEGMENTS);
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

/// Draw drag-and-drop overlay when a file is being dragged over the window
fn draw_drop_zone_overlay(window: &mut Window, valid_file: bool, file_path: &Option<String>) {
    use kiss3d::nalgebra::{Point2, Point3};
    use kiss3d::text::Font;
    
    let size = window.size();
    let width = size.x as f32;
    let height = size.y as f32;
    
    // Choose color based on file validity
    let (r, g, b) = if valid_file {
        (0.2, 0.6, 1.0) // Blue tint for valid .3mf files
    } else {
        (1.0, 0.3, 0.3) // Red tint for invalid files
    };
    
    // Draw semi-transparent fullscreen overlay using horizontal lines
    let alpha = 0.3;
    let line_spacing = 10.0;
    
    // Draw horizontal lines to create overlay effect
    let mut y = 0.0;
    while y < height {
        window.draw_planar_line(
            &Point2::new(0.0, y),
            &Point2::new(width, y),
            &Point3::new(r * alpha, g * alpha, b * alpha),
        );
        y += line_spacing;
    }
    
    // Display message text in the center
    let message = if valid_file {
        "Drop to open file"
    } else {
        "Only .3mf files supported"
    };
    
    // Draw text in center
    let font = Font::default();
    let font_size = 48.0;
    let text_x = width / 2.0 - (message.len() as f32 * font_size * 0.3);
    let text_y = height / 2.0;
    
    window.draw_text(
        message,
        &Point2::new(text_x, text_y),
        font_size,
        &font,
        &Point3::new(1.0, 1.0, 1.0), // White text
    );
    
    // Draw file name if available
    if let Some(ref path) = file_path {
        let file_name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path);
        let fname_size = 32.0;
        let fname_x = width / 2.0 - (file_name.len() as f32 * fname_size * 0.3);
        let fname_y = text_y + 60.0;
        
        window.draw_text(
            file_name,
            &Point2::new(fname_x, fname_y),
            fname_size,
            &font,
            &Point3::new(0.9, 0.9, 0.9), // Light gray text
        );
    }
}

/// Count total boolean operations in the model
fn count_boolean_operations(model: &Model) -> usize {
    model
        .resources
        .objects
        .iter()
        .filter(|obj| obj.boolean_shape.is_some())
        .count()
}

/// Check if model has displacement data
fn has_displacement_data(model: &Model) -> bool {
    !model.resources.displacement_maps.is_empty()
        || !model.resources.norm_vector_groups.is_empty()
        || !model.resources.disp2d_groups.is_empty()
        || model
            .resources
            .objects
            .iter()
            .any(|obj| obj.displacement_mesh.is_some())
}

/// Count displacement resources in the model
fn count_displacement_resources(model: &Model) -> (usize, usize, usize) {
    let maps = model.resources.displacement_maps.len();
    let norm_groups = model.resources.norm_vector_groups.len();
    let disp_groups = model.resources.disp2d_groups.len();
    (maps, norm_groups, disp_groups)
}

/// Count objects with displacement meshes
fn count_displacement_objects(model: &Model) -> usize {
    model
        .resources
        .objects
        .iter()
        .filter(|obj| obj.displacement_mesh.is_some())
        .count()
}

/// Print detailed boolean operation information
fn print_boolean_info(model: &Model) {
    let boolean_objects: Vec<_> = model
        .resources
        .objects
        .iter()
        .filter(|obj| obj.boolean_shape.is_some())
        .collect();

    if boolean_objects.is_empty() {
        println!("\n  No boolean operations found in model");
        return;
    }

    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!("  Boolean Operations Information");
    println!("═══════════════════════════════════════════════════════════");

    for obj in boolean_objects {
        if let Some(ref shape) = obj.boolean_shape {
            println!();
            println!("  Object ID: {}", obj.id);
            println!("    Operation: {}", shape.operation.as_str());
            println!("    Base Object: {}", shape.objectid);
            println!("    Operands: {} objects", shape.operands.len());

            for (i, operand) in shape.operands.iter().enumerate() {
                println!("      [{}] Object ID: {}", i + 1, operand.objectid);
                if let Some(ref path) = operand.path {
                    println!("          Path: {}", path);
                }
            }
        }
    }

    println!();
    println!("═══════════════════════════════════════════════════════════");
}

/// Create a TriMesh node from mesh data with the given color
fn create_trimesh_node(
    window: &mut Window,
    mesh_data: &lib3mf::Mesh,
    color: (f32, f32, f32),
) -> SceneNode {
    let vertices: Vec<Point3<f32>> = mesh_data
        .vertices
        .iter()
        .map(|v| Point3::new(v.x as f32, v.y as f32, v.z as f32))
        .collect();

    let faces: Vec<Point3<u32>> = mesh_data
        .triangles
        .iter()
        .filter(|t| t.v1 < vertices.len() && t.v2 < vertices.len() && t.v3 < vertices.len())
        .map(|t| Point3::new(t.v1 as u32, t.v2 as u32, t.v3 as u32))
        .collect();

    let tri_mesh = TriMesh::new(
        vertices,
        None,
        None,
        Some(kiss3d::ncollide3d::procedural::IndexBuffer::Unified(faces)),
    );

    let scale = Vector3::new(1.0, 1.0, 1.0);
    let mut mesh_node = window.add_trimesh(tri_mesh, scale);
    mesh_node.set_color(color.0, color.1, color.2);

    mesh_node
}

/// Create mesh nodes with boolean operation-aware coloring
fn create_mesh_nodes_with_boolean_mode(
    window: &mut Window,
    model: &Model,
    mode: BooleanMode,
) -> Vec<SceneNode> {
    match mode {
        BooleanMode::Normal => create_mesh_nodes(window, model),
        BooleanMode::ShowInputs => create_mesh_nodes_show_inputs(window, model),
        BooleanMode::HighlightOperands => create_mesh_nodes_highlight_operands(window, model),
    }
}

/// Create mesh nodes with optional displacement highlighting
fn create_mesh_nodes_with_displacement(
    window: &mut Window,
    model: &Model,
    mode: BooleanMode,
    show_displacement: bool,
    file_path: Option<&PathBuf>,
) -> Vec<SceneNode> {
    if show_displacement && has_displacement_data(model) {
        create_mesh_nodes_highlight_displacement(window, model, file_path)
    } else {
        create_mesh_nodes_with_boolean_mode(window, model, mode)
    }
}

/// Create mesh nodes with material rendering support
fn create_mesh_nodes_with_materials(
    window: &mut Window,
    model: &Model,
    show_materials: bool,
    mode: BooleanMode,
    show_displacement: bool,
    file_path: Option<&PathBuf>,
) -> Vec<SceneNode> {
    // If materials are disabled, use default rendering
    if !show_materials {
        return create_mesh_nodes_with_displacement(window, model, mode, show_displacement, file_path);
    }
    
    // If displacement or boolean modes are active, use those instead
    if show_displacement && has_displacement_data(model) {
        return create_mesh_nodes_highlight_displacement(window, model, file_path);
    }
    
    if mode != BooleanMode::Normal {
        return create_mesh_nodes_with_boolean_mode(window, model, mode);
    }
    
    // Create nodes with per-triangle material colors and textures
    if let Some(path) = file_path {
        create_mesh_nodes_with_triangle_colors_and_textures(window, model, path)
    } else {
        create_mesh_nodes_with_triangle_colors(window, model)
    }
}

/// Create mesh nodes with per-triangle color support and texture mapping
fn create_mesh_nodes_with_triangle_colors(window: &mut Window, model: &Model) -> Vec<SceneNode> {
    create_mesh_nodes_with_triangle_colors_impl(window, model, None)
}

/// Create mesh nodes with per-triangle color support, texture mapping, and file path for loading textures
fn create_mesh_nodes_with_triangle_colors_and_textures(
    window: &mut Window,
    model: &Model,
    file_path: &PathBuf,
) -> Vec<SceneNode> {
    // Load textures from the 3MF package
    let textures = if !model.resources.texture2d_resources.is_empty() {
        println!("\n  Loading textures from 3MF package...");
        let loaded = load_textures_from_package(file_path, model);
        if loaded.is_empty() {
            println!("  No textures loaded");
        }
        Some(loaded)
    } else {
        None
    };
    
    create_mesh_nodes_with_triangle_colors_impl(window, model, textures.as_ref())
}

/// Implementation of mesh node creation with optional texture support
fn create_mesh_nodes_with_triangle_colors_impl(
    window: &mut Window,
    model: &Model,
    textures: Option<&HashMap<usize, Rc<image::DynamicImage>>>,
) -> Vec<SceneNode> {
    let mut nodes = Vec::new();

    for item in &model.build.items {
        if let Some(obj) = model
            .resources
            .objects
            .iter()
            .find(|o| o.id == item.objectid)
        {
            if let Some(ref mesh_data) = obj.mesh {
                // Check if mesh has per-triangle colors or textures
                let has_triangle_properties = mesh_data.triangles.iter().any(|t| t.pid.is_some());
                
                if has_triangle_properties {
                    // Group triangles by their material property ID
                    let mut property_groups: HashMap<(Option<usize>, Option<usize>), Vec<usize>> = 
                        HashMap::new();
                    
                    for (tri_idx, triangle) in mesh_data.triangles.iter().enumerate() {
                        // Group by (pid, pindex) to handle both colors and textures
                        let key = (triangle.pid, triangle.pindex.or(triangle.p1));
                        property_groups.entry(key).or_default().push(tri_idx);
                    }
                    
                    // Create a mesh for each property group
                    for ((pid_opt, pindex_opt), tri_indices) in property_groups.iter() {
                        // Check if this group uses a texture
                        let texture_info = if let (Some(pid), Some(pindex)) = (pid_opt, pindex_opt) {
                            // Check if this pid refers to a texture2d group
                            model.resources.texture2d_groups.iter()
                                .find(|tg| tg.id == *pid)
                                .map(|tg| (*pid, tg.texid, *pindex))
                        } else {
                            None
                        };
                        
                        if let Some((tex_group_id, tex_id, _)) = texture_info {
                            // This group uses textures - create mesh with UV coordinates
                            let tex_group = model.resources.texture2d_groups.iter()
                                .find(|tg| tg.id == tex_group_id).unwrap();
                            
                            // Create vertices and UV coordinates
                            let mut vertices = Vec::new();
                            let mut uvs = Vec::new();
                            let mut faces = Vec::new();
                            
                            for &tri_idx in tri_indices.iter() {
                                let triangle = &mesh_data.triangles[tri_idx];
                                let base_idx = vertices.len() as u32;
                                
                                // Add vertices for this triangle
                                for &v_idx in &[triangle.v1, triangle.v2, triangle.v3] {
                                    if v_idx < mesh_data.vertices.len() {
                                        let v = &mesh_data.vertices[v_idx];
                                        vertices.push(Point3::new(v.x as f32, v.y as f32, v.z as f32));
                                        
                                        // Add corresponding UV coordinate
                                        if v_idx < tex_group.tex2coords.len() {
                                            let uv = &tex_group.tex2coords[v_idx];
                                            uvs.push(Point2::new(uv.u, uv.v));
                                        } else {
                                            uvs.push(Point2::new(0.0, 0.0));
                                        }
                                    }
                                }
                                
                                // Add face
                                faces.push(Point3::new(base_idx, base_idx + 1, base_idx + 2));
                            }
                            
                            if !faces.is_empty() {
                                let tri_mesh = TriMesh::new(
                                    vertices,
                                    None,
                                    Some(uvs),
                                    Some(kiss3d::ncollide3d::procedural::IndexBuffer::Unified(faces)),
                                );
                                
                                let scale = Vector3::new(1.0, 1.0, 1.0);
                                let mut mesh_node = window.add_trimesh(tri_mesh, scale);
                                
                                // Try to apply texture if available
                                if let Some(texture_map) = textures {
                                    if let Some(texture_img) = texture_map.get(&tex_id) {
                                        // Convert image to RGBA8 format
                                        let rgba_img = texture_img.to_rgba8();
                                        let raw_data = rgba_img.into_raw();
                                        
                                        // Create a unique texture name
                                        let texture_name = format!("texture_{}", tex_id);
                                        
                                        // Apply texture from memory
                                        mesh_node.set_texture_from_memory(&raw_data, &texture_name);
                                    } else {
                                        // Texture not loaded, use teal indicator color
                                        mesh_node.set_color(0.0, 0.8, 0.8);
                                    }
                                } else {
                                    // No textures loaded, use teal indicator color
                                    mesh_node.set_color(0.0, 0.8, 0.8);
                                }
                                
                                nodes.push(mesh_node);
                            }
                        } else {
                            // This group uses colors (not textures)
                            let color = if let Some(_pid) = pid_opt {
                                let triangle = &mesh_data.triangles[tri_indices[0]];
                                get_triangle_color(model, obj, triangle)
                            } else {
                                get_object_color(model, obj)
                            };
                            
                            let color_key = (
                                (color.0 * 255.0) as u8,
                                (color.1 * 255.0) as u8,
                                (color.2 * 255.0) as u8,
                            );
                            
                            let vertices: Vec<Point3<f32>> = mesh_data
                                .vertices
                                .iter()
                                .map(|v| Point3::new(v.x as f32, v.y as f32, v.z as f32))
                                .collect();

                            let faces: Vec<Point3<u32>> = tri_indices
                                .iter()
                                .map(|&idx| {
                                    let t = &mesh_data.triangles[idx];
                                    Point3::new(t.v1 as u32, t.v2 as u32, t.v3 as u32)
                                })
                                .collect();

                            if !faces.is_empty() {
                                let tri_mesh = TriMesh::new(
                                    vertices,
                                    None,
                                    None,
                                    Some(kiss3d::ncollide3d::procedural::IndexBuffer::Unified(faces)),
                                );

                                let scale = Vector3::new(1.0, 1.0, 1.0);
                                let mut mesh_node = window.add_trimesh(tri_mesh, scale);
                                mesh_node.set_color(
                                    color_key.0 as f32 / 255.0,
                                    color_key.1 as f32 / 255.0,
                                    color_key.2 as f32 / 255.0,
                                );

                                nodes.push(mesh_node);
                            }
                        }
                    }
                } else {
                    // No per-triangle properties, use object-level color
                    let vertices: Vec<Point3<f32>> = mesh_data
                        .vertices
                        .iter()
                        .map(|v| Point3::new(v.x as f32, v.y as f32, v.z as f32))
                        .collect();

                    let faces: Vec<Point3<u32>> = mesh_data
                        .triangles
                        .iter()
                        .filter(|t| {
                            t.v1 < vertices.len() && t.v2 < vertices.len() && t.v3 < vertices.len()
                        })
                        .map(|t| Point3::new(t.v1 as u32, t.v2 as u32, t.v3 as u32))
                        .collect();

                    let tri_mesh = TriMesh::new(
                        vertices,
                        None,
                        None,
                        Some(kiss3d::ncollide3d::procedural::IndexBuffer::Unified(faces)),
                    );

                    let color = get_object_color(model, obj);
                    let scale = Vector3::new(1.0, 1.0, 1.0);
                    let mut mesh_node = window.add_trimesh(tri_mesh, scale);
                    mesh_node.set_color(color.0, color.1, color.2);

                    nodes.push(mesh_node);
                }
            }
        }
    }

    nodes
}

/// Load displacement texture maps from a 3MF package file
/// Returns a HashMap mapping displacement map IDs to loaded texture data
fn load_displacement_maps_from_package(
    file_path: &PathBuf,
    model: &Model,
) -> HashMap<usize, Rc<image::DynamicImage>> {
    let mut displacement_maps = HashMap::new();
    
    // Open the 3MF file as a ZIP archive
    let file = match File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open 3MF file for displacement map loading: {}", e);
            return displacement_maps;
        }
    };
    
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Failed to open 3MF as ZIP archive: {}", e);
            return displacement_maps;
        }
    };
    
    // Load each displacement2d resource
    for displacement_map in &model.resources.displacement_maps {
        // Normalize path (remove leading slash if present)
        let normalized_path = displacement_map.path.trim_start_matches('/');
        
        // Try both path variants and read the file
        let image_data = {
            let mut buffer = Vec::new();
            let mut found = false;
            
            // Try normalized path first
            if let Ok(mut file) = archive.by_name(normalized_path) {
                if file.read_to_end(&mut buffer).is_ok() {
                    found = true;
                }
            }
            
            // Try original path if normalized didn't work
            if !found {
                buffer.clear();
                if let Ok(mut file) = archive.by_name(&displacement_map.path) {
                    if file.read_to_end(&mut buffer).is_ok() {
                        found = true;
                    }
                }
            }
            
            if !found {
                eprintln!(
                    "Displacement map file '{}' not found in 3MF package",
                    displacement_map.path
                );
                continue;
            }
            
            buffer
        };
        
        // Load the image from memory
        match image::load_from_memory(&image_data) {
            Ok(img) => {
                println!(
                    "  ✓ Loaded displacement map ID {}: {} ({}x{})",
                    displacement_map.id,
                    displacement_map.path,
                    img.width(),
                    img.height()
                );
                displacement_maps.insert(displacement_map.id, Rc::new(img));
            }
            Err(e) => {
                eprintln!(
                    "Failed to decode displacement map image '{}': {}",
                    displacement_map.path, e
                );
            }
        }
    }
    
    if !displacement_maps.is_empty() {
        println!("  Loaded {} displacement map(s) from 3MF package", displacement_maps.len());
    }
    
    displacement_maps
}

/// Sample a displacement value from a texture at given UV coordinates
/// Returns a normalized displacement value [0.0, 1.0]
fn sample_displacement_texture(
    image: &image::DynamicImage,
    u: f64,
    v: f64,
    channel: lib3mf::Channel,
) -> f64 {
    let width = image.width() as f64;
    let height = image.height() as f64;
    
    // Clamp UV to [0, 1] range
    let u = u.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    
    // Convert UV to pixel coordinates
    // Note: V coordinate is typically flipped in textures
    let x = ((u * width).floor() as u32).min(image.width() - 1);
    let y = (((1.0 - v) * height).floor() as u32).min(image.height() - 1);
    
    // Sample the pixel
    let pixel = image.get_pixel(x, y);
    
    // Extract the appropriate channel value
    let value = match channel {
        lib3mf::Channel::R => pixel[0],
        lib3mf::Channel::G => pixel[1],
        lib3mf::Channel::B => pixel[2],
        lib3mf::Channel::A => if pixel.0.len() > 3 { pixel[3] } else { 255 },
    };
    
    // Normalize to [0.0, 1.0]
    value as f64 / 255.0
}

/// Create mesh nodes with displacement highlighting and actual displacement rendering
fn create_mesh_nodes_highlight_displacement(window: &mut Window, model: &Model, file_path: Option<&PathBuf>) -> Vec<SceneNode> {
    let mut nodes = Vec::new();

    // Load displacement maps if file_path is available
    let displacement_maps = if let Some(path) = file_path {
        if !model.resources.displacement_maps.is_empty() {
            println!("\n  Loading displacement maps...");
            load_displacement_maps_from_package(path, model)
        } else {
            HashMap::new()
        }
    } else {
        HashMap::new()
    };

    // Collect objects with displacement meshes
    let displacement_object_ids: HashSet<usize> = model
        .resources
        .objects
        .iter()
        .filter(|obj| obj.displacement_mesh.is_some())
        .map(|obj| obj.id)
        .collect();

    for item in &model.build.items {
        if let Some(obj) = model
            .resources
            .objects
            .iter()
            .find(|o| o.id == item.objectid)
        {
            // Handle objects with displacement meshes
            if let Some(ref disp_mesh) = obj.displacement_mesh {
                // Render displacement mesh with actual displacement applied
                if let Some(node) = create_displaced_mesh_node(
                    window,
                    model,
                    obj,
                    disp_mesh,
                    &displacement_maps,
                ) {
                    nodes.push(node);
                }
            }
            // Handle regular meshes (highlight if they have displacement data)
            else if let Some(ref mesh_data) = obj.mesh {
                // Convert vertices to nalgebra Point3
                let vertices: Vec<Point3<f32>> = mesh_data
                    .vertices
                    .iter()
                    .map(|v| Point3::new(v.x as f32, v.y as f32, v.z as f32))
                    .collect();

                // Convert triangles to face indices
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
                    None,
                    None,
                    Some(kiss3d::ncollide3d::procedural::IndexBuffer::Unified(faces)),
                );

                // Use bright cyan/aqua color for objects with displacement data
                let color = if displacement_object_ids.contains(&obj.id) {
                    (0.0, 1.0, 1.0) // Bright cyan for displaced objects
                } else {
                    get_object_color(model, obj) // Normal color
                };

                let mut mesh_node = window.add_trimesh(tri_mesh, IDENTITY_SCALE);
                mesh_node.set_color(color.0, color.1, color.2);

                nodes.push(mesh_node);
            }
        }
    }

    nodes
}

/// Create a mesh node with displacement applied
/// Returns None if displacement cannot be applied
fn create_displaced_mesh_node(
    window: &mut Window,
    model: &Model,
    _obj: &lib3mf::Object,
    disp_mesh: &lib3mf::DisplacementMesh,
    displacement_maps: &HashMap<usize, Rc<image::DynamicImage>>,
) -> Option<SceneNode> {
    // Calculate vertex normals for the base mesh
    let mut temp_mesh = lib3mf::Mesh::new();
    temp_mesh.vertices = disp_mesh.vertices.clone();
    
    // Convert displacement triangles to regular triangles for normal calculation
    for dt in &disp_mesh.triangles {
        temp_mesh.triangles.push(lib3mf::Triangle {
            v1: dt.v1,
            v2: dt.v2,
            v3: dt.v3,
            pid: None,
            pindex: None,
            p1: None,
            p2: None,
            p3: None,
        });
    }
    
    let vertex_normals = lib3mf::mesh_ops::calculate_vertex_normals(&temp_mesh);
    
    // Create displaced vertices
    let mut displaced_vertices = disp_mesh.vertices.clone();
    
    // Track which vertices have been displaced
    // NOTE: This implementation applies displacement only once per vertex.
    // If a vertex is shared by multiple triangles with different displacement
    // coordinates, only the first encountered displacement will be applied.
    // A more accurate implementation would either:
    // 1. Duplicate vertices so each triangle-vertex pair has independent displacement
    // 2. Average or blend multiple displacement values for shared vertices
    // For the initial implementation, we use the simpler approach.
    let mut vertex_displaced = vec![false; displaced_vertices.len()];
    
    // Process each triangle to apply displacement
    for triangle in &disp_mesh.triangles {
        // Check if this triangle has displacement data
        if let (Some(did), Some(d1), Some(d2), Some(d3)) = (triangle.did, triangle.d1, triangle.d2, triangle.d3) {
            // Find the displacement group
            if let Some(disp_group) = model.resources.disp2d_groups.iter().find(|g| g.id == did) {
                // Find the normal vector group
                if let Some(norm_group) = model.resources.norm_vector_groups.iter().find(|g| g.id == disp_group.nid) {
                    // Find the displacement map
                    if let Some(disp_map_image) = displacement_maps.get(&disp_group.dispid) {
                        // Find the displacement map resource for channel info
                        if let Some(disp_map) = model.resources.displacement_maps.iter().find(|m| m.id == disp_group.dispid) {
                            // Apply displacement to each vertex of the triangle if not already displaced
                            let displacement_info = DisplacementInfo {
                                disp_group,
                                norm_group,
                                disp_map_image,
                                disp_map,
                            };
                            
                            apply_displacement_to_vertex(
                                &mut displaced_vertices,
                                &vertex_normals,
                                &mut vertex_displaced,
                                triangle.v1,
                                d1,
                                &displacement_info,
                            );
                            
                            apply_displacement_to_vertex(
                                &mut displaced_vertices,
                                &vertex_normals,
                                &mut vertex_displaced,
                                triangle.v2,
                                d2,
                                &displacement_info,
                            );
                            
                            apply_displacement_to_vertex(
                                &mut displaced_vertices,
                                &vertex_normals,
                                &mut vertex_displaced,
                                triangle.v3,
                                d3,
                                &displacement_info,
                            );
                        }
                    }
                }
            }
        }
    }
    
    // Convert displaced vertices to nalgebra Point3
    let vertices: Vec<Point3<f32>> = displaced_vertices
        .iter()
        .map(|v| Point3::new(v.x as f32, v.y as f32, v.z as f32))
        .collect();

    // Convert triangles to face indices
    let faces: Vec<Point3<u32>> = disp_mesh
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
        None,
        None,
        Some(kiss3d::ncollide3d::procedural::IndexBuffer::Unified(faces)),
    );

    // Use cyan color for displaced meshes to indicate they have been processed
    let mut mesh_node = window.add_trimesh(tri_mesh, IDENTITY_SCALE);
    mesh_node.set_color(0.0, 1.0, 1.0); // Bright cyan

    Some(mesh_node)
}

/// Helper struct to pass displacement information
struct DisplacementInfo<'a> {
    disp_group: &'a lib3mf::Disp2DGroup,
    norm_group: &'a lib3mf::NormVectorGroup,
    disp_map_image: &'a image::DynamicImage,
    disp_map: &'a lib3mf::Displacement2D,
}

/// Apply displacement to a single vertex
fn apply_displacement_to_vertex(
    vertices: &mut [lib3mf::Vertex],
    vertex_normals: &[(f64, f64, f64)],
    vertex_displaced: &mut [bool],
    vertex_index: usize,
    coord_index: usize,
    info: &DisplacementInfo,
) {
    // Skip if already displaced or invalid index
    if vertex_index >= vertices.len() || vertex_displaced[vertex_index] {
        return;
    }
    
    // Skip if coordinate index is out of bounds
    if coord_index >= info.disp_group.coords.len() {
        return;
    }
    
    let coord = &info.disp_group.coords[coord_index];
    
    // Sample the displacement texture at UV coordinates
    let displacement_value = sample_displacement_texture(
        info.disp_map_image,
        coord.u,
        coord.v,
        info.disp_map.channel,
    );
    
    // Get the normal vector from the norm vector group
    // The coord.n is an index into the norm vector group's vectors
    let normal = if coord.n < info.norm_group.vectors.len() {
        let norm_vec = &info.norm_group.vectors[coord.n];
        (norm_vec.x, norm_vec.y, norm_vec.z)
    } else {
        // Fallback to vertex normal if index is invalid
        if vertex_index < vertex_normals.len() {
            vertex_normals[vertex_index]
        } else {
            (0.0, 0.0, 1.0) // Default up vector
        }
    };
    
    // Calculate displacement amount: (texture_value * height + offset) * factor
    let displacement_amount = (displacement_value * info.disp_group.height + info.disp_group.offset) * coord.f;
    
    // Displace the vertex along the normal
    let vertex = &mut vertices[vertex_index];
    vertex.x += normal.0 * displacement_amount;
    vertex.y += normal.1 * displacement_amount;
    vertex.z += normal.2 * displacement_amount;
    
    // Mark as displaced
    vertex_displaced[vertex_index] = true;
}

/// Create mesh nodes with boolean inputs shown in different colors
fn create_mesh_nodes_show_inputs(window: &mut Window, model: &Model) -> Vec<SceneNode> {
    let mut nodes = Vec::new();

    // Collect all boolean operations and their operands
    let mut boolean_base_objects = std::collections::HashSet::new();
    let mut boolean_operand_objects = std::collections::HashSet::new();

    for obj in &model.resources.objects {
        if let Some(ref shape) = obj.boolean_shape {
            boolean_base_objects.insert(shape.objectid);
            for operand in &shape.operands {
                boolean_operand_objects.insert(operand.objectid);
            }
        }
    }

    // Render all objects with appropriate colors
    for item in &model.build.items {
        if let Some(obj) = model
            .resources
            .objects
            .iter()
            .find(|o| o.id == item.objectid)
        {
            // Skip objects with boolean_shape (they're the result objects)
            if obj.boolean_shape.is_some() {
                continue;
            }

            if let Some(ref mesh_data) = obj.mesh {
                // Determine color based on role in boolean operations
                let color = if boolean_base_objects.contains(&obj.id) {
                    // Base object: Blue
                    (0.3, 0.5, 0.9)
                } else if boolean_operand_objects.contains(&obj.id) {
                    // Operand object: Red
                    (0.9, 0.3, 0.3)
                } else {
                    // Regular object: Use default color
                    get_object_color(model, obj)
                };

                let mesh_node = create_trimesh_node(window, mesh_data, color);
                nodes.push(mesh_node);
            }
        }
    }

    // Also render the base and operand objects directly (not just from build items)
    for obj in &model.resources.objects {
        if boolean_base_objects.contains(&obj.id) || boolean_operand_objects.contains(&obj.id) {
            // Check if already rendered via build items
            let already_in_build = model.build.items.iter().any(|item| item.objectid == obj.id);

            if !already_in_build {
                if let Some(ref mesh_data) = obj.mesh {
                    let color = if boolean_base_objects.contains(&obj.id) {
                        (0.3, 0.5, 0.9) // Blue for base
                    } else {
                        (0.9, 0.3, 0.3) // Red for operand
                    };

                    let mesh_node = create_trimesh_node(window, mesh_data, color);
                    nodes.push(mesh_node);
                }
            }
        }
    }

    nodes
}

/// Create mesh nodes with only boolean operands highlighted
fn create_mesh_nodes_highlight_operands(window: &mut Window, model: &Model) -> Vec<SceneNode> {
    let mut nodes = Vec::new();

    // Collect all boolean operations and their operands
    let mut boolean_base_objects = std::collections::HashSet::new();
    let mut boolean_operand_objects = std::collections::HashSet::new();

    for obj in &model.resources.objects {
        if let Some(ref shape) = obj.boolean_shape {
            boolean_base_objects.insert(shape.objectid);
            for operand in &shape.operands {
                boolean_operand_objects.insert(operand.objectid);
            }
        }
    }

    // Only render base and operand objects
    for obj in &model.resources.objects {
        if boolean_base_objects.contains(&obj.id) || boolean_operand_objects.contains(&obj.id) {
            if let Some(ref mesh_data) = obj.mesh {
                // Use bright, distinct colors
                let color = if boolean_base_objects.contains(&obj.id) {
                    (0.2, 0.6, 1.0) // Bright blue for base
                } else {
                    (1.0, 0.4, 0.2) // Bright orange for operands
                };

                let mesh_node = create_trimesh_node(window, mesh_data, color);
                nodes.push(mesh_node);
            }
        }
    }

    nodes
}

/// Draw print area as a wireframe box (12 lines)
fn draw_print_area(window: &mut Window, area: &PrintArea) {
    // Get dimensions in mm for internal calculations
    let width = area.width_mm();
    let height = area.height_mm();
    let depth = area.depth_mm();
    
    // Calculate offset based on origin mode
    let (x_offset, y_offset) = match area.origin {
        Origin::Corner => (0.0, 0.0),
        Origin::CenterBottom => (-width / 2.0, -depth / 2.0),
    };

    // Define 8 corners of the box
    let corners = [
        Point3::new(x_offset, y_offset, 0.0),                     // 0: bottom front left
        Point3::new(x_offset + width, y_offset, 0.0),             // 1: bottom front right
        Point3::new(x_offset + width, y_offset + depth, 0.0),     // 2: bottom back right
        Point3::new(x_offset, y_offset + depth, 0.0),             // 3: bottom back left
        Point3::new(x_offset, y_offset, height),                  // 4: top front left
        Point3::new(x_offset + width, y_offset, height),          // 5: top front right
        Point3::new(x_offset + width, y_offset + depth, height),  // 6: top back right
        Point3::new(x_offset, y_offset + depth, height),          // 7: top back left
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
    
    // Draw origin indicator - XYZ axes at origin
    let origin_size = (width.min(depth).min(height) * 0.05).max(1.0);
    
    // Draw XYZ axes at origin
    window.draw_line(
        &Point3::new(0.0, 0.0, 0.0),
        &Point3::new(origin_size, 0.0, 0.0),
        &Point3::new(1.0, 0.0, 0.0), // Red for X
    );
    window.draw_line(
        &Point3::new(0.0, 0.0, 0.0),
        &Point3::new(0.0, origin_size, 0.0),
        &Point3::new(0.0, 1.0, 0.0), // Green for Y
    );
    window.draw_line(
        &Point3::new(0.0, 0.0, 0.0),
        &Point3::new(0.0, 0.0, origin_size),
        &Point3::new(0.0, 0.0, 1.0), // Blue for Z
    );
}

/// Calculate appropriate tick spacing based on dimension
fn calculate_tick_spacing(dimension: f32) -> f32 {
    if dimension <= 50.0 {
        5.0
    } else if dimension <= 200.0 {
        10.0
    } else if dimension <= 500.0 {
        25.0
    } else if dimension <= 1000.0 {
        50.0
    } else {
        100.0
    }
}

/// Draw ruler along the axes with tick marks and labels
fn draw_ruler(window: &mut Window, area: &PrintArea) {
    // Get dimensions in mm
    let width = area.width_mm();
    let height = area.height_mm();
    let depth = area.depth_mm();
    
    // Calculate offset based on origin mode
    let (x_offset, y_offset) = match area.origin {
        Origin::Corner => (0.0, 0.0),
        Origin::CenterBottom => (-width / 2.0, -depth / 2.0),
    };
    
    let tick_color = Point3::new(0.7, 0.7, 0.7);
    let major_tick_size = 2.0;
    let minor_tick_size = 1.0;
    
    // X-axis ruler
    let x_spacing = calculate_tick_spacing(width);
    let mut x = 0.0;
    while x <= width {
        let is_major = (x / x_spacing).round() % 5.0 < 0.1;
        let tick_size = if is_major { major_tick_size } else { minor_tick_size };
        
        let pos = Point3::new(x_offset + x, y_offset, 0.0);
        let tick_end = Point3::new(x_offset + x, y_offset - tick_size, 0.0);
        window.draw_line(&pos, &tick_end, &tick_color);
        
        x += x_spacing;
    }
    
    // Y-axis ruler
    let y_spacing = calculate_tick_spacing(depth);
    let mut y = 0.0;
    while y <= depth {
        let is_major = (y / y_spacing).round() % 5.0 < 0.1;
        let tick_size = if is_major { major_tick_size } else { minor_tick_size };
        
        let pos = Point3::new(x_offset, y_offset + y, 0.0);
        let tick_end = Point3::new(x_offset - tick_size, y_offset + y, 0.0);
        window.draw_line(&pos, &tick_end, &tick_color);
        
        y += y_spacing;
    }
    
    // Z-axis ruler
    let z_spacing = calculate_tick_spacing(height);
    let mut z = 0.0;
    while z <= height {
        let is_major = (z / z_spacing).round() % 5.0 < 0.1;
        let tick_size = if is_major { major_tick_size } else { minor_tick_size };
        
        let pos = Point3::new(x_offset, y_offset, z);
        let tick_end = Point3::new(x_offset - tick_size, y_offset, z);
        window.draw_line(&pos, &tick_end, &tick_color);
        
        z += z_spacing;
    }
}

/// Round a value to a "nice" number for display (1, 2, 5, 10, 20, 50, etc.)
fn round_to_nice_number(value: f32) -> f32 {
    let exponent = value.log10().floor();
    let fraction = value / 10f32.powf(exponent);
    
    let nice_fraction = if fraction < 1.5 {
        1.0
    } else if fraction < 3.0 {
        2.0
    } else if fraction < 7.0 {
        5.0
    } else {
        10.0
    };
    
    nice_fraction * 10f32.powf(exponent)
}

/// Draw a scale bar in the corner of the screen
/// Note: This is a simplified version - in a real implementation,
/// you'd need camera information to calculate screen-space properly
fn draw_scale_bar(window: &mut Window, area: &PrintArea) {
    // Use a fixed reference size based on the print bed
    let reference_size = round_to_nice_number(area.width_mm() * 0.1);
    
    // Draw the scale bar at a fixed position in the scene
    // In corner/bottom of print bed
    let (x_offset, y_offset) = match area.origin {
        Origin::Corner => (0.0, 0.0),
        Origin::CenterBottom => (-area.width_mm() / 2.0, -area.depth_mm() / 2.0),
    };
    
    let bar_start = Point3::new(x_offset + 10.0, y_offset + 10.0, 0.0);
    let bar_end = Point3::new(x_offset + 10.0 + reference_size, y_offset + 10.0, 0.0);
    let bar_color = Point3::new(1.0, 1.0, 0.0); // Yellow
    
    // Draw the main bar
    window.draw_line(&bar_start, &bar_end, &bar_color);
    
    // Draw tick marks at ends
    let tick_height = 2.0;
    window.draw_line(
        &bar_start,
        &Point3::new(bar_start.x, bar_start.y, bar_start.z + tick_height),
        &bar_color,
    );
    window.draw_line(
        &bar_end,
        &Point3::new(bar_end.x, bar_end.y, bar_end.z + tick_height),
        &bar_color,
    );
    
    // Note: Text rendering would require additional text rendering capabilities
    // which kiss3d doesn't provide by default. This would show as just the bar.
}

/// Render the model information panel
fn render_model_info_panel(window: &mut Window, state: &ViewerState) {
    if !state.info_panel.visible {
        return;
    }

    use kiss3d::text::Font;
    use kiss3d::nalgebra::Point2;
    
    const PANEL_X: f32 = 10.0;
    const PANEL_Y: f32 = 40.0; // Below menu bar
    const LINE_HEIGHT: f32 = 16.0;
    const FONT_SIZE: f32 = 13.0;
    const SECTION_SPACING: f32 = 8.0;
    
    let text_color = kiss3d::nalgebra::Point3::new(0.9, 0.9, 0.9);
    let header_color = kiss3d::nalgebra::Point3::new(1.0, 1.0, 0.6);
    
    let mut y = PANEL_Y;
    
    // Title
    window.draw_text(
        "Model Information",
        &Point2::new(PANEL_X, y),
        FONT_SIZE + 2.0,
        &Font::default(),
        &header_color,
    );
    y += LINE_HEIGHT + SECTION_SPACING;
    
    if let Some(ref model) = state.model {
        // File info
        if let Some(ref path) = state.file_path {
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            window.draw_text(
                &format!("File: {}", filename),
                &Point2::new(PANEL_X, y),
                FONT_SIZE,
                &Font::default(),
                &text_color,
            );
            y += LINE_HEIGHT;
            
            // File size
            if let Ok(metadata) = std::fs::metadata(path) {
                let size_mb = metadata.len() as f64 / 1_048_576.0;
                window.draw_text(
                    &format!("Size: {:.2} MB", size_mb),
                    &Point2::new(PANEL_X, y),
                    FONT_SIZE,
                    &Font::default(),
                    &text_color,
                );
                y += LINE_HEIGHT;
            }
        }
        
        y += SECTION_SPACING;
        
        // Geometry section
        window.draw_text(
            "Geometry",
            &Point2::new(PANEL_X, y),
            FONT_SIZE,
            &Font::default(),
            &header_color,
        );
        y += LINE_HEIGHT;
        
        let vertex_count = count_vertices(model);
        let triangle_count = count_triangles(model);
        let object_count = model.resources.objects.len();
        let component_count: usize = model.resources.objects.iter()
            .map(|obj| obj.components.len())
            .sum();
        
        window.draw_text(
            &format!("  Vertices: {}", vertex_count),
            &Point2::new(PANEL_X, y),
            FONT_SIZE,
            &Font::default(),
            &text_color,
        );
        y += LINE_HEIGHT;
        
        window.draw_text(
            &format!("  Triangles: {}", triangle_count),
            &Point2::new(PANEL_X, y),
            FONT_SIZE,
            &Font::default(),
            &text_color,
        );
        y += LINE_HEIGHT;
        
        window.draw_text(
            &format!("  Objects: {}", object_count),
            &Point2::new(PANEL_X, y),
            FONT_SIZE,
            &Font::default(),
            &text_color,
        );
        y += LINE_HEIGHT;
        
        window.draw_text(
            &format!("  Components: {}", component_count),
            &Point2::new(PANEL_X, y),
            FONT_SIZE,
            &Font::default(),
            &text_color,
        );
        y += LINE_HEIGHT;
        
        // Bounding box
        let (_min_bound, max_bound) = calculate_model_bounds(model);
        let size_x = max_bound.0 - _min_bound.0;
        let size_y = max_bound.1 - _min_bound.1;
        let size_z = max_bound.2 - _min_bound.2;
        
        window.draw_text(
            &format!("  Bounds: {:.1} × {:.1} × {:.1} {}", 
                size_x, size_y, size_z, model.unit),
            &Point2::new(PANEL_X, y),
            FONT_SIZE,
            &Font::default(),
            &text_color,
        );
        y += LINE_HEIGHT + SECTION_SPACING;
        
        // Extensions section
        if !model.required_extensions.is_empty() || 
           !model.resources.materials.is_empty() ||
           !model.resources.color_groups.is_empty() ||
           count_beams(model) > 0 ||
           !model.resources.slice_stacks.is_empty() {
            
            window.draw_text(
                "Extensions",
                &Point2::new(PANEL_X, y),
                FONT_SIZE,
                &Font::default(),
                &header_color,
            );
            y += LINE_HEIGHT;
            
            // Materials
            let material_count = model.resources.materials.len();
            let color_group_count = model.resources.color_groups.len();
            if material_count > 0 || color_group_count > 0 {
                window.draw_text(
                    &format!("  ✓ Materials ({} mats, {} groups)", 
                        material_count, color_group_count),
                    &Point2::new(PANEL_X, y),
                    FONT_SIZE,
                    &Font::default(),
                    &text_color,
                );
                y += LINE_HEIGHT;
            }
            
            // Beam Lattice
            let beam_count = count_beams(model);
            if beam_count > 0 {
                window.draw_text(
                    &format!("  ✓ Beam Lattice ({} beams)", beam_count),
                    &Point2::new(PANEL_X, y),
                    FONT_SIZE,
                    &Font::default(),
                    &text_color,
                );
                y += LINE_HEIGHT;
            }
            
            // Slice Stacks
            if !model.resources.slice_stacks.is_empty() {
                let slice_count: usize = model.resources.slice_stacks.iter()
                    .map(|s| s.slices.len())
                    .sum();
                window.draw_text(
                    &format!("  ✓ Slice ({} slices)", slice_count),
                    &Point2::new(PANEL_X, y),
                    FONT_SIZE,
                    &Font::default(),
                    &text_color,
                );
                y += LINE_HEIGHT;
            }
            
            // Production
            if !model.build.items.is_empty() {
                let production_items: usize = model.build.items.iter()
                    .filter(|item| item.production_uuid.is_some())
                    .count();
                if production_items > 0 {
                    window.draw_text(
                        &format!("  ✓ Production ({} items)", production_items),
                        &Point2::new(PANEL_X, y),
                        FONT_SIZE,
                        &Font::default(),
                        &text_color,
                    );
                    y += LINE_HEIGHT;
                }
            }
            
            y += SECTION_SPACING;
        }
        
        // Objects section (show first few)
        if !model.resources.objects.is_empty() {
            window.draw_text(
                "Objects",
                &Point2::new(PANEL_X, y),
                FONT_SIZE,
                &Font::default(),
                &header_color,
            );
            y += LINE_HEIGHT;
            
            for obj in model.resources.objects.iter().take(5) {
                let obj_name = if let Some(ref name) = obj.name {
                    name.clone()
                } else {
                    format!("Object {}", obj.id)
                };
                
                let obj_type = if obj.mesh.is_some() {
                    "mesh"
                } else if !obj.components.is_empty() {
                    "component"
                } else {
                    "other"
                };
                
                window.draw_text(
                    &format!("  {} ({})", obj_name, obj_type),
                    &Point2::new(PANEL_X, y),
                    FONT_SIZE - 1.0,
                    &Font::default(),
                    &text_color,
                );
                y += LINE_HEIGHT;
            }
            
            if model.resources.objects.len() > 5 {
                window.draw_text(
                    &format!("  ... and {} more", model.resources.objects.len() - 5),
                    &Point2::new(PANEL_X, y),
                    FONT_SIZE - 1.0,
                    &Font::default(),
                    &text_color,
                );
                // No need to increment y here since it's the last item
            }
        }
    } else {
        // No model loaded
        window.draw_text(
            "No model loaded",
            &Point2::new(PANEL_X, y),
            FONT_SIZE,
            &Font::default(),
            &text_color,
        );
        y += LINE_HEIGHT;
        window.draw_text(
            "Press Ctrl+O to open a file",
            &Point2::new(PANEL_X, y),
            FONT_SIZE,
            &Font::default(),
            &text_color,
        );
    }
}

/// Print the menu with current settings
/// Print menu information (kept for potential future use)
#[allow(dead_code)]
fn print_menu(state: &ViewerState) {
    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!("  Menu - Current Settings");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("  Theme:           {}", state.theme.name());
    println!(
        "  Print Area:      {}",
        if state.print_area.visible {
            "ON"
        } else {
            "OFF"
        }
    );
    println!(
        "    Width (X):     {} {}",
        state.print_area.width, state.print_area.unit.as_str()
    );
    println!(
        "    Depth (Y):     {} {}",
        state.print_area.depth, state.print_area.unit.as_str()
    );
    println!(
        "    Height (Z):    {} {}",
        state.print_area.height, state.print_area.unit.as_str()
    );
    
    // Show displacement status if data is present
    if let Some(ref model) = state.model {
        if has_displacement_data(model) {
            println!(
                "  Displacement:    {}",
                if state.show_displacement {
                    "ON"
                } else {
                    "OFF"
                }
            );
            let (maps, _, disp_groups) = count_displacement_resources(model);
            let disp_objects = count_displacement_objects(model);
            println!("    Maps:          {}", maps);
            println!("    Groups:        {}", disp_groups);
            println!("    Objects:       {}", disp_objects);
        }
    }
    
    // Show material rendering status
    println!(
        "  Materials:       {}",
        if state.show_materials { "ON" } else { "OFF (default gray)" }
    );
    
    if let Some(ref path) = state.file_path {
        println!(
            "  File:            {}",
            path.file_name().unwrap_or_default().to_string_lossy()
        );
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

    // Show printer presets
    println!("\nPrinter Presets:");
    let presets = printer_presets();
    for (i, preset) in presets.iter().enumerate() {
        println!("  {}. {} ({}x{}x{} mm)", i + 1, preset.name, preset.width, preset.depth, preset.height);
    }
    print!("\nSelect preset (1-{}) or press Enter to customize [custom]: ", presets.len());
    io::stdout().flush()?;
    let preset_input = read_line()?;
    
    if !preset_input.is_empty() {
        if let Ok(preset_idx) = preset_input.parse::<usize>() {
            if preset_idx > 0 && preset_idx <= presets.len() {
                let preset = &presets[preset_idx - 1];
                // Convert from mm to current unit
                new_area.width = current.unit.from_mm(preset.width);
                new_area.depth = current.unit.from_mm(preset.depth);
                new_area.height = current.unit.from_mm(preset.height);
                println!("✓ Applied {} preset", preset.name);
                
                // If not "Custom", return early
                if preset.name != "Custom" {
                    return Ok(new_area);
                }
            }
        }
    }

    // Configure dimensions
    new_area.width = read_dimension("Enter width (X)", current.width, current.unit.as_str())?;
    new_area.depth = read_dimension("Enter depth (Y)", current.depth, current.unit.as_str())?;
    new_area.height = read_dimension("Enter height (Z)", current.height, current.unit.as_str())?;

    // Configure unit with validation
    print!("Enter unit (mm/inch) [{}]: ", current.unit.as_str());
    io::stdout().flush()?;
    let input = read_line()?;
    if !input.is_empty() {
        if let Some(unit) = LengthUnit::from_str(&input) {
            // Convert dimensions to new unit if changed
            if unit != current.unit {
                let width_mm = current.unit.to_mm(new_area.width);
                let depth_mm = current.unit.to_mm(new_area.depth);
                let height_mm = current.unit.to_mm(new_area.height);
                
                new_area.width = unit.from_mm(width_mm);
                new_area.depth = unit.from_mm(depth_mm);
                new_area.height = unit.from_mm(height_mm);
                new_area.unit = unit;
                
                println!("✓ Converted to {}", unit.as_str());
            }
        } else {
            println!("Warning: Unknown unit '{}', keeping '{}'", input, current.unit.as_str());
        }
    }
    
    // Configure origin
    print!("Origin mode (corner/center) [{}]: ", current.origin.as_str());
    io::stdout().flush()?;
    let input = read_line()?;
    if !input.is_empty() {
        match input.to_lowercase().as_str() {
            "corner" | "c" => new_area.origin = Origin::Corner,
            "center" | "centre" | "ctr" => new_area.origin = Origin::CenterBottom,
            _ => println!("Warning: Unknown origin mode '{}', keeping '{}'", input, current.origin.as_str()),
        }
    }
    
    // Configure visibility options
    print!("Show ruler (y/n) [{}]: ", if current.show_ruler { "y" } else { "n" });
    io::stdout().flush()?;
    let input = read_line()?;
    if !input.is_empty() {
        new_area.show_ruler = matches!(input.to_lowercase().as_str(), "y" | "yes" | "true" | "1");
    }
    
    print!("Show scale bar (y/n) [{}]: ", if current.show_scale_bar { "y" } else { "n" });
    io::stdout().flush()?;
    let input = read_line()?;
    if !input.is_empty() {
        new_area.show_scale_bar = matches!(input.to_lowercase().as_str(), "y" | "yes" | "true" | "1");
    }

    Ok(new_area)
}

/// Compute intersection of a line segment with a horizontal plane at z_height
/// Returns the intersection point in 2D (x, y) if it exists
fn line_plane_intersection(
    p1: (f32, f32, f32),
    p2: (f32, f32, f32),
    z_height: f32,
) -> Option<Point2D> {
    let (x1, y1, z1) = p1;
    let (x2, y2, z2) = p2;

    // Check if line segment crosses the plane
    if (z1 <= z_height && z2 <= z_height) || (z1 >= z_height && z2 >= z_height) {
        return None; // Both points on same side of plane
    }

    // Calculate intersection parameter t
    let t = (z_height - z1) / (z2 - z1);
    
    // Calculate intersection point
    let x = x1 + t * (x2 - x1);
    let y = y1 + t * (y2 - y1);

    Some(Point2D { x, y })
}

/// Compute the intersection of a triangle with a horizontal plane at z_height
/// Returns a line segment (0, 1, or 2 points - if 2 points, they form a line segment)
fn triangle_plane_intersection(
    v1: (f32, f32, f32),
    v2: (f32, f32, f32),
    v3: (f32, f32, f32),
    z_height: f32,
) -> Option<LineSegment2D> {
    let mut intersections = Vec::new();

    // Check each edge of the triangle
    if let Some(p) = line_plane_intersection(v1, v2, z_height) {
        intersections.push(p);
    }
    if let Some(p) = line_plane_intersection(v2, v3, z_height) {
        intersections.push(p);
    }
    if let Some(p) = line_plane_intersection(v3, v1, z_height) {
        intersections.push(p);
    }

    // A triangle can intersect a plane at 0, 1 (vertex touch), or 2 points (edge crossing)
    if intersections.len() == 2 {
        Some(LineSegment2D {
            start: intersections[0],
            end: intersections[1],
        })
    } else {
        None
    }
}

/// Compute beam-plane intersection for a cylindrical beam
/// Returns a circle in 2D if the beam crosses the Z plane
/// 
/// # Arguments
/// * `p1` - First endpoint of the beam (x, y, z)
/// * `p2` - Second endpoint of the beam (x, y, z)
/// * `r1` - Radius at first endpoint (must be positive)
/// * `r2` - Radius at second endpoint (must be positive)
/// * `z_height` - Z coordinate of the plane
fn beam_plane_intersection(
    p1: (f32, f32, f32),
    p2: (f32, f32, f32),
    r1: f32,
    r2: f32,
    z_height: f32,
) -> Option<(Point2D, f32)> {
    let (x1, y1, z1) = p1;
    let (x2, y2, z2) = p2;

    // Validate radii
    if r1 <= 0.0 || r2 <= 0.0 {
        return None; // Invalid radius
    }

    // Check if beam crosses Z plane (endpoints on different sides)
    if (z1 - z_height) * (z2 - z_height) > 0.0 {
        return None; // Both endpoints on same side
    }

    // Handle edge case where beam is exactly on the plane
    // Use epsilon appropriate for f32 precision
    let epsilon = 1e-5;
    if (z1 - z_height).abs() < epsilon && (z2 - z_height).abs() < epsilon {
        return None; // Beam lies in plane - degenerate case
    }

    // Find intersection point along beam axis
    let t = (z_height - z1) / (z2 - z1);
    
    // Clamp t to [0, 1] to handle numerical precision issues
    let t = t.clamp(0.0, 1.0);
    
    let center_x = x1 + t * (x2 - x1);
    let center_y = y1 + t * (y2 - y1);

    // Interpolate radius for tapered beams
    let radius = r1 + t * (r2 - r1);

    Some((Point2D { x: center_x, y: center_y }, radius))
}

/// Compute ball-plane intersection for a spherical ball joint
/// Returns a circle in 2D if the sphere intersects the Z plane
///
/// # Arguments
/// * `center` - Center of the sphere (x, y, z)
/// * `radius` - Radius of the sphere (must be positive)
/// * `z_height` - Z coordinate of the plane
fn ball_plane_intersection(
    center: (f32, f32, f32),
    radius: f32,
    z_height: f32,
) -> Option<(Point2D, f32)> {
    let (x, y, z) = center;

    // Validate radius
    if radius <= 0.0 {
        return None; // Invalid radius
    }
    
    let dz = (z - z_height).abs();
    
    if dz > radius {
        return None; // Plane doesn't intersect sphere
    }

    // Circle radius at slice height (from sphere geometry: r^2 = r_slice^2 + dz^2)
    let slice_radius = (radius * radius - dz * dz).sqrt();

    Some((Point2D { x, y }, slice_radius))
}

/// Convert a circle to a polygon approximation with specified number of segments
/// 
/// # Arguments
/// * `center` - Center of the circle
/// * `radius` - Radius of the circle (should be positive)
/// * `segments` - Number of segments (must be >= 3)
fn circle_to_line_segments(center: Point2D, radius: f32, segments: u32) -> Vec<LineSegment2D> {
    // Validate input
    if segments < 3 {
        // Return a minimal triangle for degenerate cases
        return Vec::new();
    }

    let mut line_segments = Vec::with_capacity(segments as usize);
    let two_pi = 2.0 * std::f32::consts::PI;
    
    for i in 0..segments {
        let angle1 = two_pi * (i as f32) / (segments as f32);
        let angle2 = two_pi * ((i + 1) as f32) / (segments as f32);
        
        let p1 = Point2D {
            x: center.x + radius * angle1.cos(),
            y: center.y + radius * angle1.sin(),
        };
        let p2 = Point2D {
            x: center.x + radius * angle2.cos(),
            y: center.y + radius * angle2.sin(),
        };
        
        line_segments.push(LineSegment2D { start: p1, end: p2 });
    }
    
    line_segments
}

/// Compute all slice contours for a model at a given Z height
fn compute_slice_contours(model: &Model, z_height: f32) -> Vec<LineSegment2D> {
    let mut segments = Vec::new();

    // Iterate through all build items
    for item in &model.build.items {
        if let Some(obj) = model
            .resources
            .objects
            .iter()
            .find(|o| o.id == item.objectid)
        {
            if let Some(ref mesh) = obj.mesh {
                // For each triangle, check if it intersects the Z plane
                for triangle in &mesh.triangles {
                    if triangle.v1 >= mesh.vertices.len()
                        || triangle.v2 >= mesh.vertices.len()
                        || triangle.v3 >= mesh.vertices.len()
                    {
                        continue; // Skip invalid triangles
                    }

                    let v1 = &mesh.vertices[triangle.v1];
                    let v2 = &mesh.vertices[triangle.v2];
                    let v3 = &mesh.vertices[triangle.v3];

                    let p1 = (v1.x as f32, v1.y as f32, v1.z as f32);
                    let p2 = (v2.x as f32, v2.y as f32, v2.z as f32);
                    let p3 = (v3.x as f32, v3.y as f32, v3.z as f32);

                    if let Some(segment) = triangle_plane_intersection(p1, p2, p3, z_height) {
                        segments.push(segment);
                    }
                }

                // Process beam lattices (NEW)
                if let Some(ref beamset) = mesh.beamset {
                    // Process beams
                    for beam in &beamset.beams {
                        // Validate vertex indices
                        if beam.v1 >= mesh.vertices.len() || beam.v2 >= mesh.vertices.len() {
                            continue; // Skip invalid beams
                        }

                        let v1 = &mesh.vertices[beam.v1];
                        let v2 = &mesh.vertices[beam.v2];

                        let p1 = (v1.x as f32, v1.y as f32, v1.z as f32);
                        let p2 = (v2.x as f32, v2.y as f32, v2.z as f32);

                        // Get beam radii (with fallbacks to beamset defaults)
                        let r1 = beam.r1.unwrap_or(beamset.radius) as f32;
                        let r2 = beam.r2.or(beam.r1).unwrap_or(beamset.radius) as f32;

                        if let Some((center, radius)) = beam_plane_intersection(p1, p2, r1, r2, z_height) {
                            // Convert circle to polygon segments
                            segments.extend(circle_to_line_segments(center, radius, CIRCLE_APPROXIMATION_SEGMENTS));
                        }
                    }

                    // Process ball joints (if present)
                    for ball in &beamset.balls {
                        // Validate vertex index
                        if ball.vindex >= mesh.vertices.len() {
                            continue; // Skip invalid balls
                        }

                        let vertex = &mesh.vertices[ball.vindex];
                        let center = (vertex.x as f32, vertex.y as f32, vertex.z as f32);

                        // Get ball radius (with fallback to beamset ball_radius or default radius)
                        let radius = ball.radius
                            .or(beamset.ball_radius)
                            .unwrap_or(beamset.radius) as f32;

                        if let Some((center_2d, slice_radius)) = ball_plane_intersection(center, radius, z_height) {
                            // Convert circle to polygon segments
                            segments.extend(circle_to_line_segments(center_2d, slice_radius, CIRCLE_APPROXIMATION_SEGMENTS));
                        }
                    }
                }
            }
        }
    }

    segments
}

/// Draw slice plane in 3D view
fn draw_slice_plane(window: &mut Window, slice_view: &SliceView, model_bounds: ((f32, f32, f32), (f32, f32, f32))) {
    let (min_bound, max_bound) = model_bounds;
    let z = slice_view.z_height;
    
    // Define corners of the plane slightly larger than model bounds
    let margin = 10.0;
    let x_min = min_bound.0 - margin;
    let x_max = max_bound.0 + margin;
    let y_min = min_bound.1 - margin;
    let y_max = max_bound.1 + margin;

    // Draw plane outline as 4 lines
    let color = Point3::new(1.0, 1.0, 0.0); // Yellow color for slice plane
    
    window.draw_line(
        &Point3::new(x_min, y_min, z),
        &Point3::new(x_max, y_min, z),
        &color,
    );
    window.draw_line(
        &Point3::new(x_max, y_min, z),
        &Point3::new(x_max, y_max, z),
        &color,
    );
    window.draw_line(
        &Point3::new(x_max, y_max, z),
        &Point3::new(x_min, y_max, z),
        &color,
    );
    window.draw_line(
        &Point3::new(x_min, y_max, z),
        &Point3::new(x_min, y_min, z),
        &color,
    );
}

/// Draw slice contours in 3D view at the slice plane
fn draw_slice_contours(window: &mut Window, slice_view: &SliceView) {
    let z = slice_view.z_height;
    let color = Point3::new(1.0, 0.0, 0.0); // Red color for contour lines

    for segment in &slice_view.contours {
        window.draw_line(
            &Point3::new(segment.start.x, segment.start.y, z),
            &Point3::new(segment.end.x, segment.end.y, z),
            &color,
        );
    }
}

/// Draw a single slice from the slice stack at current index
fn draw_slice_stack_single(window: &mut Window, slice_view: &SliceView, stack: &lib3mf::model::SliceStack) {
    use lib3mf::model::Vertex2D;
    
    // Early return if slice stack is empty or index is out of bounds
    if stack.slices.is_empty() || slice_view.current_slice_index >= stack.slices.len() {
        return;
    }
    
    let slice = &stack.slices[slice_view.current_slice_index];
    let z = slice.ztop as f32;
    
    // Color based on position in stack (gradient from blue to red)
    let t = slice_view.current_slice_index as f32 / stack.slices.len() as f32;
    let color = Point3::new(t, 0.0, 1.0 - t);
    
    // Draw each polygon in the slice
    for polygon in &slice.polygons {
        if polygon.startv >= slice.vertices.len() {
            continue;
        }
        
        // Build the polygon vertices
        let mut vertices: Vec<&Vertex2D> = Vec::new();
        vertices.push(&slice.vertices[polygon.startv]);
        
        for segment in &polygon.segments {
            if segment.v2 < slice.vertices.len() {
                vertices.push(&slice.vertices[segment.v2]);
            }
        }
        
        // Draw polygon edges
        for i in 0..vertices.len() {
            let v1 = vertices[i];
            let v2 = vertices[(i + 1) % vertices.len()];
            
            window.draw_line(
                &Point3::new(v1.x as f32, v1.y as f32, z),
                &Point3::new(v2.x as f32, v2.y as f32, z),
                &color,
            );
        }
        
        // If filled mode, draw filled triangles (simple fan triangulation)
        if slice_view.filled_mode && vertices.len() >= 3 {
            // Note: kiss3d doesn't have a simple filled polygon primitive
            // So we'll just draw more lines to create a denser visualization
            let center_x = vertices.iter().map(|v| v.x as f32).sum::<f32>() / vertices.len() as f32;
            let center_y = vertices.iter().map(|v| v.y as f32).sum::<f32>() / vertices.len() as f32;
            let center = Point3::new(center_x, center_y, z);
            
            for v in vertices.iter() {
                window.draw_line(
                    &Point3::new(v.x as f32, v.y as f32, z),
                    &center,
                    &color,
                );
            }
        }
    }
}

/// Draw all slices in the stack in 3D
fn draw_slice_stack_3d(window: &mut Window, slice_view: &SliceView, stack: &lib3mf::model::SliceStack) {
    use lib3mf::model::Vertex2D;
    
    // Early return if slice stack is empty
    if stack.slices.is_empty() {
        return;
    }
    
    for (slice_idx, slice) in stack.slices.iter().enumerate() {
        // Apply spread factor
        let z = slice.ztop as f32 * slice_view.spread_factor;
        
        // Color gradient from blue (bottom) to red (top)
        let t = slice_idx as f32 / stack.slices.len() as f32;
        let color = Point3::new(t, 0.0, 1.0 - t);
        
        // Highlight current slice with brighter color
        let color = if slice_idx == slice_view.current_slice_index {
            Point3::new(1.0, 1.0, 0.0) // Yellow for current slice
        } else {
            color
        };
        
        // Draw each polygon in the slice
        for polygon in &slice.polygons {
            if polygon.startv >= slice.vertices.len() {
                continue;
            }
            
            // Build the polygon vertices
            let mut vertices: Vec<&Vertex2D> = Vec::new();
            vertices.push(&slice.vertices[polygon.startv]);
            
            for segment in &polygon.segments {
                if segment.v2 < slice.vertices.len() {
                    vertices.push(&slice.vertices[segment.v2]);
                }
            }
            
            // Draw polygon edges
            for i in 0..vertices.len() {
                let v1 = vertices[i];
                let v2 = vertices[(i + 1) % vertices.len()];
                
                window.draw_line(
                    &Point3::new(v1.x as f32, v1.y as f32, z),
                    &Point3::new(v2.x as f32, v2.y as f32, z),
                    &color,
                );
            }
        }
    }
}


/// Export slice view to PNG file
fn export_slice_to_png(
    slice_view: &SliceView,
    model_bounds: ((f32, f32, f32), (f32, f32, f32)),
    unit: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use image::{ImageBuffer, Rgb, RgbImage};

    if slice_view.contours.is_empty() {
        println!("No contours to export at Z = {}", slice_view.z_height);
        return Ok(());
    }

    let (min_bound, max_bound) = model_bounds;
    
    // Calculate bounds for the 2D view
    let x_min = min_bound.0;
    let x_max = max_bound.0;
    let y_min = min_bound.1;
    let y_max = max_bound.1;
    
    let width_units = x_max - x_min;
    let height_units = y_max - y_min;
    
    // Image dimensions (pixels) - scale to a reasonable size
    let scale = 10.0; // pixels per unit
    let img_width = (width_units * scale).max(100.0) as u32;
    let img_height = (height_units * scale).max(100.0) as u32;
    
    // Create white background image
    let mut img: RgbImage = ImageBuffer::from_pixel(img_width, img_height, Rgb([255, 255, 255]));
    
    // Draw grid lines
    let grid_color = Rgb([220, 220, 220]);
    let grid_spacing = 10.0; // units
    
    // Vertical grid lines
    let mut x = (x_min / grid_spacing).ceil() * grid_spacing;
    while x <= x_max {
        let px = ((x - x_min) * scale) as u32;
        if px < img_width {
            for py in 0..img_height {
                if let Some(pixel) = img.get_pixel_mut_checked(px, py) {
                    *pixel = grid_color;
                }
            }
        }
        x += grid_spacing;
    }
    
    // Horizontal grid lines
    let mut y = (y_min / grid_spacing).ceil() * grid_spacing;
    while y <= y_max {
        let py = img_height - ((y - y_min) * scale) as u32 - 1;
        if py < img_height {
            for px in 0..img_width {
                if let Some(pixel) = img.get_pixel_mut_checked(px, py) {
                    *pixel = grid_color;
                }
            }
        }
        y += grid_spacing;
    }
    
    // Draw contour lines in red
    let line_color = Rgb([255, 0, 0]);
    for segment in &slice_view.contours {
        let x1 = ((segment.start.x - x_min) * scale) as i32;
        let y1 = img_height as i32 - ((segment.start.y - y_min) * scale) as i32 - 1;
        let x2 = ((segment.end.x - x_min) * scale) as i32;
        let y2 = img_height as i32 - ((segment.end.y - y_min) * scale) as i32 - 1;
        
        // Simple line drawing using Bresenham's algorithm
        draw_line(&mut img, x1, y1, x2, y2, line_color);
    }
    
    // Generate filename with Z height
    let now = chrono::Local::now();
    let filename = format!(
        "slice_z_{:.2}{unit}_{}.png",
        slice_view.z_height,
        now.format("%Y%m%d_%H%M%S")
    );
    
    // Save image
    img.save(&filename)?;
    println!("\n✓ Slice exported to: {}", filename);
    println!("  Z height: {} {}", slice_view.z_height, unit);
    println!("  Contours: {} segments", slice_view.contours.len());
    
    Ok(())
}

/// Simple line drawing using Bresenham's algorithm
fn draw_line(img: &mut RgbImage, x1: i32, y1: i32, x2: i32, y2: i32, color: Rgb<u8>) {
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut x = x1;
    let mut y = y1;

    let (width, height) = img.dimensions();

    loop {
        // Draw pixel if within bounds
        if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
            if let Some(pixel) = img.get_pixel_mut_checked(x as u32, y as u32) {
                *pixel = color;
            }
        }

        if x == x2 && y == y2 {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
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
        assert!(matches!(area.unit, LengthUnit::Millimeters));
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

    #[test]
    fn test_create_camera_for_empty_scene() {
        // Test camera creation for empty scene
        let camera = create_camera_for_model(None);
        
        // Should create a default camera
        assert!(camera.dist() > 0.0, "Camera distance should be positive");
    }

    #[test]
    fn test_zoom_camera() {
        // Create a default camera
        let mut camera = create_camera_for_model(None);
        let initial_dist = camera.dist();
        
        // Zoom in
        zoom_camera(&mut camera, ZOOM_STEP);
        assert!(camera.dist() < initial_dist, "Zoom in should decrease distance");
        
        // Zoom out
        zoom_camera(&mut camera, 1.0 / ZOOM_STEP);
        assert!((camera.dist() - initial_dist).abs() < 0.01, "Zoom in/out should be reversible");
    }

    #[test]
    fn test_pan_camera() {
        // Create a default camera
        let mut camera = create_camera_for_model(None);
        let initial_at = camera.at();
        
        // Pan right (positive X)
        pan_camera(&mut camera, 1.0, 0.0, 0.0);
        let after_pan = camera.at();
        
        assert!(after_pan.x > initial_at.x, "Pan right should increase X coordinate");
        assert_eq!(after_pan.y, initial_at.y, "Pan right should not change Y");
        assert_eq!(after_pan.z, initial_at.z, "Pan right should not change Z");
    }

    #[test]
    fn test_beam_plane_intersection_crossing() {
        // Beam crossing Z plane at z=5
        let p1 = (0.0, 0.0, 0.0);
        let p2 = (10.0, 10.0, 10.0);
        let r1 = 2.0;
        let r2 = 4.0;
        let z_height = 5.0;
        
        let result = beam_plane_intersection(p1, p2, r1, r2, z_height);
        assert!(result.is_some(), "Beam crossing plane should intersect");
        
        let (center, radius) = result.unwrap();
        // At z=5, t should be 0.5
        assert!((center.x - 5.0).abs() < 0.001, "X should be at midpoint");
        assert!((center.y - 5.0).abs() < 0.001, "Y should be at midpoint");
        assert!((radius - 3.0).abs() < 0.001, "Radius should be interpolated to 3.0");
    }

    #[test]
    fn test_beam_plane_intersection_invalid_radius() {
        // Test negative radius
        let p1 = (0.0, 0.0, 0.0);
        let p2 = (10.0, 10.0, 10.0);
        let z_height = 5.0;
        
        let result = beam_plane_intersection(p1, p2, -1.0, 2.0, z_height);
        assert!(result.is_none(), "Negative radius should return None");
        
        let result = beam_plane_intersection(p1, p2, 2.0, 0.0, z_height);
        assert!(result.is_none(), "Zero radius should return None");
    }

    #[test]
    fn test_beam_plane_intersection_no_crossing() {
        // Beam entirely above Z plane
        let p1 = (0.0, 0.0, 10.0);
        let p2 = (10.0, 10.0, 20.0);
        let r1 = 2.0;
        let r2 = 4.0;
        let z_height = 5.0;
        
        let result = beam_plane_intersection(p1, p2, r1, r2, z_height);
        assert!(result.is_none(), "Beam not crossing plane should not intersect");
    }

    #[test]
    fn test_beam_plane_intersection_constant_radius() {
        // Beam with constant radius
        let p1 = (0.0, 0.0, 0.0);
        let p2 = (0.0, 0.0, 10.0);
        let r1 = 2.5;
        let r2 = 2.5;
        let z_height = 7.0;
        
        let result = beam_plane_intersection(p1, p2, r1, r2, z_height);
        assert!(result.is_some(), "Vertical beam should intersect");
        
        let (center, radius) = result.unwrap();
        assert!((center.x - 0.0).abs() < 0.001, "X should be at beam center");
        assert!((center.y - 0.0).abs() < 0.001, "Y should be at beam center");
        assert!((radius - 2.5).abs() < 0.001, "Radius should remain constant");
    }

    #[test]
    fn test_ball_plane_intersection_crossing() {
        // Ball centered at z=5 with radius 3
        let center = (10.0, 20.0, 5.0);
        let radius = 3.0;
        let z_height = 6.0; // 1 unit above center
        
        let result = ball_plane_intersection(center, radius, z_height);
        assert!(result.is_some(), "Ball should intersect plane");
        
        let (center_2d, slice_radius) = result.unwrap();
        assert!((center_2d.x - 10.0).abs() < 0.001, "X should match ball center");
        assert!((center_2d.y - 20.0).abs() < 0.001, "Y should match ball center");
        
        // At dz=1, slice_radius = sqrt(3^2 - 1^2) = sqrt(8) ≈ 2.828
        let expected_radius = (radius * radius - 1.0 * 1.0).sqrt();
        assert!((slice_radius - expected_radius).abs() < 0.001, "Slice radius should be sqrt(8)");
    }

    #[test]
    fn test_ball_plane_intersection_no_crossing() {
        // Ball far from plane
        let center = (10.0, 20.0, 5.0);
        let radius = 2.0;
        let z_height = 10.0; // 5 units above center
        
        let result = ball_plane_intersection(center, radius, z_height);
        assert!(result.is_none(), "Ball should not intersect distant plane");
    }

    #[test]
    fn test_ball_plane_intersection_invalid_radius() {
        // Test negative and zero radius
        let center = (10.0, 20.0, 5.0);
        let z_height = 5.0;
        
        let result = ball_plane_intersection(center, -1.0, z_height);
        assert!(result.is_none(), "Negative radius should return None");
        
        let result = ball_plane_intersection(center, 0.0, z_height);
        assert!(result.is_none(), "Zero radius should return None");
    }

    #[test]
    fn test_ball_plane_intersection_at_center() {
        // Plane passing through ball center
        let center = (10.0, 20.0, 5.0);
        let radius = 3.0;
        let z_height = 5.0;
        
        let result = ball_plane_intersection(center, radius, z_height);
        assert!(result.is_some(), "Ball should intersect plane at center");
        
        let (_, slice_radius) = result.unwrap();
        assert!((slice_radius - radius).abs() < 0.001, "Slice radius should equal ball radius");
    }

    #[test]
    fn test_circle_to_line_segments() {
        let center = Point2D { x: 10.0, y: 20.0 };
        let radius = 5.0;
        let segments = 8;
        
        let line_segs = circle_to_line_segments(center, radius, segments);
        
        assert_eq!(line_segs.len(), segments as usize, "Should have correct number of segments");
        
        // Check that all segments are connected (end of one is start of next)
        for i in 0..segments as usize {
            let current = &line_segs[i];
            let next = &line_segs[(i + 1) % segments as usize];
            
            assert!((current.end.x - next.start.x).abs() < 0.001, "Segments should be connected");
            assert!((current.end.y - next.start.y).abs() < 0.001, "Segments should be connected");
        }
        
        // Check that points are approximately at the right distance from center
        for seg in &line_segs {
            let dx_start = seg.start.x - center.x;
            let dy_start = seg.start.y - center.y;
            let dist_start = (dx_start * dx_start + dy_start * dy_start).sqrt();
            
            assert!((dist_start - radius).abs() < 0.001, "Start point should be at radius distance");
        }
    }

    #[test]
    fn test_compute_slice_with_beam_lattice() {
        use std::fs::File;
        
        // Load the pyramid beam lattice test file
        let file = File::open("../../test_files/beam_lattice/pyramid.3mf")
            .expect("Failed to open pyramid.3mf test file");
        let model = Model::from_reader(file).expect("Failed to parse pyramid.3mf");
        
        // Compute slices at different heights
        let z_heights = [0.0, 25.0, 50.0, 75.0, 100.0];
        
        for z_height in z_heights {
            let segments = compute_slice_contours(&model, z_height);
            
            // At each height, we should have some segments from beams crossing
            // The exact number depends on the beam lattice structure
            // For z > 0 and z < 100, we expect some beam intersections
            if z_height > 0.0 && z_height < 100.0 {
                assert!(segments.len() > 0, 
                    "Expected beam lattice slices at z={}, but got {} segments", 
                    z_height, segments.len());
            }
        }
    }
    
    #[test]
    fn test_length_unit_conversions() {
        // Test mm to mm
        assert!((LengthUnit::Millimeters.to_mm(100.0) - 100.0).abs() < 0.01);
        assert!((LengthUnit::Millimeters.from_mm(100.0) - 100.0).abs() < 0.01);
        
        // Test inches to mm
        assert!((LengthUnit::Inches.to_mm(1.0) - 25.4).abs() < 0.01);
        assert!((LengthUnit::Inches.from_mm(25.4) - 1.0).abs() < 0.01);
        
        // Test round trip conversion
        let original = 100.0;
        let converted = LengthUnit::Inches.to_mm(original);
        let back = LengthUnit::Inches.from_mm(converted);
        assert!((back - original).abs() < 0.01);
    }
    
    #[test]
    fn test_print_area_dimensions() {
        let mut area = PrintArea::new();
        
        // Test default values
        assert_eq!(area.width, 200.0);
        assert_eq!(area.depth, 200.0);
        assert_eq!(area.height, 200.0);
        assert!(matches!(area.unit, LengthUnit::Millimeters));
        assert!(matches!(area.origin, Origin::Corner));
        
        // Test width_mm, depth_mm, height_mm
        assert!((area.width_mm() - 200.0).abs() < 0.01);
        assert!((area.depth_mm() - 200.0).abs() < 0.01);
        assert!((area.height_mm() - 200.0).abs() < 0.01);
        
        // Test with inches
        area.unit = LengthUnit::Inches;
        area.width = 10.0;
        area.depth = 8.0;
        area.height = 10.0;
        
        assert!((area.width_mm() - 254.0).abs() < 0.1);
        assert!((area.depth_mm() - 203.2).abs() < 0.1);
        assert!((area.height_mm() - 254.0).abs() < 0.1);
    }
    
    #[test]
    fn test_printer_presets() {
        let presets = printer_presets();
        
        // Check we have expected presets
        assert!(presets.len() >= 4);
        
        // Check Prusa MK3S+ preset
        let prusa = presets.iter().find(|p| p.name == "Prusa MK3S+").unwrap();
        assert_eq!(prusa.width, 250.0);
        assert_eq!(prusa.depth, 210.0);
        assert_eq!(prusa.height, 210.0);
        
        // Check Ender 3 preset
        let ender = presets.iter().find(|p| p.name == "Ender 3").unwrap();
        assert_eq!(ender.width, 220.0);
        assert_eq!(ender.depth, 220.0);
        assert_eq!(ender.height, 250.0);
        
        // Check Bambu Lab X1 preset
        let bambu = presets.iter().find(|p| p.name == "Bambu Lab X1").unwrap();
        assert_eq!(bambu.width, 256.0);
        assert_eq!(bambu.depth, 256.0);
        assert_eq!(bambu.height, 256.0);
    }
    
    #[test]
    fn test_calculate_tick_spacing() {
        assert_eq!(calculate_tick_spacing(30.0), 5.0);
        assert_eq!(calculate_tick_spacing(100.0), 10.0);
        assert_eq!(calculate_tick_spacing(300.0), 25.0);
        assert_eq!(calculate_tick_spacing(700.0), 50.0);
        assert_eq!(calculate_tick_spacing(1500.0), 100.0);
    }
    
    #[test]
    fn test_round_to_nice_number() {
        // Test various inputs round to nice numbers
        assert!((round_to_nice_number(12.5) - 10.0).abs() < 0.01);
        assert!((round_to_nice_number(23.0) - 20.0).abs() < 0.01);
        assert!((round_to_nice_number(47.0) - 50.0).abs() < 0.01);
        assert!((round_to_nice_number(89.0) - 100.0).abs() < 0.01);
    }
    
    #[test]
    fn test_print_area_toggles() {
        let mut area = PrintArea::new();
        
        // Test visibility toggle
        assert!(area.visible);
        area.toggle_visibility();
        assert!(!area.visible);
        area.toggle_visibility();
        assert!(area.visible);
        
        // Test ruler toggle
        assert!(!area.show_ruler);
        area.toggle_ruler();
        assert!(area.show_ruler);
        area.toggle_ruler();
        assert!(!area.show_ruler);
        
        // Test scale bar toggle
        assert!(!area.show_scale_bar);
        area.toggle_scale_bar();
        assert!(area.show_scale_bar);
        area.toggle_scale_bar();
        assert!(!area.show_scale_bar);
    }
}
