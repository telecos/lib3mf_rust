//! Tests for Secure Content Extension awareness
//!
//! These tests validate that the parser recognizes the Secure Content extension
//! and properly handles files that declare it in requiredextensions.
//!
//! **Note**: These tests do NOT implement cryptographic operations.
//! See SECURE_CONTENT_SUPPORT.md for security considerations.

use lib3mf::{Extension, Model, ParserConfig};

/// Test that the SecureContent extension is recognized in validation
#[test]
fn test_secure_content_extension_recognized() {
    // Verify the extension is properly defined
    assert_eq!(
        Extension::SecureContent.namespace(),
        "http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07"
    );
    assert_eq!(Extension::SecureContent.name(), "SecureContent");
}

/// Test that SecureContent can be roundtripped through namespace
#[test]
fn test_secure_content_namespace_roundtrip() {
    let namespace = Extension::SecureContent.namespace();
    let extension = Extension::from_namespace(namespace);
    assert_eq!(extension, Some(Extension::SecureContent));
}

/// Test that ParserConfig can be configured to support SecureContent
#[test]
fn test_parser_config_supports_secure_content() {
    let config = ParserConfig::new().with_extension(Extension::SecureContent);
    assert!(config.supports(&Extension::SecureContent));
}

/// Test that all extensions config includes SecureContent
#[test]
fn test_all_extensions_includes_secure_content() {
    let config = ParserConfig::with_all_extensions();
    assert!(config.supports(&Extension::SecureContent));
}

/// Test parsing a minimal 3MF with secure content extension declared
#[test]
fn test_parse_secure_content_declaration() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" 
       xml:lang="en-US" 
       xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:sc="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07"
       requiredextensions="sc">
    <metadata name="Application">lib3mf_rust</metadata>
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="100" y="0" z="0" />
                    <vertex x="100" y="100" z="0" />
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2" />
                </triangles>
            </mesh>
        </object>
    </resources>
    <build>
        <item objectid="1" />
    </build>
</model>"#;

    // Parse with secure content support using test-only export
    let config = ParserConfig::with_all_extensions();
    let model = lib3mf::parser::parse_model_xml_with_config(xml, config);

    assert!(model.is_ok(), "Failed to parse: {:?}", model.err());
    let model = model.unwrap();

    // Verify the extension was recognized
    assert!(
        model
            .required_extensions
            .contains(&Extension::SecureContent),
        "SecureContent extension not recognized in required_extensions"
    );
}

/// Test that parsing fails when SecureContent is required but not supported
#[test]
fn test_secure_content_validation_fails_when_unsupported() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" 
       xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:sc="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07"
       requiredextensions="sc">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="100" y="0" z="0" />
                    <vertex x="100" y="100" z="0" />
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2" />
                </triangles>
            </mesh>
        </object>
    </resources>
    <build>
        <item objectid="1" />
    </build>
</model>"#;

    // Parse without secure content support (core only)
    let config = ParserConfig::new();
    let result = lib3mf::parser::parse_model_xml_with_config(xml, config);

    // Should fail because SecureContent is required but not supported
    assert!(
        result.is_err(),
        "Should fail when SecureContent is required but not supported"
    );

    let err = result.unwrap_err();
    let err_msg = format!("{:?}", err);
    assert!(
        err_msg.contains("SecureContent") || err_msg.contains("UnsupportedExtension"),
        "Error should mention SecureContent or UnsupportedExtension, got: {}",
        err_msg
    );
}

/// Test that Model initializes with None for secure_content
#[test]
fn test_model_secure_content_default() {
    let model = Model::new();
    assert!(model.secure_content.is_none());
}

/// Test that parsing a file without secure content leaves field as None
#[test]
fn test_parse_without_secure_content() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" 
       xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="100" y="0" z="0" />
                    <vertex x="100" y="100" z="0" />
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2" />
                </triangles>
            </mesh>
        </object>
    </resources>
    <build>
        <item objectid="1" />
    </build>
</model>"#;

    let model = lib3mf::parser::parse_model_xml(xml).unwrap();
    assert!(model.secure_content.is_none());
    assert!(!model
        .required_extensions
        .contains(&Extension::SecureContent));
}

/// Test parsing keystore.xml from a 3MF package
#[test]
fn test_keystore_parsing() {
    use std::fs::File;

    // Use a positive test case that has keystore but doesn't fail validation
    // This file has encrypted texture but the model itself is valid
    let file =
        File::open("test_suites/suite8_secure/positive_test_cases/P_EPX_2102_01_materialExt.3mf")
            .unwrap();

    // This test file uses the older 2019/04 namespace and requires Production + Material extensions
    let config = ParserConfig::new()
        .with_extension(Extension::SecureContent)
        .with_extension(Extension::Production)
        .with_extension(Extension::Material)
        .with_custom_extension(
            "http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/04",
            "SecureContent 2019/04",
        );

    let model = lib3mf::parser::parse_3mf_with_config(file, config).unwrap();

    // Verify secure_content was populated
    assert!(
        model.secure_content.is_some(),
        "SecureContent info should be populated"
    );

    let sc = model.secure_content.unwrap();

    // Verify keystore UUID was extracted
    assert!(
        sc.keystore_uuid.is_some(),
        "Keystore UUID should be present"
    );
    assert_eq!(
        sc.keystore_uuid.unwrap(),
        "9a39333b-a20c-4932-9ddb-762dde47d06e"
    );

    // Verify encrypted files were extracted
    assert!(
        !sc.encrypted_files.is_empty(),
        "Should have at least one encrypted file"
    );
    assert!(sc
        .encrypted_files
        .contains(&"/3D/Texture/photo_1_encrypted.jpg".to_string()));
}
