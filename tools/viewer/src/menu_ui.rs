//! Simple overlay menu system for the 3MF viewer
//!
//! This module provides a clickable menu bar and context menu system
//! using kiss3d's text rendering and mouse event handling.

#![forbid(unsafe_code)]

use kiss3d::event::{Action, MouseButton, WindowEvent};
use kiss3d::text::Font;
use kiss3d::window::Window;

/// Menu item definition
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub shortcut: Option<String>,
    pub action: MenuAction,
    pub enabled: bool,
    pub checked: bool,
}

/// Menu actions that can be triggered
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Some menu actions are not yet implemented
pub enum MenuAction {
    // File menu
    Open,
    OpenRecent,
    BrowseTests,
    ExportScreenshot,
    Exit,
    
    // View menu
    ToggleAxes,
    TogglePrintBed,
    ToggleGrid,
    ToggleRulers,
    ResetCamera,
    FitToModel,
    TopView,
    FrontView,
    SideView,
    
    // Settings menu
    ThemeLight,
    ThemeDark,
    ThemeCustom,
    PrintBedSettings,
    Preferences,
    
    // Extensions menu
    ToggleMaterials,
    ToggleBeamLattice,
    ToggleSliceStack,
    ToggleDisplacement,
    ToggleBooleanOps,
    
    // Help menu
    KeyboardShortcuts,
    About,
    
    // Internal
    None,
}

/// Top-level menu definition
#[derive(Debug, Clone)]
pub struct Menu {
    pub label: String,
    pub items: Vec<MenuItem>,
    pub open: bool,
}

/// Menu bar state
pub struct MenuBar {
    pub menus: Vec<Menu>,
    pub active_menu: Option<usize>,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub window_width: u32,
    pub window_height: u32,
    pub visible: bool,
}

impl MenuBar {
    /// Create a new menu bar with default menus
    pub fn new() -> Self {
        let menus = vec![
            Menu {
                label: "File".to_string(),
                items: vec![
                    MenuItem {
                        label: "Open...".to_string(),
                        shortcut: Some("Ctrl+O".to_string()),
                        action: MenuAction::Open,
                        enabled: true,
                        checked: false,
                    },
                    MenuItem {
                        label: "Browse Test Suites...".to_string(),
                        shortcut: Some("Ctrl+T".to_string()),
                        action: MenuAction::BrowseTests,
                        enabled: true,
                        checked: false,
                    },
                    MenuItem {
                        label: "Export Screenshot...".to_string(),
                        shortcut: Some("S".to_string()),
                        action: MenuAction::ExportScreenshot,
                        enabled: true,
                        checked: false,
                    },
                    MenuItem {
                        label: "Exit".to_string(),
                        shortcut: Some("ESC".to_string()),
                        action: MenuAction::Exit,
                        enabled: true,
                        checked: false,
                    },
                ],
                open: false,
            },
            Menu {
                label: "View".to_string(),
                items: vec![
                    MenuItem {
                        label: "Show Axes".to_string(),
                        shortcut: Some("A".to_string()),
                        action: MenuAction::ToggleAxes,
                        enabled: true,
                        checked: true,
                    },
                    MenuItem {
                        label: "Show Print Bed".to_string(),
                        shortcut: Some("P".to_string()),
                        action: MenuAction::TogglePrintBed,
                        enabled: true,
                        checked: true,
                    },
                    MenuItem {
                        label: "Show Grid".to_string(),
                        shortcut: Some("G".to_string()),
                        action: MenuAction::ToggleGrid,
                        enabled: true,
                        checked: false,
                    },
                    MenuItem {
                        label: "Reset Camera".to_string(),
                        shortcut: Some("Home".to_string()),
                        action: MenuAction::ResetCamera,
                        enabled: true,
                        checked: false,
                    },
                    MenuItem {
                        label: "Fit to Model".to_string(),
                        shortcut: Some("F".to_string()),
                        action: MenuAction::FitToModel,
                        enabled: true,
                        checked: false,
                    },
                ],
                open: false,
            },
            Menu {
                label: "Settings".to_string(),
                items: vec![
                    MenuItem {
                        label: "Theme: Light".to_string(),
                        shortcut: None,
                        action: MenuAction::ThemeLight,
                        enabled: true,
                        checked: false,
                    },
                    MenuItem {
                        label: "Theme: Dark".to_string(),
                        shortcut: Some("T".to_string()),
                        action: MenuAction::ThemeDark,
                        enabled: true,
                        checked: true,
                    },
                ],
                open: false,
            },
            Menu {
                label: "Extensions".to_string(),
                items: vec![
                    MenuItem {
                        label: "Materials/Colors".to_string(),
                        shortcut: None,
                        action: MenuAction::ToggleMaterials,
                        enabled: true,
                        checked: true,
                    },
                    MenuItem {
                        label: "Beam Lattice".to_string(),
                        shortcut: Some("B".to_string()),
                        action: MenuAction::ToggleBeamLattice,
                        enabled: true,
                        checked: true,
                    },
                    MenuItem {
                        label: "Slice Stack".to_string(),
                        shortcut: Some("Z".to_string()),
                        action: MenuAction::ToggleSliceStack,
                        enabled: true,
                        checked: false,
                    },
                    MenuItem {
                        label: "Displacement".to_string(),
                        shortcut: Some("D".to_string()),
                        action: MenuAction::ToggleDisplacement,
                        enabled: true,
                        checked: false,
                    },
                    MenuItem {
                        label: "Boolean Operations".to_string(),
                        shortcut: Some("V".to_string()),
                        action: MenuAction::ToggleBooleanOps,
                        enabled: true,
                        checked: false,
                    },
                ],
                open: false,
            },
            Menu {
                label: "Help".to_string(),
                items: vec![
                    MenuItem {
                        label: "Keyboard Shortcuts".to_string(),
                        shortcut: Some("M".to_string()),
                        action: MenuAction::KeyboardShortcuts,
                        enabled: true,
                        checked: false,
                    },
                    MenuItem {
                        label: "About".to_string(),
                        shortcut: None,
                        action: MenuAction::About,
                        enabled: true,
                        checked: false,
                    },
                ],
                open: false,
            },
        ];

        Self {
            menus,
            active_menu: None,
            mouse_x: 0.0,
            mouse_y: 0.0,
            window_width: 800,
            window_height: 600,
            visible: true,
        }
    }

