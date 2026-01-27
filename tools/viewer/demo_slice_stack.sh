#!/bin/bash
# Demo script for slice stack visualization feature
# This script demonstrates how to use the viewer with slice stack test files

set -e

VIEWER_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$VIEWER_DIR/../.." && pwd)"
SLICE_TEST_FILE="$PROJECT_ROOT/test_files/slices/box_sliced.3mf"

echo "=========================================="
echo "Slice Stack Visualization Demo"
echo "=========================================="
echo ""

# Check if test file exists
if [ ! -f "$SLICE_TEST_FILE" ]; then
    echo "Error: Test file not found: $SLICE_TEST_FILE"
    exit 1
fi

echo "✓ Found test file: box_sliced.3mf"
echo ""

# First, analyze the slice data using the example
echo "Step 1: Analyzing slice stack data..."
echo "----------------------------------------"
cd "$PROJECT_ROOT"
cargo run --release --example slice_extension_demo "$SLICE_TEST_FILE"
echo ""

echo "=========================================="
echo "Interactive Viewer Instructions"
echo "=========================================="
echo ""
echo "The viewer will now launch. To explore slice stacks:"
echo ""
echo "1. Press 'Z' to enable slice view"
echo "   → Slice stack will be automatically detected"
echo "   → Initial slice information will be displayed"
echo ""
echo "2. Navigate through slices:"
echo "   → Up/Down arrows: Move through slices one by one"
echo "   → Home: Jump to first slice"
echo "   → End: Jump to last slice"
echo ""
echo "3. Enable 3D stack visualization:"
echo "   → Press 'K' to show all 378 slices in 3D"
echo "   → Use Shift+Up/Down to spread slices apart"
echo "   → Current slice is highlighted in yellow"
echo ""
echo "4. Start animation:"
echo "   → Press Space to play/pause"
echo "   → Press '[' to slow down"
echo "   → Press ']' to speed up"
echo ""
echo "5. Toggle rendering modes:"
echo "   → Press 'N' for filled/outline mode"
echo "   → Press 'L' to show/hide slice plane"
echo ""
echo "6. Other controls:"
echo "   → Press 'S' to toggle slice stack mode"
echo "   → Press 'X' to export current slice to PNG"
echo "   → Press 'M' to see current settings menu"
echo "   → Press Ctrl+Q to quit"
echo ""
echo "Press Enter to launch the viewer..."
read

# Launch the viewer
cd "$VIEWER_DIR"
echo "Launching viewer..."
cargo run --release -- "$SLICE_TEST_FILE"

echo ""
echo "Demo complete!"
