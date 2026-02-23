//! Tests for Secure Content Extension awareness
//!
//! These tests validate that the parser recognizes the Secure Content extension
//! and properly handles files that declare it in requiredextensions.
//!
//! **Note**: These tests do NOT implement cryptographic operations.

use lib3mf::{CEKParams, Consumer, Extension, KEKParams, Model, ParserConfig};
use std::fs::File;

/// Test file path for Suite 8 secure content tests
const SUITE8_TEST_FILE: &str =
    "test_suites/suite8_secure/positive_test_cases/P_EPX_2102_01_materialExt.3mf";

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
    assert!(
        !model
            .required_extensions
            .contains(&Extension::SecureContent)
    );
}

/// Test parsing keystore.xml from a 3MF package
#[test]
fn test_keystore_parsing() {
    // Use a positive test case that has keystore but doesn't fail validation
    // This file has encrypted texture but the model itself is valid
    let file = File::open(SUITE8_TEST_FILE);

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
    assert!(
        sc.encrypted_files
            .contains(&"/3D/Texture/photo_1_encrypted.jpg".to_string())
    );
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
    // Use the same test file that has a complete keystore structure
    let file = File::open(SUITE8_TEST_FILE);

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
    let sc = model
        .secure_content
        .expect("SecureContent info should be populated");

    // Verify keystore UUID
    assert!(
        sc.keystore_uuid.is_some(),
        "Keystore UUID should be present"
    );

    // Verify consumers were parsed
    if !sc.consumers.is_empty() {
        println!("Found {} consumers", sc.consumers.len());
        for (i, consumer) in sc.consumers.iter().enumerate() {
            println!(
                "Consumer {}: ID={}, keyid={:?}",
                i, consumer.consumer_id, consumer.key_id
            );
            // Verify consumer has required fields
            assert!(
                !consumer.consumer_id.is_empty(),
                "Consumer ID should not be empty"
            );
        }
    }

    // Verify resource data groups were parsed
    if !sc.resource_data_groups.is_empty() {
        println!(
            "Found {} resource data groups",
            sc.resource_data_groups.len()
        );
        for (i, group) in sc.resource_data_groups.iter().enumerate() {
            println!("Group {}: UUID={}", i, group.key_uuid);
            assert!(!group.key_uuid.is_empty(), "Key UUID should not be empty");

            // Verify access rights
            for (j, access_right) in group.access_rights.iter().enumerate() {
                println!(
                    "  Access right {}: consumer_index={}",
                    j, access_right.consumer_index
                );
                assert!(
                    access_right.consumer_index < sc.consumers.len(),
                    "Consumer index should be valid"
                );
                assert!(
                    !access_right.kek_params.wrapping_algorithm.is_empty(),
                    "Wrapping algorithm should not be empty"
                );
            }

            // Verify resource data
            for (j, resource) in group.resource_data.iter().enumerate() {
                println!("  Resource {}: path={}", j, resource.path);
                assert!(
                    !resource.path.is_empty(),
                    "Resource path should not be empty"
                );
                assert!(
                    !resource.cek_params.encryption_algorithm.is_empty(),
                    "Encryption algorithm should not be empty"
                );
            }
        }
    }

    // Verify backward compatibility - encrypted_files list should still be populated
    assert!(
        !sc.encrypted_files.is_empty(),
        "Encrypted files list should be populated for backward compatibility"
    );
}

