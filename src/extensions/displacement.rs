//! Displacement extension handler
//!
//! Implements the `ExtensionHandler` trait for the Displacement extension.

use crate::error::Result;
use crate::extension::ExtensionHandler;
use crate::model::{Extension, Model};
use crate::validator;

/// Extension handler for the Displacement extension
///
/// This handler provides validation and utility functions for 3MF models
/// that use the Displacement extension (displacement mapping for 3D printing).
///
/// # Example
///
/// ```rust
/// use lib3mf::extensions::DisplacementExtensionHandler;
/// use lib3mf::extension::ExtensionHandler;
///
/// let handler = DisplacementExtensionHandler;
/// assert_eq!(handler.name(), "Displacement");
/// ```
pub struct DisplacementExtensionHandler;

impl ExtensionHandler for DisplacementExtensionHandler {
    fn extension_type(&self) -> Extension {
        Extension::Displacement
    }

    fn validate(&self, model: &Model) -> Result<()> {
        // Call the existing displacement validation from the validator module
        validator::validate_displacement_extension(model)
    }

    fn is_used_in_model(&self, model: &Model) -> bool {
        // Check if extension is required or if any displacement resources or elements are present
        model.required_extensions.contains(&Extension::Displacement)
            || !model.resources.displacement_maps.is_empty()
            || !model.resources.norm_vector_groups.is_empty()
            || !model.resources.disp2d_groups.is_empty()
            || model
                .resources
                .objects
                .iter()
                .any(|obj| obj.displacement_mesh.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        Disp2DCoords, Disp2DGroup, Displacement2D, DisplacementMesh, DisplacementTriangle,
        NormVector, NormVectorGroup, Object, Vertex,
    };

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

        // Empty model should not use displacement extension
        assert!(!handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_with_displacement_map() {
        let handler = DisplacementExtensionHandler;
        let mut model = Model::new();

        // Add a displacement map
        model
            .resources
            .displacement_maps
            .push(Displacement2D::new(1, "/3D/Textures/disp.png".to_string()));

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_with_norm_vector_group() {
        let handler = DisplacementExtensionHandler;
        let mut model = Model::new();

        // Add a norm vector group
        model
            .resources
            .norm_vector_groups
            .push(NormVectorGroup::new(1));

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_with_disp2d_group() {
        let handler = DisplacementExtensionHandler;
        let mut model = Model::new();

        // Add a disp2d group
        model
            .resources
            .disp2d_groups
            .push(Disp2DGroup::new(1, 10, 20, 1.0));

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_with_displacement_mesh() {
        let handler = DisplacementExtensionHandler;
        let mut model = Model::new();

        // Create an object with a displacement mesh
        let mut obj = Object::new(1);
        let mut displacement_mesh = DisplacementMesh::new();

        // Add vertices
        displacement_mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        displacement_mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        displacement_mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));
        displacement_mesh.vertices.push(Vertex::new(0.0, 0.0, 1.0));

        // Add triangles (minimum 4 for a tetrahedron)
        displacement_mesh
            .triangles
            .push(DisplacementTriangle::new(0, 1, 2));
        displacement_mesh
            .triangles
            .push(DisplacementTriangle::new(0, 1, 3));
        displacement_mesh
            .triangles
            .push(DisplacementTriangle::new(0, 2, 3));
        displacement_mesh
            .triangles
            .push(DisplacementTriangle::new(1, 2, 3));

        obj.displacement_mesh = Some(displacement_mesh);
        model.resources.objects.push(obj);

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_validate_empty_model() {
        let handler = DisplacementExtensionHandler;
        let model = Model::new();

        // Empty model should pass validation (no displacement resources to validate)
        assert!(handler.validate(&model).is_ok());
    }

    #[test]
    fn test_validate_with_valid_displacement_resources() {
        let handler = DisplacementExtensionHandler;
        let mut model = Model::new();

        // Add displacement extension to required extensions
        model.required_extensions.push(Extension::Displacement);

        // Add a displacement map with valid path
        model
            .resources
            .displacement_maps
            .push(Displacement2D::new(1, "/3D/Textures/disp.png".to_string()));

        // Add a norm vector group with a valid normalized vector
        let mut norm_group = NormVectorGroup::new(2);
        norm_group.vectors.push(NormVector::new(0.0, 0.0, 1.0));
        model.resources.norm_vector_groups.push(norm_group);

        // Add a disp2d group that references the above resources
        let mut disp_group = Disp2DGroup::new(3, 1, 2, 1.0);
        disp_group.coords.push(Disp2DCoords::new(0.0, 0.0, 0));
        model.resources.disp2d_groups.push(disp_group);

        // This should pass validation
        assert!(handler.validate(&model).is_ok());
    }

    #[test]
    fn test_validate_missing_extension_declaration() {
        let handler = DisplacementExtensionHandler;
        let mut model = Model::new();

        // Add displacement resources but DON'T add extension to required_extensions
        model
            .resources
            .displacement_maps
            .push(Displacement2D::new(1, "/3D/Textures/disp.png".to_string()));

        // This should fail validation
        let result = handler.validate(&model);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("displacement extension is not declared"));
    }

    #[test]
    fn test_validate_invalid_path() {
        let handler = DisplacementExtensionHandler;
        let mut model = Model::new();

        // Add displacement extension to required extensions
        model.required_extensions.push(Extension::Displacement);

        // Add a displacement map with invalid path (not in /3D/Textures/)
        model
            .resources
            .displacement_maps
            .push(Displacement2D::new(1, "/invalid/path.png".to_string()));

        // This should fail validation
        let result = handler.validate(&model);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not in /3D/Textures/"));
    }

    #[test]
    fn test_validate_invalid_reference() {
        let handler = DisplacementExtensionHandler;
        let mut model = Model::new();

        // Add displacement extension to required extensions
        model.required_extensions.push(Extension::Displacement);

        // Add a disp2d group that references non-existent resources
        let disp_group = Disp2DGroup::new(1, 999, 888, 1.0); // Invalid IDs
        model.resources.disp2d_groups.push(disp_group);

        // This should fail validation
        let result = handler.validate(&model);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("non-existent"));
    }

    #[test]
    fn test_default_post_parse() {
        let handler = DisplacementExtensionHandler;
        let mut model = Model::new();

        // Default post_parse should do nothing and return Ok
        assert!(handler.post_parse(&mut model).is_ok());
    }

    #[test]
    fn test_default_pre_write() {
        let handler = DisplacementExtensionHandler;
        let mut model = Model::new();

        // Default pre_write should do nothing and return Ok
        assert!(handler.pre_write(&mut model).is_ok());
    }
}
