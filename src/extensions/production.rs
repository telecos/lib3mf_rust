//! Production Extension Handler
//!
//! Concrete implementation of ExtensionHandler for the Production extension.

use crate::error::Result;
use crate::extension::ExtensionHandler;
use crate::model::{Extension, Model};
use crate::validator::{validate_production_extension, validate_production_paths};

/// Handler for the Production extension
///
/// This handler consolidates all production-related validation and processing,
/// including validation of production paths, UUIDs, and production-specific attributes.
///
/// # Example
///
/// ```
/// use lib3mf::extensions::ProductionExtensionHandler;
/// use lib3mf::{ExtensionHandler, ExtensionRegistry, Model};
///
/// let handler = ProductionExtensionHandler;
/// let model = Model::new();
///
/// // Validate production extension data
/// assert!(handler.validate(&model).is_ok());
/// ```
pub struct ProductionExtensionHandler;

impl ExtensionHandler for ProductionExtensionHandler {
    fn extension_type(&self) -> Extension {
        Extension::Production
    }

    fn validate(&self, model: &Model) -> Result<()> {
        // Call existing production validators
        validate_production_extension(model)?;
        validate_production_paths(model)?;
        Ok(())
    }

    fn is_used_in_model(&self, model: &Model) -> bool {
        // Check if any objects have production info
        let has_object_production = model
            .resources
            .objects
            .iter()
            .any(|obj| obj.production.is_some());

        // Check if any build items have production attributes
        let has_build_production = model
            .build
            .items
            .iter()
            .any(|item| item.production_uuid.is_some() || item.production_path.is_some());

        // Check if build has production UUID
        let has_build_uuid = model.build.production_uuid.is_some();

        has_object_production || has_build_production || has_build_uuid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BuildItem, Object, ProductionInfo};

    #[test]
    fn test_extension_type() {
        let handler = ProductionExtensionHandler;
        assert_eq!(handler.extension_type(), Extension::Production);
    }

    #[test]
    fn test_namespace() {
        let handler = ProductionExtensionHandler;
        assert_eq!(
            handler.namespace(),
            "http://schemas.microsoft.com/3dmanufacturing/production/2015/06"
        );
    }

    #[test]
    fn test_name() {
        let handler = ProductionExtensionHandler;
        assert_eq!(handler.name(), "Production");
    }

    #[test]
    fn test_is_used_in_model_empty() {
        let handler = ProductionExtensionHandler;
        let model = Model::new();
        assert!(!handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_with_object_production() {
        let handler = ProductionExtensionHandler;
        let mut model = Model::new();

        // Add an object with production info
        let mut obj = Object::new(1);
        obj.production = Some(ProductionInfo::with_uuid("test-uuid".to_string()));
        model.resources.objects.push(obj);

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_with_build_item_production() {
        let handler = ProductionExtensionHandler;
        let mut model = Model::new();

        // Add a build item with production UUID
        let mut item = BuildItem::new(1);
        item.production_uuid = Some("build-item-uuid".to_string());
        model.build.items.push(item);

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_with_build_production_uuid() {
        let handler = ProductionExtensionHandler;
        let mut model = Model::new();

        // Set build production UUID
        model.build.production_uuid = Some("build-uuid".to_string());

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_validate_empty_model() {
        let handler = ProductionExtensionHandler;
        let model = Model::new();

        // Should pass validation for empty model
        assert!(handler.validate(&model).is_ok());
    }

    #[test]
    fn test_validate_valid_production_path() {
        let handler = ProductionExtensionHandler;
        let mut model = Model::new();

        // Add object with valid production path
        let mut obj = Object::new(1);
        obj.production = Some(ProductionInfo {
            uuid: Some("test-uuid".to_string()),
            path: Some("/3D/other_part.model".to_string()),
        });
        model.resources.objects.push(obj);

        assert!(handler.validate(&model).is_ok());
    }

    #[test]
    fn test_validate_invalid_production_path() {
        let handler = ProductionExtensionHandler;
        let mut model = Model::new();

        // Add object with invalid production path (doesn't start with /)
        let mut obj = Object::new(1);
        obj.production = Some(ProductionInfo {
            uuid: Some("test-uuid".to_string()),
            path: Some("relative/path.model".to_string()),
        });
        model.resources.objects.push(obj);

        assert!(handler.validate(&model).is_err());
    }

    #[test]
    fn test_post_parse_default() {
        let handler = ProductionExtensionHandler;
        let mut model = Model::new();

        // Default implementation should succeed
        assert!(handler.post_parse(&mut model).is_ok());
    }

    #[test]
    fn test_pre_write_default() {
        let handler = ProductionExtensionHandler;
        let mut model = Model::new();

        // Default implementation should succeed
        assert!(handler.pre_write(&mut model).is_ok());
    }
}