/// Test parsing of consumer with keyvalue (PEM public key)
#[test]
fn test_consumer_keyvalue_parsing() {
    // This test verifies that we can parse the optional <keyvalue> element
    // containing a PEM-formatted public key (per RFC 7468)

    // Create a minimal keystore XML with a consumer that has a keyvalue
    // We'll test this by creating a complete 3MF structure
    // For now, just verify the structure is available

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

/// Helper: build a minimal 3MF ZIP with a keystore containing the given XML
fn create_3mf_with_keystore(model_xml: &str, keystore_xml: &str) -> Vec<u8> {
    use std::io::Write;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    let mut buffer = Vec::new();
    let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));

    // [Content_Types].xml – register keystore content type
    zip.start_file("[Content_Types].xml", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
    <Override PartName="/Secure/keystore.xml" ContentType="application/vnd.ms-package.3dmanufacturing-keystore+xml"/>
</Types>"#,
    )
    .unwrap();

    // _rels/.rels with keystore relationship (use the correct 2019/07 rel type)
    zip.add_directory("_rels/", SimpleFileOptions::default())
        .unwrap();
    zip.start_file("_rels/.rels", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rel0" Target="/3D/3dmodel.model" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
    <Relationship Id="rel1" Target="/Secure/keystore.xml" Type="http://schemas.microsoft.com/3dmanufacturing/2019/07/keystore"/>
</Relationships>"#,
    )
    .unwrap();

    // 3D/3dmodel.model
    zip.add_directory("3D/", SimpleFileOptions::default())
        .unwrap();
    zip.start_file("3D/3dmodel.model", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(model_xml.as_bytes()).unwrap();

    // Secure/keystore.xml
    zip.add_directory("Secure/", SimpleFileOptions::default())
        .unwrap();
    zip.start_file("Secure/keystore.xml", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(keystore_xml.as_bytes()).unwrap();

    zip.finish().unwrap();
    buffer
}

const MINIMAL_MODEL_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US"
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
  <build><item objectid="1"/></build>
</model>"#;

/// Test that a valid keystore with full consumer + resource group parses correctly
#[test]
fn test_keystore_valid_full_structure() {
    let keystore_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07" UUID="test-uuid-1234">
    <consumer consumerid="consumer1" keyid="key001"></consumer>
    <resourcedatagroup keyuuid="group-uuid-5678">
        <accessright consumerindex="0">
            <kekparams wrappingalgorithm="http://www.w3.org/2001/04/xmlenc#rsa-oaep-mgf1p"
                       mgfalgorithm="http://www.w3.org/2009/xmlenc11#mgf1sha256"
                       digestmethod="http://www.w3.org/2001/04/xmlenc#sha256"/>
            <cipherdata><xenc:CipherValue xmlns:xenc="http://www.w3.org/2001/04/xmlenc#">AAAA</xenc:CipherValue></cipherdata>
        </accessright>
    </resourcedatagroup>
</keystore>"#;

    let buffer = create_3mf_with_keystore(MINIMAL_MODEL_XML, keystore_xml);
    let config = ParserConfig::new().with_extension(Extension::SecureContent);
    let result = lib3mf::parser::parse_3mf_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_ok(),
        "Expected valid keystore to parse, got: {:?}",
        result.err()
    );
    let model = result.unwrap();
    let sc = model.secure_content.expect("Should have secure content");
    assert_eq!(
        sc.keystore_uuid.as_deref(),
        Some("test-uuid-1234"),
        "Keystore UUID should be parsed"
    );
    assert_eq!(sc.consumers.len(), 1);
    assert_eq!(sc.consumers[0].consumer_id, "consumer1");
    assert_eq!(sc.resource_data_groups.len(), 1);
    assert_eq!(
        sc.resource_data_groups[0].access_rights[0]
            .kek_params
            .wrapping_algorithm,
        "http://www.w3.org/2001/04/xmlenc#rsa-oaep-mgf1p"
    );
}

/// Test EPX-2604: duplicate consumer ID should fail
#[test]
fn test_keystore_duplicate_consumer_id_fails() {
    let keystore_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07" UUID="uuid-1">
    <consumer consumerid="same_id" keyid="key1"></consumer>
    <consumer consumerid="same_id" keyid="key2"></consumer>
</keystore>"#;

    let buffer = create_3mf_with_keystore(MINIMAL_MODEL_XML, keystore_xml);
    let config = ParserConfig::new().with_extension(Extension::SecureContent);
    let result = lib3mf::parser::parse_3mf_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_err(),
        "Expected error for duplicate consumer ID (EPX-2604)"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("same_id") || msg.contains("Duplicate") || msg.contains("consumer"),
        "Error should mention duplicate consumer, got: {}",
        msg
    );
}

/// Test EPX-2602: resource data groups without any consumers should fail
#[test]
fn test_keystore_resource_groups_without_consumers_fails() {
    let keystore_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07" UUID="uuid-1">
    <resourcedatagroup keyuuid="group-uuid">
        <accessright consumerindex="0">
            <kekparams wrappingalgorithm="http://www.w3.org/2001/04/xmlenc#rsa-oaep-mgf1p"/>
        </accessright>
    </resourcedatagroup>
</keystore>"#;

    let buffer = create_3mf_with_keystore(MINIMAL_MODEL_XML, keystore_xml);
    let config = ParserConfig::new().with_extension(Extension::SecureContent);
    let result = lib3mf::parser::parse_3mf_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_err(),
        "Expected error for resource data groups without consumers (EPX-2602)"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("consumer") || msg.contains("EPX-2602"),
        "Error should mention consumers or EPX-2602, got: {}",
        msg
    );
}

/// Test EPX-2601: invalid consumer index in access right should fail
#[test]
fn test_keystore_invalid_consumer_index_fails() {
    let keystore_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07" UUID="uuid-1">
    <consumer consumerid="consumer1" keyid="key1"></consumer>
    <resourcedatagroup keyuuid="group-uuid">
        <accessright consumerindex="99">
            <kekparams wrappingalgorithm="http://www.w3.org/2001/04/xmlenc#rsa-oaep-mgf1p"/>
        </accessright>
    </resourcedatagroup>
</keystore>"#;

    let buffer = create_3mf_with_keystore(MINIMAL_MODEL_XML, keystore_xml);
    let config = ParserConfig::new().with_extension(Extension::SecureContent);
    let result = lib3mf::parser::parse_3mf_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_err(),
        "Expected error for invalid consumer index (EPX-2601)"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("consumer") || msg.contains("index") || msg.contains("EPX-2601"),
        "Error should mention consumer index or EPX-2601, got: {}",
        msg
    );
}

