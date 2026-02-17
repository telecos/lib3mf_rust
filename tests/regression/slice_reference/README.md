# Slice Regression Tests

This directory contains regression tests for slice generation, validating that sample 3MF files generate the expected number of slices with expected pixel content.

## Test Structure

The regression tests are implemented in `slice_tests.rs` and use the following approach:

1. **Run the slicer** on sample 3MF files using predefined configurations
2. **Validate slice count** - Check that the expected number of slices are generated
3. **Compare pixel content** - Compare selected slices against reference PNG images

## Reference Files

The `slice_reference/` directory contains:
- **Configuration files** (`*_config.json`) - Slicer configurations for each test case
- **Reference PNG images** (`*.png`) - Expected slice output at specific Z heights

### Current Test Cases

#### 1. Pyramid Beam Lattice (`pyramid`)
- **Model**: `tools/slicer/samples/pyramid/pyramid.3mf`
- **Expected slices**: 11 slices from Z=0mm to Z=100mm (10mm intervals)
- **Reference slices**: Z=0mm, Z=50mm, Z=100mm
- **Features tested**: Beam lattice slicing, circular cross-sections

#### 2. Box with Slice Stack (`box_sliced`)
- **Model**: `test_files/slices/box_sliced.3mf`
- **Expected slices**: 400 slices from Z=14mm to Z=46mm (0.08mm intervals)
- **Reference slices**: Z=20mm, Z=30mm, Z=40mm
- **Features tested**: Slice stack extraction, transform handling

#### 3. Cube Gears (`cube_gears`)
- **Model**: `tools/slicer/samples/cube_gears/cube_gears.3mf`
- **Expected slices**: 8 slices from Z=20mm to Z=55mm (5mm intervals)
- **Reference slices**: Z=30mm, Z=40mm
- **Features tested**: Multi-component slicing, complex geometry

## Running Tests

### Run all slice regression tests:
```bash
cargo test --test regression slice_regression
```

### Run a specific test:
```bash
cargo test --test regression test_pyramid_slice_regression
cargo test --test regression test_box_sliced_regression
cargo test --test regression test_cube_gears_slice_regression
```

### Run with verbose output:
```bash
cargo test --test regression slice_regression -- --nocapture
```

## Updating Reference Images

If the slicer output changes (e.g., due to algorithm improvements), you may need to update the reference images:

### 1. Generate new reference slices

From the project root:

```bash
cd tools/slicer

# Build the slicer
cargo build --release

# For pyramid
./target/release/lib3mf-slicer \
    samples/pyramid/pyramid.3mf \
    ../../tests/regression/slice_reference/pyramid_config.json \
    -o /tmp/new_pyramid

# For box_sliced
./target/release/lib3mf-slicer \
    ../../test_files/slices/box_sliced.3mf \
    ../../tests/regression/slice_reference/box_sliced_config.json \
    -o /tmp/new_box_sliced

# For cube_gears
./target/release/lib3mf-slicer \
    samples/cube_gears/cube_gears.3mf \
    ../../tests/regression/slice_reference/cube_gears_config.json \
    -o /tmp/new_cube_gears
```

### 2. Copy the reference slices

```bash
# Pyramid references (Z=0, 50, 100mm)
cp /tmp/new_pyramid/slice_00000_z0.000mm.png \
   tests/regression/slice_reference/pyramid_z0.000mm.png
cp /tmp/new_pyramid/slice_00005_z50.000mm.png \
   tests/regression/slice_reference/pyramid_z50.000mm.png
cp /tmp/new_pyramid/slice_00010_z100.000mm.png \
   tests/regression/slice_reference/pyramid_z100.000mm.png

# Box_sliced references (Z=20, 30, 40mm)
cp /tmp/new_box_sliced/slice_00075_z20.000mm.png \
   tests/regression/slice_reference/box_sliced_z20.000mm.png
cp /tmp/new_box_sliced/slice_00200_z30.000mm.png \
   tests/regression/slice_reference/box_sliced_z30.000mm.png
cp /tmp/new_box_sliced/slice_00325_z40.000mm.png \
   tests/regression/slice_reference/box_sliced_z40.000mm.png

# Cube_gears references (Z=30, 40mm)
cp /tmp/new_cube_gears/slice_00002_z30.000mm.png \
   tests/regression/slice_reference/cube_gears_z30.000mm.png
cp /tmp/new_cube_gears/slice_00004_z40.000mm.png \
   tests/regression/slice_reference/cube_gears_z40.000mm.png
```

### 3. Verify the tests pass

```bash
cargo test --test regression slice_regression
```

## Adding New Test Cases

To add a new slice regression test:

1. **Choose a sample file** - Select a 3MF file that exercises specific features
2. **Create a configuration** - Add a `*_config.json` file to `slice_reference/`
3. **Generate reference slices** - Use the slicer to generate output
4. **Select representative slices** - Choose 2-3 slices at different Z heights
5. **Copy reference images** - Save the selected slices to `slice_reference/`
6. **Add a test function** - Create a new test in `slice_tests.rs`:

```rust
#[test]
fn test_my_new_sample_regression() {
    let test_case = SliceTestCase {
        name: "my new sample",
        model_path: "path/to/sample.3mf",
        config_path: "my_sample_config.json",
        expected_slice_count: 42,
        reference_slices: vec![
            (10.0, "my_sample_z10.000mm.png"),
            (50.0, "my_sample_z50.000mm.png"),
        ],
    };
    
    run_slice_test(&test_case);
}
```

## Image Comparison

The tests use a pixel-by-pixel comparison with a tolerance of **0.1%** difference to account for minor rendering variations. Images are compared in RGB color space.

If a test fails with a difference close to the threshold, inspect the generated images manually to determine if the difference is acceptable or indicates a regression.
