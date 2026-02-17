//! Regression tests for slice generation
//!
//! These tests validate that sample files generate the expected number of slices
//! with expected pixel content by comparing against reference PNG images.

use image::{RgbImage, open};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Test configuration for a sample file
struct SliceTestCase {
    /// Name of the test case
    name: &'static str,
    /// Path to the 3MF file relative to project root
    model_path: &'static str,
    /// Path to the slicer config relative to test directory
    config_path: &'static str,
    /// Expected total number of slices
    expected_slice_count: usize,
    /// Reference slices to check (Z height in mm, reference filename)
    reference_slices: Vec<(f64, &'static str)>,
}

/// Calculate the percentage difference between two images
/// Returns a value between 0.0 (identical) and 1.0 (completely different)
fn compare_images(img1: &RgbImage, img2: &RgbImage) -> f64 {
    if img1.dimensions() != img2.dimensions() {
        return 1.0; // Completely different if dimensions don't match
    }

    let (width, height) = img1.dimensions();
    let total_pixels = (width * height * 3) as f64; // 3 channels (RGB)

    let mut diff_sum = 0u64;
    for y in 0..height {
        for x in 0..width {
            let p1 = img1.get_pixel(x, y);
            let p2 = img2.get_pixel(x, y);

            // Calculate absolute difference for each channel
            diff_sum += (p1[0] as i32 - p2[0] as i32).unsigned_abs() as u64;
            diff_sum += (p1[1] as i32 - p2[1] as i32).unsigned_abs() as u64;
            diff_sum += (p1[2] as i32 - p2[2] as i32).unsigned_abs() as u64;
        }
    }

    // Normalize to 0.0-1.0 range
    (diff_sum as f64) / (total_pixels * 255.0)
}

/// Run the slicer on a test case and validate the output
fn run_slice_test(test_case: &SliceTestCase) {
    // Build the slicer if needed
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let slicer_dir = project_root.join("tools/slicer");

    // Check if the slicer binary exists, build it if not
    // First try the workspace target directory (when running from workspace root)
    let mut slicer_binary = project_root
        .parent()
        .map(|p| p.join("tools/slicer/target/release/lib3mf-slicer"))
        .filter(|p| p.exists())
        .unwrap_or_else(|| {
            // Fall back to the slicer's own target directory
            slicer_dir.join("target/release/lib3mf-slicer")
        });

    // If still not found, try debug build
    if !slicer_binary.exists() {
        slicer_binary = slicer_dir.join("target/debug/lib3mf-slicer");
    }

    if !slicer_binary.exists() {
        println!("Building slicer binary...");
        let build_status = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(&slicer_dir)
            .status()
            .expect("Failed to build slicer");
        assert!(build_status.success(), "Failed to build slicer");

        // Update binary path after build
        slicer_binary = slicer_dir.join("target/release/lib3mf-slicer");
    }

    // Create temporary directory for output
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_dir = temp_dir.path();

    // Run the slicer
    let model_path = project_root.join(test_case.model_path);
    let config_path = project_root
        .join("tests/regression/slice_reference")
        .join(test_case.config_path);

    println!("Running slicer for test case: {}", test_case.name);
    println!("  Model: {}", model_path.display());
    println!("  Config: {}", config_path.display());
    println!("  Output: {}", output_dir.display());

    let output = Command::new(&slicer_binary)
        .arg(&model_path)
        .arg(&config_path)
        .arg("-o")
        .arg(output_dir)
        .output()
        .expect("Failed to run slicer");

    if !output.status.success() {
        eprintln!("Slicer stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("Slicer stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Slicer failed with status: {}", output.status);
    }

    // Count generated slices
    let slice_files: Vec<_> = fs::read_dir(output_dir)
        .expect("Failed to read output directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "png" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    println!("  Generated {} slices", slice_files.len());
    assert_eq!(
        slice_files.len(),
        test_case.expected_slice_count,
        "Expected {} slices but got {}",
        test_case.expected_slice_count,
        slice_files.len()
    );

    // Compare reference slices
    let reference_dir = project_root.join("tests/regression/slice_reference");

    for (z_height, reference_filename) in &test_case.reference_slices {
        // Find the generated slice at this Z height
        let z_str = format!("z{:.3}mm.png", z_height);
        let generated_slice = slice_files
            .iter()
            .find(|path| {
                path.file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .ends_with(&z_str)
            })
            .unwrap_or_else(|| panic!("Could not find generated slice at Z={}", z_height));

        let reference_slice = reference_dir.join(reference_filename);

        println!("  Comparing slice at Z={} mm", z_height);
        println!("    Generated: {}", generated_slice.display());
        println!("    Reference: {}", reference_slice.display());

        // Load images
        let generated_img = open(generated_slice)
            .expect("Failed to load generated image")
            .to_rgb8();
        let reference_img = open(&reference_slice)
            .unwrap_or_else(|_| {
                panic!(
                    "Failed to load reference image: {}",
                    reference_slice.display()
                )
            })
            .to_rgb8();

        // Compare dimensions
        assert_eq!(
            generated_img.dimensions(),
            reference_img.dimensions(),
            "Image dimensions don't match for slice at Z={} mm",
            z_height
        );

        // Compare pixel content
        let difference = compare_images(&generated_img, &reference_img);
        println!("    Difference: {:.4}%", difference * 100.0);

        // Allow up to 0.1% difference to account for minor rendering variations
        assert!(
            difference < 0.001,
            "Slice at Z={} mm differs too much from reference (difference: {:.4}%)",
            z_height,
            difference * 100.0
        );
    }

    println!("✓ Test case '{}' passed", test_case.name);
}

#[test]
fn test_pyramid_slice_regression() {
    let test_case = SliceTestCase {
        name: "pyramid beam lattice",
        model_path: "tools/slicer/samples/pyramid/pyramid.3mf",
        config_path: "pyramid_config.json",
        expected_slice_count: 11,
        reference_slices: vec![
            (0.0, "pyramid_z0.000mm.png"),
            (50.0, "pyramid_z50.000mm.png"),
            (100.0, "pyramid_z100.000mm.png"),
        ],
    };

    run_slice_test(&test_case);
}

#[test]
fn test_box_sliced_regression() {
    let test_case = SliceTestCase {
        name: "box with slice stack",
        model_path: "test_files/slices/box_sliced.3mf",
        config_path: "box_sliced_config.json",
        expected_slice_count: 400,
        reference_slices: vec![
            (20.0, "box_sliced_z20.000mm.png"),
            (30.0, "box_sliced_z30.000mm.png"),
            (40.0, "box_sliced_z40.000mm.png"),
        ],
    };

    run_slice_test(&test_case);
}

#[test]
fn test_cube_gears_slice_regression() {
    let test_case = SliceTestCase {
        name: "cube gears",
        model_path: "tools/slicer/samples/cube_gears/cube_gears.3mf",
        config_path: "cube_gears_config.json",
        expected_slice_count: 8,
        reference_slices: vec![
            (30.0, "cube_gears_z30.000mm.png"),
            (40.0, "cube_gears_z40.000mm.png"),
        ],
    };

    run_slice_test(&test_case);
}
