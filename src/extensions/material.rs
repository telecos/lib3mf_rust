//! Material extension handler implementation

use crate::error::Result;
use crate::extension::ExtensionHandler;
use crate::model::{Extension, Model};
use crate::validator::{
    validate_material_references, validate_multiproperties_references, validate_texture_paths,
    validate_triangle_properties,
};

/// Extension handler for the Material extension
///
/// This handler provides validation and processing for the Material & Properties
/// extension, which includes:
/// - Base material groups
/// - Color groups  
/// - Texture2D groups and resources
/// - Composite materials
/// - Multi-properties
///
/// # Example
///
/// ```ignore
/// use lib3mf::extensions::MaterialExtensionHandler;
/// use lib3mf::extension::{ExtensionHandler, ExtensionRegistry};
///
/// let handler = MaterialExtensionHandler;
/// let mut registry = ExtensionRegistry::new();
/// registry.register(Box::new(handler));
/// ```
#[derive(Debug, Clone, Copy)]
pub struct MaterialExtensionHandler;

impl ExtensionHandler for MaterialExtensionHandler {
    fn extension_type(&self) -> Extension {
        Extension::Material
    }

    fn validate(&self, model: &Model) -> Result<()> {
        // Run all material-specific validators
        validate_material_references(model)?;
        validate_texture_paths(model)?;
        validate_multiproperties_references(model)?;
        validate_triangle_properties(model)?;
        Ok(())
    }

    fn is_used_in_model(&self, model: &Model) -> bool {
        // Check if the model uses any material-related resources
        !model.resources.base_material_groups.is_empty()
            || !model.resources.color_groups.is_empty()
            || !model.resources.texture2d_groups.is_empty()
            || !model.resources.composite_materials.is_empty()
            || !model.resources.multi_properties.is_empty()
            || !model.resources.texture2d_resources.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        BaseMaterial, BaseMaterialGroup, ColorGroup, CompositeMaterials, MultiProperties,
        Texture2D, Texture2DGroup,
    };

    #[test]
    fn test_extension_type() {
        let handler = MaterialExtensionHandler;
        assert_eq!(handler.extension_type(), Extension::Material);
    }

    #[test]
    fn test_namespace() {
        let handler = MaterialExtensionHandler;
        assert_eq!(
            handler.namespace(),
            "http://schemas.microsoft.com/3dmanufacturing/material/2015/02"
        );
    }

    #[test]
    fn test_name() {
        let handler = MaterialExtensionHandler;
        assert_eq!(handler.name(), "Material");
    }

    #[test]
    fn test_is_used_in_model_empty() {
        let handler = MaterialExtensionHandler;
        let model = Model::new();
        // Empty model should not be using the material extension
        assert!(!handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_with_base_materials() {
        let handler = MaterialExtensionHandler;
        let mut model = Model::new();

        // Add a base material group
        let mut group = BaseMaterialGroup::new(1);
        group
            .materials
            .push(BaseMaterial::new("Steel".to_string(), (128, 128, 128, 255)));
        model.resources.base_material_groups.push(group);

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_with_color_groups() {
        let handler = MaterialExtensionHandler;
        let mut model = Model::new();

        // Add a color group
        let mut group = ColorGroup::new(1);
        group.colors.push((255, 0, 0, 255));
        model.resources.color_groups.push(group);

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_with_texture2d_groups() {
        let handler = MaterialExtensionHandler;
        let mut model = Model::new();

        // Add a texture2d group
        let group = Texture2DGroup::new(1, 10);
        model.resources.texture2d_groups.push(group);

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_with_texture2d_resources() {
        let handler = MaterialExtensionHandler;
        let mut model = Model::new();

        // Add a texture2d resource
        let texture = Texture2D::new(
            10,
            "/Textures/texture.png".to_string(),
            "image/png".to_string(),
        );
        model.resources.texture2d_resources.push(texture);

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_with_composite_materials() {
        let handler = MaterialExtensionHandler;
        let mut model = Model::new();

        // Add a composite materials group
        let group = CompositeMaterials::new(1, 10, vec![0, 1]);
        model.resources.composite_materials.push(group);

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_with_multi_properties() {
        let handler = MaterialExtensionHandler;
        let mut model = Model::new();

        // Add a multi-properties group
        let group = MultiProperties::new(1, vec![10, 20]);
        model.resources.multi_properties.push(group);

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_validate_empty_model() {
        let handler = MaterialExtensionHandler;
        let model = Model::new();
        // Empty model should pass validation
        assert!(handler.validate(&model).is_ok());
    }

    #[test]
    fn test_validate_valid_base_materials() {
        let handler = MaterialExtensionHandler;
        let mut model = Model::new();

        // Add a valid base material group
        let mut group = BaseMaterialGroup::new(1);
        group
            .materials
            .push(BaseMaterial::new("Steel".to_string(), (128, 128, 128, 255)));
        model.resources.base_material_groups.push(group);

        assert!(handler.validate(&model).is_ok());
    }

    #[test]
    fn test_validate_duplicate_property_group_ids() {
        let handler = MaterialExtensionHandler;
        let mut model = Model::new();

        // Add two property groups with the same ID (should fail validation)
        let group1 = ColorGroup::new(1);
        let group2 = BaseMaterialGroup::new(1); // Same ID as color group

        model.resources.color_groups.push(group1);
        model.resources.base_material_groups.push(group2);

        // This should fail validation because IDs must be unique
        assert!(handler.validate(&model).is_err());
    }
}
