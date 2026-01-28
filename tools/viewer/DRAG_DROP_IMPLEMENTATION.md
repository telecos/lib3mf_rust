# Drag-and-Drop File Loading Implementation

## Overview

This document describes the implementation of drag-and-drop file loading for the 3MF Viewer. Users can now drag `.3mf` files directly onto the viewer window to load them, providing a more intuitive file loading experience.

## Changes Made

### 1. kiss3d Patch (Vendored Copy)

Since kiss3d 0.35 does not expose file drop events from the underlying winit library, we created a vendored copy with minimal patches:

**Location:** `tools/viewer/kiss3d-patch/`

**Key Changes:**

#### `src/event/window_event.rs`
- Changed `WindowEvent` enum from `Copy` to `Clone` (required for String fields)
- Added three new event variants:
  - `HoveredFile(String)` - Triggered when a file is dragged over the window
  - `HoveredFileCancelled` - Triggered when the file drag is cancelled
  - `DroppedFile(String)` - Triggered when a file is dropped onto the window

#### `src/event/event_manager.rs`
- Modified `Event::drop()` to clone events instead of moving them (required after removing `Copy` trait)

#### `src/window/gl_canvas.rs`
- Added handlers to convert glutin/winit file drop events to kiss3d WindowEvents
- File paths are converted to strings using `to_string_lossy()` for cross-platform compatibility

#### `src/lib.rs`
- Added lint allows for compatibility with newer Rust versions:
  - `#![allow(static_mut_refs)]`
  - `#![allow(unused_parens)]`
  - Changed `deny(unused_qualifications)` to `allow` for compatibility

### 2. Viewer State (`src/ui_viewer.rs`)

#### New State Fields
Added to `ViewerState` struct:
- `show_drop_zone: bool` - Controls visibility of the drop zone overlay
- `drop_file_valid: bool` - Indicates if the hovered file is a valid .3mf file
- `hovered_file_path: Option<String>` - Path of the file currently being dragged

#### Event Handlers
Added three new event handlers in the main event loop:

1. **`WindowEvent::HoveredFile(path)`**
   - Sets `show_drop_zone = true`
   - Validates file extension (checks for `.3mf`)
   - Stores file path for display

2. **`WindowEvent::HoveredFileCancelled`**
   - Hides drop zone overlay
   - Clears state

3. **`WindowEvent::DroppedFile(path)`**
   - Validates file extension
   - Loads the 3MF file if valid
   - Recreates mesh and beam nodes
   - Resets camera to fit new model
   - Shows error message for invalid file types

#### Visual Feedback Function
**`draw_drop_zone_overlay(window, valid_file, file_path)`**

Creates a full-screen semi-transparent overlay with:
- **Blue tint (0.2, 0.6, 1.0)** for valid .3mf files
- **Red tint (1.0, 0.3, 0.3)** for invalid files
- Horizontal lines with 30% alpha for overlay effect
- Centered message text:
  - "Drop to open file" (valid)
  - "Only .3mf files supported" (invalid)
- File name display below the main message

### 3. Keybindings Help (`src/keybindings.rs`)

Added a new entry to the File category:
```
Drag & Drop    Drag .3mf file onto window to load
```

### 4. Cargo Configuration

Updated `tools/viewer/Cargo.toml`:
```toml
kiss3d = { path = "kiss3d-patch" }
```

## How It Works

### File Drag Flow

1. **User drags file over window**
   - winit/glutin generates `WinitWindowEvent::HoveredFile`
   - kiss3d-patch converts to `WindowEvent::HoveredFile`
   - Viewer shows blue or red overlay based on file extension

2. **User cancels drag (moves file away)**
   - winit/glutin generates `WinitWindowEvent::HoveredFileCancelled`
   - kiss3d-patch converts to `WindowEvent::HoveredFileCancelled`
   - Viewer hides overlay

3. **User drops file**
   - winit/glutin generates `WinitWindowEvent::DroppedFile`
   - kiss3d-patch converts to `WindowEvent::DroppedFile`
   - Viewer validates extension and loads file if valid

### Visual Feedback

The overlay is rendered using kiss3d's planar (2D) drawing API:
- `window.draw_planar_line()` for the overlay effect
- `window.draw_text()` for messages
- Rendered after all 3D content but before the menu bar
- Uses `Point2` for 2D coordinates and `Point3` for colors

## Cross-Platform Compatibility

- File paths are converted to strings using `to_string_lossy()` to handle platform-specific path formats
- Uses winit 0.24 (via glutin 0.26) which supports file drop on:
  - **Windows** - Native file drop API
  - **macOS** - Cocoa drag-and-drop
  - **Linux** - X11/Wayland drag-and-drop

## Testing

### Manual Testing Steps

1. **Test valid .3mf file:**
   ```bash
   cd tools/viewer
   cargo run --release -- --ui
   # Drag a .3mf file onto the window
   # Expected: Blue overlay appears, file loads on drop
   ```

2. **Test invalid file type:**
   ```bash
   cargo run --release -- --ui
   # Drag a .txt or .obj file onto the window
   # Expected: Red overlay appears, error message on drop
   ```

3. **Test drag cancellation:**
   ```bash
   cargo run --release -- --ui
   # Drag a file over the window, then move it away
   # Expected: Overlay disappears
   ```

## Known Limitations

1. **Single file only** - Multiple files dropped simultaneously will only load the first .3mf file found
2. **No confirmation dialog** - Current model is immediately replaced when a file is dropped
3. **kiss3d patch** - Requires maintaining a vendored copy of kiss3d 0.35 with our modifications

## Future Enhancements

Potential improvements for the future:
- Add confirmation dialog before replacing current model
- Support loading multiple files (open in new windows or merge)
- Add drag-and-drop for other file formats (OBJ, STL for comparison)
- Upstream the file drop patches to kiss3d repository

## References

- kiss3d: https://github.com/sebcrozet/kiss3d
- winit file drop events: https://docs.rs/winit/0.24.0/winit/event/enum.WindowEvent.html
- Issue: telecos/lib3mf_rust#[issue_number] (Viewer drag-and-drop file loading)
