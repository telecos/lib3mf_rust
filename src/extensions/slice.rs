//! Slice extension handler implementation

use crate::error::Result;
use crate::extension::ExtensionHandler;
use crate::model::{Extension, Model};

/// Extension handler for the Slice extension
///
/// This handler provides validation and processing for the Slice extension,
/// which enables slicing data for additive manufacturing processes.
///
/// # Example
///
/// ```ignore
/// use lib3mf::extensions::SliceExtensionHandler;
/// use lib3mf::extension::{ExtensionHandler, ExtensionRegistry};
///
/// let handler = SliceExtensionHandler;
/// let mut registry = ExtensionRegistry::new();
/// registry.register(Box::new(handler));
/// ```
#[derive(Debug, Clone, Copy)]
pub struct SliceExtensionHandler;

impl ExtensionHandler for SliceExtensionHandler {
    fn extension_type(&self) -> Extension {
        Extension::Slice
    }

    fn validate(&self, _model: &Model) -> Result<()> {
        // TODO: Implement slice-specific validation
        // - Validate slice references in build items
        // - Validate polygon winding order
        // - Validate slice stack references
        Ok(())
    }

    fn is_used_in_model(&self, model: &Model) -> bool {
        // Check if any objects have slice stack references
        model
            .resources
            .objects
            .iter()
            .any(|obj| obj.slicestackid.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Object;

    #[test]
    fn test_extension_type() {
        let handler = SliceExtensionHandler;
        assert_eq!(handler.extension_type(), Extension::Slice);
    }

    #[test]
    fn test_namespace() {
        let handler = SliceExtensionHandler;
        assert_eq!(
            handler.namespace(),
            "http://schemas.microsoft.com/3dmanufacturing/slice/2015/07"
        );
    }

    #[test]
    fn test_name() {
        let handler = SliceExtensionHandler;
        assert_eq!(handler.name(), "Slice");
    }

    #[test]
    fn test_is_used_in_model_empty() {
        let handler = SliceExtensionHandler;
        let model = Model::new();
        assert!(!handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_with_slice_ref() {
        let handler = SliceExtensionHandler;
        let mut model = Model::new();

        let mut obj = Object::new(1);
        obj.slicestackid = Some(100);
        model.resources.objects.push(obj);

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_validate_empty_model() {
        let handler = SliceExtensionHandler;
        let model = Model::new();
        assert!(handler.validate(&model).is_ok());
    }
}
