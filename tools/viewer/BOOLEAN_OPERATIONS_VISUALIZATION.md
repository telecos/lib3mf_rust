# Boolean Operations Visualization

This document describes the boolean operations visualization features in the lib3mf viewer.

## Overview

The 3MF Boolean Operations extension allows objects to be defined using volumetric boolean operations (union, difference, intersection). The lib3mf viewer provides interactive visualization of these boolean operations to help understand their structure and composition.

## Boolean Operations Support

The viewer supports visualization of all boolean operation types defined in the 3MF Boolean Operations extension:

- **Union**: Combines two or more volumes into a single volume
- **Difference**: Subtracts one volume from another
- **Intersection**: Keeps only the overlapping region of volumes

## Visualization Modes

The viewer provides three visualization modes for boolean operations, accessible by pressing the **V** key:

### 1. Normal Mode

- Default mode showing all meshes with their standard colors
- Boolean operation structure is not visually differentiated
- Useful for seeing the overall model appearance

### 2. Show Inputs Mode

- Color-codes boolean operation components:
  - **Blue**: Base objects (the first operand)
  - **Red**: Operand objects (objects being combined with the base)
  - **Default colors**: Non-boolean objects
- All objects remain visible
- Useful for understanding which objects participate in boolean operations

### 3. Highlight Operands Mode

- Shows only objects involved in boolean operations
- Uses bright, high-contrast colors:
  - **Bright Blue**: Base objects
  - **Bright Orange**: Operand objects
- Non-boolean objects are hidden
- Useful for isolating and examining boolean operation inputs

## Using the Viewer

### Command Line

Run the viewer with a 3MF file containing boolean operations:

```bash
# Launch in UI mode
cargo run --release --bin lib3mf-viewer -- --ui test_files/boolean_ops/simple_union.3mf

# Or for non-UI mode (displays information only)
cargo run --release --bin lib3mf-viewer -- test_files/boolean_ops/simple_union.3mf
```

### Interactive Controls

When viewing a model in UI mode:

- **V**: Cycle through visualization modes (Normal → Show Inputs → Highlight Operands → Normal)
- **T**: Cycle through background themes
- **A**: Toggle coordinate axes
- **B**: Toggle beam lattice (if present)
- **Ctrl+O**: Open a new file
- **Ctrl+T**: Browse 3MF Consortium test suites
- **ESC**: Exit viewer

### Boolean Operation Information

When in Show Inputs or Highlight Operands mode, the viewer prints detailed information about boolean operations to the console:

```
═══════════════════════════════════════════════════════════
  Boolean Operations Information
═══════════════════════════════════════════════════════════

  Object ID: 3
    Operation: union
    Base Object: 1
    Operands: 1 objects
      [1] Object ID: 2

═══════════════════════════════════════════════════════════
```

## Example Usage

A complete example demonstrating boolean operations visualization is available:

```bash
cargo run --example boolean_operations_demo test_files/boolean_ops/simple_union.3mf
```

This example:
1. Loads a 3MF file with boolean operations
2. Detects and lists all boolean operations
3. Displays operation details (type, base object, operands)
4. Provides instructions for interactive visualization

## Creating Test Files

The repository includes a test file generator for boolean operations:

```bash
python3 /tmp/create_boolean_test.py
```

This creates `test_files/boolean_ops/simple_union.3mf` with:
- Two overlapping cube meshes
- A boolean union operation combining them
- Proper triangle winding for valid geometry

## Implementation Details

### Data Structures

Boolean operations are represented in the model using:

```rust
pub struct BooleanShape {
    pub objectid: usize,          // Base object ID
    pub operation: BooleanOpType, // union, difference, or intersection
    pub path: Option<String>,     // Optional external file path
    pub operands: Vec<BooleanRef>, // List of operand objects
}

pub struct BooleanRef {
    pub objectid: usize,      // Operand object ID
    pub path: Option<String>, // Optional external file path
}

pub enum BooleanOpType {
    Union,
    Difference,
    Intersection,
}
```

### Visualization Algorithm

1. **Detection**: Scan all objects for `boolean_shape` field
2. **Classification**: Build sets of base objects and operand objects
3. **Rendering**:
   - Normal mode: Render all objects with default colors
   - Show Inputs mode: Apply color coding while rendering all objects
   - Highlight Operands mode: Render only boolean-related objects with high-contrast colors

### Color Scheme

| Mode               | Base Object    | Operand Object | Other Objects |
|--------------------|----------------|----------------|---------------|
| Normal             | Default        | Default        | Default       |
| Show Inputs        | Blue (0.3, 0.5, 0.9) | Red (0.9, 0.3, 0.3) | Default |
| Highlight Operands | Bright Blue (0.2, 0.6, 1.0) | Bright Orange (1.0, 0.4, 0.2) | Hidden |

## Limitations

- The viewer currently displays the input meshes of boolean operations, not the computed result
- Actual boolean CSG computation is not performed (this would require a mesh processing library)
- Nested boolean operations are supported in the data model but visualization shows the flat structure
- No animation or interpolation between states

## Future Enhancements

Possible improvements include:

1. **Result Computation**: Integrate a CSG library to compute and display actual boolean results
2. **Animation**: Smooth transitions when cycling between visualization modes
3. **Transparency**: Semi-transparent rendering to show overlapping regions
4. **Hierarchy View**: Visual representation of nested boolean operations
5. **Edge Highlighting**: Highlight intersection edges between operands
6. **Info Overlay**: On-screen display of operation details instead of console output

## See Also

- [3MF Boolean Operations Specification](https://github.com/3MFConsortium/spec_booleanoperations)
- [examples/boolean_operations_demo.rs](../examples/boolean_operations_demo.rs)
- [AXIS_VISUALIZATION.md](../../tools/viewer/AXIS_VISUALIZATION.md) - Similar visualization feature for axes
- [BEAM_LATTICE_RENDERING.md](../../tools/viewer/BEAM_LATTICE_RENDERING.md) - Beam lattice visualization
