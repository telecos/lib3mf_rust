# lib3mf-slicer

A command-line tool for slicing 3MF (3D Manufacturing Format) files into 2D images.

## Overview

`lib3mf-slicer` is a specialized tool that processes 3MF files and generates slice images at specified intervals within a defined printable volume. It supports various 3MF features including:

- Standard mesh geometries
- Materials and surface colors
- Boolean operations
- Beam lattice structures
- Displacement maps
- Slice extension data
- Encrypted content (with optional crypto feature)

## Installation

From the tools/slicer directory:

```bash
cargo build --release
```

The binary will be available at `../../target/release/lib3mf-slicer`

### Optional Features

To enable support for encrypted 3MF files:

```bash
cargo build --release --features crypto
```

## Usage

```bash
lib3mf-slicer <INPUT_FILE> <CONFIG_FILE> [OPTIONS]
```

### Arguments

- `INPUT_FILE`: Path to the 3MF file to slice
- `CONFIG_FILE`: Path to the JSON configuration file

### Options

- `-o, --output <OUTPUT_DIR>`: Output directory for slice images (default: ./slices)
- `-v, --verbose`: Show detailed model information
- `-h, --help`: Print help information
- `-V, --version`: Print version information

### Example

```bash
# Basic usage
lib3mf-slicer model.3mf config.json

# With custom output directory
lib3mf-slicer model.3mf config.json --output my_slices

# With verbose output
lib3mf-slicer model.3mf config.json --verbose
```

## Configuration File Format

The configuration file is a JSON file with the following structure:

```json
{
  "slice_thickness_um": 50,
  "printable_box": {
    "origin": {
      "x": 0.0,
      "y": 0.0,
      "z": 0.0
    },
    "end": {
      "x": 200.0,
      "y": 200.0,
      "z": 200.0
    }
  },
  "resolution": {
    "width": 1920,
    "height": 1080
  },
  "key_file": null,
  "spec_support": {
    "meshes": true,
    "materials": true,
    "beam_lattice": true,
    "boolean_ops": true,
    "displacement": true,
    "slice_extension": true
  }
}
```

### Configuration Parameters

#### Required Parameters

- **slice_thickness_um** (number): Thickness of each slice layer in micrometers (μm)
  - Must be positive
  - Example: `50` = 0.05 mm layers

- **printable_box** (object): Defines the printable volume
  - **origin** (object): Minimum corner point in millimeters
    - **x**, **y**, **z** (numbers): Coordinates in mm
  - **end** (object): Maximum corner point in millimeters
    - **x**, **y**, **z** (numbers): Coordinates in mm
  - All dimensions must be positive (end > origin)

- **resolution** (object): Output image resolution
  - **width** (number): Image width in pixels
  - **height** (number): Image height in pixels
  - Both must be positive integers

#### Optional Parameters

- **key_file** (string or null): Path to public key file for encrypted 3MF files
  - Only used when built with `crypto` feature
  - Default: `null` (no decryption)

- **spec_support** (object or null): Enable/disable specific 3MF features
  - **meshes** (boolean): Support mesh geometries (default: `true`)
  - **materials** (boolean): Support materials (default: `true`)
  - **beam_lattice** (boolean): Support beam lattice structures (default: `true`)
  - **boolean_ops** (boolean): Support boolean operations (default: `true`)
  - **displacement** (boolean): Support displacement maps (default: `true`)
  - **slice_extension** (boolean): Support slice extension (default: `true`)

## Output

The tool generates PNG images for each slice layer, named in the format:

```
slice_00000_z0.000mm.png
slice_00001_z0.050mm.png
slice_00002_z0.100mm.png
...
```

Each image:
- Has a white background
- Shows sliced geometry in black
- Uses the specified resolution
- Covers the defined printable box area

## Examples

### Example 1: Standard Slicing

Create a configuration file `config.json`:

```json
{
  "slice_thickness_um": 50,
  "printable_box": {
    "origin": {"x": 0, "y": 0, "z": 0},
    "end": {"x": 200, "y": 200, "z": 200}
  },
  "resolution": {
    "width": 1920,
    "height": 1080
  }
}
```

Run the slicer:

```bash
lib3mf-slicer model.3mf config.json
```

### Example 2: High-Resolution Slicing

For higher quality output:

```json
{
  "slice_thickness_um": 25,
  "printable_box": {
    "origin": {"x": -100, "y": -100, "z": 0},
    "end": {"x": 100, "y": 100, "z": 150}
  },
  "resolution": {
    "width": 3840,
    "height": 2160
  }
}
```

### Example 3: Selective Feature Support

To disable certain features:

```json
{
  "slice_thickness_um": 50,
  "printable_box": {
    "origin": {"x": 0, "y": 0, "z": 0},
    "end": {"x": 200, "y": 200, "z": 200}
  },
  "resolution": {
    "width": 1920,
    "height": 1080
  },
  "spec_support": {
    "meshes": true,
    "materials": false,
    "beam_lattice": false,
    "boolean_ops": true,
    "displacement": false,
    "slice_extension": true
  }
}
```

## Technical Details

### Slicing Algorithm

1. **Model Loading**: Parse the 3MF file and extract all geometry
2. **Layer Calculation**: Compute Z-heights for each layer based on slice thickness
3. **Mesh Intersection**: For each layer, intersect all meshes with the Z-plane
4. **Contour Assembly**: Collect intersection segments and assemble closed contours
5. **Rasterization**: Triangulate polygons and render to PNG with specified resolution

### Coordinate System

- **Input coordinates**: Millimeters (mm)
- **Slice thickness**: Micrometers (μm)
- **Output images**: Pixels (specified resolution)

The printable box defines the world-space region to slice. The tool automatically scales this region to fit the output image resolution while maintaining aspect ratio.

## Dependencies

- **lib3mf**: 3MF file parsing and mesh operations
- **clap**: Command-line argument parsing
- **serde/serde_json**: JSON configuration parsing
- **image**: PNG image generation
- **earcutr**: Polygon triangulation
- **thiserror**: Error handling

## License

MIT License - See LICENSE file for details

## Contributing

Contributions are welcome! Please see the main repository's CONTRIBUTING.md for guidelines.

## Related Tools

- **lib3mf-viewer**: Interactive 3D viewer for 3MF files
