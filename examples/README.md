# lib3mf_rust Examples

This directory contains examples demonstrating various features and use cases of the lib3mf_rust library.

## Basic Usage

### Core Functionality

#### `parse_3mf.rs` - Basic Parsing
Demonstrates how to create and parse a simple 3MF file in memory, inspecting model data, metadata, objects, and build items.

```bash
cargo run --example parse_3mf
```

#### `write_3mf.rs` - Creating 3MF Files
Shows how to create a new 3MF model from scratch, add geometry, materials, colors, and metadata, then write it to a file.

```bash
cargo run --example write_3mf
```

Creates `colored_cube.3mf` in the current directory.

#### `extension_support.rs` - Extension Handling
Demonstrates how to configure the parser to accept specific extensions and handle files requiring unsupported extensions.

```bash
# Accept all extensions
cargo run --example extension_support test_files/material/kinect_scan.3mf all

# Core only
cargo run --example extension_support test_files/core/box.3mf core-only

# Core + Materials
cargo run --example extension_support test_files/material/kinect_scan.3mf core-mat
```

#### `validation_errors.rs` - Validation & Error Handling
Shows how to handle parsing errors, validate file structure, and provide meaningful error messages.

```bash
cargo run --example validation_errors test_files/core/box.3mf permissive
cargo run --example validation_errors test_files/core/box.3mf strict
```

#### `custom_extension.rs` - Custom Extension Support
Demonstrates how to register and handle custom/proprietary 3MF extensions with element and validation handlers.

```bash
cargo run --example custom_extension
```

## Geometry and Mesh Operations

#### `mesh_analysis.rs` - Mesh Operations
Demonstrates triangle mesh operations including volume computation, bounding boxes, transformations, and build volume analysis.

```bash
cargo run --example mesh_analysis test_files/core/box.3mf
```

#### `build_transformations.rs` - Build Items & Transformations
Shows how to work with build items, parse transformation matrices, and apply transformations to geometry.

```bash
cargo run --example build_transformations test_files/core/box.3mf
```

#### `components.rs` - Component Hierarchies
Demonstrates how components are parsed and validated in 3MF files.

```bash
cargo run --example components
```

## Extension-Specific Examples

### Materials Extension

#### `advanced_materials.rs` - Advanced Materials
Shows how to access advanced materials including Texture2D, composite materials, and multi-properties.

```bash
cargo run --example advanced_materials test_files/material/kinect_scan.3mf
```

#### `extract_colors.rs` - Color Extraction
Demonstrates extracting material and color information for rendering, mapping colors to triangles.

```bash
cargo run --example extract_colors test_files/material/kinect_scan.3mf
```

### Beam Lattice Extension

#### `beam_lattice_demo.rs` - Beam Lattice Structures
Shows how to parse and access beam lattice data including beamsets, beams, and beam properties.

```bash
cargo run --example beam_lattice_demo
```

### Slice Extension

#### `slice_extension_demo.rs` - Slice Data
Demonstrates slice extension support for layer-by-layer manufacturing.

```bash
cargo run --example slice_extension_demo
```

### Production Extension

#### `production_handler_demo.rs` - Production Information
Shows how to validate and access production extension data including UUIDs and paths.

```bash
cargo run --example production_handler_demo
```

### Secure Content Extension

#### `secure_content_handler.rs` - Secure Content
Demonstrates validation of secure content in 3MF models.

```bash
cargo run --example secure_content_handler
```

## Polygon Operations

#### `polygon_clipping_demo.rs` - Polygon Clipping
Shows how to use polygon clipping operations for resolving self-intersections in slice data.

```bash
cargo run --example polygon_clipping_demo
```

## Format Conversion

#### `export_to_stl.rs` - STL Export
Converts 3MF files to STL (STereoLithography) format for 3D printing.

```bash
cargo run --example export_to_stl test_files/core/box.3mf output.stl
```

#### `export_to_obj.rs` - OBJ Export
Converts 3MF files to OBJ format with MTL material file for 3D modeling tools.

```bash
cargo run --example export_to_obj test_files/core/box.3mf output.obj
```

## Utilities

#### `extract_thumbnail.rs` - Thumbnail Extraction
Shows how to check for and extract thumbnail images from 3MF files.

```bash
cargo run --example extract_thumbnail test_files/core/box.3mf
```

## Example Categories

### By Skill Level

**Beginner:**
- `parse_3mf.rs` - Start here to understand basic parsing
- `write_3mf.rs` - Learn to create 3MF files
- `extension_support.rs` - Understand extension configuration

**Intermediate:**
- `mesh_analysis.rs` - Work with mesh geometry
- `build_transformations.rs` - Handle transformations
- `validation_errors.rs` - Robust error handling
- `extract_colors.rs` - Material and color mapping
- `export_to_stl.rs` / `export_to_obj.rs` - Format conversion

**Advanced:**
- `custom_extension.rs` - Implement custom extensions
- `polygon_clipping_demo.rs` - Advanced polygon operations
- Extension-specific demos for specialized use cases

### By Use Case

**3D Printing Applications:**
- `parse_3mf.rs` - Load models
- `export_to_stl.rs` - Convert for slicers
- `slice_extension_demo.rs` - Pre-sliced data
- `mesh_analysis.rs` - Validate printability

**3D Modeling Tools:**
- `export_to_obj.rs` - Export for modeling software
- `build_transformations.rs` - Instance placement
- `components.rs` - Assembly hierarchies

**Manufacturing Systems:**
- `production_handler_demo.rs` - Production workflows
- `validation_errors.rs` - Quality control
- `beam_lattice_demo.rs` - Lattice structures

**Rendering/Visualization:**
- `extract_colors.rs` - Color data for rendering
- `mesh_analysis.rs` - Bounding boxes and volumes
- `extract_thumbnail.rs` - Preview images

## Notes

- All examples require the library to be built: `cargo build`
- Test files are located in the `test_files/` directory at the repository root
- Many examples accept command-line arguments - run without arguments to see usage
- Examples are self-contained and can be used as templates for your own code
- See the main [README.md](../README.md) for library documentation and installation instructions

## Contributing

When adding new examples:
1. Add clear documentation comments at the top explaining what the example demonstrates
2. Include command-line usage instructions
3. Make examples self-contained and runnable
4. Update this README with the new example
5. Group by category (basic/extensions/utilities)
