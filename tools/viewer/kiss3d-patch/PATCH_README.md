# kiss3d Patch for File Drop Support

This is a minimal vendored copy of kiss3d v0.35.0 with patches to support file drag-and-drop events.

## Why Vendored?

kiss3d 0.35 does not expose file drop events from the underlying winit library. This patch adds support for:
- `WindowEvent::HoveredFile(String)` - File dragged over window
- `WindowEvent::HoveredFileCancelled` - File drag cancelled  
- `WindowEvent::DroppedFile(String)` - File dropped onto window

## Changes Made

### 1. `src/event/window_event.rs`
- Changed `WindowEvent` enum from `Copy` to `Clone` (required for String fields)
- Added three new event variants: `HoveredFile`, `HoveredFileCancelled`, `DroppedFile`

### 2. `src/event/event_manager.rs`
- Modified `Event::drop()` to clone events instead of moving them

### 3. `src/window/gl_canvas.rs`
- Added handlers to convert glutin/winit file drop events to kiss3d WindowEvents

### 4. `src/lib.rs`
- Added lint allows for compatibility with newer Rust versions

## Size Optimization

To reduce repository size, the following directories have been removed:
- `examples/` - Example programs (not needed for library usage)
- `website/` - Documentation website source
- `.circleci/` - CI configuration

## Base Version

Based on kiss3d commit 97192552 (v0.35.0 release).

## Upstream

Original repository: https://github.com/sebcrozet/kiss3d

If this feature is accepted upstream, this vendored copy can be removed.
