# lib3mf Viewer

A powerful tool for viewing and analyzing 3MF (3D Manufacturing Format) files, built using the `lib3mf_rust` library.

## Features

- **Interactive 3D Viewer**: Real-time 3D visualization with mouse controls
  - **GUI Menu Bar** (NEW!): Clickable menu bar with File, View, Settings, Extensions, and Help menus
    - Press 'M' to show/hide menu bar
    - Click menu items for easy access to all features
    - Visual checkmarks show active features
    - Keyboard shortcuts displayed alongside menu items
    - See [GUI_MENU_FEATURE.md](GUI_MENU_FEATURE.md) for detailed documentation
  - **Rotate view**: Left mouse drag
  - **Pan view**: Right mouse drag  
  - **Zoom**: Mouse scroll wheel
  - **XYZ Axes**: Toggle coordinate axes with 'A' key (X=Red, Y=Green, Z=Blue)
  - **Screenshot capture**: Save current view to PNG with 'S' key (auto-timestamped filenames)
  - **Print Area Visualization**: Configurable build volume wireframe box
    - Toggle visibility with 'P' key
    - Configure dimensions with 'C' key
    - Default: 200x200x200mm
  - **2D Slice View**: Interactive cross-section visualization
    - **Toggle slice view**: Press 'Z' to enable/disable 2D slice visualization
    - **Adjust Z height**: Use Shift+Up/Down to move the slice plane
    - **Slice plane**: Yellow rectangle showing the current slice position
    - **Contour display**: Red lines showing model intersection with the plane
    - **Export to PNG**: Press 'X' to export current slice with grid and contours
    - **Toggle plane**: Press 'L' to show/hide the slice plane rectangle
    - See [SLICE_VIEW_FEATURE.md](SLICE_VIEW_FEATURE.md) for detailed documentation
  - **Live Slice Preview Window** (NEW!): Secondary 2D window for real-time slice viewing
    - **Dual window setup**: Separate 2D window alongside 3D viewer
    - **Real-time updates**: Slice preview updates instantly as Z-height changes
    - **Toggle preview**: Press 'W' to open/close the slice preview window
    - **Independent controls**: Up/Down arrows and PageUp/PageDown in preview window
    - **Bidirectional sync**: Changes in either window affect both views
    - **Visual Z-slider**: See current slice position at a glance
    - **Grid overlay**: Toggle with 'G' key for coordinate reference
    - **Export capability**: PNG export from preview window
    - See [LIVE_SLICE_PREVIEW.md](LIVE_SLICE_PREVIEW.md) for detailed documentation
  - **Material Rendering** (NEW!): View materials and colors from 3MF files
    - **Toggle materials**: Press 'R' to toggle between material colors and default gray
    - **Per-triangle colors**: Supports different colors for each triangle face
    - **Material types**: Base materials, color groups, and base material groups
    - **Color information**: Shows material counts in model info and menu
  - **Slice Stack Visualization** (NEW!): Comprehensive slice extension support
    - **Automatic detection**: Recognizes 3MF files with pre-computed slices
    - **Single slice navigation**: Step through slices with Up/Down arrows
    - **3D stack mode**: View all slices simultaneously with color gradient
    - **Animation**: Play/pause with Space, adjust speed with [/]
    - **Spread control**: Separate slices in 3D with Shift+Up/Down
    - **Rendering modes**: Toggle filled/outline with 'N'
    - See [SLICE_STACK_FEATURE.md](SLICE_STACK_FEATURE.md) for detailed documentation
  - **Hardware-accelerated rendering** using OpenGL
  - **Color support** from materials and color groups
  - **Theme customization**: 5 built-in background themes (Dark, Light, Blue, White, Black)
  - **Keyboard shortcuts**: T for themes, B for backgrounds, V for boolean mode, Ctrl+O for file loading
  - **Open files**: Ctrl+O to open file dialog
  - **Browse test suites**: Ctrl+T to browse 3MF Consortium test files from GitHub
  - **Boolean Operations Visualization** (NEW!): Interactive visualization of boolean operations
    - **Three visualization modes**: Normal, Show Inputs, Highlight Operands
    - **Color-coded operands**: Blue for base objects, Red/Orange for operands
    - **Mode cycling**: Press 'V' to cycle through visualization modes
    - **Operation details**: Console output of boolean operation information
- **Test Suite Browser** (NEW!): Browse and download official 3MF Consortium test files
  - **Direct GitHub integration**: Fetch test files from the official repository
  - **Interactive navigation**: Browse through test suite directories
  - **Test categorization**: Identify positive/negative tests and categories
  - **Local caching**: Downloaded files are cached for quick access
  - **Automatic loading**: Selected files load directly into the viewer
- **Load and Display 3MF Files**: Parse and display comprehensive 3D model information
- **Model Analysis**: Show detailed information about:
  - Model properties (unit, namespace, language)
  - Metadata entries
  - Objects and meshes (vertices, triangles, bounding boxes)
  - Materials and color groups
  - Build items and transformations
