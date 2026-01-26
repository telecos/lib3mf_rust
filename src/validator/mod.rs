//! Validation logic for 3MF models
//!
//! This module contains functions to validate 3MF models according to the
//! 3MF Core Specification requirements. Validation ensures that:
//! - All object IDs are unique and positive
//! - Triangle vertex indices reference valid vertices
//! - Triangles are not degenerate (all three vertices must be distinct)
//! - Build items reference existing objects
//! - Material, color group, and base material references are valid

mod beam_lattice;
mod boolean_ops;
mod core;
mod displacement;
mod material;
mod production;
mod slice;

// Re-export public API functions
pub use beam_lattice::validate_beam_lattice;
pub use boolean_ops::validate_boolean_operations;
pub use core::{
    detect_circular_components, validate_build_references, validate_component_properties,
    validate_component_references, validate_mesh_geometry, validate_mesh_manifold,
};
pub use displacement::validate_displacement_extension;
pub use material::{
    get_property_resource_size, validate_color_formats, validate_duplicate_resource_ids,
    validate_material_references, validate_multiproperties_references,
    validate_object_triangle_materials, validate_resource_ordering, validate_texture_paths,
    validate_triangle_properties,
};
pub use production::{
    validate_component_chain, validate_duplicate_uuids, validate_production_extension,
    validate_production_extension_with_config, validate_production_paths,
    validate_production_uuids_required, validate_uuid_formats,
};
pub use slice::{
    validate_planar_transform, validate_slice, validate_slice_extension, validate_slices,
};

// Re-import internal functions from submodules for use within this module
use core::{
    sorted_ids_from_set, validate_dtd_declaration, validate_mesh_volume, validate_object_ids,
    validate_required_extensions, validate_required_structure, validate_thumbnail_format,
    validate_thumbnail_jpeg_colorspace, validate_transform_matrices, validate_vertex_order,
};

use crate::error::{Error, Result};
use crate::model::{Model, ParserConfig};

/// Validate a parsed 3MF model
///
/// Performs comprehensive validation of the model structure, including:
/// - Required model structure (objects and build items)
/// - Object ID uniqueness
/// - Triangle vertex bounds and degeneracy checks
/// - Build item object references
/// - Material, color group, and base material references
/// - Component references and circular dependency detection
/// - Mesh requirements (must have vertices)
///
/// Note: This function uses a default parser config for backward compatibility.
/// For more control, use `validate_model_with_config`.
#[allow(dead_code)] // Public API but may not be used internally
pub fn validate_model(model: &Model) -> Result<()> {
    // Delegate to validate_model_with_config with default config
    validate_model_with_config(model, &ParserConfig::with_all_extensions())
}

