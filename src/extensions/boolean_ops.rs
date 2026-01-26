//! Boolean Operations extension handler
//!
//! Implements the `ExtensionHandler` trait for the Boolean Operations extension.

use crate::error::Result;
use crate::extension::ExtensionHandler;
use crate::model::{Extension, Model};
use crate::validator::validate_boolean_operations;

/// Extension handler for the Boolean Operations extension
///
/// This handler provides validation for boolean operations in 3MF models,
/// which allow objects to be defined using volumetric boolean operations
/// (union, intersection, difference) on other objects.
///
/// # Example
///
/// ```ignore
/// use lib3mf::extensions::BooleanOperationsExtensionHandler;
/// use lib3mf::extension::{ExtensionHandler, ExtensionRegistry};
/// use std::sync::Arc;
///
/// let handler = BooleanOperationsExtensionHandler;
/// let mut registry = ExtensionRegistry::new();
/// registry.register(Arc::new(handler));
/// ```
pub struct BooleanOperationsExtensionHandler;

impl ExtensionHandler for BooleanOperationsExtensionHandler {
    fn extension_type(&self) -> Extension {
        Extension::BooleanOperations
    }

    fn validate(&self, model: &Model) -> Result<()> {
        validate_boolean_operations(model)
    }

    fn is_used_in_model(&self, model: &Model) -> bool {
        // Check if extension is required or if any object has a boolean_shape field
        model
            .required_extensions
            .contains(&Extension::BooleanOperations)
            || model
                .resources
                .objects
                .iter()
                .any(|obj| obj.boolean_shape.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BooleanOpType, BooleanRef, BooleanShape, Mesh, Object};

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
    fn test_is_used_in_model_with_boolean_shape() {
        let handler = BooleanOperationsExtensionHandler;
        let mut model = Model::new();

        // Add a base object
        let mut base_obj = Object::new(1);
        base_obj.mesh = Some(Mesh::new());
        model.resources.objects.push(base_obj);

        // Add an object with boolean_shape
        let mut boolean_shape = BooleanShape::new(1, BooleanOpType::Union);
        boolean_shape.operands.push(BooleanRef::new(1));

        let mut obj = Object::new(2);
        obj.boolean_shape = Some(boolean_shape);
        model.resources.objects.push(obj);

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_without_boolean_shape() {
        let handler = BooleanOperationsExtensionHandler;
        let mut model = Model::new();

        // Add a regular object without boolean_shape
        let mut obj = Object::new(1);
        obj.mesh = Some(Mesh::new());
        model.resources.objects.push(obj);

        assert!(!handler.is_used_in_model(&model));
    }

    #[test]
    fn test_validate_valid_model() {
        let handler = BooleanOperationsExtensionHandler;
        let mut model = Model::new();

        // Create a base mesh object
        let mut base_obj = Object::new(1);
        base_obj.mesh = Some(Mesh::new());
        model.resources.objects.push(base_obj);

        // Create an operand mesh object
        let mut operand_obj = Object::new(2);
        operand_obj.mesh = Some(Mesh::new());
        model.resources.objects.push(operand_obj);

        // Create an object with boolean_shape
        let mut boolean_shape = BooleanShape::new(1, BooleanOpType::Union);
        boolean_shape.operands.push(BooleanRef::new(2));

        let mut obj = Object::new(3);
        obj.boolean_shape = Some(boolean_shape);
        model.resources.objects.push(obj);

        // Validation should pass
        assert!(handler.validate(&model).is_ok());
    }

    #[test]
    fn test_validate_invalid_model_no_operands() {
        let handler = BooleanOperationsExtensionHandler;
        let mut model = Model::new();

        // Create a base object
        let mut base_obj = Object::new(1);
        base_obj.mesh = Some(Mesh::new());
        model.resources.objects.push(base_obj);

        // Create an object with boolean_shape but no operands (invalid)
        let boolean_shape = BooleanShape::new(1, BooleanOpType::Union);

        let mut obj = Object::new(2);
        obj.boolean_shape = Some(boolean_shape);
        model.resources.objects.push(obj);

        // Validation should fail
        let result = handler.validate(&model);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Boolean shape has no operands"));
    }

    #[test]
    fn test_validate_invalid_model_nonexistent_base() {
        let handler = BooleanOperationsExtensionHandler;
        let mut model = Model::new();

        // Create an object with boolean_shape referencing non-existent base
        let mut boolean_shape = BooleanShape::new(999, BooleanOpType::Union);
        boolean_shape.operands.push(BooleanRef::new(1));

        let mut obj = Object::new(1);
        obj.boolean_shape = Some(boolean_shape);
        model.resources.objects.push(obj);

        // Validation should fail
        let result = handler.validate(&model);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("non-existent object ID"));
    }
}
