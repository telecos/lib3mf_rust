//! Example demonstrating the SecureContentExtensionHandler
//!
//! This example shows how to use the SecureContentExtensionHandler with
//! the ExtensionRegistry to validate secure content in 3MF models.

use lib3mf::{
    extensions::SecureContentExtensionHandler, AccessRight, CEKParams, Consumer,
    ExtensionRegistry, KEKParams, Model, ResourceData, ResourceDataGroup, Result,
    SecureContentInfo,
};

fn main() -> Result<()> {
    println!("=== SecureContentExtensionHandler Example ===\n");

    // Create an extension registry and register the handler
    let mut registry = ExtensionRegistry::new();
    registry.register(Box::new(SecureContentExtensionHandler));
    println!("✓ Registered SecureContentExtensionHandler\n");

    // Example 1: Model without secure content
    println!("Example 1: Model without secure content");
    let model1 = Model::new();
    match registry.validate_all(&model1) {
        Ok(_) => println!("  ✓ Validation passed (no secure content)\n"),
        Err(e) => println!("  ✗ Validation failed: {}\n", e),
    }

    // Example 2: Model with valid secure content
    println!("Example 2: Model with valid secure content");
    let mut model2 = Model::new();

    let mut sc_info = SecureContentInfo::default();
    sc_info.keystore_uuid = Some("ks-12345678-1234-1234-1234-123456789abc".to_string());

    // Add consumer
    sc_info.consumers.push(Consumer {
        consumer_id: "alice".to_string(),
        key_id: Some("alice-key-001".to_string()),
        key_value: None,
    });

    // Add resource data group with encryption info
    let group = ResourceDataGroup {
        key_uuid: "cek-87654321-4321-4321-4321-cba987654321".to_string(),
        access_rights: vec![AccessRight {
            consumer_index: 0,
            kek_params: KEKParams {
                wrapping_algorithm: "http://www.w3.org/2001/04/xmlenc#rsa-oaep-mgf1p"
                    .to_string(),
                mgf_algorithm: Some("http://www.w3.org/2009/xmlenc11#mgf1sha256".to_string()),
                digest_method: Some("http://www.w3.org/2001/04/xmlenc#sha256".to_string()),
            },
            cipher_value: "YWJjZGVmZ2hpamtsbW5vcHFyc3R1dnd4eXo=".to_string(),
        }],
        resource_data: vec![ResourceData {
            path: "/3D/3dmodel.model".to_string(),
            cek_params: CEKParams {
                encryption_algorithm: "http://www.w3.org/2009/xmlenc11#aes256-gcm".to_string(),
                compression: "none".to_string(),
                iv: Some("MTIzNDU2Nzg5MGFiY2RlZg==".to_string()),
                tag: Some("YWJjZGVmZ2hpamtsbW5vcA==".to_string()),
                aad: None,
            },
        }],
    };

    sc_info.resource_data_groups.push(group);
    model2.secure_content = Some(sc_info);

    println!("  Keystore UUID: ks-12345678-...");
    println!("  Consumers: 1 (alice)");
    println!("  Resource Data Groups: 1");
    println!("  Encrypted Files: 1 (/3D/3dmodel.model)");

    match registry.validate_all(&model2) {
        Ok(_) => println!("  ✓ Validation passed\n"),
        Err(e) => println!("  ✗ Validation failed: {}\n", e),
    }

    // Example 3: Model with invalid secure content (duplicate consumer IDs)
    println!("Example 3: Model with invalid secure content (duplicate consumer IDs)");
    let mut model3 = Model::new();

    let mut sc_info_invalid = SecureContentInfo::default();
    sc_info_invalid.consumers.push(Consumer {
        consumer_id: "bob".to_string(),
        key_id: None,
        key_value: None,
    });
    sc_info_invalid.consumers.push(Consumer {
        consumer_id: "bob".to_string(), // Duplicate!
        key_id: None,
        key_value: None,
    });

    model3.secure_content = Some(sc_info_invalid);

    match registry.validate_all(&model3) {
        Ok(_) => println!("  ✓ Validation passed (unexpected!)\n"),
        Err(e) => println!("  ✗ Validation failed as expected: {}\n", e),
    }

    // Example 4: Model with invalid consumer index
    println!("Example 4: Model with invalid consumer index");
    let mut model4 = Model::new();

    let mut sc_info_bad_index = SecureContentInfo::default();
    sc_info_bad_index.consumers.push(Consumer {
        consumer_id: "charlie".to_string(),
        key_id: None,
        key_value: None,
    });

    let bad_group = ResourceDataGroup {
        key_uuid: "key-uuid".to_string(),
        access_rights: vec![AccessRight {
            consumer_index: 10, // Invalid - only 1 consumer exists!
            kek_params: KEKParams {
                wrapping_algorithm: "rsa-oaep-mgf1p".to_string(),
                mgf_algorithm: None,
                digest_method: None,
            },
            cipher_value: "cipher".to_string(),
        }],
        resource_data: vec![],
    };

    sc_info_bad_index.resource_data_groups.push(bad_group);
    model4.secure_content = Some(sc_info_bad_index);

    match registry.validate_all(&model4) {
        Ok(_) => println!("  ✓ Validation passed (unexpected!)\n"),
        Err(e) => println!("  ✗ Validation failed as expected: {}\n", e),
    }

    println!("=== Summary ===");
    println!("SecureContentExtensionHandler validates:");
    println!("  • Consumer ID uniqueness");
    println!("  • Valid consumer index references");
    println!("  • Non-empty required fields (UUIDs, algorithms, paths)");
    println!("  • Structural consistency of secure content data");
    println!("\nNote: This handler validates metadata only.");
    println!("Actual cryptographic operations are not performed.");
    println!("See SECURE_CONTENT_SUPPORT.md for details.");

    Ok(())
}
