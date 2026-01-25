# lib3mf Viewer

A command-line tool for viewing and analyzing 3MF (3D Manufacturing Format) files, built using the `lib3mf_rust` library.

## Features

- **Load and Display 3MF Files**: Parse and display comprehensive 3D model information
- **Model Analysis**: Show detailed information about:
  - Model properties (unit, namespace, language)
  - Metadata entries
  - Objects and meshes (vertices, triangles, bounding boxes)
  - Materials and color groups
  - Build items and transformations
- **Wireframe Export**: Generate preview images of the model (top-view orthographic projection)
- **Detailed Inspection**: View vertex and triangle data
- **Extension Support**: Works with all 3MF extensions

## Installation

Navigate to the viewer directory and build:

```bash
cd tools/viewer
cargo build --release
```

## Usage

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

Export a wireframe preview image:
```bash
cargo run --release -- <path-to-3mf-file> --export-preview output.png
```

Or run the compiled binary directly:
```bash
./target/release/lib3mf-viewer <path-to-3mf-file> [OPTIONS]
```

### Command-Line Options

- `--detailed, -d`: Show detailed mesh information (vertex/triangle counts, bounding boxes)
- `--show-all, -a`: Show all vertices and triangles (can be very verbose)
- `--export-preview <FILE>, -e <FILE>`: Export a wireframe preview image to the specified file

### Examples

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

The viewer provides:
- Formatted text output for easy reading
- Bounding box calculations
- Wireframe preview generation using the `image` crate
- Detailed mesh inspection capabilities

## Use Cases

- **Quick Inspection**: Rapidly examine 3MF file contents without opening a full 3D viewer
- **Debugging**: Verify that 3MF files are correctly formed
- **Analysis**: Understand model structure and properties
- **Documentation**: Generate text reports of model contents
- **Testing**: Validate lib3mf_rust parsing capabilities
- **Preview Generation**: Create simple wireframe previews for documentation

## License

This tool is part of lib3mf_rust and is licensed under MIT OR Apache-2.0.
