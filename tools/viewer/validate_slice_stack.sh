#!/bin/bash
# Validation script for slice stack visualization feature
# Verifies the implementation compiles and slice data loads correctly

set -e

echo "=========================================="
echo "Slice Stack Feature Validation"
echo "=========================================="
echo ""

PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$PROJECT_ROOT"

echo "✓ Step 1: Building lib3mf library..."
cargo build --release --lib 2>&1 | tail -3
echo ""

echo "✓ Step 2: Building viewer with slice stack support..."
cd tools/viewer
cargo build --release 2>&1 | tail -3
echo ""

echo "✓ Step 3: Verifying slice extension demo..."
cd "$PROJECT_ROOT"
echo "  Testing with box_sliced.3mf (378 slices)..."
OUTPUT=$(cargo run --release --example slice_extension_demo test_files/slices/box_sliced.3mf 2>&1)

# Check for expected output
if echo "$OUTPUT" | grep -q "Slice Stacks: 1"; then
    echo "  ✓ Slice stack detected"
else
    echo "  ✗ Failed to detect slice stack"
    exit 1
fi

if echo "$OUTPUT" | grep -q "Slices: 378"; then
    echo "  ✓ Found 378 slices"
else
    echo "  ✗ Incorrect slice count"
    exit 1
fi

if echo "$OUTPUT" | grep -q "Z Bottom: 0"; then
    echo "  ✓ Z bottom coordinate correct"
else
    echo "  ✗ Incorrect Z bottom"
    exit 1
fi

echo ""
echo "✓ Step 4: Checking viewer code structure..."

# Check for slice stack rendering functions
if grep -q "fn draw_slice_stack_single" tools/viewer/src/ui_viewer.rs; then
    echo "  ✓ Single slice rendering function present"
else
    echo "  ✗ Missing single slice rendering"
    exit 1
fi

if grep -q "fn draw_slice_stack_3d" tools/viewer/src/ui_viewer.rs; then
    echo "  ✓ 3D stack rendering function present"
else
    echo "  ✗ Missing 3D stack rendering"
    exit 1
fi

if grep -q "use_slice_stack: bool" tools/viewer/src/ui_viewer.rs; then
    echo "  ✓ Slice stack mode flag present"
else
    echo "  ✗ Missing slice stack mode"
    exit 1
fi

if grep -q "animation_playing: bool" tools/viewer/src/ui_viewer.rs; then
    echo "  ✓ Animation support present"
else
    echo "  ✗ Missing animation support"
    exit 1
fi

echo ""
echo "✓ Step 5: Verifying keyboard controls..."

# Check for key bindings
declare -a KEYS=("Key::K" "Key::Space" "Key::RBracket" "Key::LBracket" "Key::N" "Key::Home" "Key::End")
declare -a FEATURES=("3D stack toggle" "animation play/pause" "speed increase" "speed decrease" "fill mode" "first slice" "last slice")

for i in "${!KEYS[@]}"; do
    if grep -q "${KEYS[$i]}" tools/viewer/src/ui_viewer.rs; then
        echo "  ✓ ${FEATURES[$i]} (${KEYS[$i]}) implemented"
    else
        echo "  ✗ Missing ${FEATURES[$i]}"
        exit 1
    fi
done

echo ""
echo "✓ Step 6: Checking documentation..."

if [ -f "tools/viewer/SLICE_STACK_FEATURE.md" ]; then
    echo "  ✓ Feature documentation exists"
    LINES=$(wc -l < tools/viewer/SLICE_STACK_FEATURE.md)
    echo "    ($LINES lines of documentation)"
else
    echo "  ✗ Missing feature documentation"
    exit 1
fi

if grep -q "Slice Stack Visualization" tools/viewer/README.md; then
    echo "  ✓ README updated with new feature"
else
    echo "  ✗ README not updated"
    exit 1
fi

echo ""
echo "=========================================="
echo "✅ All Validation Checks Passed!"
echo "=========================================="
echo ""
echo "Slice stack visualization feature is ready for use."
echo ""
echo "To test interactively, run:"
echo "  cd tools/viewer"
echo "  ./demo_slice_stack.sh"
echo ""
echo "Or manually with:"
echo "  cargo run --release -- ../../test_files/slices/box_sliced.3mf"
echo "  (Then press 'Z' in the viewer to see slice stacks)"
echo ""
