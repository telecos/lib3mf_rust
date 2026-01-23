//! Test to verify texture path validation accepts both /3D/Texture/ and /3D/Textures/

use lib3mf::{Extension, Model, ParserConfig};
use std::fs::File;
use std::path::Path;

#[test]
fn test_texture_path_validation_accepts_singular() {
    // This test verifies that texture paths in /3D/Texture/ (singular) are accepted
    // These are official 3MF consortium test files that should pass

    let config = ParserConfig::new().with_extension(Extension::Material);

    let test_file = "test_suites/suite6_core_matl/positive_test_cases/P_XXM_0508_13.3mf";

    if !Path::new(test_file).exists() {
        eprintln!("Warning: {} not found, skipping test", test_file);
        return;
    }

    let file = File::open(test_file).expect("Failed to open test file");
    let result = Model::from_reader_with_config(file, config);

    assert!(
        result.is_ok(),
        "File with /3D/Texture/ path should parse successfully: {:?}",
        result.err()
    );

    let model = result.unwrap();
    assert_eq!(
        model.resources.texture2d_resources.len(),
        1,
        "Should have 1 texture"
    );
    assert!(
        model.resources.texture2d_resources[0]
            .path
            .to_lowercase()
            .starts_with("/3d/texture/"),
        "Texture path should start with /3D/Texture/"
    );
}

#[test]
fn test_all_pxxm_0508_files() {
    // Test all 14 P_XXM_0508 files that use /3D/Texture/ paths

    let config = ParserConfig::new().with_extension(Extension::Material);

    for i in 1..=14 {
        let test_file = format!(
            "test_suites/suite6_core_matl/positive_test_cases/P_XXM_0508_{:02}.3mf",
            i
        );

        if !Path::new(&test_file).exists() {
            eprintln!("Warning: {} not found, skipping", test_file);
            continue;
        }

        let file =
            File::open(&test_file).unwrap_or_else(|_| panic!("Failed to open {}", test_file));
        let result = Model::from_reader_with_config(file, config.clone());

        assert!(
            result.is_ok(),
            "P_XXM_0508_{:02}.3mf should parse successfully: {:?}",
            i,
            result.err()
        );
    }
}
