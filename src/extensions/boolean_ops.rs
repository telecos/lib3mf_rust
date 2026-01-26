//! Boolean Operations extension handler implementation

use crate::error::Result;
use crate::extension::ExtensionHandler;
use crate::model::{Extension, Model};

/// Extension handler for the Boolean Operations extension
///
/// This handler provides validation and processing for the Boolean Operations extension,
/// which enables boolean operations (union, intersection, difference) on meshes.
///
/// # Example
///
/// ```ignore
/// use lib3mf::extensions::BooleanOperationsExtensionHandler;
/// use lib3mf::extension::{ExtensionHandler, ExtensionRegistry};
///
/// let handler = BooleanOperationsExtensionHandler;
/// let mut registry = ExtensionRegistry::new();
/// registry.register(Box::new(handler));
/// ```
#[derive(Debug, Clone, Copy)]
pub struct BooleanOperationsExtensionHandler;

impl ExtensionHandler for BooleanOperationsExtensionHandler {
    fn extension_type(&self) -> Extension {
        Extension::BooleanOperations
    }

    fn validate(&self, _model: &Model) -> Result<()> {
        // TODO: Implement boolean operations-specific validation
        // - Validate boolean shape references
        // - Validate operation types
        // - Validate mesh references in boolean operations
        Ok(())
    }

    fn is_used_in_model(&self, model: &Model) -> bool {
        // Check if any objects have boolean shapes
        model
            .resources
            .objects
            .iter()
            .any(|obj| obj.boolean_shape.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BooleanOpType, BooleanRef, BooleanShape, Object};

    #[test]
    fn test_extension_type() {
        let handler = BooleanOperationsExtensionHandler;
        assert_eq!(handler.extension_type(), Extension::BooleanOperations);
    }

    #[test]
    fn test_namespace() {
        let handler = BooleanOperationsExtensionHandler;
        assert_eq!(
            handler.namespace(),
            "http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07"
        );
    }

    #[test]
    fn test_name() {
        let handler = BooleanOperationsExtensionHandler;
        assert_eq!(handler.name(), "BooleanOperations");
    }

    #[test]
    fn test_is_used_in_model_empty() {
        let handler = BooleanOperationsExtensionHandler;
        let model = Model::new();
        assert!(!handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_with_boolean_shape() {
        let handler = BooleanOperationsExtensionHandler;
        let mut model = Model::new();

        let mut obj = Object::new(1);
        obj.boolean_shape = Some(BooleanShape {
            objectid: 2,
            operation: BooleanOpType::Union,
            path: None,
            operands: vec![BooleanRef::new(3)],
        });
        model.resources.objects.push(obj);

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_validate_empty_model() {
        let handler = BooleanOperationsExtensionHandler;
        let model = Model::new();
        assert!(handler.validate(&model).is_ok());
    }
}
