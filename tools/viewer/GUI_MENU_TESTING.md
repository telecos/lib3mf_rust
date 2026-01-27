# GUI Menu Bar - Testing and Verification Guide

## What Was Implemented

A clickable GUI menu bar was successfully added to the 3MF viewer with the following features:

### Menu Structure
```
┌────────────────────────────────────────────────────┐
│ File   View   Settings   Extensions   Help         │
└────────────────────────────────────────────────────┘
```

**File Menu:**
- Open... (Ctrl+O)
- Browse Test Suites... (Ctrl+T)  
- Export Screenshot... (S)
- Exit (ESC)

**View Menu:**
- [✓] Show Axes (A)
- [✓] Show Print Bed (P)
- [ ] Show Grid (G) - placeholder
- Reset Camera (Home)
- Fit to Model (F)

**Settings Menu:**
- Theme: Light
- [✓] Theme: Dark (T)
- Print Bed Settings

**Extensions Menu:**
- [✓] Materials/Colors
- [✓] Beam Lattice (B)
- [ ] Slice Stack (Z)
- [ ] Displacement (D)
- [ ] Boolean Operations (V)

**Help Menu:**
- Keyboard Shortcuts (M)
- About

## How to Test

### Prerequisites
- A system with a display (X11/Wayland on Linux, or Windows/macOS)
- Rust toolchain installed
- Test 3MF files available

### Building the Viewer

```bash
cd /home/runner/work/lib3mf_rust/lib3mf_rust/tools/viewer
cargo build --release
```

The binary will be at: `target/release/lib3mf-viewer`

### Running the Viewer

#### Test 1: Launch with Empty Scene
```bash
cargo run --release -- --ui
```

**Expected:**
- Window opens with menu bar at top
- Menu shows: File, View, Settings, Extensions, Help
- Clicking menu labels opens dropdown menus

#### Test 2: Launch with 3MF File
```bash
cargo run --release -- ../../test_files/core/box.3mf --ui
```

**Expected:**
- Window opens with box model displayed
- Menu bar visible at top
- Axes visible by default (red=X, green=Y, blue=Z)

#### Test 3: Menu Interactions

**File Menu:**
1. Click "File" → Should open dropdown
2. Click "Open..." → Should open file dialog
3. Select a .3mf file → Should load into viewer
4. Click "File" → "Exit" → Should close application

**View Menu:**
1. Click "View" → Should show dropdown
2. Click "Show Axes" → Should toggle axes visibility
3. Verify checkmark appears/disappears
4. Click "Fit to Model" → Camera should adjust to show full model
5. Click "Reset Camera" → Camera should return to default position

**Settings Menu:**
1. Click "Settings" → Should show dropdown
2. Click "Theme: Light" → Background should turn light gray
3. Click "Settings" → "Theme: Dark" → Background should turn dark gray

**Extensions Menu:**
1. Click "Extensions" → Should show dropdown
2. Click "Beam Lattice" → If model has beams, they should toggle on/off
3. Click "Slice Stack" → Slice view should toggle

**Help Menu:**
1. Click "Help" → Should show dropdown
2. Click "Keyboard Shortcuts" → Should print controls to console
3. Click "About" → Should print version info to console

#### Test 4: Keyboard Shortcuts Still Work
After testing menus, verify keyboard shortcuts still function:

```
M - Toggle menu bar visibility
A - Toggle axes
P - Toggle print bed
T - Cycle themes
S - Capture screenshot
Ctrl+O - Open file
F - Fit to model
Home - Reset camera
ESC - Exit
```

#### Test 5: Menu Bar Toggle
1. Press M key → Menu should disappear
2. Press M key again → Menu should reappear
3. Verify viewport gets more space when menu is hidden

### Visual Verification Checklist

- [ ] Menu bar appears at top of window
- [ ] Menu text is readable (light gray on dark background)
- [ ] Clicking menu label opens dropdown
- [ ] Dropdown appears below clicked menu
- [ ] Menu items have proper spacing
- [ ] Checkmarks appear for active features
- [ ] Keyboard shortcuts shown on right side
- [ ] Hovering over item highlights it
- [ ] Clicking item executes action
- [ ] Menu closes after selecting item
- [ ] Disabled items appear grayed out

