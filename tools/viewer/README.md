# lib3mf Viewer

A powerful tool for viewing and analyzing 3MF (3D Manufacturing Format) files, built using the `lib3mf_rust` library.

## Features

- **Interactive 3D Viewer**: Real-time 3D visualization with mouse controls
  - **Rotate view**: Left mouse drag
  - **Pan view**: Right mouse drag  
  - **Zoom**: Mouse scroll wheel
  - **Hardware-accelerated rendering** using OpenGL
  - **Color support** from materials and color groups
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

### Interactive 3D Viewer (NEW!)

Launch the interactive 3D viewer window:
```bash
cargo run --release -- <path-to-3mf-file> --ui
```

**Controls:**
- 🖱️ **Left Mouse + Drag**: Rotate view around the model
- 🖱️ **Right Mouse + Drag**: Pan the view
- 🖱️ **Scroll Wheel**: Zoom in/out
- ⌨️ **Arrow Keys**: Pan the view
- ⌨️ **ESC / Close Window**: Exit viewer

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

- `--ui, -u`: Launch interactive 3D viewer window (NEW!)
- `--detailed, -d`: Show detailed mesh information (vertex/triangle counts, bounding boxes)
- `--show-all, -a`: Show all vertices and triangles (can be very verbose)
- `--export-preview <FILE>, -e <FILE>`: Export a preview image to the specified file
- `--view-angle <ANGLE>`: Choose view angle for preview (isometric, top, front, side). Default: isometric
- `--render-style <STYLE>`: Choose render style (shaded, wireframe). Default: shaded

### Examples

**Interactive 3D viewer (recommended):**
```bash
cargo run --release -- ../../test_files/core/box.3mf --ui
cargo run --release -- ../../test_files/core/sphere.3mf --ui
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

## Use Cases

- **Interactive Exploration**: Examine 3MF models in real-time with full 3D controls
- **Quick Inspection**: Rapidly examine 3MF file contents without opening a full 3D viewer
- **Debugging**: Verify that 3MF files are correctly formed
- **Analysis**: Understand model structure and properties
- **Documentation**: Generate text reports of model contents
- **Testing**: Validate lib3mf_rust parsing capabilities
- **Preview Generation**: Create static preview images for documentation

## License

This tool is part of lib3mf_rust and is licensed under MIT OR Apache-2.0.
