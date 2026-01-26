//! Slice extension handler implementation

use crate::error::Result;
use crate::extension::ExtensionHandler;
use crate::model::{Extension, Model};
use crate::validator::{validate_slice_extension, validate_slices};

/// Handler for the Slice extension
///
/// The Slice extension provides support for sliced 3D models, allowing models to be
/// represented as a stack of 2D cross-sections at different Z heights. This is commonly
/// used for additive manufacturing workflows.
///
/// # Validation
///
/// This handler validates:
/// - Slice stack structure and Z-coordinate ordering
/// - Polygon validity (closed, proper vertex references)
/// - Planar transform constraints for objects with slicestacks
/// - SliceRef path requirements and external file references
///
/// # Example
///
/// ```ignore
/// use lib3mf::extension::ExtensionRegistry;
/// use lib3mf::extensions::slice::SliceExtensionHandler;
/// use std::sync::Arc;
///
/// let mut registry = ExtensionRegistry::new();
/// registry.register(Arc::new(SliceExtensionHandler));
/// ```
pub struct SliceExtensionHandler;

impl ExtensionHandler for SliceExtensionHandler {
    fn extension_type(&self) -> Extension {
        Extension::Slice
    }

    fn validate(&self, model: &Model) -> Result<()> {
        // Run all slice-specific validations
        validate_slices(model)?;
        validate_slice_extension(model)?;
        Ok(())
    }

    fn is_used_in_model(&self, model: &Model) -> bool {
        // Check if model has any slice stacks or objects referencing slicestacks
        if !model.resources.slice_stacks.is_empty() {
            return true;
        }

        // Check if any objects reference slicestacks
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
    use crate::model::{Object, Slice, SlicePolygon, SliceSegment, SliceStack, Vertex2D};

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
    fn test_is_used_in_model_with_slice_stacks() {
        let handler = SliceExtensionHandler;
        let mut model = Model::new();

        // Initially no slices
        assert!(!handler.is_used_in_model(&model));

        // Add a slice stack
        let slice_stack = SliceStack::new(1, 0.0);
        model.resources.slice_stacks.push(slice_stack);

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_with_object_reference() {
        let handler = SliceExtensionHandler;
        let mut model = Model::new();

        // Initially no slices
        assert!(!handler.is_used_in_model(&model));

        // Add an object that references a slicestack
        let mut object = Object::new(1);
        object.slicestackid = Some(1);
        model.resources.objects.push(object);

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_validate_empty_model() {
        let handler = SliceExtensionHandler;
        let model = Model::new();

        // Empty model should validate successfully
        assert!(handler.validate(&model).is_ok());
    }

    #[test]
    fn test_validate_valid_slice_stack() {
        let handler = SliceExtensionHandler;
        let mut model = Model::new();

        // Create a valid slice stack with one slice
        let mut slice_stack = SliceStack::new(1, 0.0);

        // Create a slice with a valid triangle polygon
        let mut slice = Slice::new(1.0);
        slice.vertices.push(Vertex2D::new(0.0, 0.0));
        slice.vertices.push(Vertex2D::new(1.0, 0.0));
        slice.vertices.push(Vertex2D::new(0.5, 1.0));

        let mut polygon = SlicePolygon::new(0);
        polygon.segments.push(SliceSegment::new(1));
        polygon.segments.push(SliceSegment::new(2));
        polygon.segments.push(SliceSegment::new(0)); // Close the polygon

        slice.polygons.push(polygon);
        slice_stack.slices.push(slice);

        model.resources.slice_stacks.push(slice_stack);

        // Should validate successfully
        assert!(handler.validate(&model).is_ok());
    }

    #[test]
    fn test_validate_invalid_ztop_below_zbottom() {
        let handler = SliceExtensionHandler;
        let mut model = Model::new();

        // Create a slice stack with zbottom > slice ztop (invalid)
        let mut slice_stack = SliceStack::new(1, 5.0); // zbottom = 5.0

        let slice = Slice::new(1.0); // ztop = 1.0 < zbottom (invalid!)
        slice_stack.slices.push(slice);

        model.resources.slice_stacks.push(slice_stack);

        // Should fail validation
        assert!(handler.validate(&model).is_err());
    }

    #[test]
    fn test_validate_non_increasing_ztop() {
        let handler = SliceExtensionHandler;
        let mut model = Model::new();

        let mut slice_stack = SliceStack::new(1, 0.0);

        // Add slices with non-strictly increasing ztop values
        slice_stack.slices.push(Slice::new(1.0));
        slice_stack.slices.push(Slice::new(1.0)); // Same as previous (invalid!)

        model.resources.slice_stacks.push(slice_stack);

        // Should fail validation
        assert!(handler.validate(&model).is_err());
    }

    #[test]
    fn test_validate_invalid_polygon_not_closed() {
        let handler = SliceExtensionHandler;
        let mut model = Model::new();

        let mut slice_stack = SliceStack::new(1, 0.0);

        let mut slice = Slice::new(1.0);
        slice.vertices.push(Vertex2D::new(0.0, 0.0));
        slice.vertices.push(Vertex2D::new(1.0, 0.0));
        slice.vertices.push(Vertex2D::new(0.5, 1.0));

        let mut polygon = SlicePolygon::new(0);
        polygon.segments.push(SliceSegment::new(1));
        polygon.segments.push(SliceSegment::new(2));
        // Missing segment back to startv=0 (not closed!)

        slice.polygons.push(polygon);
        slice_stack.slices.push(slice);

        model.resources.slice_stacks.push(slice_stack);

        // Should fail validation
        assert!(handler.validate(&model).is_err());
    }

    #[test]
    fn test_post_parse_default() {
        let handler = SliceExtensionHandler;
        let mut model = Model::new();

        // Default implementation should do nothing and succeed
        assert!(handler.post_parse(&mut model).is_ok());
    }

    #[test]
    fn test_pre_write_default() {
        let handler = SliceExtensionHandler;
        let mut model = Model::new();

        // Default implementation should do nothing and succeed
        assert!(handler.pre_write(&mut model).is_ok());
    }
}
