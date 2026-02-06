//! Volumetric extension handler
//!
//! Implements the `ExtensionHandler` trait for the Volumetric extension.

use crate::error::Result;
use crate::extension::ExtensionHandler;
use crate::model::{Extension, Model};
use crate::validator;

/// Extension handler for the Volumetric extension
///
/// This handler provides validation and utility functions for 3MF models
/// that use the Volumetric extension (volumetric data for 3D printing).
///
/// # Example
///
/// ```rust
/// use lib3mf::extensions::VolumetricExtensionHandler;
/// use lib3mf::extension::ExtensionHandler;
///
/// let handler = VolumetricExtensionHandler;
/// assert_eq!(handler.name(), "Volumetric");
/// ```
pub struct VolumetricExtensionHandler;

impl ExtensionHandler for VolumetricExtensionHandler {
    fn extension_type(&self) -> Extension {
        Extension::Volumetric
    }

    fn validate(&self, model: &Model) -> Result<()> {
        // Call the existing volumetric validation from the validator module
        validator::validate_volumetric_extension(model)
    }

    fn is_used_in_model(&self, model: &Model) -> bool {
        // Check if any volumetric resources are present in the model
        !model.resources.volumetric_data.is_empty()
            || !model.resources.volumetric_property_groups.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{VolumetricData, VolumetricPropertyGroup};

    #[test]
    fn test_extension_type() {
        let handler = VolumetricExtensionHandler;
        assert_eq!(handler.extension_type(), Extension::Volumetric);
    }

    #[test]
    fn test_namespace() {
        let handler = VolumetricExtensionHandler;
        assert_eq!(
            handler.namespace(),
            "http://schemas.3mf.io/volumetric/2023/02"
        );
    }

    #[test]
    fn test_name() {
        let handler = VolumetricExtensionHandler;
        assert_eq!(handler.name(), "Volumetric");
    }

    #[test]
    fn test_is_used_in_model_empty() {
        let handler = VolumetricExtensionHandler;
        let model = Model::new();

        // Empty model should not use volumetric extension
        assert!(!handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_with_volumetric_data() {
        let handler = VolumetricExtensionHandler;
        let mut model = Model::new();

        // Add volumetric data
        model.resources.volumetric_data.push(VolumetricData::new(1));

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_with_property_group() {
        let handler = VolumetricExtensionHandler;
        let mut model = Model::new();

        // Add volumetric property group
        model
            .resources
            .volumetric_property_groups
            .push(VolumetricPropertyGroup::new(1));

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_validate_empty_model() {
        let handler = VolumetricExtensionHandler;
        let model = Model::new();

        // Empty model should pass validation (no volumetric resources to validate)
        assert!(handler.validate(&model).is_ok());
    }

    #[test]
    fn test_default_post_parse() {
        let handler = VolumetricExtensionHandler;
        let mut model = Model::new();

        // Default post_parse should do nothing and return Ok
        assert!(handler.post_parse(&mut model).is_ok());
    }

    #[test]
    fn test_default_pre_write() {
        let handler = VolumetricExtensionHandler;
        let mut model = Model::new();

        // Default pre_write should do nothing and return Ok
        assert!(handler.pre_write(&mut model).is_ok());
    }
}
