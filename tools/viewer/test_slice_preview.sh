#!/bin/bash
# Manual test script for Live Slice Preview feature
# This script provides guided instructions for manual testing

echo "═══════════════════════════════════════════════════════════"
echo "  Live Slice Preview - Manual Test Script"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Build the viewer
echo "Building viewer..."
cd "$(dirname "$0")"
cargo build --bin lib3mf-viewer 2>&1 | grep -E "(Finished|error)" || true
echo ""

if [ ! -f "../../target/debug/lib3mf-viewer" ]; then
    echo "❌ Build failed. Please check errors above."
    exit 1
fi

echo "✓ Build successful!"
echo ""

echo "═══════════════════════════════════════════════════════════"
echo "  Test 1: Basic Window Opening"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Instructions:"
echo "1. The viewer will launch with a sample file"
echo "2. Press 'W' key to open the slice preview window"
echo "3. Verify that a new window opens"
echo "4. Close the preview window (ESC or X button)"
echo "5. Press 'W' again to reopen it"
echo ""
read -p "Press Enter to start Test 1..."

../../target/debug/lib3mf-viewer ../../test_files/core/box.3mf &
VIEWER_PID=$!

echo ""
echo "Viewer launched (PID: $VIEWER_PID)"
echo "Perform the test steps above, then close the viewer when done."
echo ""

wait $VIEWER_PID

echo "═══════════════════════════════════════════════════════════"
echo "  Test 2: Z-Height Synchronization"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Instructions:"
echo "1. Press 'W' to open slice preview window"
echo "2. Press 'Z' in 3D viewer to enable slice view"
echo "3. In preview window, press Up/Down arrows"
echo "4. Observe that:"
echo "   - The red slider in preview window moves"
echo "   - The slice plane in 3D view moves in sync"
echo "   - The contours update in real-time"
echo "5. Try Shift+Up/Down in 3D viewer"
echo "6. Verify preview window updates"
echo ""
read -p "Press Enter to start Test 2..."

../../target/debug/lib3mf-viewer ../../test_files/core/torus.3mf &
VIEWER_PID=$!

echo ""
echo "Viewer launched (PID: $VIEWER_PID)"
echo "Perform the test steps above, then close the viewer when done."
echo ""

wait $VIEWER_PID

echo "═══════════════════════════════════════════════════════════"
echo "  Test 3: Grid and Controls"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Instructions:"
echo "1. Press 'W' to open slice preview window"
echo "2. Press 'G' in preview window to toggle grid"
echo "3. Verify grid appears/disappears"
echo "4. Press 'F' (future: filled mode)"
echo "5. Use PageUp/PageDown for coarse Z adjustment"
echo "6. Verify smooth operation"
echo ""
read -p "Press Enter to start Test 3..."

../../target/debug/lib3mf-viewer ../../test_files/core/box.3mf &
VIEWER_PID=$!

echo ""
echo "Viewer launched (PID: $VIEWER_PID)"
echo "Perform the test steps above, then close the viewer when done."
echo ""

wait $VIEWER_PID

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "  Test Results"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Please verify the following behaviors:"
echo ""
echo "✓ Preview window opens with 'W' key"
echo "✓ Preview window shows model contours"
echo "✓ Grid overlay visible (toggle with 'G')"
echo "✓ Z-slider visible at bottom of preview window"
echo "✓ Up/Down arrows adjust Z-height"
echo "✓ PageUp/PageDown for coarse adjustment"
echo "✓ Changes sync between 3D and 2D windows"
echo "✓ Window can be resized and repositioned"
echo "✓ Window can be closed and reopened"
echo "✓ No crashes or freezes during operation"
echo ""
echo "═══════════════════════════════════════════════════════════"
echo ""
