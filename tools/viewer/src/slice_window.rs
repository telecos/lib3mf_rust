//! Slice Preview Window - Secondary 2D window for live slice preview
//!
//! This module provides a separate 2D window that displays live slice previews
//! of the 3D model, updating in real-time as the user adjusts the Z height.

#![forbid(unsafe_code)]

use minifb::{Key, Window, WindowOptions};
use std::time::Instant;

/// Width of the slice preview window in pixels
const WINDOW_WIDTH: usize = 800;
/// Height of the slice preview window in pixels
const WINDOW_HEIGHT: usize = 600;
/// Background color (light gray)
const BG_COLOR: u32 = 0x00F0F0F0;
/// Grid color (medium gray)
const GRID_COLOR: u32 = 0x00C0C0C0;
/// Contour line color (red)
const CONTOUR_COLOR: u32 = 0x00FF0000;
/// Text color (dark gray)
const TEXT_COLOR: u32 = 0x00202020;
/// UI panel background (white)
const PANEL_BG_COLOR: u32 = 0x00FFFFFF;

/// 2D point for rendering
#[derive(Debug, Clone, Copy)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

/// Line segment for slice contours
#[derive(Debug, Clone, Copy)]
pub struct LineSegment2D {
    pub start: Point2D,
    pub end: Point2D,
}

/// Configuration for slice preview rendering
#[derive(Debug, Clone)]
pub struct SliceConfig {
    /// Current Z height of the slice
    pub z_height: f32,
    /// Minimum Z bound of the model
    pub min_z: f32,
    /// Maximum Z bound of the model
    pub max_z: f32,
    /// Whether to show filled polygons (vs outline only)
    pub filled_mode: bool,
    /// Whether to show coordinate grid
    pub show_grid: bool,
    /// Slice contour segments
    pub contours: Vec<LineSegment2D>,
}

impl Default for SliceConfig {
    fn default() -> Self {
        Self {
            z_height: 0.0,
            min_z: 0.0,
            max_z: 100.0,
            filled_mode: false,
            show_grid: true,
            contours: Vec::new(),
        }
    }
}

/// Secondary window for live 2D slice preview
pub struct SlicePreviewWindow {
    /// The minifb window
    window: Window,
    /// Pixel buffer for rendering
    buffer: Vec<u32>,
    /// Current slice configuration
    config: SliceConfig,
    /// Model bounds for coordinate transformation
    model_min: (f32, f32),
    model_max: (f32, f32),
    /// Scale factor for rendering (pixels per unit)
    scale: f32,
    /// Offset for centering the view
    offset_x: f32,
    offset_y: f32,
    /// Last frame time for FPS calculation
    last_frame: Instant,
    /// Whether the window is visible
    visible: bool,
}

impl SlicePreviewWindow {
    /// Create a new slice preview window
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut window = Window::new(
            "Slice Preview - lib3mf Viewer",
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            WindowOptions {
                resize: false, // Disable resizing to keep buffer size consistent
                ..WindowOptions::default()
            },
        )?;

        // Set update rate to 60 FPS
        window.set_target_fps(60);

        let buffer = vec![BG_COLOR; WINDOW_WIDTH * WINDOW_HEIGHT];

