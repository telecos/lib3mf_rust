//! Secure Content extension handler

use crate::error::Result;
use crate::extension::ExtensionHandler;
use crate::model::{Extension, Model};

/// Handler for the Secure Content extension
///
/// This handler validates secure content metadata including keystore structure,
/// consumer definitions, and encryption references. It does not perform actual
/// cryptographic operations - see SECURE_CONTENT_SUPPORT.md for details.
///
/// # Example
///
/// ```
/// use lib3mf::extensions::SecureContentExtensionHandler;
/// use lib3mf::{ExtensionHandler, Extension};
///
/// let handler = SecureContentExtensionHandler;
/// assert_eq!(handler.extension_type(), Extension::SecureContent);
/// assert_eq!(handler.name(), "SecureContent");
/// ```
pub struct SecureContentExtensionHandler;

impl ExtensionHandler for SecureContentExtensionHandler {
    fn extension_type(&self) -> Extension {
        Extension::SecureContent
    }

    fn validate(&self, model: &Model) -> Result<()> {
        // If there's no secure content info, nothing to validate
        let Some(ref sc_info) = model.secure_content else {
            return Ok(());
        };

        // Validate consumer IDs are unique
        let mut seen_ids = std::collections::HashSet::new();
        for consumer in &sc_info.consumers {
            if !seen_ids.insert(&consumer.consumer_id) {
                return Err(crate::Error::InvalidModel(format!(
                    "Duplicate consumer ID: {}",
                    consumer.consumer_id
                )));
            }
        }

        // Validate resource data groups
        for group in &sc_info.resource_data_groups {
            // Validate key UUID is not empty
            if group.key_uuid.is_empty() {
                return Err(crate::Error::InvalidModel(
                    "Resource data group key UUID cannot be empty".to_string(),
                ));
            }

            // Validate access rights
            for access_right in &group.access_rights {
                // Check consumer index is valid
                if access_right.consumer_index >= sc_info.consumers.len() {
                    return Err(crate::Error::InvalidModel(format!(
                        "Invalid consumer index {} (only {} consumers defined)",
                        access_right.consumer_index,
                        sc_info.consumers.len()
                    )));
                }

                // Validate wrapping algorithm is not empty
                if access_right.kek_params.wrapping_algorithm.is_empty() {
                    return Err(crate::Error::InvalidModel(
                        "Wrapping algorithm cannot be empty".to_string(),
                    ));
                }

                // Validate cipher value is not empty
                if access_right.cipher_value.is_empty() {
                    return Err(crate::Error::InvalidModel(
                        "Cipher value cannot be empty".to_string(),
                    ));
                }
            }

            // Validate resource data
            for resource in &group.resource_data {
                // Validate path is not empty
                if resource.path.is_empty() {
                    return Err(crate::Error::InvalidModel(
                        "Resource data path cannot be empty".to_string(),
                    ));
                }

                // Validate encryption algorithm is not empty
                if resource.cek_params.encryption_algorithm.is_empty() {
                    return Err(crate::Error::InvalidModel(
                        "Encryption algorithm cannot be empty".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    fn is_used_in_model(&self, model: &Model) -> bool {
        model.required_extensions.contains(&Extension::SecureContent)
            || model.secure_content.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        AccessRight, CEKParams, Consumer, KEKParams, ResourceData, ResourceDataGroup,
        SecureContentInfo,
    };

    #[test]
    fn test_extension_type() {
        let handler = SecureContentExtensionHandler;
        assert_eq!(handler.extension_type(), Extension::SecureContent);
    }

    #[test]
    fn test_namespace() {
        let handler = SecureContentExtensionHandler;
        assert_eq!(
            handler.namespace(),
            "http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07"
        );
    }

    #[test]
    fn test_name() {
        let handler = SecureContentExtensionHandler;
        assert_eq!(handler.name(), "SecureContent");
    }

    #[test]
    fn test_is_used_in_model_none() {
        let handler = SecureContentExtensionHandler;
        let model = Model::new();
        assert!(!handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_some() {
        let handler = SecureContentExtensionHandler;
        let mut model = Model::new();
        model.secure_content = Some(SecureContentInfo::default());
        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_validate_empty_model() {
        let handler = SecureContentExtensionHandler;
        let model = Model::new();
        assert!(handler.validate(&model).is_ok());
    }

    #[test]
    fn test_validate_model_with_empty_secure_content() {
        let handler = SecureContentExtensionHandler;
        let mut model = Model::new();
        model.secure_content = Some(SecureContentInfo::default());
        assert!(handler.validate(&model).is_ok());
    }

    #[test]
    fn test_validate_duplicate_consumer_ids() {
        let handler = SecureContentExtensionHandler;
        let mut model = Model::new();

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

        let result = handler.validate(&model);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Duplicate consumer ID"));
    }

    #[test]
    fn test_validate_invalid_consumer_index() {
        let handler = SecureContentExtensionHandler;
        let mut model = Model::new();

        let mut sc_info = SecureContentInfo::default();
        sc_info.consumers = vec![Consumer {
            consumer_id: "consumer1".to_string(),
            key_id: None,
            key_value: None,
        }];

        let group = ResourceDataGroup {
            key_uuid: "key-uuid-1".to_string(),
            access_rights: vec![AccessRight {
                consumer_index: 5, // Invalid - only 1 consumer defined
                kek_params: KEKParams {
                    wrapping_algorithm: "rsa-oaep-mgf1p".to_string(),
                    mgf_algorithm: None,
                    digest_method: None,
                },
                cipher_value: "base64data".to_string(),
            }],
            resource_data: vec![],
        };

        sc_info.resource_data_groups = vec![group];
        model.secure_content = Some(sc_info);

        let result = handler.validate(&model);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid consumer index"));
    }

    #[test]
    fn test_validate_empty_key_uuid() {
        let handler = SecureContentExtensionHandler;
        let mut model = Model::new();

        let mut sc_info = SecureContentInfo::default();
        let group = ResourceDataGroup {
            key_uuid: "".to_string(), // Empty
            access_rights: vec![],
            resource_data: vec![],
        };

        sc_info.resource_data_groups = vec![group];
        model.secure_content = Some(sc_info);

        let result = handler.validate(&model);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("key UUID cannot be empty"));
    }

    #[test]
    fn test_validate_empty_wrapping_algorithm() {
        let handler = SecureContentExtensionHandler;
        let mut model = Model::new();

        let mut sc_info = SecureContentInfo::default();
        sc_info.consumers = vec![Consumer {
            consumer_id: "consumer1".to_string(),
            key_id: None,
            key_value: None,
        }];

        let group = ResourceDataGroup {
            key_uuid: "key-uuid-1".to_string(),
            access_rights: vec![AccessRight {
                consumer_index: 0,
                kek_params: KEKParams {
                    wrapping_algorithm: "".to_string(), // Empty
                    mgf_algorithm: None,
                    digest_method: None,
                },
                cipher_value: "base64data".to_string(),
            }],
            resource_data: vec![],
        };

        sc_info.resource_data_groups = vec![group];
        model.secure_content = Some(sc_info);

        let result = handler.validate(&model);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Wrapping algorithm cannot be empty"));
    }

    #[test]
    fn test_validate_empty_cipher_value() {
        let handler = SecureContentExtensionHandler;
        let mut model = Model::new();

        let mut sc_info = SecureContentInfo::default();
        sc_info.consumers = vec![Consumer {
            consumer_id: "consumer1".to_string(),
            key_id: None,
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
                cipher_value: "".to_string(), // Empty
            }],
            resource_data: vec![],
        };

        sc_info.resource_data_groups = vec![group];
        model.secure_content = Some(sc_info);

        let result = handler.validate(&model);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cipher value cannot be empty"));
    }

    #[test]
    fn test_validate_empty_resource_path() {
        let handler = SecureContentExtensionHandler;
        let mut model = Model::new();

        let mut sc_info = SecureContentInfo::default();
        let group = ResourceDataGroup {
            key_uuid: "key-uuid-1".to_string(),
            access_rights: vec![],
            resource_data: vec![ResourceData {
                path: "".to_string(), // Empty
                cek_params: CEKParams {
                    encryption_algorithm: "AES-256-GCM".to_string(),
                    compression: "none".to_string(),
                    iv: None,
                    tag: None,
                    aad: None,
                },
            }],
        };

        sc_info.resource_data_groups = vec![group];
        model.secure_content = Some(sc_info);

        let result = handler.validate(&model);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Resource data path cannot be empty"));
    }

    #[test]
    fn test_validate_empty_encryption_algorithm() {
        let handler = SecureContentExtensionHandler;
        let mut model = Model::new();

        let mut sc_info = SecureContentInfo::default();
        let group = ResourceDataGroup {
            key_uuid: "key-uuid-1".to_string(),
            access_rights: vec![],
            resource_data: vec![ResourceData {
                path: "/3D/3dmodel.model".to_string(),
                cek_params: CEKParams {
                    encryption_algorithm: "".to_string(), // Empty
                    compression: "none".to_string(),
                    iv: None,
                    tag: None,
                    aad: None,
                },
            }],
        };

        sc_info.resource_data_groups = vec![group];
        model.secure_content = Some(sc_info);

        let result = handler.validate(&model);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Encryption algorithm cannot be empty"));
    }

    #[test]
    fn test_validate_valid_secure_content() {
        let handler = SecureContentExtensionHandler;
        let mut model = Model::new();

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
                    mgf_algorithm: Some("mgf1sha256".to_string()),
                    digest_method: Some("sha256".to_string()),
                },
                cipher_value: "YmFzZTY0ZGF0YQ==".to_string(),
            }],
            resource_data: vec![ResourceData {
                path: "/3D/3dmodel.model".to_string(),
                cek_params: CEKParams {
                    encryption_algorithm: "http://www.w3.org/2009/xmlenc11#aes256-gcm".to_string(),
                    compression: "none".to_string(),
                    iv: Some("aXZkYXRh".to_string()),
                    tag: Some("dGFnZGF0YQ==".to_string()),
                    aad: None,
                },
            }],
        };

        sc_info.resource_data_groups = vec![group];
        model.secure_content = Some(sc_info);

        assert!(handler.validate(&model).is_ok());
    }

    #[test]
    fn test_validate_multiple_consumers_and_groups() {
        let handler = SecureContentExtensionHandler;
        let mut model = Model::new();

        let mut sc_info = SecureContentInfo::default();
        sc_info.consumers = vec![
            Consumer {
                consumer_id: "consumer1".to_string(),
                key_id: None,
                key_value: None,
            },
            Consumer {
                consumer_id: "consumer2".to_string(),
                key_id: None,
                key_value: None,
            },
        ];

        let group1 = ResourceDataGroup {
            key_uuid: "key-uuid-1".to_string(),
            access_rights: vec![
                AccessRight {
                    consumer_index: 0,
                    kek_params: KEKParams {
                        wrapping_algorithm: "rsa-oaep-mgf1p".to_string(),
                        mgf_algorithm: None,
                        digest_method: None,
                    },
                    cipher_value: "cipher1".to_string(),
                },
                AccessRight {
                    consumer_index: 1,
                    kek_params: KEKParams {
                        wrapping_algorithm: "rsa-oaep-mgf1p".to_string(),
                        mgf_algorithm: None,
                        digest_method: None,
                    },
                    cipher_value: "cipher2".to_string(),
                },
            ],
            resource_data: vec![ResourceData {
                path: "/3D/3dmodel.model".to_string(),
                cek_params: CEKParams {
                    encryption_algorithm: "aes256-gcm".to_string(),
                    compression: "none".to_string(),
                    iv: None,
                    tag: None,
                    aad: None,
                },
            }],
        };

        sc_info.resource_data_groups = vec![group1];
        model.secure_content = Some(sc_info);

        assert!(handler.validate(&model).is_ok());
    }
}
