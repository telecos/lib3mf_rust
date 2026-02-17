#!/bin/bash
# Example script demonstrating lib3mf-slicer usage

set -e

echo "=== lib3mf-slicer Usage Examples ==="
echo ""

# Build the slicer
echo "Building lib3mf-slicer..."
cargo build --release
echo "Done."
echo ""

# Example 1: Basic slicing with default configuration
echo "Example 1: Slicing a simple box model"
cat > /tmp/example_config_1.json << 'EOF'
{
  "slice_thickness_um": 100,
  "printable_box": {
    "origin": { "x": -50.0, "y": -50.0, "z": 0.0 },
    "end": { "x": 50.0, "y": 50.0, "z": 35.0 }
  },
  "resolution": {
    "width": 800,
    "height": 800
  }
}
EOF

echo "Configuration:"
cat /tmp/example_config_1.json
echo ""

../../target/release/lib3mf-slicer \
    ../../test_files/core/box.3mf \
    /tmp/example_config_1.json \
    --output /tmp/example_slices_1 \
    --verbose

echo ""
echo "Slices generated in /tmp/example_slices_1"
echo ""

# Example 2: High-resolution slicing with material support
echo "Example 2: High-resolution slicing with materials"
cat > /tmp/example_config_2.json << 'EOF'
{
  "slice_thickness_um": 50,
  "printable_box": {
    "origin": { "x": 0.0, "y": 0.0, "z": 0.0 },
    "end": { "x": 150.0, "y": 150.0, "z": 100.0 }
  },
  "resolution": {
    "width": 1920,
    "height": 1080
  },
  "spec_support": {
    "meshes": true,
    "materials": true,
    "beam_lattice": true,
    "boolean_ops": true,
    "displacement": true,
    "slice_extension": true
  }
}
EOF

echo "Configuration:"
cat /tmp/example_config_2.json
echo ""

../../target/release/lib3mf-slicer \
    ../../test_files/material/kinect_scan.3mf \
    /tmp/example_config_2.json \
    --output /tmp/example_slices_2

echo ""
echo "Slices generated in /tmp/example_slices_2"
echo ""

echo "=== Examples Complete ==="
echo ""
echo "View the generated PNG files in:"
echo "  /tmp/example_slices_1/"
echo "  /tmp/example_slices_2/"