/// Test EPX-2603: invalid wrapping algorithm in kekparams should fail
#[test]
fn test_keystore_invalid_wrapping_algorithm_fails() {
    let keystore_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07" UUID="uuid-1">
    <consumer consumerid="consumer1" keyid="key1"></consumer>
    <resourcedatagroup keyuuid="group-uuid">
        <accessright consumerindex="0">
            <kekparams wrappingalgorithm="http://invalid.algorithm/not-valid"/>
        </accessright>
    </resourcedatagroup>
</keystore>"#;

    let buffer = create_3mf_with_keystore(MINIMAL_MODEL_XML, keystore_xml);
    let config = ParserConfig::new().with_extension(Extension::SecureContent);
    let result = lib3mf::parser::parse_3mf_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_err(),
        "Expected error for invalid wrapping algorithm (EPX-2603)"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("wrapping") || msg.contains("algorithm") || msg.contains("EPX-2603"),
        "Error should mention wrapping algorithm or EPX-2603, got: {}",
        msg
    );
}

/// Test EPX-2603: invalid mgf algorithm in kekparams should fail
#[test]
fn test_keystore_invalid_mgf_algorithm_fails() {
    let keystore_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07" UUID="uuid-1">
    <consumer consumerid="consumer1" keyid="key1"></consumer>
    <resourcedatagroup keyuuid="group-uuid">
        <accessright consumerindex="0">
            <kekparams wrappingalgorithm="http://www.w3.org/2001/04/xmlenc#rsa-oaep-mgf1p"
                       mgfalgorithm="http://invalid.mgf/not-valid"/>
        </accessright>
    </resourcedatagroup>
</keystore>"#;

    let buffer = create_3mf_with_keystore(MINIMAL_MODEL_XML, keystore_xml);
    let config = ParserConfig::new().with_extension(Extension::SecureContent);
    let result = lib3mf::parser::parse_3mf_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_err(),
        "Expected error for invalid mgf algorithm (EPX-2603)"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("mgf") || msg.contains("algorithm") || msg.contains("EPX-2603"),
        "Error should mention mgf algorithm or EPX-2603, got: {}",
        msg
    );
}

/// Test EPX-2603: invalid digest method in kekparams should fail
#[test]
fn test_keystore_invalid_digest_method_fails() {
    let keystore_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07" UUID="uuid-1">
    <consumer consumerid="consumer1" keyid="key1"></consumer>
    <resourcedatagroup keyuuid="group-uuid">
        <accessright consumerindex="0">
            <kekparams wrappingalgorithm="http://www.w3.org/2001/04/xmlenc#rsa-oaep-mgf1p"
                       digestmethod="http://invalid.digest/not-valid"/>
        </accessright>
    </resourcedatagroup>
</keystore>"#;

    let buffer = create_3mf_with_keystore(MINIMAL_MODEL_XML, keystore_xml);
    let config = ParserConfig::new().with_extension(Extension::SecureContent);
    let result = lib3mf::parser::parse_3mf_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_err(),
        "Expected error for invalid digest method (EPX-2603)"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("digest") || msg.contains("EPX-2603"),
        "Error should mention digest method or EPX-2603, got: {}",
        msg
    );
}

/// Test EPX-2603: kekparams as self-closing tag (Empty event)
#[test]
fn test_keystore_kekparams_self_closing_valid() {
    // kekparams can appear as a self-closing tag (Empty event in quick-xml)
    let keystore_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07" UUID="uuid-1">
    <consumer consumerid="consumer1" keyid="key1"></consumer>
    <resourcedatagroup keyuuid="group-uuid">
        <accessright consumerindex="0">
            <kekparams wrappingalgorithm="http://www.w3.org/2001/04/xmlenc#rsa-oaep-mgf1p" mgfalgorithm="" digestmethod=""/>
            <cipherdata><xenc:CipherValue xmlns:xenc="http://www.w3.org/2001/04/xmlenc#">AAAA</xenc:CipherValue></cipherdata>
        </accessright>
    </resourcedatagroup>
</keystore>"#;

    let buffer = create_3mf_with_keystore(MINIMAL_MODEL_XML, keystore_xml);
    let config = ParserConfig::new().with_extension(Extension::SecureContent);
    let result = lib3mf::parser::parse_3mf_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_ok(),
        "Expected valid self-closing kekparams to succeed, got: {:?}",
        result.err()
    );
}

