# Print Area (Build Volume) Visualization Feature

## Overview

The 3MF viewer includes a comprehensive configurable print area visualization feature that displays a wireframe box representing the build volume of a 3D printer. This helps users visualize whether their models fit within the available print area. The feature now includes support for different origin modes, unit systems, ruler visualization, and a scale bar.

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
  - Origin indicator with RGB axes (Red=X, Green=Y, Blue=Z)
  - Box placement depends on origin mode (Corner or CenterBottom)
  - Box extends from Z=0 (build plate) to Z=height

### 3. Print Area Configuration
- **Key**: Press `C` to configure print area dimensions
- **Function**: Opens an interactive console-based configuration dialog
- **Configurable Parameters**:
  - Printer preset selection (Prusa MK3S+, Ender 3, Bambu Lab X1, Creality CR-10, Custom)
  - Width (X dimension) - default: 200mm
  - Depth (Y dimension) - default: 200mm
  - Height (Z dimension) - default: 200mm
  - Unit of measurement (mm or inch) - default: "mm"
  - Origin mode (corner or center) - default: "corner"
  - Ruler visibility (y/n) - default: "n"
  - Scale bar visibility (y/n) - default: "n"
- **Usage**:
  1. Press `C` to open configuration
  2. Select a printer preset (1-5) or press Enter for custom
  3. Enter new values when prompted (press Enter to keep current value)
  4. Configuration is applied immediately

### 4. Ruler Visualization
- **Key**: Press `U` to toggle ruler visibility
- **Function**: Shows/hides graduated rulers along the X, Y, and Z axes
- **Visual Style**:
  - Gray tick marks at regular intervals
  - Major ticks every 5 intervals (larger size)
  - Minor ticks at each interval (smaller size)
  - Tick spacing automatically calculated based on dimensions (5mm, 10mm, 25mm, 50mm, or 100mm)
  - Aligned with print bed edges

### 5. Scale Bar
- **Key**: Press `J` to toggle scale bar visibility
- **Function**: Shows/hides a reference scale bar
- **Visual Style**:
  - Yellow bar in the corner of the print bed
  - Shows a "nice" rounded length (e.g., 10mm, 20mm, 50mm)
  - Length is approximately 10% of the bed width
  - Tick marks at both ends
  - Helps understand model scale at a glance

## Unit System

The viewer now supports two unit systems:
- **Millimeters (mm)**: Default unit, standard for 3MF files
- **Inches (inch)**: Alternative unit system

When switching units, all dimensions are automatically converted:
- 1 inch = 25.4 mm
- Conversion happens transparently during configuration

## Origin Modes

### Corner Origin (Default)
- Origin (0,0,0) is at the front-left corner of the build plate
- Print bed extends in positive X, Y, and Z directions
- This is the traditional 3D printer coordinate system

### Center Bottom Origin
- Origin (0,0,0) is at the center of the build plate
- Print bed extends equally in positive and negative X and Y directions
- Z is still at the build plate level
- Useful for models designed with centered coordinates

## Printer Presets

Built-in presets for popular 3D printers:

| Printer | Width | Depth | Height |
|---------|-------|-------|--------|
| Prusa MK3S+ | 250mm | 210mm | 210mm |
| Ender 3 | 220mm | 220mm | 250mm |
| Bambu Lab X1 | 256mm | 256mm | 256mm |
| Creality CR-10 | 300mm | 300mm | 400mm |
| Custom | User-defined | User-defined | User-defined |

Selecting a preset instantly configures the print bed to match that printer's specifications.

## Default Settings

The print area initializes with the following default values:
- **Width**: 200.0 mm
- **Depth**: 200.0 mm
- **Height**: 200.0 mm
- **Unit**: Millimeters
- **Origin**: Corner
- **Visible**: true (enabled by default)
- **Ruler**: false (disabled by default)
- **Scale Bar**: false (disabled by default)

## Keyboard Controls

| Key | Function |
|-----|----------|
| `M` | Toggle menu display |
| `P` | Toggle print area visibility |
| `C` | Configure print area (dimensions, unit, origin, ruler, scale bar) |
| `U` | Toggle ruler visibility |
| `J` | Toggle scale bar visibility |

## Implementation Details

### Data Structures

```rust
enum LengthUnit {
    Millimeters,
    Inches,
}

enum Origin {
    Corner,           // Origin at (0, 0, 0) corner
    CenterBottom,     // Origin at center of bed, Z=0
}

struct PrintArea {
    width: f32,          // X dimension
    depth: f32,          // Y dimension
    height: f32,         // Z dimension
    unit: LengthUnit,
    origin: Origin,
    visible: bool,
    show_ruler: bool,
    show_scale_bar: bool,
}

struct PrinterPreset {
    name: &'static str,
    width: f32,
    depth: f32,
    height: f32,
}
```

