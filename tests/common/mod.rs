//! Shared utilities for conformance tests
//!
//! This module provides common functionality used across multiple conformance test files,
//! particularly for configuring parser settings based on test suite requirements.

use lib3mf::{Extension, ParserConfig};

/// Get parser configuration for a specific test suite
///
/// Each suite tests specific extensions, so we configure the parser
/// to support only the extensions relevant to that suite. This ensures
/// proper validation of extension support.
///
/// # Arguments
///
/// * `suite_name` - The directory name of the test suite (e.g., "suite1_core_slice_prod")
///
/// # Returns
///
/// A `ParserConfig` configured with the appropriate extensions for the suite
pub fn get_suite_config(suite_name: &str) -> ParserConfig {
    match suite_name {
        // Suite 1: Core + Production + Slice
        "suite1_core_slice_prod" => ParserConfig::new()
            .with_extension(Extension::Production)
            .with_extension(Extension::Slice),

        // Suite 2: Core + Production + Materials
        "suite2_core_prod_matl" => ParserConfig::new()
            .with_extension(Extension::Production)
            .with_extension(Extension::Material),

        // Suite 3: Core only
        "suite3_core" => ParserConfig::new(),

        // Suite 4: Core + Slice
        "suite4_core_slice" => ParserConfig::new().with_extension(Extension::Slice),

        // Suite 5: Core + Production
        "suite5_core_prod" => ParserConfig::new().with_extension(Extension::Production),

        // Suite 6: Core + Materials
        "suite6_core_matl" => ParserConfig::new().with_extension(Extension::Material),

        // Suite 7: Beam Lattice
        // Also register the balls sub-extension namespace
        "suite7_beam" => ParserConfig::new()
            .with_extension(Extension::BeamLattice)
            .with_custom_extension(
                "http://schemas.microsoft.com/3dmanufacturing/beamlattice/balls/2020/07",
                "BeamLattice Balls",
            ),

        // Suite 8: Secure Content
        // Some test files also use Production extension
        // Also register the older 2019/04 SecureContent namespace
        "suite8_secure" => ParserConfig::new()
            .with_extension(Extension::SecureContent)
            .with_extension(Extension::Production)
            .with_custom_extension(
                "http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/04",
                "SecureContent 2019/04",
            ),

        // Suite 9: Core Extensions - support all for compatibility
        // Also register custom extensions like trianglesets
        "suite9_core_ext" => ParserConfig::with_all_extensions().with_custom_extension(
            "http://schemas.microsoft.com/3dmanufacturing/trianglesets/2021/07",
            "TriangleSets",
        ),

        // Suite 10: Boolean Operations
        // Some test files also use Production and BeamLattice extensions
        "suite10_boolean" => ParserConfig::new()
            .with_extension(Extension::BooleanOperations)
            .with_extension(Extension::Production)
            .with_extension(Extension::BeamLattice),

        // Suite 11: Displacement
        // Register both the built-in 2022/07 namespace and the newer 2023/10 namespace
        // Some test files also use BooleanOperations and Production extensions
        "suite11_Displacement" => ParserConfig::new()
            .with_extension(Extension::Displacement)
            .with_extension(Extension::BooleanOperations)
            .with_extension(Extension::Production)
            .with_custom_extension(
                "http://schemas.3mf.io/3dmanufacturing/displacement/2023/10",
                "Displacement 2023/10",
            ),

        // Default: support all extensions for unknown suites
        _ => ParserConfig::with_all_extensions(),
    }
}