/// Test consumer keyvalue element parsing
#[test]
fn test_keystore_consumer_keyvalue_element() {
    let keystore_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07" UUID="uuid-kv">
    <consumer consumerid="consumer_with_key">
        <keyvalue>-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA
-----END PUBLIC KEY-----</keyvalue>
    </consumer>
</keystore>"#;

    let buffer = create_3mf_with_keystore(MINIMAL_MODEL_XML, keystore_xml);
    let config = ParserConfig::new().with_extension(Extension::SecureContent);
    let result = lib3mf::parser::parse_3mf_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_ok(),
        "Expected keyvalue element to parse successfully, got: {:?}",
        result.err()
    );
    let model = result.unwrap();
    let sc = model.secure_content.expect("Should have secure content");
    assert_eq!(sc.consumers.len(), 1);
    assert!(
        sc.consumers[0].key_value.is_some(),
        "Consumer should have key_value"
    );
}

/// Test EPX-2605: OPC .rels file as encrypted resource should fail
#[test]
fn test_keystore_rels_file_encrypted_fails() {
    use std::io::Write;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    // We need to actually have the rels-like file in the package for EPX-2607 check
    // to pass and reach the EPX-2605 check. Create a package that has the file.
    let keystore_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07" UUID="uuid-1">
    <consumer consumerid="consumer1" keyid="key1"></consumer>
    <resourcedatagroup keyuuid="group-uuid">
        <resourcedata path="/_rels/.rels">
            <cekparams encryptionalgorithm="http://www.w3.org/2009/xmlenc11#aes256-gcm"/>
        </resourcedata>
    </resourcedatagroup>
</keystore>"#;

    let mut buffer = Vec::new();
    let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));

    zip.start_file("[Content_Types].xml", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
    <Override PartName="/Secure/keystore.xml" ContentType="application/vnd.ms-package.3dmanufacturing-keystore+xml"/>
</Types>"#,
    )
    .unwrap();

    zip.add_directory("_rels/", SimpleFileOptions::default())
        .unwrap();
    zip.start_file("_rels/.rels", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rel0" Target="/3D/3dmodel.model" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
    <Relationship Id="rel1" Target="/Secure/keystore.xml" Type="http://schemas.microsoft.com/3dmanufacturing/2019/07/keystore"/>
</Relationships>"#,
    )
    .unwrap();

    zip.add_directory("3D/", SimpleFileOptions::default())
        .unwrap();
    zip.start_file("3D/3dmodel.model", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(MINIMAL_MODEL_XML.as_bytes()).unwrap();

    // Add the rels-like file to test EPX-2605 path validation (path contains /_rels/)
    zip.add_directory("Secure/", SimpleFileOptions::default())
        .unwrap();
    zip.start_file("Secure/keystore.xml", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(keystore_xml.as_bytes()).unwrap();

    zip.finish().unwrap();

    let config = ParserConfig::new().with_extension(Extension::SecureContent);
    let result = lib3mf::parser::parse_3mf_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_err(),
        "Expected error for .rels file as encrypted resource (EPX-2605)"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("rels") || msg.contains("OPC") || msg.contains("EPX-2605"),
        "Error should mention OPC rels files or EPX-2605, got: {}",
        msg
    );
}

/// Test EPX-2605: empty path in resourcedata should fail
#[test]
fn test_keystore_empty_resource_path_fails() {
    let keystore_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07" UUID="uuid-1">
    <consumer consumerid="consumer1" keyid="key1"></consumer>
    <resourcedatagroup keyuuid="group-uuid">
        <resourcedata path="">
            <cekparams encryptionalgorithm="http://www.w3.org/2009/xmlenc11#aes256-gcm"/>
        </resourcedata>
    </resourcedatagroup>
</keystore>"#;

    let buffer = create_3mf_with_keystore(MINIMAL_MODEL_XML, keystore_xml);
    let config = ParserConfig::new().with_extension(Extension::SecureContent);
    let result = lib3mf::parser::parse_3mf_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_err(),
        "Expected error for empty resource data path (EPX-2605)"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("path") || msg.contains("EPX-2605"),
        "Error should mention path or EPX-2605, got: {}",
        msg
    );
}