- **Enhanced 3D Preview**: Generate high-quality preview images with:
  - **Isometric 3D projection** for proper depth perception (default)
  - **Shaded rendering** with face normals for realistic lighting
  - **Color support** from materials and color groups
  - **Multiple view angles**: isometric, top, front, side
  - **Render styles**: shaded or wireframe
- **Detailed Inspection**: View vertex and triangle data
- **Extension Support**: Works with all 3MF extensions

## Installation

Navigate to the viewer directory and build:

```bash
cd tools/viewer
cargo build --release
```

### System Dependencies

On Linux, you may need to install some system libraries:

```bash
sudo apt-get update
sudo apt-get install -y libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev
```

On macOS and Windows, no additional dependencies are required.

## Usage

### Test Suite Browser (NEW!)

Browse and download test files directly from the 3MF Consortium GitHub repository:

```bash
cargo run --release -- --browse-tests
# or short form
cargo run --release -- -t
```

**Features:**
- Navigate through official test suite directories
- View file sizes and test categories
- Download files directly to your local cache
- Automatically open downloaded files in the viewer

**Navigation:**
- Enter a number to select a directory or file
- `b` or `back` - Go to parent directory
- `r` or `refresh` - Clear cache and reload
- `q` or `quit` - Exit browser
- `h` or `help` - Show help

**Within the 3D viewer:**
- Press `Ctrl+T` to open the test suite browser at any time

### Interactive 3D Viewer (NEW!)

Launch the interactive 3D viewer window:
```bash
cargo run --release -- <path-to-3mf-file> --ui
```

**Controls:**

**GUI Menu Bar** (NEW!):
- Click on menu labels (File, View, Settings, Extensions, Help) to access features
- ⌨️ **M Key**: Toggle menu bar visibility
- See [GUI_MENU_FEATURE.md](GUI_MENU_FEATURE.md) for complete menu documentation

**Mouse Controls:**
- 🖱️ **Left Mouse + Drag**: Rotate view around the model
- 🖱️ **Right Mouse + Drag**: Pan the view
- 🖱️ **Scroll Wheel**: Zoom in/out

**Keyboard Shortcuts:**

For a complete, organized list of all keyboard shortcuts, press **H** or **?** in the viewer, or see [KEYBOARD_CONTROLS_GUIDE.md](KEYBOARD_CONTROLS_GUIDE.md).

Key shortcuts include:
- ⌨️ **H or ?**: Show complete help with all shortcuts
- ⌨️ **Ctrl+O**: Open file dialog
- ⌨️ **Ctrl+T**: Browse test suites from GitHub
- ⌨️ **S**: Capture screenshot
- ⌨️ **A**: Toggle XYZ axes
- ⌨️ **M**: Toggle menu bar
- ⌨️ **T**: Cycle background themes
- ⌨️ **F**: Fit model to view
- ⌨️ **ESC**: Exit viewer

See the full categorized list in the viewer (press **H**) or in [KEYBOARD_CONTROLS_GUIDE.md](KEYBOARD_CONTROLS_GUIDE.md).

### Command-Line Mode

Basic usage:
```bash
cargo run --release -- <path-to-3mf-file>
```

Show detailed mesh information:
```bash
cargo run --release -- <path-to-3mf-file> --detailed
```

Show all vertices and triangles (verbose):
```bash
cargo run --release -- <path-to-3mf-file> --show-all
```

Export a preview image:
```bash
cargo run --release -- <path-to-3mf-file> --export-preview output.png
```

Export with different view angles:
```bash
# Isometric view (default) - best for 3D visualization
cargo run --release -- <path-to-3mf-file> --export-preview output.png --view-angle isometric

# Top view - looking down from above
cargo run --release -- <path-to-3mf-file> --export-preview output.png --view-angle top

# Front view - looking from the front
cargo run --release -- <path-to-3mf-file> --export-preview output.png --view-angle front

# Side view - looking from the side
cargo run --release -- <path-to-3mf-file> --export-preview output.png --view-angle side
```

Export with different render styles:
```bash
# Shaded rendering (default) - realistic lighting with face normals
cargo run --release -- <path-to-3mf-file> --export-preview output.png --render-style shaded

# Wireframe rendering - show mesh structure
cargo run --release -- <path-to-3mf-file> --export-preview output.png --render-style wireframe
```

Or run the compiled binary directly:
```bash
./target/release/lib3mf-viewer <path-to-3mf-file> [OPTIONS]
```

### Command-Line Options

- `--browse-tests, -t`: Browse 3MF Consortium test suites from GitHub (NEW!)
- `--ui, -u`: Launch interactive 3D viewer window
- `--detailed, -d`: Show detailed mesh information (vertex/triangle counts, bounding boxes)
- `--show-all, -a`: Show all vertices and triangles (can be very verbose)
- `--export-preview <FILE>, -e <FILE>`: Export a preview image to the specified file
- `--view-angle <ANGLE>`: Choose view angle for preview (isometric, top, front, side). Default: isometric
- `--render-style <STYLE>`: Choose render style (shaded, wireframe). Default: shaded