### Unit Conversion

```rust
impl LengthUnit {
    fn to_mm(&self, value: f32) -> f32 {
        match self {
            LengthUnit::Millimeters => value,
            LengthUnit::Inches => value * 25.4,
        }
    }
    
    fn from_mm(&self, value: f32) -> f32 {
        match self {
            LengthUnit::Millimeters => value,
            LengthUnit::Inches => value / 25.4,
        }
    }
}
```

### Wireframe Box Construction

The print area is rendered as a wireframe box consisting of 12 lines:
- 4 lines for the bottom face (build plate level, Z=0)
- 4 lines for the top face (Z=height)
- 4 vertical edges connecting bottom to top
- Origin indicator showing RGB axes (5% of smallest dimension)

Box placement depends on the origin mode:
- **Corner mode**: X range: [0, width], Y range: [0, depth], Z range: [0, height]
- **Center mode**: X range: [-width/2, width/2], Y range: [-depth/2, depth/2], Z range: [0, height]

### Ruler Implementation

Rulers are drawn along each axis with automatic tick spacing:
- **Small beds** (≤50mm): 5mm spacing
- **Medium beds** (≤200mm): 10mm spacing  
- **Large beds** (≤500mm): 25mm spacing
- **Extra large beds** (≤1000mm): 50mm spacing
- **Very large beds** (>1000mm): 100mm spacing

Major ticks appear every 5 minor intervals.

### Scale Bar Implementation

The scale bar uses a "nice number" algorithm to show readable lengths:
- Reference size is approximately 10% of bed width
- Rounded to nice values: 1, 2, 5, 10, 20, 50, 100, etc.
- Drawn as a horizontal line with vertical tick marks at each end

### Coordinate System

The print area visualization uses the same coordinate system as the model:
- **X axis**: Width (Red axis)
- **Y axis**: Depth (Green axis)
- **Z axis**: Height (Blue axis)
- **Origin**: Configurable (Corner or Center Bottom)

## Use Cases

1. **Print Size Verification**: Check if a model fits within your printer's build volume
2. **Multi-Model Layout**: Visualize available space when viewing assemblies
3. **Scale Comparison**: Compare model size against known printer dimensions using rulers
4. **Print Planning**: Determine if model needs to be scaled or rotated
5. **Quick Printer Setup**: Use presets to instantly configure for your specific printer
6. **Unit Conversion**: View dimensions in millimeters or inches as needed
7. **Origin Alignment**: Match the viewer's coordinate system to your slicer or CAD tool

## Session Persistence

Print area configuration persists during the viewer session:
- Settings are maintained when loading new files
- Configuration survives theme changes
- Settings reset to defaults when the application is closed

## Examples

### Setting Up for a Prusa MK3S+ Using Preset
1. Press `C` to open configuration
2. Enter `1` to select "Prusa MK3S+" preset
3. The bed is automatically configured to 250x210x210mm

### Setting Up Custom Dimensions in Inches
1. Press `C` to open configuration
2. Press Enter to skip preset selection
3. Enter `10` for width
4. Enter `8` for depth
5. Enter `10` for height
6. Enter `inch` for unit
7. The bed is now configured as 10"x8"x10" (254x203.2x254mm internally)

### Enabling Full Visualization
1. Press `P` to show print area (if not visible)
2. Press `U` to enable ruler
3. Press `J` to enable scale bar
4. You now have a complete reference grid

### Switching to Center Origin Mode
1. Press `C` to open configuration
2. Skip through presets and dimensions (press Enter)
3. When prompted for origin mode, enter `center`
4. The print bed is now centered at (0,0,0)

### Hiding Print Area for Screenshots
1. Press `P` to toggle off
2. The wireframe box, rulers, and scale bar will disappear
3. Press `P` again to show it

## Testing

Comprehensive unit tests verify all functionality:
- `test_length_unit_conversions()`: Unit conversion accuracy
- `test_print_area_dimensions()`: Dimension calculations with different units
- `test_printer_presets()`: Preset values are correct
- `test_calculate_tick_spacing()`: Ruler tick spacing algorithm
- `test_round_to_nice_number()`: Scale bar nice number rounding
- `test_print_area_toggles()`: All toggle functions work correctly
- `test_print_area_new()`: Default initialization values
- `test_print_area_toggle_visibility()`: Visibility toggle functionality

Run tests with:
```bash
cd tools/viewer
cargo test
```

## Technical Notes

### Coordinate Space
All internal calculations use millimeters regardless of the display unit. This ensures:
- Consistent rendering across different unit systems
- Accurate unit conversions
- No precision loss during unit changes

### Rendering Order
The visualization is rendered in this order:
1. Print area wireframe box
2. Origin indicator (RGB axes)
3. Rulers (if enabled)
4. Scale bar (if enabled)

This ensures proper layering and visibility of all elements.
