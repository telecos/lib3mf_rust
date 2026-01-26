//! Integration test for SecureContentExtensionHandler with ExtensionRegistry

use std::sync::Arc;

use lib3mf::{
    extensions::SecureContentExtensionHandler, AccessRight, CEKParams, Consumer, Extension,
    ExtensionHandler, ExtensionRegistry, KEKParams, Model, ResourceData, ResourceDataGroup,
    SecureContentInfo,
};

#[test]
fn test_secure_content_handler_with_registry() {
    let mut registry = ExtensionRegistry::new();
    registry.register(Arc::new(SecureContentExtensionHandler));

    // Verify handler is registered
    assert_eq!(registry.handlers().len(), 1);
    assert!(registry.get_handler(Extension::SecureContent).is_some());
}

#[test]
fn test_registry_validate_empty_model() {
    let mut registry = ExtensionRegistry::new();
    registry.register(Arc::new(SecureContentExtensionHandler));

    let model = Model::new();

    // Should pass validation for model without secure content
    assert!(registry.validate_all(&model).is_ok());
}

#[test]
fn test_registry_validate_model_with_secure_content() {
    let mut registry = ExtensionRegistry::new();
    registry.register(Arc::new(SecureContentExtensionHandler));

    let mut model = Model::new();

    // Create valid secure content info
    let mut sc_info = SecureContentInfo::default();
    sc_info.keystore_uuid = Some("ks-uuid-1".to_string());
    sc_info.consumers = vec![Consumer {
        consumer_id: "consumer1".to_string(),
        key_id: Some("key-123".to_string()),
        key_value: None,
    }];

    let group = ResourceDataGroup {
        key_uuid: "key-uuid-1".to_string(),
        access_rights: vec![AccessRight {
            consumer_index: 0,
            kek_params: KEKParams {
                wrapping_algorithm: "rsa-oaep-mgf1p".to_string(),
                mgf_algorithm: None,
                digest_method: None,
            },
            cipher_value: "base64data".to_string(),
        }],
        resource_data: vec![ResourceData {
            path: "/3D/3dmodel.model".to_string(),
            cek_params: CEKParams {
                encryption_algorithm: "aes256-gcm".to_string(),
                compression: "none".to_string(),
                iv: Some("iv".to_string()),
                tag: Some("tag".to_string()),
                aad: None,
            },
        }],
    };

    sc_info.resource_data_groups = vec![group];
    model.secure_content = Some(sc_info);

    // Should pass validation
    assert!(registry.validate_all(&model).is_ok());
}

#[test]
fn test_registry_validate_invalid_secure_content() {
    let mut registry = ExtensionRegistry::new();
    registry.register(Arc::new(SecureContentExtensionHandler));

    let mut model = Model::new();

    // Create invalid secure content (duplicate consumer IDs)
    let mut sc_info = SecureContentInfo::default();
    sc_info.consumers = vec![
        Consumer {
            consumer_id: "consumer1".to_string(),
            key_id: None,
            key_value: None,
        },
        Consumer {
            consumer_id: "consumer1".to_string(), // Duplicate
            key_id: None,
            key_value: None,
        },
    ];

    model.secure_content = Some(sc_info);

    // Should fail validation
    let result = registry.validate_all(&model);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Duplicate consumer ID"));
}

#[test]
fn test_registry_post_parse_all() {
    let mut registry = ExtensionRegistry::new();
    registry.register(Arc::new(SecureContentExtensionHandler));

    let mut model = Model::new();
    model.secure_content = Some(SecureContentInfo::default());

    // Should succeed (default implementation does nothing)
    assert!(registry.post_parse_all(&mut model).is_ok());
}

#[test]
fn test_registry_pre_write_all() {
    let mut registry = ExtensionRegistry::new();
    registry.register(Arc::new(SecureContentExtensionHandler));

    let mut model = Model::new();
    model.secure_content = Some(SecureContentInfo::default());

    // Should succeed (default implementation does nothing)
    assert!(registry.pre_write_all(&mut model).is_ok());
}

#[test]
fn test_handler_properties() {
    let handler = SecureContentExtensionHandler;

    assert_eq!(handler.extension_type(), Extension::SecureContent);
    assert_eq!(handler.name(), "SecureContent");
    assert_eq!(
        handler.namespace(),
        "http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07"
    );
}
