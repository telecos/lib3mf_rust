# lib3mf Viewer - Usage Examples

## Interactive 3D Viewer

### Basic Usage
Launch the interactive 3D viewer with any 3MF file:
```bash
cargo run --release -- ../../test_files/core/box.3mf --ui
```

When the window opens, you'll see:
- The 3D model rendered in the center
- Interactive controls via mouse
- Model information printed in the console

### Mouse Controls
- **Rotate**: Click and drag with LEFT mouse button
- **Pan**: Click and drag with RIGHT mouse button
- **Zoom**: Use mouse scroll wheel

### Example Commands

View a sphere:
```bash
cargo run --release -- ../../test_files/core/sphere.3mf -u
```

View a more complex model:
```bash
cargo run --release -- ../../test_files/core/cube_gears.3mf --ui
```

View a model with materials:
```bash
cargo run --release -- ../../test_files/material/kinect_scan.3mf --ui
```

## Command-Line Mode (Traditional)

### View model information without UI
```bash
cargo run --release -- ../../test_files/core/box.3mf
```

### Export static preview image
```bash
cargo run --release -- ../../test_files/core/sphere.3mf --export-preview sphere.png
```

### Detailed analysis
```bash
cargo run --release -- ../../test_files/core/torus.3mf --detailed
```

### Export with specific view angle
```bash
# Top view
cargo run --release -- ../../test_files/core/cylinder.3mf -e cylinder_top.png --view-angle top

# Isometric view (default)
cargo run --release -- ../../test_files/core/box.3mf -e box_iso.png --view-angle isometric

# Front view
cargo run --release -- ../../test_files/core/sphere.3mf -e sphere_front.png --view-angle front
```

### Wireframe rendering
```bash
cargo run --release -- ../../test_files/core/torus.3mf -e torus_wire.png --render-style wireframe
```

## Combining Options

View detailed info AND launch UI:
```bash
cargo run --release -- ../../test_files/core/cube_gears.3mf --detailed --ui
```

Export preview AND view in UI:
```bash
cargo run --release -- ../../test_files/core/box.3mf -e box.png --ui
```

## Running the Compiled Binary

After building, you can run the viewer directly:
```bash
# Build first
cargo build --release

# Run the binary
./target/release/lib3mf-viewer ../../test_files/core/sphere.3mf --ui
```

## System Requirements

- **Linux**: Requires X11 libraries (automatically installed by CI)
- **Windows**: No additional dependencies
- **macOS**: No additional dependencies

All platforms require OpenGL support for the interactive viewer.
