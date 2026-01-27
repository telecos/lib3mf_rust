# lib3mf Viewer - Usage Examples

## Test Suite Browser (NEW!)

### Overview
Browse and download official 3MF Consortium test files directly from GitHub without leaving the viewer.

### Starting the Browser
```bash
cargo run --release -- --browse-tests
# or short form:
cargo run --release -- -t
```

### Interactive Workflow

1. **Navigate to a test suite**
   ```
   > 7  # Select suite3_core
   ```

2. **Browse test categories**
   ```
   > 3  # Select positive_test_cases
   ```

3. **Download and open a file**
   ```
   > 1  # Download first test file
   ```
   The file automatically downloads and opens in the 3D viewer!

### Commands
- **Number** - Select item by number
- **b** or **back** - Go to parent directory
- **r** or **refresh** - Clear cache and reload
- **q** or **quit** - Exit browser
- **h** or **help** - Show help

### Within the 3D Viewer
Press **Ctrl+T** anytime to launch the browser while viewing a model.

### File Caching
Downloaded files are cached locally:
- Linux: `~/.cache/lib3mf_viewer/github_cache/`
- macOS: `~/Library/Caches/lib3mf_viewer/github_cache/`
- Windows: `%LOCALAPPDATA%\lib3mf_viewer\github_cache\`

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

### Keyboard Shortcuts
- **Ctrl+O**: Open a local 3MF file
- **Ctrl+T**: Browse test suites from GitHub
- **ESC**: Exit viewer

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
