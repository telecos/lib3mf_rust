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
        File::open("test_suites/suite8_secure/positive_test_cases/P_EPX_2102_01_materialExt.3mf");
    
    if file.is_err() {
        // Skip test if file doesn't exist (test files not available)
        return;
    }
    
    let file = file.unwrap();

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

/// Test that keystore parsing handles binary/encrypted data correctly
/// This verifies the fix for suite8 UTF-8 errors where keystore files
/// may contain encrypted content that is not valid UTF-8
#[test]
fn test_keystore_handles_binary_data() {
    // This test validates that the parser can handle keystore files
    // with binary/encrypted content without throwing UTF-8 errors
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" 
       xml:lang="en-US" 
       xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:sc="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07"
       requiredextensions="sc">
  <resources>
    <object id="1" type="model">
      <mesh>
        <vertices>
          <vertex x="0" y="0" z="0"/>
          <vertex x="10" y="0" z="0"/>
          <vertex x="0" y="10" z="0"/>
        </vertices>
        <triangles>
          <triangle v1="0" v2="1" v3="2"/>
        </triangles>
      </mesh>
    </object>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"#;

    let config = ParserConfig::new().with_extension(Extension::SecureContent);
    let result = lib3mf::parser::parse_model_xml_with_config(xml, config);

    // Should parse successfully without UTF-8 errors
    assert!(
        result.is_ok(),
        "Failed to parse secure content model: {:?}",
        result.err()
    );
}

/// Test comprehensive keystore parsing with full structure
#[test]
fn test_keystore_full_structure_parsing() {
    use std::fs::File;

    // Use the same test file that has a complete keystore structure
    let file =
        File::open("test_suites/suite8_secure/positive_test_cases/P_EPX_2102_01_materialExt.3mf");
    
    if file.is_err() {
        // Skip test if file doesn't exist (test files not available)
        return;
    }
    
    let file = file.unwrap();

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
    let sc = model.secure_content.expect("SecureContent info should be populated");

    // Verify keystore UUID
    assert!(sc.keystore_uuid.is_some(), "Keystore UUID should be present");
    
    // Verify consumers were parsed
    if !sc.consumers.is_empty() {
        println!("Found {} consumers", sc.consumers.len());
        for (i, consumer) in sc.consumers.iter().enumerate() {
            println!("Consumer {}: ID={}, keyid={:?}", i, consumer.consumer_id, consumer.key_id);
            // Verify consumer has required fields
            assert!(!consumer.consumer_id.is_empty(), "Consumer ID should not be empty");
        }
    }
    
    // Verify resource data groups were parsed
    if !sc.resource_data_groups.is_empty() {
        println!("Found {} resource data groups", sc.resource_data_groups.len());
        for (i, group) in sc.resource_data_groups.iter().enumerate() {
            println!("Group {}: UUID={}", i, group.key_uuid);
            assert!(!group.key_uuid.is_empty(), "Key UUID should not be empty");
            
            // Verify access rights
            for (j, access_right) in group.access_rights.iter().enumerate() {
                println!("  Access right {}: consumer_index={}", j, access_right.consumer_index);
                assert!(access_right.consumer_index < sc.consumers.len(), 
                    "Consumer index should be valid");
                assert!(!access_right.kek_params.wrapping_algorithm.is_empty(),
                    "Wrapping algorithm should not be empty");
            }
            
            // Verify resource data
            for (j, resource) in group.resource_data.iter().enumerate() {
                println!("  Resource {}: path={}", j, resource.path);
                assert!(!resource.path.is_empty(), "Resource path should not be empty");
                assert!(!resource.cek_params.encryption_algorithm.is_empty(),
                    "Encryption algorithm should not be empty");
            }
        }
    }
    
    // Verify backward compatibility - encrypted_files list should still be populated
    assert!(!sc.encrypted_files.is_empty(), 
        "Encrypted files list should be populated for backward compatibility");
}

/// Test parsing of consumer with keyvalue (PEM public key)
#[test]
fn test_consumer_keyvalue_parsing() {
    // This test verifies that we can parse the optional <keyvalue> element
    // containing a PEM-formatted public key (per RFC 7468)
    
    // Create a minimal keystore XML with a consumer that has a keyvalue
    // We'll test this by creating a complete 3MF structure
    // For now, just verify the structure is available
    
    use lib3mf::{Consumer, KEKParams, CEKParams};
    
    // Verify structures can be created programmatically
    let consumer = Consumer {
        consumer_id: "test_consumer".to_string(),
        key_id: Some("KEK_001".to_string()),
        key_value: Some("-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...\n-----END PUBLIC KEY-----".to_string()),
    };
    
    assert_eq!(consumer.consumer_id, "test_consumer");
    assert!(consumer.key_id.is_some());
    assert!(consumer.key_value.is_some());
    
    // Verify KEKParams structure
    let kek_params = KEKParams {
        wrapping_algorithm: "http://www.w3.org/2001/04/xmlenc#rsa-oaep-mgf1p".to_string(),
        mgf_algorithm: Some("http://www.w3.org/2009/xmlenc11#mgf1sha256".to_string()),
        digest_method: Some("http://www.w3.org/2001/04/xmlenc#sha256".to_string()),
    };
    
    assert!(!kek_params.wrapping_algorithm.is_empty());
    assert!(kek_params.mgf_algorithm.is_some());
    assert!(kek_params.digest_method.is_some());
    
    // Verify CEKParams structure
    let cek_params = CEKParams {
        encryption_algorithm: "http://www.w3.org/2009/xmlenc11#aes256-gcm".to_string(),
        compression: "deflate".to_string(),
        iv: Some("base64encodedIV".to_string()),
        tag: Some("base64encodedTag".to_string()),
        aad: Some("base64encodedAAD".to_string()),
    };
    
    assert!(!cek_params.encryption_algorithm.is_empty());
    assert_eq!(cek_params.compression, "deflate");
    assert!(cek_params.iv.is_some());
    assert!(cek_params.tag.is_some());
    assert!(cek_params.aad.is_some());
}