        Ok(Self {
            window,
            buffer,
            config: SliceConfig::default(),
            model_min: (0.0, 0.0),
            model_max: (100.0, 100.0),
            scale: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            last_frame: Instant::now(),
            visible: true,
        })
    }

    /// Update the slice configuration
    pub fn update_config(&mut self, config: SliceConfig) {
        self.config = config;
        self.calculate_transform();
    }

    /// Set model bounds for coordinate transformation
    pub fn set_model_bounds(&mut self, min: (f32, f32), max: (f32, f32)) {
        self.model_min = min;
        self.model_max = max;
        self.calculate_transform();
    }

    /// Calculate transformation parameters for rendering
    fn calculate_transform(&mut self) {
        let model_width = self.model_max.0 - self.model_min.0;
        let model_height = self.model_max.1 - self.model_min.1;

        // Handle degenerate cases where model has zero or very small dimensions
        const MIN_DIMENSION: f32 = 0.001;
        let width = if model_width < MIN_DIMENSION { 1.0 } else { model_width };
        let height = if model_height < MIN_DIMENSION { 1.0 } else { model_height };

        // Add some margin
        let margin = 50.0;
        let available_width = WINDOW_WIDTH as f32 - 2.0 * margin;
        let available_height = WINDOW_HEIGHT as f32 - 100.0 - 2.0 * margin; // Reserve 100px for UI panel

        // Calculate scale to fit model in window
        let scale_x = available_width / width;
        let scale_y = available_height / height;
        self.scale = scale_x.min(scale_y);

        // Calculate offsets to center the model
        self.offset_x = margin + (available_width - width * self.scale) / 2.0;
        self.offset_y = margin + (available_height - height * self.scale) / 2.0;
    }

    /// Transform model coordinates to screen coordinates
    fn to_screen(&self, x: f32, y: f32) -> (i32, i32) {
        let screen_x = (x - self.model_min.0) * self.scale + self.offset_x;
        // Flip Y axis (screen Y grows downward, model Y grows upward)
        let screen_y = WINDOW_HEIGHT as f32 - ((y - self.model_min.1) * self.scale + self.offset_y);
        (screen_x as i32, screen_y as i32)
    }

    /// Clear the buffer with background color
    fn clear(&mut self) {
        self.buffer.fill(BG_COLOR);
    }

    /// Draw a pixel at the given screen coordinates
    fn draw_pixel(&mut self, x: i32, y: i32, color: u32) {
        if x >= 0 && x < WINDOW_WIDTH as i32 && y >= 0 && y < WINDOW_HEIGHT as i32 {
            let index = (y as usize * WINDOW_WIDTH) + x as usize;
            if index < self.buffer.len() {
                self.buffer[index] = color;
            }
        }
    }

    /// Draw a line using Bresenham's algorithm
    fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
        let mut x0 = x0;
        let mut y0 = y0;
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;

        loop {
            self.draw_pixel(x0, y0, color);

            if x0 == x1 && y0 == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x0 += sx;
            }
            if e2 < dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Draw a filled rectangle
    fn draw_rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: u32) {
        for dy in 0..height {
            for dx in 0..width {
                self.draw_pixel(x + dx, y + dy, color);
            }
        }
    }

    /// Draw the coordinate grid
    fn draw_grid(&mut self) {
        if !self.config.show_grid {
            return;
        }

        let grid_spacing = 10.0; // Grid every 10 units in model space
        
        // Calculate grid line positions in model space
        let min_x = (self.model_min.0 / grid_spacing).floor() * grid_spacing;
        let max_x = (self.model_max.0 / grid_spacing).ceil() * grid_spacing;
        let min_y = (self.model_min.1 / grid_spacing).floor() * grid_spacing;
        let max_y = (self.model_max.1 / grid_spacing).ceil() * grid_spacing;

        // Draw vertical grid lines
        let mut x = min_x;
        while x <= max_x {
            let (sx, sy0) = self.to_screen(x, self.model_min.1);
            let (_, sy1) = self.to_screen(x, self.model_max.1);
            self.draw_line(sx, sy0, sx, sy1, GRID_COLOR);
            x += grid_spacing;
        }

        // Draw horizontal grid lines
        let mut y = min_y;
        while y <= max_y {
            let (sx0, sy) = self.to_screen(self.model_min.0, y);
            let (sx1, _) = self.to_screen(self.model_max.0, y);
            self.draw_line(sx0, sy, sx1, sy, GRID_COLOR);
            y += grid_spacing;
        }
    }

    /// Draw slice contours
    fn draw_contours(&mut self) {
        // Clone contours to avoid borrow issues
        let contours = self.config.contours.clone();
        for segment in &contours {
            let (x0, y0) = self.to_screen(segment.start.x, segment.start.y);
            let (x1, y1) = self.to_screen(segment.end.x, segment.end.y);
            self.draw_line(x0, y0, x1, y1, CONTOUR_COLOR);
        }
    }

    /// Draw UI panel with Z height controls
    fn draw_ui_panel(&mut self) {
        let panel_height = 80;
        let panel_y = WINDOW_HEIGHT as i32 - panel_height;

        // Draw panel background
        self.draw_rect(0, panel_y, WINDOW_WIDTH as i32, panel_height, PANEL_BG_COLOR);

        // Draw separator line
        self.draw_line(0, panel_y, WINDOW_WIDTH as i32, panel_y, TEXT_COLOR);

        // Draw Z height indicator
        // Note: minifb doesn't have text rendering, so we'll draw a simple visual indicator
        // Draw a slider bar representing Z position
        let slider_x = 50;
        let slider_y = panel_y + 30;
        let slider_width = WINDOW_WIDTH as i32 - 100;
        let slider_height = 20;

        // Slider background
        self.draw_rect(slider_x, slider_y, slider_width, slider_height, GRID_COLOR);

        // Calculate slider position
        let z_range = self.config.max_z - self.config.min_z;
        let z_position = if z_range > 0.0 {
            ((self.config.z_height - self.config.min_z) / z_range).clamp(0.0, 1.0)
        } else {
            0.5
        };
        let slider_pos = slider_x + (z_position * slider_width as f32) as i32;

        // Slider indicator
        self.draw_rect(
            slider_pos - 5,
            slider_y - 5,
            10,
            slider_height + 10,
            CONTOUR_COLOR,
        );
    }

    /// Render the current frame
    pub fn render(&mut self) {
        self.clear();
        self.draw_grid();
        self.draw_contours();
        self.draw_ui_panel();
    }

    /// Update the window (returns false if window should close)
    pub fn update(&mut self) -> bool {
        if !self.window.is_open() {
            self.visible = false;
            return false;
        }

        // Handle keyboard input for Z height adjustment
        let z_step = (self.config.max_z - self.config.min_z) * 0.02; // 2% of range

        if self.window.is_key_down(Key::Up) {
            self.config.z_height = (self.config.z_height + z_step).min(self.config.max_z);
        }
        if self.window.is_key_down(Key::Down) {
            self.config.z_height = (self.config.z_height - z_step).max(self.config.min_z);
        }
        if self.window.is_key_down(Key::PageUp) {
            self.config.z_height = (self.config.z_height + z_step * 5.0).min(self.config.max_z);
        }
        if self.window.is_key_down(Key::PageDown) {
            self.config.z_height = (self.config.z_height - z_step * 5.0).max(self.config.min_z);
        }
        if self.window.is_key_pressed(Key::G, minifb::KeyRepeat::No) {
            self.config.show_grid = !self.config.show_grid;
        }
        if self.window.is_key_pressed(Key::F, minifb::KeyRepeat::No) {
            self.config.filled_mode = !self.config.filled_mode;
        }

        // Render and update the window
        self.render();
        self.window
            .update_with_buffer(&self.buffer, WINDOW_WIDTH, WINDOW_HEIGHT)
            .is_ok()
    }

    /// Get the current Z height (for synchronization with 3D view)
    pub fn get_z_height(&self) -> f32 {
        self.config.z_height
    }

    /// Get the current grid visibility state
    pub fn get_show_grid(&self) -> bool {
        self.config.show_grid
    }

    /// Set the Z height (for synchronization from 3D view)
    pub fn set_z_height(&mut self, z: f32) {
        self.config.z_height = z.clamp(self.config.min_z, self.config.max_z);
    }

    /// Check if the window is visible
    pub fn is_visible(&self) -> bool {
        self.visible && self.window.is_open()
    }

    /// Export current slice to PNG
    pub fn export_to_png(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        use image::{ImageBuffer, Rgb};

        let mut img = ImageBuffer::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32);

        for y in 0..WINDOW_HEIGHT {
            for x in 0..WINDOW_WIDTH {
                let pixel = self.buffer[y * WINDOW_WIDTH + x];
                let r = ((pixel >> 16) & 0xFF) as u8;
                let g = ((pixel >> 8) & 0xFF) as u8;
                let b = (pixel & 0xFF) as u8;
                img.put_pixel(x as u32, y as u32, Rgb([r, g, b]));
            }
        }

        img.save(path)?;
        Ok(())
    }
}
