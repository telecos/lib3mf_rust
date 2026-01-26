//! Displacement extension handler implementation

use crate::error::Result;
use crate::extension::ExtensionHandler;
use crate::model::{Extension, Model};

/// Extension handler for the Displacement extension
///
/// This handler provides validation and processing for the Displacement extension,
/// which enables displacement mapping for surface detail.
///
/// # Example
///
/// ```ignore
/// use lib3mf::extensions::DisplacementExtensionHandler;
/// use lib3mf::extension::{ExtensionHandler, ExtensionRegistry};
///
/// let handler = DisplacementExtensionHandler;
/// let mut registry = ExtensionRegistry::new();
/// registry.register(Box::new(handler));
/// ```
#[derive(Debug, Clone, Copy)]
pub struct DisplacementExtensionHandler;

impl ExtensionHandler for DisplacementExtensionHandler {
    fn extension_type(&self) -> Extension {
        Extension::Displacement
    }

    fn validate(&self, _model: &Model) -> Result<()> {
        // TODO: Implement displacement-specific validation
        // - Validate displacement group references
        // - Validate texture coordinates
        // - Validate displacement values
        Ok(())
    }

    fn is_used_in_model(&self, model: &Model) -> bool {
        // Check if any displacement2d groups exist
        !model.resources.disp2d_groups.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Disp2DGroup;

    #[test]
    fn test_extension_type() {
        let handler = DisplacementExtensionHandler;
        assert_eq!(handler.extension_type(), Extension::Displacement);
    }

    #[test]
    fn test_namespace() {
        let handler = DisplacementExtensionHandler;
        assert_eq!(
            handler.namespace(),
            "http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07"
        );
    }

    #[test]
    fn test_name() {
        let handler = DisplacementExtensionHandler;
        assert_eq!(handler.name(), "Displacement");
    }

    #[test]
    fn test_is_used_in_model_empty() {
        let handler = DisplacementExtensionHandler;
        let model = Model::new();
        assert!(!handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_with_displacement() {
        let handler = DisplacementExtensionHandler;
        let mut model = Model::new();

        // Create a minimal displacement group with required parameters
        let group = Disp2DGroup::new(1, 10, 20, 0.5);
        model.resources.disp2d_groups.push(group);

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_validate_empty_model() {
        let handler = DisplacementExtensionHandler;
        let model = Model::new();
        assert!(handler.validate(&model).is_ok());
    }
}
