#!/bin/bash
# Setup script for 3MF conformance testing

set -e

echo "=== 3MF Conformance Test Setup ==="
echo

# Check if test_suites directory exists
if [ -d "test_suites" ]; then
    echo "✓ Test suites already cloned"
else
    echo "Cloning official 3MF test suites..."
    git clone https://github.com/3MFConsortium/test_suites.git
    echo "✓ Test suites cloned successfully"
fi

echo
echo "=== Running Conformance Tests ==="
echo

# Run the summary test
cargo test --test conformance_tests summary -- --ignored --nocapture

echo
echo "=== Setup Complete ==="
echo
echo "You can now run individual test suites:"
echo "  cargo test --test conformance_tests suite3_core"
echo "  cargo test --test conformance_tests suite7_beam"
echo
echo "Or run the summary again:"
echo "  cargo test --test conformance_tests summary -- --ignored --nocapture"