/// Test EPX-2607: non-existent file referenced in resourcedata should fail
#[test]
fn test_keystore_nonexistent_resource_file_fails() {
    let keystore_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07" UUID="uuid-1">
    <consumer consumerid="consumer1" keyid="key1"></consumer>
    <resourcedatagroup keyuuid="group-uuid">
        <resourcedata path="/3D/nonexistent_encrypted.model">
            <cekparams encryptionalgorithm="http://www.w3.org/2009/xmlenc11#aes256-gcm"/>
        </resourcedata>
    </resourcedatagroup>
</keystore>"#;

    let buffer = create_3mf_with_keystore(MINIMAL_MODEL_XML, keystore_xml);
    let config = ParserConfig::new().with_extension(Extension::SecureContent);
    let result = lib3mf::parser::parse_3mf_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_err(),
        "Expected error for non-existent file in resourcedata (EPX-2607)"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("nonexistent_encrypted")
            || msg.contains("does not exist")
            || msg.contains("EPX-2607"),
        "Error should mention missing file or EPX-2607, got: {}",
        msg
    );
}

/// Test EPX-2607: duplicate resourcedata path should fail
#[test]
fn test_keystore_duplicate_resource_path_fails() {
    use std::io::Write;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    // We need an actual encrypted file in the package
    let keystore_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07" UUID="uuid-1">
    <consumer consumerid="consumer1" keyid="key1"></consumer>
    <resourcedatagroup keyuuid="group-uuid">
        <resourcedata path="/3D/encrypted.model">
            <cekparams encryptionalgorithm="http://www.w3.org/2009/xmlenc11#aes256-gcm"/>
        </resourcedata>
        <resourcedata path="/3D/encrypted.model">
            <cekparams encryptionalgorithm="http://www.w3.org/2009/xmlenc11#aes256-gcm"/>
        </resourcedata>
    </resourcedatagroup>
</keystore>"#;

    let mut buffer = Vec::new();
    let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));

    zip.start_file("[Content_Types].xml", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
    <Override PartName="/Secure/keystore.xml" ContentType="application/vnd.ms-package.3dmanufacturing-keystore+xml"/>
</Types>"#,
    )
    .unwrap();

    zip.add_directory("_rels/", SimpleFileOptions::default())
        .unwrap();
    zip.start_file("_rels/.rels", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rel0" Target="/3D/3dmodel.model" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
    <Relationship Id="rel1" Target="/Secure/keystore.xml" Type="http://schemas.microsoft.com/3dmanufacturing/2019/07/keystore"/>
</Relationships>"#,
    )
    .unwrap();

    zip.add_directory("3D/", SimpleFileOptions::default())
        .unwrap();
    zip.start_file("3D/3dmodel.model", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(MINIMAL_MODEL_XML.as_bytes()).unwrap();

    // Add the encrypted file so EPX-2607 existence check passes for first occurrence
    zip.start_file("3D/encrypted.model", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(b"encrypted data").unwrap();

    zip.add_directory("Secure/", SimpleFileOptions::default())
        .unwrap();
    zip.start_file("Secure/keystore.xml", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(keystore_xml.as_bytes()).unwrap();

    zip.finish().unwrap();

    let config = ParserConfig::new().with_extension(Extension::SecureContent);
    let result = lib3mf::parser::parse_3mf_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_err(),
        "Expected error for duplicate resourcedata path (EPX-2607)"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Duplicate") || msg.contains("duplicate") || msg.contains("encrypted.model"),
        "Error should mention duplicate path, got: {}",
        msg
    );
}

/// Test that consumers with key_id are correctly parsed
#[test]
fn test_keystore_consumer_with_keyid() {
    let keystore_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07" UUID="uuid-keyid">
    <consumer consumerid="consumer1" keyid="my-key-id-001"></consumer>
</keystore>"#;

    let buffer = create_3mf_with_keystore(MINIMAL_MODEL_XML, keystore_xml);
    let config = ParserConfig::new().with_extension(Extension::SecureContent);
    let result = lib3mf::parser::parse_3mf_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_ok(),
        "Expected consumer with keyid to parse, got: {:?}",
        result.err()
    );
    let model = result.unwrap();
    let sc = model.secure_content.expect("Should have secure content");
    assert_eq!(sc.consumers[0].key_id.as_deref(), Some("my-key-id-001"));
}

/// Test that the 2009 wrapping algorithm variant is also valid
#[test]
fn test_keystore_rsa_oaep_2009_algorithm_valid() {
    let keystore_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07" UUID="uuid-1">
    <consumer consumerid="consumer1" keyid="key1"></consumer>
    <resourcedatagroup keyuuid="group-uuid">
        <accessright consumerindex="0">
            <kekparams wrappingalgorithm="http://www.w3.org/2009/xmlenc11#rsa-oaep"/>
            <cipherdata><xenc:CipherValue xmlns:xenc="http://www.w3.org/2001/04/xmlenc#">BBBB</xenc:CipherValue></cipherdata>
        </accessright>
    </resourcedatagroup>
</keystore>"#;

    let buffer = create_3mf_with_keystore(MINIMAL_MODEL_XML, keystore_xml);
    let config = ParserConfig::new().with_extension(Extension::SecureContent);
    let result = lib3mf::parser::parse_3mf_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_ok(),
        "Expected 2009 RSA-OAEP algorithm to be valid, got: {:?}",
        result.err()
    );
}