### Expected Console Output

When launching the viewer, you should see:
```
═══════════════════════════════════════════════════════════
  Interactive 3D Viewer Controls
═══════════════════════════════════════════════════════════

  🖱️  Left Mouse + Drag      : Rotate view
  🖱️  Right Mouse + Drag     : Pan view
  🖱️  Scroll Wheel           : Zoom in/out
  ⌨️  M Key                  : Toggle menu
  ...
═══════════════════════════════════════════════════════════
```

When clicking menu items:
```
Theme changed to: Light
Axes: ON
Beam Lattice: OFF
Screenshot saved: screenshot_2025-01-27_231234.png
```

### Screenshots to Capture

Please capture the following screenshots:

1. **Menu bar closed** - Empty window with menu bar
2. **File menu open** - Showing File dropdown
3. **View menu with checkmarks** - Showing active features
4. **Settings menu open** - Showing theme options  
5. **Extensions menu open** - Showing extension toggles
6. **Help menu open** - Showing help options
7. **Menu bar hidden** - With M key, showing full viewport
8. **Menu with model** - Menu bar with 3D model displayed

Save screenshots as:
```
gui_menu_1_closed.png
gui_menu_2_file_menu.png
gui_menu_3_view_menu.png
gui_menu_4_settings_menu.png
gui_menu_5_extensions_menu.png
gui_menu_6_help_menu.png
gui_menu_7_hidden.png
gui_menu_8_with_model.png
```

### Known Issues to Check

1. **Menu rendering**: Does menu appear on top of 3D viewport?
2. **Click detection**: Do clicks register in correct menu areas?
3. **Window resize**: Does menu adjust to window size changes?
4. **Multiple clicks**: Can you open/close menus multiple times?
5. **Checkbox accuracy**: Do checkmarks match actual feature state?

### Performance Testing

1. Open a large 3MF file (many triangles)
2. Open and close menus rapidly
3. Verify no lag or stuttering
4. Check FPS counter (should stay at 60)

### Platform-Specific Testing

**Linux (X11/Wayland):**
- [ ] Menu renders correctly
- [ ] Mouse clicks detected accurately
- [ ] Text is readable
- [ ] No rendering artifacts

**Windows:**
- [ ] Menu renders correctly
- [ ] Mouse clicks detected accurately
- [ ] Text is readable
- [ ] DPI scaling works

**macOS:**
- [ ] Menu renders correctly
- [ ] Mouse clicks detected accurately
- [ ] Text is readable
- [ ] Retina display support

## Reporting Issues

If you find any issues, please report:

1. **What you did**: Steps to reproduce
2. **What happened**: Actual behavior
3. **What you expected**: Expected behavior
4. **Screenshots**: Visual evidence
5. **Console output**: Any error messages
6. **Platform**: OS and version
7. **Window size**: Resolution when issue occurred

## Success Criteria

The implementation is successful if:

✅ Menu bar appears at top of window
✅ All 5 menus are present and clickable
✅ Dropdown menus open when clicking menu labels
✅ Menu items execute correct actions
✅ Checkmarks appear for active features
✅ Keyboard shortcuts still work
✅ M key toggles menu visibility
✅ No performance degradation
✅ No crashes or errors
✅ Console output is helpful

## Next Steps After Testing

Once testing is complete and screenshots are captured:

1. Add screenshots to the repository
2. Update GUI_MENU_VISUAL_GUIDE.md with actual screenshots
3. Create a comparison showing before/after
4. Document any issues found
5. Plan additional enhancements (toolbar, context menu, etc.)

## Additional Notes

- The menu uses kiss3d's native text rendering
- No external GUI library dependencies
- Menu is rendered last (on top of 3D viewport)
- State synchronization happens each frame
- Menu actions are type-safe via MenuAction enum

## Contact

If you have questions about testing or need help:
- Check GUI_MENU_FEATURE.md for feature details
- Check GUI_MENU_IMPLEMENTATION.md for technical details
- Check GUI_MENU_VISUAL_GUIDE.md for visual examples