/// Validate a parsed 3MF model with custom extension validation
///
/// This function performs the same validation as `validate_model` and additionally
/// invokes any custom validation handlers registered in the parser configuration.
#[allow(dead_code)] // Currently called during parsing; may be exposed publicly in future
pub fn validate_model_with_config(model: &Model, config: &ParserConfig) -> Result<()> {
    // Core validation (always required regardless of extensions)
    validate_required_structure(model)?;
    validate_object_ids(model)?;
    validate_mesh_geometry(model)?;
    validate_build_references(model)?;
    validate_required_extensions(model)?;
    validate_component_references(model)?;
    
    // Production extension validation with config support for backward compatibility
    // This is kept separate because it checks config.supports() for lenient validation
    validate_production_extension_with_config(model, config)?;

    // Extension registry validation (unified approach)
    // This calls validate() on all registered extension handlers, which now handle:
    // - Material extension: material references, texture paths, multiproperties, 
    //   triangle properties, color formats, resource ordering, duplicate resource IDs
    // - Production extension: production extension, production paths, UUID formats,
    //   production UUIDs required, duplicate UUIDs, component chain
    // - Boolean operations extension
    // - Displacement extension
    // - Slice extension
    // - Beam lattice extension
    config.registry().validate_all(model)?;

    // Custom extension validation (legacy pattern - deprecated)
    // NOTE: This callback-based pattern is deprecated in favor of the ExtensionRegistry system.
    // New code should use ExtensionHandler trait and register handlers via config.registry().
    // This is kept for backward compatibility and will be removed in a future major version.
    for ext_info in config.custom_extensions().values() {
        if let Some(validator) = &ext_info.validation_handler {
            validator(model)
                .map_err(|e| Error::InvalidModel(format!("Custom validation failed: {}", e)))?;
        }
    }

    // Additional core validations (not extension-specific)
    validate_transform_matrices(model)?;
    validate_thumbnail_format(model)?;
    validate_mesh_volume(model)?;
    validate_vertex_order(model)?;
    validate_thumbnail_jpeg_colorspace(model)?;
    validate_dtd_declaration(model)?;
    validate_component_properties(model)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::model::{BuildItem, Mesh, Model, Multi, MultiProperties, Object, Triangle, Vertex};
    use crate::validator::*;

    #[test]
    fn test_validate_duplicate_object_ids() {
        let mut model = Model::new();
        model.resources.objects.push(Object::new(1));
        model.resources.objects.push(Object::new(1)); // Duplicate!

        let result = validate_object_ids(&model);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Duplicate object ID"));
    }

    #[test]
    fn test_validate_zero_object_id() {
        let mut model = Model::new();
        model.resources.objects.push(Object::new(0)); // Invalid!

        let result = validate_object_ids(&model);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("positive integer"));
    }

    #[test]
    fn test_validate_degenerate_triangle() {
        let mut model = Model::new();
        let mut object = Object::new(1);
        let mut mesh = Mesh::new();

        // Add vertices
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));

        // Add degenerate triangle (v1 == v2)
        mesh.triangles.push(Triangle::new(0, 0, 2));

        object.mesh = Some(mesh);
        model.resources.objects.push(object);

        let result = validate_mesh_geometry(&model);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("degenerate"));
    }

    #[test]
    fn test_validate_vertex_out_of_bounds() {
        let mut model = Model::new();
        let mut object = Object::new(1);
        let mut mesh = Mesh::new();

        // Add only 2 vertices
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));

        // Add triangle with out-of-bounds vertex (v3 = 5, but only have 2 vertices)
        mesh.triangles.push(Triangle::new(0, 1, 5));

        object.mesh = Some(mesh);
        model.resources.objects.push(object);

        let result = validate_mesh_geometry(&model);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("out of bounds"));
    }

    #[test]
    fn test_validate_build_item_invalid_reference() {
        let mut model = Model::new();
        model.resources.objects.push(Object::new(1));

        // Build item references non-existent object 99
        model.build.items.push(BuildItem::new(99));

        let result = validate_build_references(&model);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("non-existent object"));
    }

    #[test]
    fn test_validate_valid_model() {
        let mut model = Model::new();
        let mut object = Object::new(1);
        let mut mesh = Mesh::new();

        // Add valid vertices
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));

        // Add valid triangle
        mesh.triangles.push(Triangle::new(0, 1, 2));

        object.mesh = Some(mesh);
        model.resources.objects.push(object);

        // Add valid build item
        model.build.items.push(BuildItem::new(1));

        let result = validate_model(&model);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_empty_mesh() {
        let mut model = Model::new();
        let mut object = Object::new(1);
        let mut mesh = Mesh::new();

        // Add triangles but no vertices - this should fail
        mesh.triangles.push(Triangle::new(0, 1, 2));

        object.mesh = Some(mesh);
        model.resources.objects.push(object);

        let result = validate_mesh_geometry(&model);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("triangle"),
            "Error message should mention triangles"
        );
        assert!(
            err_msg.contains("no vertices"),
            "Error message should mention missing vertices"
        );
    }

    #[test]
    fn test_validate_base_material_reference() {
        use crate::model::{BaseMaterial, BaseMaterialGroup};

        let mut model = Model::new();

        // Add a base material group with id=5
        let mut base_group = BaseMaterialGroup::new(5);
        base_group
            .materials
            .push(BaseMaterial::new("Red".to_string(), (255, 0, 0, 255)));
        base_group
            .materials
            .push(BaseMaterial::new("Blue".to_string(), (0, 0, 255, 255)));
        model.resources.base_material_groups.push(base_group);

        // Create an object that references the base material group
        let mut object = Object::new(1);
        object.pid = Some(5);
        object.pindex = Some(0);

        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));

        object.mesh = Some(mesh);
        model.resources.objects.push(object);
        model.build.items.push(BuildItem::new(1));

        // Should pass validation
        let result = validate_model(&model);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_base_material_reference() {
        use crate::model::{BaseMaterial, BaseMaterialGroup};

        let mut model = Model::new();

        // Add a base material group with id=5
        let mut base_group = BaseMaterialGroup::new(5);
        base_group
            .materials
            .push(BaseMaterial::new("Red".to_string(), (255, 0, 0, 255)));
        model.resources.base_material_groups.push(base_group);

        // Create an object that references a non-existent material group id=99
        let mut object = Object::new(1);
        object.pid = Some(99); // Invalid reference!

        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));

        object.mesh = Some(mesh);
        model.resources.objects.push(object);
        model.build.items.push(BuildItem::new(1));

        // Should fail validation
        let result = validate_material_references(&model);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("non-existent"));
    }

    #[test]
    fn test_validate_base_material_pindex_out_of_bounds() {
        use crate::model::{BaseMaterial, BaseMaterialGroup};

        let mut model = Model::new();

        // Add a base material group with only 1 material
        let mut base_group = BaseMaterialGroup::new(5);
        base_group
            .materials
            .push(BaseMaterial::new("Red".to_string(), (255, 0, 0, 255)));
        model.resources.base_material_groups.push(base_group);

        // Create an object with pindex=5 (out of bounds)
        let mut object = Object::new(1);
        object.pid = Some(5);
        object.pindex = Some(5); // Out of bounds! Only index 0 is valid

        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));

        object.mesh = Some(mesh);
        model.resources.objects.push(object);
        model.build.items.push(BuildItem::new(1));

        // Should fail validation
        let result = validate_material_references(&model);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("out of bounds"));
    }

    #[test]
    fn test_validate_basematerialid_valid() {
        use crate::model::{BaseMaterial, BaseMaterialGroup};

        let mut model = Model::new();

        // Add a base material group with ID 5
        let mut base_group = BaseMaterialGroup::new(5);
        base_group.materials.push(BaseMaterial::new(
            "Red Plastic".to_string(),
            (255, 0, 0, 255),
        ));
        model.resources.base_material_groups.push(base_group);

        // Create an object that references the base material group via basematerialid
        let mut object = Object::new(1);
        object.basematerialid = Some(5); // Valid reference

        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));

        object.mesh = Some(mesh);
        model.resources.objects.push(object);
        model.build.items.push(BuildItem::new(1));

        // Should pass validation
        let result = validate_material_references(&model);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_basematerialid_invalid() {
        use crate::model::{BaseMaterial, BaseMaterialGroup};

        let mut model = Model::new();

        // Add a base material group with ID 5
        let mut base_group = BaseMaterialGroup::new(5);
        base_group.materials.push(BaseMaterial::new(
            "Red Plastic".to_string(),
            (255, 0, 0, 255),
        ));
        model.resources.base_material_groups.push(base_group);

        // Create an object that references a non-existent base material group
        let mut object = Object::new(1);
        object.basematerialid = Some(99); // Invalid reference!

        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));

        object.mesh = Some(mesh);
        model.resources.objects.push(object);
        model.build.items.push(BuildItem::new(1));

        // Should fail validation
        let result = validate_material_references(&model);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("non-existent base material group"));
    }

    #[test]
    fn test_validate_component_reference_invalid() {
        use crate::model::Component;

        let mut model = Model::new();

        // Create object 1 with a component referencing non-existent object 99
        let mut object1 = Object::new(1);
        object1.components.push(Component::new(99));

        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));
        object1.mesh = Some(mesh);

        model.resources.objects.push(object1);
        model.build.items.push(BuildItem::new(1));

        // Should fail validation
        let result = validate_component_references(&model);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("non-existent object"));
    }

    #[test]
    fn test_validate_component_circular_dependency() {
        use crate::model::Component;

        let mut model = Model::new();

        // Create object 1 with component referencing object 2
        let mut object1 = Object::new(1);
        object1.components.push(Component::new(2));

        let mut mesh1 = Mesh::new();
        mesh1.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh1.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh1.vertices.push(Vertex::new(0.5, 1.0, 0.0));
        mesh1.triangles.push(Triangle::new(0, 1, 2));
        object1.mesh = Some(mesh1);

        // Create object 2 with component referencing object 1 (circular!)
        let mut object2 = Object::new(2);
        object2.components.push(Component::new(1));

        let mut mesh2 = Mesh::new();
        mesh2.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh2.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh2.vertices.push(Vertex::new(0.5, 1.0, 0.0));
        mesh2.triangles.push(Triangle::new(0, 1, 2));
        object2.mesh = Some(mesh2);

        model.resources.objects.push(object1);
        model.resources.objects.push(object2);
        model.build.items.push(BuildItem::new(1));

        // Should fail validation
        let result = validate_component_references(&model);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Circular component reference"));
    }

    #[test]
    fn test_validate_component_self_reference() {
        use crate::model::Component;

        let mut model = Model::new();

        // Create object 1 with component referencing itself
        let mut object1 = Object::new(1);
        object1.components.push(Component::new(1));

        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));
        object1.mesh = Some(mesh);

        model.resources.objects.push(object1);
        model.build.items.push(BuildItem::new(1));

        // Should fail validation (self-reference is a circular dependency)
        let result = validate_component_references(&model);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Circular component reference"));
    }

    #[test]
    fn test_validate_component_valid() {
        use crate::model::Component;

        let mut model = Model::new();

        // Create base object 2 (no components)
        let mut object2 = Object::new(2);
        let mut mesh2 = Mesh::new();
        mesh2.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh2.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh2.vertices.push(Vertex::new(0.5, 1.0, 0.0));
        mesh2.triangles.push(Triangle::new(0, 1, 2));
        object2.mesh = Some(mesh2);

        // Create object 1 with component referencing object 2
        let mut object1 = Object::new(1);
        object1.components.push(Component::new(2));

        let mut mesh1 = Mesh::new();
        mesh1.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh1.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh1.vertices.push(Vertex::new(0.5, 1.0, 0.0));
        mesh1.triangles.push(Triangle::new(0, 1, 2));
        object1.mesh = Some(mesh1);

        model.resources.objects.push(object1);
        model.resources.objects.push(object2);
        model.build.items.push(BuildItem::new(1));

        // Should pass validation
        let result = validate_component_references(&model);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_multiproperties_reference() {
        use crate::model::{Multi, MultiProperties};

        let mut model = Model::new();

        // Add a multiproperties group with ID 12
        let mut multi_props = MultiProperties {
            id: 12,
            pids: vec![6, 9],
            blendmethods: vec![],
            multis: vec![],
            parse_order: 0,
        };
        multi_props.multis.push(Multi {
            pindices: vec![0, 0],
        });
        model.resources.multi_properties.push(multi_props);

        // Create an object that references the multiproperties group
        let mut object = Object::new(1);
        object.pid = Some(12); // Should reference the multiproperties group
        object.pindex = Some(0);

        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));

        object.mesh = Some(mesh);
        model.resources.objects.push(object);
        model.build.items.push(BuildItem::new(1));

        // Should pass validation (multiproperties is a valid property group)
        let result = validate_material_references(&model);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_texture2d_group_reference() {
        use crate::model::{Tex2Coord, Texture2DGroup};

        let mut model = Model::new();

        // Add a texture2d group with ID 9
        let mut tex_group = Texture2DGroup::new(9, 4);
        tex_group.tex2coords.push(Tex2Coord { u: 0.0, v: 0.0 });
        tex_group.tex2coords.push(Tex2Coord { u: 1.0, v: 1.0 });
        model.resources.texture2d_groups.push(tex_group);

        // Create an object that references the texture2d group
        let mut object = Object::new(1);
        object.pid = Some(9); // Should reference the texture2d group

        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));

        object.mesh = Some(mesh);
        model.resources.objects.push(object);
        model.build.items.push(BuildItem::new(1));

        // Should pass validation (texture2d group is a valid property group)
        let result = validate_material_references(&model);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sliced_object_allows_negative_volume_mesh() {
        use crate::model::SliceStack;

        let mut model = Model::new();

        // Add a slicestack
        let slice_stack = SliceStack::new(1, 0.0);
        model.resources.slice_stacks.push(slice_stack);

        // Create an object with negative volume (inverted mesh) but with slicestackid
        let mut object = Object::new(1);
        object.slicestackid = Some(1); // References the slicestack

        // Create a mesh with inverted triangles (negative volume)
        // This is a simple inverted tetrahedron
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));
        mesh.vertices.push(Vertex::new(5.0, 5.0, 10.0));

        // Deliberately inverted winding order to create negative volume
        mesh.triangles.push(Triangle::new(0, 2, 1)); // Inverted
        mesh.triangles.push(Triangle::new(0, 3, 2)); // Inverted
        mesh.triangles.push(Triangle::new(0, 1, 3)); // Inverted
        mesh.triangles.push(Triangle::new(1, 2, 3)); // Inverted

        object.mesh = Some(mesh);
        model.resources.objects.push(object);
        model.build.items.push(BuildItem::new(1));

        // Should pass validation because object has slicestackid
        let result = validate_mesh_volume(&model);
        assert!(
            result.is_ok(),
            "Sliced object should allow negative volume mesh"
        );
    }

    #[test]
    fn test_non_sliced_object_rejects_negative_volume() {
        let mut model = Model::new();

        // Create an object WITHOUT slicestackid
        let mut object = Object::new(1);

        // Create a box mesh with ALL triangles in inverted winding order
        // Based on the standard box from test_files/core/box.3mf but with reversed winding
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0)); // 0
        mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0)); // 1
        mesh.vertices.push(Vertex::new(10.0, 20.0, 0.0)); // 2
        mesh.vertices.push(Vertex::new(0.0, 20.0, 0.0)); // 3
        mesh.vertices.push(Vertex::new(0.0, 0.0, 30.0)); // 4
        mesh.vertices.push(Vertex::new(10.0, 0.0, 30.0)); // 5
        mesh.vertices.push(Vertex::new(10.0, 20.0, 30.0)); // 6
        mesh.vertices.push(Vertex::new(0.0, 20.0, 30.0)); // 7

        // Correct winding from box.3mf:
        // <triangle v1="3" v2="2" v3="1" />
        // For negative volume, swap the first and third vertex indices: v1="1" v2="2" v3="3"
        // All triangles with INVERTED winding (first and third indices swapped)
        mesh.triangles.push(Triangle::new(1, 2, 3)); // Was (3, 2, 1)
        mesh.triangles.push(Triangle::new(3, 0, 1)); // Was (1, 0, 3)
        mesh.triangles.push(Triangle::new(6, 5, 4)); // Was (4, 5, 6)
        mesh.triangles.push(Triangle::new(4, 7, 6)); // Was (6, 7, 4)
        mesh.triangles.push(Triangle::new(5, 1, 0)); // Was (0, 1, 5)
        mesh.triangles.push(Triangle::new(0, 4, 5)); // Was (5, 4, 0)
        mesh.triangles.push(Triangle::new(6, 2, 1)); // Was (1, 2, 6)
        mesh.triangles.push(Triangle::new(1, 5, 6)); // Was (6, 5, 1)
        mesh.triangles.push(Triangle::new(7, 3, 2)); // Was (2, 3, 7)
        mesh.triangles.push(Triangle::new(2, 6, 7)); // Was (7, 6, 2)
        mesh.triangles.push(Triangle::new(4, 0, 3)); // Was (3, 0, 4)
        mesh.triangles.push(Triangle::new(3, 7, 4)); // Was (4, 7, 3)

        object.mesh = Some(mesh);
        model.resources.objects.push(object);
        model.build.items.push(BuildItem::new(1));

        // Should fail validation for non-sliced object
        let result = validate_mesh_volume(&model);
        assert!(
            result.is_err(),
            "Non-sliced object should reject negative volume mesh"
        );
        assert!(result.unwrap_err().to_string().contains("negative volume"));
    }

    #[test]
    fn test_sliced_object_allows_mirror_transform() {
        use crate::model::SliceStack;

        let mut model = Model::new();

        // Add a slicestack
        let slice_stack = SliceStack::new(1, 0.0);
        model.resources.slice_stacks.push(slice_stack);

        // Create an object with slicestackid
        let mut object = Object::new(1);
        object.slicestackid = Some(1);

        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));

        object.mesh = Some(mesh);
        model.resources.objects.push(object);

        // Add build item with mirror transformation (negative determinant)
        // Transform with -1 scale in X axis (mirror): [-1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0]
        let mut item = BuildItem::new(1);
        item.transform = Some([-1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0]);
        model.build.items.push(item);

        // Should pass validation because object has slicestackid
        let result = validate_transform_matrices(&model);
        assert!(
            result.is_ok(),
            "Sliced object should allow mirror transformation"
        );
    }

    #[test]
    fn test_non_sliced_object_rejects_mirror_transform() {
        let mut model = Model::new();

        // Create an object WITHOUT slicestackid
        let mut object = Object::new(1);

        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));

        object.mesh = Some(mesh);
        model.resources.objects.push(object);

        // Add build item with mirror transformation (negative determinant)
        let mut item = BuildItem::new(1);
        item.transform = Some([-1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0]);
        model.build.items.push(item);

        // Should fail validation for non-sliced object
        let result = validate_transform_matrices(&model);
        assert!(
            result.is_err(),
            "Non-sliced object should reject mirror transformation"
        );
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("negative determinant"));
    }

    #[test]
    fn test_multiproperties_duplicate_colorgroup() {
        use crate::model::ColorGroup;

        let mut model = Model::new();

        // Add a colorgroup
        let mut color_group = ColorGroup::new(10);
        color_group.parse_order = 1;
        color_group.colors.push((255, 0, 0, 255)); // Red
        model.resources.color_groups.push(color_group);

        // Add multiproperties that references the same colorgroup twice
        let mut multi = MultiProperties::new(20, vec![10, 10]); // Duplicate reference to colorgroup 10
        multi.parse_order = 2;
        multi.multis.push(Multi::new(vec![0, 0]));
        model.resources.multi_properties.push(multi);

        // Add object and build item for basic structure
        let mut object = Object::new(1);
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));
        object.mesh = Some(mesh);
        model.resources.objects.push(object);
        model.build.items.push(BuildItem::new(1));

        // Should fail validation (N_XXM_0604_01)
        let result = validate_multiproperties_references(&model);
        assert!(
            result.is_err(),
            "Should reject multiproperties with duplicate colorgroup references"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("colorgroup"));
        assert!(error_msg.contains("multiple times"));
    }

    #[test]
    fn test_multiproperties_basematerials_at_layer_2() {
        use crate::model::{BaseMaterial, BaseMaterialGroup, ColorGroup};

        let mut model = Model::new();

        // Add a basematerials group
        let mut base_mat = BaseMaterialGroup::new(5);
        base_mat.parse_order = 1;
        base_mat
            .materials
            .push(BaseMaterial::new("Steel".to_string(), (128, 128, 128, 255)));
        model.resources.base_material_groups.push(base_mat);

        // Add colorgroups for layers 0 and 1
        let mut cg1 = ColorGroup::new(6);
        cg1.parse_order = 2;
        cg1.colors.push((255, 0, 0, 255));
        model.resources.color_groups.push(cg1);

        let mut cg2 = ColorGroup::new(7);
        cg2.parse_order = 3;
        cg2.colors.push((0, 255, 0, 255));
        model.resources.color_groups.push(cg2);

        // Add multiproperties with basematerials at layer 2 (index 2) - INVALID
        let mut multi = MultiProperties::new(20, vec![6, 7, 5]); // basematerials at index 2
        multi.parse_order = 4;
        multi.multis.push(Multi::new(vec![0, 0, 0]));
        model.resources.multi_properties.push(multi);

        // Add object and build item for basic structure
        let mut object = Object::new(1);
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));
        object.mesh = Some(mesh);
        model.resources.objects.push(object);
        model.build.items.push(BuildItem::new(1));

        // Should fail validation (N_XXM_0604_03)
        let result = validate_multiproperties_references(&model);
        assert!(result.is_err(), "Should reject basematerials at layer >= 2");
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("basematerials"));
        assert!(error_msg.contains("layer"));
    }

    #[test]
    fn test_multiproperties_basematerials_at_layer_1() {
        // Test case N_XXM_0604_03: basematerials at layer 1 (index 1) should be rejected
        // Per spec, basematerials MUST be at layer 0 if included
        use crate::model::{BaseMaterial, BaseMaterialGroup, ColorGroup};

        let mut model = Model::new();

        // Add a colorgroup for layer 0
        let mut cg = ColorGroup::new(6);
        cg.parse_order = 1;
        cg.colors.push((255, 0, 0, 255));
        model.resources.color_groups.push(cg);

        // Add a basematerials group
        let mut base_mat = BaseMaterialGroup::new(1);
        base_mat.parse_order = 2;
        base_mat
            .materials
            .push(BaseMaterial::new("Steel".to_string(), (128, 128, 128, 255)));
        model.resources.base_material_groups.push(base_mat);

        // Add multiproperties with basematerials at layer 1 (index 1) - INVALID
        let mut multi = MultiProperties::new(12, vec![6, 1]); // colorgroup at 0, basematerials at 1
        multi.parse_order = 3;
        multi.multis.push(Multi::new(vec![0, 0]));
        model.resources.multi_properties.push(multi);

        // Add object and build item for basic structure
        let mut object = Object::new(1);
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));
        object.mesh = Some(mesh);
        model.resources.objects.push(object);
        model.build.items.push(BuildItem::new(1));

        // Should fail validation (N_XXM_0604_03)
        let result = validate_multiproperties_references(&model);
        assert!(
            result.is_err(),
            "Should reject basematerials at layer 1 (must be at layer 0)"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("basematerials"));
        assert!(error_msg.contains("layer 1"));
        assert!(error_msg.contains("first element"));
    }

    #[test]
    fn test_multiproperties_two_different_colorgroups() {
        // Test case N_XXM_0604_01: multiple different colorgroups should be rejected
        // Per spec, pids list MUST NOT contain more than one reference to a colorgroup
        use crate::model::ColorGroup;

        let mut model = Model::new();

        // Add two different colorgroups
        let mut cg1 = ColorGroup::new(5);
        cg1.parse_order = 1;
        cg1.colors.push((255, 0, 0, 255));
        model.resources.color_groups.push(cg1);

        let mut cg2 = ColorGroup::new(6);
        cg2.parse_order = 2;
        cg2.colors.push((0, 255, 0, 255));
        model.resources.color_groups.push(cg2);

        // Add multiproperties that references both colorgroups
        let mut multi = MultiProperties::new(12, vec![5, 6]); // Two different colorgroups
        multi.parse_order = 3;
        multi.multis.push(Multi::new(vec![0, 0]));
        model.resources.multi_properties.push(multi);

        // Add object and build item for basic structure
        let mut object = Object::new(1);
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));
        object.mesh = Some(mesh);
        model.resources.objects.push(object);
        model.build.items.push(BuildItem::new(1));

        // Should fail validation (N_XXM_0604_01)
        let result = validate_multiproperties_references(&model);
        assert!(
            result.is_err(),
            "Should reject multiproperties with multiple different colorgroups"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("multiple colorgroups"));
        assert!(error_msg.contains("[5, 6]") || error_msg.contains("[6, 5]"));
    }

    #[test]
    fn test_triangle_material_without_object_default() {
        let mut model = Model::new();

        // Create object WITHOUT default material (no pid)
        let mut object = Object::new(1);
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));

        // Triangle with material property but object has no default
        let mut triangle = Triangle::new(0, 1, 2);
        triangle.p1 = Some(0); // Triangle has material property
        mesh.triangles.push(triangle);

        object.mesh = Some(mesh);
        model.resources.objects.push(object);
        model.build.items.push(BuildItem::new(1));

        // Should fail validation (N_XXM_0601_01)
        let result = validate_triangle_properties(&model);
        assert!(
        result.is_err(),
        "Should reject triangle with per-vertex properties when neither triangle nor object has pid"
    );
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("per-vertex material properties"));
    }

    #[test]
    fn test_forward_reference_texture2dgroup_to_texture2d() {
        use crate::model::{Texture2D, Texture2DGroup};

        let mut model = Model::new();

        // Add texture2dgroup BEFORE texture2d (forward reference)
        let mut tex_group = Texture2DGroup::new(10, 20);
        tex_group.parse_order = 1; // Earlier in parse order
        model.resources.texture2d_groups.push(tex_group);

        // Add texture2d AFTER texture2dgroup
        let mut texture = Texture2D::new(
            20,
            "/3D/Texture/image.png".to_string(),
            "image/png".to_string(),
        );
        texture.parse_order = 2; // Later in parse order
        model.resources.texture2d_resources.push(texture);

        // Add object and build item for basic structure
        let mut object = Object::new(1);
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));
        object.mesh = Some(mesh);
        model.resources.objects.push(object);
        model.build.items.push(BuildItem::new(1));

        // Should fail validation (N_XXM_0606_01)
        let result = validate_resource_ordering(&model);
        assert!(
            result.is_err(),
            "Should reject forward reference from texture2dgroup to texture2d"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Forward reference"));
        assert!(error_msg.contains("texture2d"));
    }

    #[test]
    fn test_forward_reference_multiproperties_to_colorgroup() {
        use crate::model::ColorGroup;

        let mut model = Model::new();

        // Add multiproperties BEFORE colorgroup (forward reference)
        let mut multi = MultiProperties::new(10, vec![20]);
        multi.parse_order = 1; // Earlier in parse order
        multi.multis.push(Multi::new(vec![0]));
        model.resources.multi_properties.push(multi);

        // Add colorgroup AFTER multiproperties
        let mut color_group = ColorGroup::new(20);
        color_group.parse_order = 2; // Later in parse order
        color_group.colors.push((255, 0, 0, 255));
        model.resources.color_groups.push(color_group);

        // Add object and build item for basic structure
        let mut object = Object::new(1);
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));
        object.mesh = Some(mesh);
        model.resources.objects.push(object);
        model.build.items.push(BuildItem::new(1));

        // Should fail validation (N_XXM_0606_03)
        let result = validate_resource_ordering(&model);
        assert!(
            result.is_err(),
            "Should reject forward reference from multiproperties to colorgroup"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Forward reference"));
        assert!(error_msg.contains("colorgroup"));
    }

    #[test]
    fn test_texture_path_with_backslash() {
        use crate::model::Texture2D;

        let mut model = Model::new();

        // Add texture with backslash in path (invalid per OPC spec)
        let mut texture = Texture2D::new(
            10,
            "/3D\\Texture\\image.png".to_string(),
            "image/png".to_string(),
        );
        texture.parse_order = 1;
        model.resources.texture2d_resources.push(texture);

        // Add object and build item for basic structure
        let mut object = Object::new(1);
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));
        object.mesh = Some(mesh);
        model.resources.objects.push(object);
        model.build.items.push(BuildItem::new(1));

        // Should fail validation (N_XXM_0610_01)
        let result = validate_texture_paths(&model);
        assert!(
            result.is_err(),
            "Should reject texture path with backslashes"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("backslash"));
    }

    #[test]
    fn test_texture_path_empty() {
        use crate::model::Texture2D;

        let mut model = Model::new();

        // Add texture with empty path
        let mut texture = Texture2D::new(10, "".to_string(), "image/png".to_string());
        texture.parse_order = 1;
        model.resources.texture2d_resources.push(texture);

        // Add object and build item for basic structure
        let mut object = Object::new(1);
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));
        object.mesh = Some(mesh);
        model.resources.objects.push(object);
        model.build.items.push(BuildItem::new(1));

        // Should fail validation (N_XXM_0610_01)
        let result = validate_texture_paths(&model);
        assert!(result.is_err(), "Should reject empty texture path");
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("empty"));
    }
}