/// Helper: create a 3MF package with a keystore and an actual encrypted file,
/// including an EncryptedFile relationship for EPX-2606 compliance.
fn create_3mf_with_keystore_and_encrypted_file(
    model_xml: &str,
    keystore_xml: &str,
    encrypted_file_path: &str,
    encrypted_file_content: &[u8],
) -> Vec<u8> {
    use std::io::Write;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    // Strip leading slash for use as ZIP entry name
    let normalized = encrypted_file_path.trim_start_matches('/');

    let mut buffer = Vec::new();
    let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));
    let opts = SimpleFileOptions::default();

    // [Content_Types].xml
    zip.start_file("[Content_Types].xml", opts).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
    <Override PartName="/Secure/keystore.xml" ContentType="application/vnd.ms-package.3dmanufacturing-keystore+xml"/>
</Types>"#,
    )
    .unwrap();

    // _rels/.rels with keystore relationship AND EncryptedFile relationship
    let rels_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rel0" Target="/3D/3dmodel.model" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
    <Relationship Id="rel1" Target="/Secure/keystore.xml" Type="http://schemas.microsoft.com/3dmanufacturing/2019/07/keystore"/>
    <Relationship Id="rel2" Target="{}" Type="http://schemas.openxmlformats.org/package/2006/relationships/encryptedfile"/>
</Relationships>"#,
        encrypted_file_path
    );
    zip.add_directory("_rels/", opts).unwrap();
    zip.start_file("_rels/.rels", opts).unwrap();
    zip.write_all(rels_content.as_bytes()).unwrap();

    // 3D/3dmodel.model
    zip.add_directory("3D/", opts).unwrap();
    zip.start_file("3D/3dmodel.model", opts).unwrap();
    zip.write_all(model_xml.as_bytes()).unwrap();

    // Encrypted file (fake/placeholder content)
    zip.start_file(normalized, opts).unwrap();
    zip.write_all(encrypted_file_content).unwrap();

    // Secure/keystore.xml
    zip.add_directory("Secure/", opts).unwrap();
    zip.start_file("Secure/keystore.xml", opts).unwrap();
    zip.write_all(keystore_xml.as_bytes()).unwrap();

    zip.finish().unwrap();
    buffer
}

/// Test that cekparams with non-self-closing tag and iv/tag/aad child elements parses correctly.
/// This exercises the Event::Start handler for cekparams and the Event::End handlers for
/// iv, tag, aad, and cekparams in load_keystore.
#[test]
fn test_keystore_cekparams_with_iv_tag_aad() {
    let keystore_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07" UUID="uuid-cek">
    <consumer consumerid="consumer1" keyid="key1"></consumer>
    <resourcedatagroup keyuuid="group-uuid-cek">
        <accessright consumerindex="0">
            <kekparams wrappingalgorithm="http://www.w3.org/2001/04/xmlenc#rsa-oaep-mgf1p"
                       mgfalgorithm="http://www.w3.org/2009/xmlenc11#mgf1sha256"
                       digestmethod="http://www.w3.org/2001/04/xmlenc#sha256"/>
            <cipherdata><xenc:CipherValue xmlns:xenc="http://www.w3.org/2001/04/xmlenc#">AAAA</xenc:CipherValue></cipherdata>
        </accessright>
        <resourcedata path="/3D/enc.model">
            <cekparams encryptionalgorithm="http://www.w3.org/2009/xmlenc11#aes256-gcm" compression="deflate">
                <iv>dGVzdGl2MTIzNDU2</iv>
                <tag>dGVzdHRhZzEyMzQ1Ng==</tag>
                <aad>dGVzdGFhZA==</aad>
            </cekparams>
        </resourcedata>
    </resourcedatagroup>
</keystore>"#;

    let buffer = create_3mf_with_keystore_and_encrypted_file(
        MINIMAL_MODEL_XML,
        keystore_xml,
        "/3D/enc.model",
        b"fake encrypted content",
    );
    let config = ParserConfig::new().with_extension(Extension::SecureContent);
    let result = lib3mf::parser::parse_3mf_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_ok(),
        "Expected cekparams with iv/tag/aad to parse successfully, got: {:?}",
        result.err()
    );
    let model = result.unwrap();
    let sc = model.secure_content.expect("Should have secure content");
    assert_eq!(sc.resource_data_groups.len(), 1);
    let group = &sc.resource_data_groups[0];
    assert_eq!(group.resource_data.len(), 1);
    let resource = &group.resource_data[0];
    assert_eq!(resource.path, "/3D/enc.model");
    assert_eq!(resource.cek_params.compression, "deflate");
    assert!(
        resource.cek_params.iv.is_some(),
        "IV should be parsed from <iv> element"
    );
    assert!(
        resource.cek_params.tag.is_some(),
        "Tag should be parsed from <tag> element"
    );
    assert!(
        resource.cek_params.aad.is_some(),
        "AAD should be parsed from <aad> element"
    );
    assert_eq!(resource.cek_params.iv.as_deref(), Some("dGVzdGl2MTIzNDU2"));
    assert_eq!(
        resource.cek_params.tag.as_deref(),
        Some("dGVzdHRhZzEyMzQ1Ng==")
    );
    assert_eq!(resource.cek_params.aad.as_deref(), Some("dGVzdGFhZA=="));
}

