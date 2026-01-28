# Keyboard Controls - Visual Guide

This document provides a visual reference for all keyboard shortcuts and controls in the 3MF Viewer.

## Organized Help Display

The viewer now features a well-organized, categorized display of all keyboard controls. This help is shown:
- On startup when the viewer launches
- On demand by pressing **H** or **?** keys

### Example Output

```
╔══════════════════════════════════════════════════════════════╗
║                    3MF Viewer - Controls                      ║
╠══════════════════════════════════════════════════════════════╣
║  FILE                                                        ║
║    Ctrl+O       Open file                                    ║
║    S            Save screenshot                              ║
║    Escape       Exit                                         ║
║    Ctrl+T       Browse test suites                           ║
║                                                              ║
║  VIEW                                                        ║
║    A            Toggle axes                                  ║
║    P            Toggle print bed                             ║
║    M            Toggle menu                                  ║
║    R            Toggle materials                             ║
║    B            Toggle beam lattice                          ║
║    V            Cycle boolean visualization                  ║
║    D            Toggle displacement                          ║
║                                                              ║
║  CAMERA                                                      ║
║    F            Fit model to view                            ║
║    Home         Reset camera                                 ║
║    Mouse Left   Rotate view                                  ║
║    Mouse Right  Pan view                                     ║
║    Scroll       Zoom in/out                                  ║
║    +/PgUp       Zoom in                                      ║
║    -/PgDn       Zoom out                                     ║
║    Arrow Keys   Pan view                                     ║
║                                                              ║
║  SLICE                                                       ║
║    Z            Toggle slice view                            ║
║    Shift+↑      Move slice up                                ║
║    Shift+↓      Move slice down                              ║
║    L            Toggle slice plane                           ║
║    X            Export slice to PNG                          ║
║    K            Toggle 3D stack view                         ║
║    N            Toggle filled/outline mode                   ║
║                                                              ║
║  ANIMATION                                                   ║
║    Space        Play/pause animation                         ║
║    Home         First slice                                  ║
║    End          Last slice                                   ║
║    ]            Increase speed                               ║
║    [            Decrease speed                               ║
║                                                              ║
║  THEME                                                       ║
║    T            Cycle themes                                 ║
║                                                              ║
║  SETTINGS                                                    ║
║    C            Configure print bed                          ║
║                                                              ║
║  HELP                                                        ║
║    H or ?       Show this help                               ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
```

## Implementation Details

### Centralized Keybinding Registry

All keybindings are now managed in `src/keybindings.rs`, which provides:

1. **Single Source of Truth**: All keyboard shortcuts defined in one place
2. **Category-Based Organization**: Shortcuts grouped into logical categories
3. **Easy to Maintain**: Adding new shortcuts is simple and consistent
4. **Prevents Conflicts**: Easy to spot duplicate or conflicting bindings

### Category Structure

Controls are organized into these categories:

- **FILE**: File operations (open, save, exit)
- **VIEW**: Display toggles (axes, print bed, menu, materials, etc.)
- **CAMERA**: Camera controls (rotate, pan, zoom, fit, reset)
- **SLICE**: Slice view operations
- **ANIMATION**: Slice animation controls
- **THEME**: Visual theme selection
- **SETTINGS**: Configuration options
- **HELP**: Help display

### Testing the Help Display

You can test the help display without launching the full UI:

```bash
cd tools/viewer
cargo run --example show_help
```

This will display the formatted help output in your terminal.

## Benefits

### For Users
- **Easy to Find**: Controls grouped by function
- **Clear Layout**: Consistent formatting with alignment
- **On-Demand Access**: Press H or ? anytime to see help
- **Complete Reference**: All shortcuts documented in one place

### For Developers
- **Easy to Add**: New keybindings added to central registry
- **No Conflicts**: Easy to check for duplicate bindings
- **Maintainable**: Single location to update documentation
- **Testable**: Unit tests verify no duplicate keys and all categories have bindings

## Usage Examples

### Viewing Help on Startup
When you launch the viewer, the organized help is displayed in the console.

### Showing Help On Demand
While using the viewer:
1. Press **H** key to display help in the console
2. Or press **?** (Shift+/) to display help

The help will be printed to the console where you launched the viewer.

## Related Features

- GUI Menu Bar (#278): Menus will show keyboard shortcuts next to menu items
- In-app Help Overlay: Future enhancement to show help directly in the viewer window

## Files Modified

- `tools/viewer/src/keybindings.rs` - New module with centralized registry
- `tools/viewer/src/ui_viewer.rs` - Updated to use new help system, added H/? handlers
- `tools/viewer/src/main.rs` - Added keybindings module
- `tools/viewer/examples/show_help.rs` - Example to demonstrate help display

## Future Enhancements

Potential improvements for the future:
1. **In-App Overlay**: Show help as a semi-transparent overlay in the viewer window
2. **Contextual Hints**: Display relevant shortcuts at bottom of screen based on current mode
3. **Status Bar**: Show current state (e.g., "Wireframe: ON | Theme: Dark")
4. **Customizable Bindings**: Allow users to configure their own keyboard shortcuts
