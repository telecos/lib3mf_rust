#!/bin/bash
# Demo script for displacement visualization in the 3MF viewer

echo "==================================================================="
echo "  3MF Displacement Visualization Demo"
echo "==================================================================="
echo ""
echo "This demo shows how to use the displacement visualization feature."
echo ""
echo "Features:"
echo "  - Automatic detection of displacement data"
echo "  - Visual highlighting with bright cyan color"
echo "  - Detailed displacement statistics"
echo "  - Toggle on/off with 'D' key"
echo ""
echo "==================================================================="
echo ""

# Navigate to viewer directory
cd "$(dirname "$0")" || exit 1

echo "Building the viewer..."
cargo build --release
echo ""

echo "==================================================================="
echo "  Starting Viewer"
echo "==================================================================="
echo ""
echo "Controls:"
echo "  D        - Toggle displacement visualization"
echo "  M        - Show menu with displacement info"
echo "  Ctrl+O   - Open a 3MF file"
echo "  ESC      - Exit"
echo ""
echo "Try loading a 3MF file with displacement data!"
echo "==================================================================="
echo ""

# Run the viewer
cargo run --release

echo ""
echo "Demo completed!"