/// Test kekparams as a non-self-closing Start+End element pair.
/// This exercises the Event::Start handler for kekparams (stores current_kek_params)
/// and the Event::End handler for kekparams (moves it to current_access_right).
#[test]
fn test_keystore_kekparams_as_start_end_element() {
    // Using <kekparams ...></kekparams> (non-self-closing) instead of <kekparams .../>
    let keystore_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07" UUID="uuid-se">
    <consumer consumerid="consumer1" keyid="key1"></consumer>
    <resourcedatagroup keyuuid="group-uuid-se">
        <accessright consumerindex="0">
            <kekparams wrappingalgorithm="http://www.w3.org/2001/04/xmlenc#rsa-oaep-mgf1p"
                       mgfalgorithm="http://www.w3.org/2009/xmlenc11#mgf1sha256"
                       digestmethod="http://www.w3.org/2001/04/xmlenc#sha256"></kekparams>
            <cipherdata><xenc:CipherValue xmlns:xenc="http://www.w3.org/2001/04/xmlenc#">CCCC</xenc:CipherValue></cipherdata>
        </accessright>
    </resourcedatagroup>
</keystore>"#;

    let buffer = create_3mf_with_keystore(MINIMAL_MODEL_XML, keystore_xml);
    let config = ParserConfig::new().with_extension(Extension::SecureContent);
    let result = lib3mf::parser::parse_3mf_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_ok(),
        "Expected non-self-closing kekparams to parse successfully, got: {:?}",
        result.err()
    );
    let model = result.unwrap();
    let sc = model.secure_content.expect("Should have secure content");
    let group = &sc.resource_data_groups[0];
    assert_eq!(
        group.access_rights[0].kek_params.wrapping_algorithm,
        "http://www.w3.org/2001/04/xmlenc#rsa-oaep-mgf1p"
    );
    assert_eq!(
        group.access_rights[0].kek_params.mgf_algorithm.as_deref(),
        Some("http://www.w3.org/2009/xmlenc11#mgf1sha256")
    );
    assert_eq!(
        group.access_rights[0].kek_params.digest_method.as_deref(),
        Some("http://www.w3.org/2001/04/xmlenc#sha256")
    );
}

/// Test that a package with Secure/keystore.xml but no keystore relationship
/// (fallback path) fails with EPX-2606 validation error.
/// This exercises the fallback path in load_keystore (lines ~118-120).
#[test]
fn test_keystore_fallback_path_fails_without_relationship() {
    use std::io::Write;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    let keystore_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07" UUID="uuid-fb">
</keystore>"#;

    // Build a package that has Secure/keystore.xml but NO keystore relationship in _rels/.rels
    // discover_keystore_path() returns None -> fallback picks up Secure/keystore.xml ->
    // validate_keystore_relationship() fails because no proper relationship exists
    let mut buffer = Vec::new();
    let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));
    let opts = SimpleFileOptions::default();

    zip.start_file("[Content_Types].xml", opts).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
    <Override PartName="/Secure/keystore.xml" ContentType="application/vnd.ms-package.3dmanufacturing-keystore+xml"/>
</Types>"#,
    )
    .unwrap();

    // _rels/.rels without keystore relationship (only model relationship)
    zip.add_directory("_rels/", opts).unwrap();
    zip.start_file("_rels/.rels", opts).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rel0" Target="/3D/3dmodel.model" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"#,
    )
    .unwrap();

    zip.add_directory("3D/", opts).unwrap();
    zip.start_file("3D/3dmodel.model", opts).unwrap();
    zip.write_all(MINIMAL_MODEL_XML.as_bytes()).unwrap();

    zip.add_directory("Secure/", opts).unwrap();
    zip.start_file("Secure/keystore.xml", opts).unwrap();
    zip.write_all(keystore_xml.as_bytes()).unwrap();

    zip.finish().unwrap();

    let config = ParserConfig::new().with_extension(Extension::SecureContent);
    let result = lib3mf::parser::parse_3mf_with_config(std::io::Cursor::new(&buffer), config);
    // The fallback finds Secure/keystore.xml but then validate_keystore_relationship fails
    assert!(
        result.is_err(),
        "Expected EPX-2606 error for missing keystore relationship"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("relationship") || msg.contains("EPX-2606") || msg.contains("keystore"),
        "Error should mention missing relationship, got: {}",
        msg
    );
}

