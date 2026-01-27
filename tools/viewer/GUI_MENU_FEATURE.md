# GUI Menu Bar Feature

## Overview

The 3MF viewer now includes a clickable GUI menu bar that provides easy access to all viewer features through a familiar menu interface. This complements the existing keyboard shortcuts, making the viewer more intuitive and discoverable.

## Features

### Menu Structure

The menu bar includes five top-level menus:

#### 1. File Menu
- **Open...** (Ctrl+O) - Open a 3MF file using a file dialog
- **Browse Test Suites...** (Ctrl+T) - Browse 3MF Consortium test suites from GitHub
- **Export Screenshot...** (S) - Capture the current view as a PNG image
- **Exit** (ESC) - Close the viewer

#### 2. View Menu
- **Show Axes** (A) - Toggle XYZ coordinate axes visualization
- **Show Print Bed** (P) - Toggle print bed/build volume display
- **Show Grid** (G) - Toggle grid overlay (not yet implemented)
- **Reset Camera** (Home) - Reset camera to default position
- **Fit to Model** (F) - Adjust camera to fit the model in view

#### 3. Settings Menu
- **Theme: Light** - Switch to light background theme
- **Theme: Dark** (T) - Switch to dark background theme (default)
- **Print Bed Settings** - Configure print bed dimensions (use C key)

#### 4. Extensions Menu
- **Materials/Colors** - Material and color support (always enabled)
- **Beam Lattice** (B) - Toggle beam lattice structure visualization
- **Slice Stack** (Z) - Toggle slice stack view mode
- **Displacement** (D) - Toggle displacement map visualization
- **Boolean Operations** (V) - Cycle through boolean operation visualization modes

#### 5. Help Menu
- **Keyboard Shortcuts** (M) - Display list of keyboard controls
- **About** - Show viewer information and version

### Usage

#### Activating the Menu Bar

The menu bar is visible by default when you launch the viewer. You can interact with it in two ways:

1. **Mouse Click**: Click on any menu label to open the dropdown menu
2. **Keyboard Toggle**: Press `M` to show/hide the menu bar

#### Using Menus

1. Click on a menu label (e.g., "File") to open the dropdown
2. Click on a menu item to execute the action
3. Items with checkmarks (✓) indicate toggleable features that are currently enabled
4. Keyboard shortcuts are shown on the right side of menu items

#### Visual Indicators

- **Highlighted menu**: The currently open menu is shown in yellow
- **Checkmarks**: Items with [✓] are enabled/active
- **Hover effect**: Menu items are highlighted when you hover over them
- **Shortcuts**: Keyboard shortcuts are displayed in gray text on the right

### Integration with Keyboard Controls

All menu actions can still be triggered using keyboard shortcuts. The menu bar provides a discoverable interface for users who prefer mouse interaction or want to explore available features.

Key shortcuts remain active:
- `Ctrl+O` - Open file
- `Ctrl+T` - Browse test suites
- `S` - Screenshot
- `A` - Toggle axes
- `P` - Toggle print bed
- `T` - Cycle themes
- `B` - Toggle beam lattice
- `Z` - Toggle slice view
- `D` - Toggle displacement
- `V` - Cycle boolean modes
- `M` - Toggle menu bar visibility
- `F` - Fit model to view
- `Home` - Reset camera

### Implementation Details

#### Menu Bar Rendering

The menu bar is rendered as an overlay on top of the 3D viewport using kiss3d's text rendering capabilities:

- Position: Top of the window (25 pixels high)
- Background: Semi-transparent dark gray
- Text: Light gray, with yellow highlight for active items

#### Menu State Management

The `MenuBar` struct in `menu_ui.rs` manages:
- Menu items and their checked/enabled states
- Mouse position tracking for hit detection
- Active menu tracking for dropdown display
- Menu action triggering

#### Action Handling

When a menu item is clicked, it triggers a `MenuAction` which is then processed by `handle_menu_action()` in the main viewer loop. This function:
- Updates viewer state (themes, visibility toggles, etc.)
- Reloads mesh nodes when needed (displacement, boolean modes)
- Provides user feedback through console messages

### Known Limitations

Some menu items are placeholders for future features:
- Grid overlay
- Ruler display
- View presets (Top, Front, Side)
- Custom themes
- Print bed settings dialog
- Preferences dialog

These are marked as "not yet implemented" and will display a console message when selected.

### Future Enhancements

Potential improvements for future versions:
- Toolbar with icon buttons for common actions
- Right-click context menu in the viewport
- Tooltips on menu hover
- Recent files list under File menu
- Customizable keyboard shortcuts
- Preferences dialog for persistent settings
- View preset buttons (Top/Front/Side/Isometric)

## Technical Architecture

### Files Modified

1. **tools/viewer/src/menu_ui.rs** (new)
   - `MenuBar` struct and implementation
   - `MenuAction` enum
   - `MenuItem` and `Menu` structures
   - Event handling and rendering logic

2. **tools/viewer/src/ui_viewer.rs** (modified)
   - Integration of menu bar into main render loop
   - `handle_menu_action()` function for action processing
   - Menu bar state initialization and updates

3. **tools/viewer/src/main.rs** (modified)
   - Added menu_ui module declaration

### Code Organization

```
MenuBar
├── menus: Vec<Menu>
│   └── items: Vec<MenuItem>
│       ├── label: String
│       ├── shortcut: Option<String>
│       ├── action: MenuAction
│       ├── enabled: bool
│       └── checked: bool
├── handle_event() -> Option<MenuAction>
├── render()
└── update_dimensions()
```

### Event Flow

1. Mouse/keyboard event occurs
2. `MenuBar::handle_event()` processes the event
3. If menu item clicked, returns corresponding `MenuAction`
4. `handle_menu_action()` executes the action
5. Menu state updated (checkmarks, visibility, etc.)
6. Changes reflected in next render frame

## Examples

### Opening a File via Menu

1. Click "File" in the menu bar
2. Click "Open..." in the dropdown
3. File dialog appears
4. Select a 3MF file
5. Model loads and displays

### Toggling Beam Lattice

1. Click "Extensions" in the menu bar
2. Click "Beam Lattice" (checkmark indicates current state)
3. Beam lattice visibility toggles
4. Checkmark updates to reflect new state

### Changing Theme

1. Click "Settings" in the menu bar
2. Click "Theme: Light" or "Theme: Dark"
3. Background color changes immediately
4. Console shows confirmation message

## Conclusion

The GUI menu bar makes the 3MF viewer more accessible and user-friendly while preserving the efficiency of keyboard shortcuts for power users. All features are now discoverable through the menu interface, with visual feedback for toggleable options.