    /// Update menu state with window events
    pub fn handle_event(&mut self, event: &WindowEvent) -> Option<MenuAction> {
        match event {
            WindowEvent::CursorPos(x, y, _) => {
                self.mouse_x = *x as f32;
                self.mouse_y = *y as f32;
                None
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                self.handle_click()
            }
            _ => None,
        }
    }

    /// Handle mouse click on menu items
    fn handle_click(&mut self) -> Option<MenuAction> {
        const MENU_BAR_HEIGHT: f32 = 25.0;
        const MENU_ITEM_HEIGHT: f32 = 20.0;
        const MENU_ITEM_WIDTH: f32 = 200.0;

        // Check if click is in menu bar
        if self.mouse_y < MENU_BAR_HEIGHT {
            // Check which menu was clicked
            let mut x_offset = 10.0;
            let mut clicked_menu_index: Option<usize> = None;
            
            for (i, menu) in self.menus.iter().enumerate() {
                let menu_width = (menu.label.len() as f32 * 8.0) + 20.0;
                if self.mouse_x >= x_offset && self.mouse_x < x_offset + menu_width {
                    clicked_menu_index = Some(i);
                    break;
                }
                x_offset += menu_width + 5.0;
            }
            
            if let Some(i) = clicked_menu_index {
                // Toggle menu open state
                if self.active_menu == Some(i) {
                    self.active_menu = None;
                    self.menus[i].open = false;
                } else {
                    // Close all menus first
                    for m in &mut self.menus {
                        m.open = false;
                    }
                    self.active_menu = Some(i);
                    self.menus[i].open = true;
                }
                return None;
            }
            
            // Click outside menus - close all
            self.close_all_menus();
            return None;
        }

        // Check if click is in a dropdown menu
        if let Some(menu_index) = self.active_menu {
            if menu_index < self.menus.len() {
                // Calculate menu x offset
                let mut x_offset = 10.0;
                for i in 0..menu_index {
                    x_offset += (self.menus[i].label.len() as f32 * 8.0) + 25.0;
                }

                let menu_x = x_offset;
                let menu_y = MENU_BAR_HEIGHT;

                // Check each menu item
                let menu = &self.menus[menu_index];
                for (i, item) in menu.items.iter().enumerate() {
                    let item_y = menu_y + (i as f32 * MENU_ITEM_HEIGHT);
                    if self.mouse_x >= menu_x
                        && self.mouse_x < menu_x + MENU_ITEM_WIDTH
                        && self.mouse_y >= item_y
                        && self.mouse_y < item_y + MENU_ITEM_HEIGHT
                        && item.enabled
                    {
                        let action = item.action;
                        self.close_all_menus();
                        return Some(action);
                    }
                }
            }
        }

        // Click outside menus - close all
        self.close_all_menus();
        None
    }

