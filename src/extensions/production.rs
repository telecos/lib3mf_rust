//! Production extension handler implementation

use crate::error::Result;
use crate::extension::ExtensionHandler;
use crate::model::{Extension, Model};

/// Extension handler for the Production extension
///
/// This handler provides validation and processing for the Production extension,
/// which includes production-specific metadata such as UUIDs and paths.
///
/// # Example
///
/// ```ignore
/// use lib3mf::extensions::ProductionExtensionHandler;
/// use lib3mf::extension::{ExtensionHandler, ExtensionRegistry};
///
/// let handler = ProductionExtensionHandler;
/// let mut registry = ExtensionRegistry::new();
/// registry.register(Box::new(handler));
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ProductionExtensionHandler;

impl ExtensionHandler for ProductionExtensionHandler {
    fn extension_type(&self) -> Extension {
        Extension::Production
    }

    fn validate(&self, _model: &Model) -> Result<()> {
        // TODO: Implement production-specific validation
        // - Validate UUID format
        // - Validate path references
        Ok(())
    }

    fn is_used_in_model(&self, model: &Model) -> bool {
        // Check if any objects have production info
        model
            .resources
            .objects
            .iter()
            .any(|obj| obj.production.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Object, ProductionInfo};

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
    fn test_is_used_in_model_with_production_info() {
        let handler = ProductionExtensionHandler;
        let mut model = Model::new();

        let mut obj = Object::new(1);
        obj.production = Some(ProductionInfo {
            uuid: Some("550e8400-e29b-41d4-a716-446655440000".to_string()),
            path: None,
        });
        model.resources.objects.push(obj);

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_validate_empty_model() {
        let handler = ProductionExtensionHandler;
        let model = Model::new();
        assert!(handler.validate(&model).is_ok());
    }
}
