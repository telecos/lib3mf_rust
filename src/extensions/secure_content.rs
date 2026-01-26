//! Secure Content extension handler implementation

use crate::error::Result;
use crate::extension::ExtensionHandler;
use crate::model::{Extension, Model};

/// Extension handler for the Secure Content extension
///
/// This handler provides validation and processing for the Secure Content extension,
/// which enables encryption and digital rights management for 3MF content.
///
/// # Example
///
/// ```ignore
/// use lib3mf::extensions::SecureContentExtensionHandler;
/// use lib3mf::extension::{ExtensionHandler, ExtensionRegistry};
///
/// let handler = SecureContentExtensionHandler;
/// let mut registry = ExtensionRegistry::new();
/// registry.register(Box::new(handler));
/// ```
#[derive(Debug, Clone, Copy)]
pub struct SecureContentExtensionHandler;

impl ExtensionHandler for SecureContentExtensionHandler {
    fn extension_type(&self) -> Extension {
        Extension::SecureContent
    }

    fn validate(&self, _model: &Model) -> Result<()> {
        // TODO: Implement secure content-specific validation
        // - Validate encryption parameters
        // - Validate consumer/access rights
        // - Validate key encryption
        Ok(())
    }

    fn is_used_in_model(&self, model: &Model) -> bool {
        // Check if secure content info exists
        model.secure_content.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SecureContentInfo;

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
    fn test_is_used_in_model_empty() {
        let handler = SecureContentExtensionHandler;
        let model = Model::new();
        assert!(!handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_with_secure_content() {
        let handler = SecureContentExtensionHandler;
        let mut model = Model::new();

        model.secure_content = Some(SecureContentInfo {
            keystore_uuid: Some("550e8400-e29b-41d4-a716-446655440000".to_string()),
            encrypted_files: vec![],
            consumers: vec![],
            resource_data_groups: vec![],
            consumer_ids: vec![],
            consumer_count: 0,
            wrapping_algorithms: vec![],
        });

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_validate_empty_model() {
        let handler = SecureContentExtensionHandler;
        let model = Model::new();
        assert!(handler.validate(&model).is_ok());
    }
}