    /// Close all open menus
    fn close_all_menus(&mut self) {
        self.active_menu = None;
        for menu in &mut self.menus {
            menu.open = false;
        }
    }

    /// Update window dimensions
    pub fn update_dimensions(&mut self, width: u32, height: u32) {
        self.window_width = width;
        self.window_height = height;
    }

    /// Toggle menu bar visibility
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
        if !self.visible {
            self.close_all_menus();
        }
    }

    /// Update menu item checked state
    pub fn set_checked(&mut self, action: MenuAction, checked: bool) {
        for menu in &mut self.menus {
            for item in &mut menu.items {
                if item.action == action {
                    item.checked = checked;
                }
            }
        }
    }

    /// Render the menu bar
    pub fn render(&self, window: &mut Window) {
        if !self.visible {
            return;
        }

        const MENU_BAR_HEIGHT: f32 = 25.0;
        const MENU_ITEM_HEIGHT: f32 = 20.0;
        const MENU_ITEM_WIDTH: f32 = 200.0;
        const FONT_SIZE: f32 = 14.0;

        // Draw menu bar background (semi-transparent dark)
        window.draw_planar_line(
            &kiss3d::nalgebra::Point2::new(0.0, 0.0),
            &kiss3d::nalgebra::Point2::new(self.window_width as f32, 0.0),
            &kiss3d::nalgebra::Point3::new(0.2, 0.2, 0.2),
        );
        window.draw_planar_line(
            &kiss3d::nalgebra::Point2::new(0.0, MENU_BAR_HEIGHT),
            &kiss3d::nalgebra::Point2::new(self.window_width as f32, MENU_BAR_HEIGHT),
            &kiss3d::nalgebra::Point3::new(0.3, 0.3, 0.3),
        );

        // Draw menu items
        let mut x_offset = 10.0;
        for (i, menu) in self.menus.iter().enumerate() {
            // Draw menu label
            let color = if Some(i) == self.active_menu {
                kiss3d::nalgebra::Point3::new(1.0, 1.0, 0.5) // Highlight active menu
            } else {
                kiss3d::nalgebra::Point3::new(0.9, 0.9, 0.9) // Normal text color
            };

            window.draw_text(
                &menu.label,
                &kiss3d::nalgebra::Point2::new(x_offset, 5.0),
                FONT_SIZE,
                &Font::default(),
                &color,
            );

            let menu_width = (menu.label.len() as f32 * 8.0) + 20.0;

            // Draw dropdown menu if open
            if menu.open {
                let menu_x = x_offset;
                let menu_y = MENU_BAR_HEIGHT;

                // Draw menu background
                for (j, item) in menu.items.iter().enumerate() {
                    let item_y = menu_y + (j as f32 * MENU_ITEM_HEIGHT);
                    
                    // Draw item background (darker for hover)
                    let is_hovered = self.mouse_x >= menu_x
                        && self.mouse_x < menu_x + MENU_ITEM_WIDTH
                        && self.mouse_y >= item_y
                        && self.mouse_y < item_y + MENU_ITEM_HEIGHT;

                    // Draw item text with checkbox indicator
                    let checkbox = if item.checked { "[✓] " } else { "    " };
                    let text = format!("{}{}", checkbox, item.label);
                    
                    let text_color = if !item.enabled {
                        kiss3d::nalgebra::Point3::new(0.5, 0.5, 0.5) // Disabled
                    } else if is_hovered {
                        kiss3d::nalgebra::Point3::new(1.0, 1.0, 0.5) // Highlighted
                    } else {
                        kiss3d::nalgebra::Point3::new(0.9, 0.9, 0.9) // Normal
                    };

                    window.draw_text(
                        &text,
                        &kiss3d::nalgebra::Point2::new(menu_x + 5.0, item_y + 2.0),
                        FONT_SIZE * 0.9,
                        &Font::default(),
                        &text_color,
                    );

                    // Draw shortcut hint if present
                    if let Some(ref shortcut) = item.shortcut {
                        let shortcut_x = menu_x + MENU_ITEM_WIDTH - (shortcut.len() as f32 * 7.0);
                        window.draw_text(
                            shortcut,
                            &kiss3d::nalgebra::Point2::new(shortcut_x, item_y + 2.0),
                            FONT_SIZE * 0.8,
                            &Font::default(),
                            &kiss3d::nalgebra::Point3::new(0.6, 0.6, 0.6),
                        );
                    }
                }
            }

            x_offset += menu_width + 5.0;
        }
    }
}

impl Default for MenuBar {
    fn default() -> Self {
        Self::new()
    }
}
