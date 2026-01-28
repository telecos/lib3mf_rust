# Print Area (Build Volume) Visualization Feature

## Overview

The 3MF viewer now includes a configurable print area visualization feature that displays a wireframe box representing the build volume of a 3D printer. This helps users visualize whether their models fit within the available print area.

## Features

### 1. Menu System
- **Key**: Press `M` to toggle the menu display
- **Function**: Shows current viewer settings including theme, print area status, and dimensions
- The menu displays:
  - Current theme
  - Print area visibility status (ON/OFF)
  - Print area dimensions (width, depth, height)
  - Current unit of measurement
  - Loaded file name (if any)

### 2. Print Area Visualization
- **Key**: Press `P` to toggle print area visibility
- **Function**: Shows/hides a wireframe box representing the print area
- **Visual Style**: 
  - Light blue/gray wireframe (12 lines forming a box)
  - Semi-transparent appearance that doesn't obscure the model
  - Centered at the origin (0, 0, 0)
  - Box extends from Z=0 (build plate) to Z=height

### 3. Print Area Configuration
- **Key**: Press `C` to configure print area dimensions
- **Function**: Opens an interactive console-based configuration dialog
- **Configurable Parameters**:
  - Width (X dimension) - default: 200mm
  - Depth (Y dimension) - default: 200mm
  - Height (Z dimension) - default: 200mm
  - Unit of measurement - default: "mm"
- **Usage**:
  1. Press `C` to open configuration
  2. Enter new values when prompted (press Enter to keep current value)
  3. Configuration is applied immediately

## Default Settings

The print area initializes with the following default values:
- **Width**: 200.0 mm
- **Depth**: 200.0 mm
- **Height**: 200.0 mm
- **Unit**: "mm"
- **Visible**: true (enabled by default)

## Keyboard Controls

| Key | Function |
|-----|----------|
| `M` | Toggle menu display |
| `P` | Toggle print area visibility |
| `C` | Configure print area dimensions |

## Implementation Details

### Data Structure
```rust
struct PrintArea {
    width: f32,   // X dimension
    depth: f32,   // Y dimension
    height: f32,  // Z dimension
    unit: String, // "mm", "inch", "cm", etc.
    visible: bool,
}
```

### Wireframe Box Construction
The print area is rendered as a wireframe box consisting of 12 lines:
- 4 lines for the bottom face (build plate level, Z=0)
- 4 lines for the top face (Z=height)
- 4 vertical edges connecting bottom to top

The box is centered at the origin with:
- X range: [-width/2, width/2]
- Y range: [-depth/2, depth/2]
- Z range: [0, height]

### Coordinate System
The print area visualization uses the same coordinate system as the model:
- **X axis**: Width (Red axis)
- **Y axis**: Depth (Green axis)
- **Z axis**: Height (Blue axis)
- **Origin**: Center of the build plate at Z=0

## Use Cases

1. **Print Size Verification**: Check if a model fits within your printer's build volume
2. **Multi-Model Layout**: Visualize available space when viewing assemblies
3. **Scale Comparison**: Compare model size against known printer dimensions
4. **Print Planning**: Determine if model needs to be scaled or rotated

## Session Persistence

Print area configuration persists during the viewer session:
- Settings are maintained when loading new files
- Configuration survives theme changes
- Settings reset to defaults when the application is closed

## Examples

### Setting Up for a Prusa i3 MK3S (250x210x210mm)
1. Press `C` to open configuration
2. Enter `250` for width
3. Enter `210` for depth
4. Enter `210` for height
5. Press Enter to keep "mm" as unit

### Setting Up for a Creality Ender 3 (220x220x250mm)
1. Press `C` to open configuration
2. Enter `220` for width
3. Enter `220` for depth
4. Enter `250` for height
5. Press Enter to keep "mm" as unit

### Hiding Print Area for Screenshots
1. Press `P` to toggle off
2. The wireframe box will disappear
3. Press `P` again to show it

## Future Enhancements

Potential improvements for future versions:
- Persistent configuration (save/load from config file)
- Preset printer profiles (e.g., "Prusa MK3", "Ender 3", etc.)
- Visual indicators when model exceeds print area
- Build plate grid visualization
- Multiple print area presets
- Print area opacity control
- Custom colors for print area wireframe

## Testing

Unit tests are included to verify:
- `test_print_area_new()`: Default initialization values
- `test_print_area_toggle_visibility()`: Visibility toggle functionality

Run tests with:
```bash
cd tools/viewer
cargo test
```