### Examples

**Browse and load test files from GitHub:**
```bash
# Launch the test suite browser
cargo run --release -- --browse-tests

# From within the 3D viewer, press Ctrl+T to browse test suites
cargo run --release -- --ui
```

**Interactive 3D viewer (recommended):**
```bash
cargo run --release -- ../../test_files/core/box.3mf --ui
cargo run --release -- ../../test_files/core/sphere.3mf --ui

# View boolean operations
cargo run --release -- ../../test_files/boolean_ops/simple_union.3mf --ui
# (Press 'V' to cycle through visualization modes)
# While the viewer is running, press 'S' to capture screenshots
# Screenshots are automatically saved with timestamped filenames like:
# screenshot_2025-01-27_145230.png
```

View a basic 3MF file:
```bash
cargo run --release -- ../../test_files/core/box.3mf
```

View with detailed information:
```bash
cargo run --release -- ../../test_files/core/cube_gears.3mf --detailed
```

Export preview image:
```bash
cargo run --release -- ../../test_files/core/sphere.3mf --export-preview sphere_preview.png
```

Export with isometric shaded view (best for 3D visualization):
```bash
cargo run --release -- ../../test_files/core/torus.3mf --export-preview torus.png
```

Export wireframe view:
```bash
cargo run --release -- ../../test_files/core/box.3mf --export-preview box_wire.png --render-style wireframe
```

Export from different angle:
```bash
cargo run --release -- ../../test_files/core/cylinder.3mf --export-preview cylinder_front.png --view-angle front
```

View all data (very verbose):
```bash
cargo run --release -- ../../test_files/core/box.3mf --show-all
```

## Output Format

The viewer displays information in a structured, easy-to-read format:

```
═══════════════════════════════════════════════════════════
  3MF File Viewer
═══════════════════════════════════════════════════════════

Loading: test_files/core/box.3mf

✓ Model loaded successfully!

┌─ Model Information ────────────────────────────────────┐
│ Unit:                 millimeter                        │
│ XML Namespace:        http://schemas.microsoft.com/3... │
└────────────────────────────────────────────────────────┘

┌─ Metadata ─────────────────────────────────────────────┐
│ Title                Simple Box                         │
│ Designer             lib3mf_rust                        │
└────────────────────────────────────────────────────────┘

... (more sections)
```

## Implementation Details

This viewer demonstrates the following capabilities of lib3mf_rust:

1. **Model Parsing**: Using `Model::from_reader()` to load 3MF files
2. **Resource Inspection**: Accessing objects, materials, and other resources
3. **Mesh Analysis**: Extracting and analyzing vertices and triangles
4. **Metadata Access**: Reading model metadata entries
5. **Build Processing**: Examining build items and transformations
6. **Extension Support**: Working with various 3MF extensions
7. **3D Visualization**: Interactive real-time rendering and static image generation

The viewer provides:
- **Interactive 3D viewer** with:
  - Hardware-accelerated OpenGL rendering using kiss3d
  - Real-time mouse-controlled camera (ArcBall)
  - Material/color group support for colored rendering
  - XYZ coordinate axes (X=Red, Y=Green, Z=Blue)
  - Smooth 60 FPS rendering
- Formatted text output for easy reading
- Bounding box calculations
- **Enhanced 3D preview generation** with:
  - Isometric projection for realistic 3D depth perception
  - Face normal-based shading for better visualization
  - Material/color group support for colored rendering
  - Multiple view angles (isometric, top, front, side)
  - Shaded and wireframe rendering modes
- Detailed mesh inspection capabilities

## Print Area Visualization

The viewer includes a configurable print area (build volume) visualization feature:

- **Toggle Visibility**: Press `P` to show/hide the print area wireframe
- **Configure Dimensions**: Press `C` to set custom dimensions (width, depth, height)
- **View Menu**: Press `M` to see current print area settings
- **Default Size**: 200x200x200mm (suitable for common desktop 3D printers)
- **Visual Style**: Light blue/gray wireframe box that doesn't obscure the model
- **Coordinate System**: Centered at origin, extends from Z=0 (build plate) upward

See [PRINT_AREA_FEATURE.md](PRINT_AREA_FEATURE.md) for detailed documentation.

## Use Cases

- **Test Suite Exploration**: Browse and test official 3MF Consortium test files
- **Interactive Exploration**: Examine 3MF models in real-time with full 3D controls
- **Quick Inspection**: Rapidly examine 3MF file contents without opening a full 3D viewer
- **Debugging**: Verify that 3MF files are correctly formed
- **Analysis**: Understand model structure and properties
- **Documentation**: Generate text reports of model contents
- **Testing**: Validate lib3mf_rust parsing capabilities
- **Preview Generation**: Create static preview images for documentation

## License

This tool is part of lib3mf_rust and is licensed under MIT OR Apache-2.0.
