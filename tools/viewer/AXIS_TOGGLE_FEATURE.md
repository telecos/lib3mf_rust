# XYZ Axis Visualization Toggle Feature

## Overview
This document describes the XYZ axis visualization toggle feature added to the lib3mf viewer.

## Implementation Details

### Components Added
1. **Axis Rendering Function** (`draw_axes`)
   - Draws three lines from the origin representing X, Y, Z axes
   - Colors: X=Red (1.0, 0.0, 0.0), Y=Green (0.0, 1.0, 0.0), Z=Blue (0.0, 0.0, 1.0)
   - Axis length: Auto-scaled to 50% of model size

2. **Toggle State Management**
   - `show_axes`: Boolean flag to track visibility state
   - Default: `true` (axes visible on startup)
   - Toggle behavior: Press 'A' key to switch between ON/OFF

3. **Keyboard Event Handling**
   - Uses kiss3d's event system
   - Listens for `Key::A` release events
   - Prints toggle status to console: "XYZ Axes: ON" or "XYZ Axes: OFF"

### Code Changes

#### File: `tools/viewer/src/ui_viewer.rs`

**Added imports:**
```rust
use kiss3d::event::{Action, Key, WindowEvent};
```

**Added in main event loop:**
```rust
// Track axis visualization state (default: visible)
let mut show_axes = true;

// Calculate axis length based on model size
let axis_length = max_size * 0.5; // 50% of model size

// Main event loop
while window.render() {
    // Handle keyboard events
    for event in window.events().iter() {
        if let WindowEvent::Key(key, Action::Release, _) = event.value {
            if key == Key::A {
                show_axes = !show_axes;
                println!("XYZ Axes: {}", if show_axes { "ON" } else { "OFF" });
            }
        }
    }
    
    // Draw XYZ axes if visible
    if show_axes {
        draw_axes(&mut window, axis_length);
    }
}
```

**Added helper function:**
```rust
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
```

## Usage

### Starting the Viewer
```bash
cd tools/viewer
cargo run --release -- ../../test_files/core/box.3mf --ui
```

### Controls
- **A Key**: Toggle XYZ axes visibility
- The console will display "XYZ Axes: ON" or "XYZ Axes: OFF" when toggled

### Expected Behavior
1. **On Startup**: Axes are visible by default
2. **Press 'A'**: Axes disappear, console prints "XYZ Axes: OFF"
3. **Press 'A' Again**: Axes reappear, console prints "XYZ Axes: ON"

## Acceptance Criteria Status

✅ **XYZ axes render correctly at origin**
- Three lines extending from (0, 0, 0) in X, Y, Z directions

✅ **Axes use standard RGB color coding**
- X axis: Red (1.0, 0.0, 0.0)
- Y axis: Green (0.0, 1.0, 0.0)
- Z axis: Blue (0.0, 0.0, 1.0)

✅ **Toggle visibility with keyboard shortcut**
- 'A' key toggles the axes on/off

✅ **Axes scale appropriately with model size**
- Length set to 50% of the maximum model dimension

✅ **Print axis toggle status in console**
- Displays "XYZ Axes: ON" or "XYZ Axes: OFF" when toggled

## Testing

### Manual Testing Steps
1. Launch the viewer with any 3MF file using the `--ui` flag
2. Verify that XYZ axes are visible at the origin
3. Verify the axes colors (X=Red, Y=Green, Z=Blue)
4. Press the 'A' key
5. Verify axes disappear and console shows "XYZ Axes: OFF"
6. Press 'A' again
7. Verify axes reappear and console shows "XYZ Axes: ON"

### Test Files
Any 3MF file from the test suite can be used:
- `test_files/core/box.3mf`
- `test_files/core/sphere.3mf`
- `test_files/core/cube_gears.3mf`

## Code Quality
- ✅ No unsafe code (enforced by `#![forbid(unsafe_code)]`)
- ✅ Passes `cargo clippy -- -D warnings` with no warnings
- ✅ Follows existing code style and conventions
- ✅ Uses kiss3d's built-in APIs (no custom OpenGL code)
- ✅ Minimal changes to existing codebase

## Future Enhancements (Not Implemented)
The following were suggested but marked as optional:
- Arrow heads at axis ends
- Axis labels (X, Y, Z text)
- Configurable axis length via UI
- Menu option for toggling (no menu system exists yet)

These can be added in future iterations if needed.