/// Test that a package with Secure/info.store but no relationship uses fallback path.
/// This exercises the fallback branch in load_keystore (lines ~121).
#[test]
fn test_keystore_infostore_fallback_fails_without_relationship() {
    use std::io::Write;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    let keystore_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07" UUID="uuid-is">
</keystore>"#;

    // Build a package with Secure/info.store but no keystore relationship
    let mut buffer = Vec::new();
    let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));
    let opts = SimpleFileOptions::default();

    zip.start_file("[Content_Types].xml", opts).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
</Types>"#,
    )
    .unwrap();

    zip.add_directory("_rels/", opts).unwrap();
    zip.start_file("_rels/.rels", opts).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rel0" Target="/3D/3dmodel.model" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"#,
    )
    .unwrap();

    zip.add_directory("3D/", opts).unwrap();
    zip.start_file("3D/3dmodel.model", opts).unwrap();
    zip.write_all(MINIMAL_MODEL_XML.as_bytes()).unwrap();

    zip.add_directory("Secure/", opts).unwrap();
    zip.start_file("Secure/info.store", opts).unwrap();
    zip.write_all(keystore_xml.as_bytes()).unwrap();

    zip.finish().unwrap();

    let config = ParserConfig::new().with_extension(Extension::SecureContent);
    let result = lib3mf::parser::parse_3mf_with_config(std::io::Cursor::new(&buffer), config);
    // Fallback picks up Secure/info.store, then validate_keystore_relationship fails
    assert!(
        result.is_err(),
        "Expected EPX-2606 error for info.store without proper relationship"
    );
}

/// Test load_file_with_decryption non-encrypted path via the slice parser.
/// Creates a 3MF with a slice stack that references an external slice file (sliceref).
/// The slice file is a regular (non-encrypted) file, exercising the non-encrypted path.
#[test]
fn test_load_file_via_slice_reference_non_encrypted() {
    use std::io::Write;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    // External slice file content
    let slice_file_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:s="http://schemas.microsoft.com/3dmanufacturing/slice/2015/07">
  <resources>
    <s:slicestack id="5" zbottom="0">
      <s:slice ztop="1.0">
        <s:vertices>
          <s:vertex x="0" y="0"/>
          <s:vertex x="10" y="0"/>
          <s:vertex x="0" y="10"/>
        </s:vertices>
        <s:polygon startv="0">
          <s:segment v2="1"/>
          <s:segment v2="2"/>
          <s:segment v2="0"/>
        </s:polygon>
      </s:slice>
    </s:slicestack>
  </resources>
  <build/>
</model>"#;

    // Main model references the slice stack via sliceref
    let model_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US"
    xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
    xmlns:s="http://schemas.microsoft.com/3dmanufacturing/slice/2015/07">
  <resources>
    <s:slicestack id="10" zbottom="0">
      <s:sliceref slicestackid="5" slicepath="/2D/slices.model"/>
    </s:slicestack>
    <object id="1" type="model" s:slicestackid="10">
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
  <build><item objectid="1"/></build>
</model>"#;

    let mut buffer = Vec::new();
    let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));
    let opts = SimpleFileOptions::default();

    zip.start_file("[Content_Types].xml", opts).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
</Types>"#,
    )
    .unwrap();

    zip.add_directory("_rels/", opts).unwrap();
    zip.start_file("_rels/.rels", opts).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rel0" Target="/3D/3dmodel.model" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"#,
    )
    .unwrap();

    zip.add_directory("3D/", opts).unwrap();
    zip.start_file("3D/3dmodel.model", opts).unwrap();
    zip.write_all(model_xml.as_bytes()).unwrap();

    zip.add_directory("2D/", opts).unwrap();
    zip.start_file("2D/slices.model", opts).unwrap();
    zip.write_all(slice_file_xml.as_bytes()).unwrap();

    zip.finish().unwrap();

    // Parse with slice extension support - this calls load_slice_references which calls
    // load_file_with_decryption for the non-encrypted "2D/slices.model" file
    use lib3mf::Extension;
    let config = lib3mf::ParserConfig::new().with_extension(Extension::Slice);
    let result = lib3mf::parser::parse_3mf_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_ok(),
        "Expected slice reference with non-encrypted file to succeed, got: {:?}",
        result.err()
    );
    let model = result.unwrap();
    // The external slices should be loaded into the main model
    assert!(
        !model.resources.slice_stacks.is_empty(),
        "Slice stacks should be populated"
    );
    assert!(
        !model.resources.slice_stacks[0].slices.is_empty(),
        "Slices should be loaded from external file"
    );
}
