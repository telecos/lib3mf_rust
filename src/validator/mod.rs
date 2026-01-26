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
    get_property_resource_size, validate_material_references, validate_multiproperties_references,
    validate_object_triangle_materials, validate_texture_paths, validate_triangle_properties,
};
pub use production::{
    validate_production_extension, validate_production_extension_with_config,
    validate_production_paths, validate_production_uuids_required,
};
pub use slice::{
    validate_planar_transform, validate_slice, validate_slice_extension, validate_slices,
};

use crate::error::{Error, Result};
use crate::mesh_ops;
use crate::model::{Extension, Model, ParserConfig};
use std::collections::HashSet;

/// Helper function to convert a HashSet of IDs to a sorted Vec for error messages
pub(crate) fn sorted_ids_from_set(ids: &HashSet<usize>) -> Vec<usize> {
    let mut sorted: Vec<usize> = ids.iter().copied().collect();
    sorted.sort();
    sorted
}

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
    // Standard validation
    validate_required_structure(model)?;
    validate_object_ids(model)?;
    validate_mesh_geometry(model)?;
    validate_build_references(model)?;
    validate_material_references(model)?;
    validate_required_extensions(model)?;
    validate_boolean_operations(model)?;
    validate_component_references(model)?;
    validate_production_extension_with_config(model, config)?;
    validate_displacement_extension(model)?;
    validate_slices(model)?;
    validate_slice_extension(model)?;
    validate_beam_lattice(model)?;

    // Custom extension validation
    for ext_info in config.custom_extensions().values() {
        if let Some(validator) = &ext_info.validation_handler {
            validator(model)
                .map_err(|e| Error::InvalidModel(format!("Custom validation failed: {}", e)))?;
        }
    }

    // Additional Material and Production validations
    validate_texture_paths(model)?;
    validate_color_formats(model)?;
    validate_uuid_formats(model)?;
    validate_production_paths(model)?;
    validate_transform_matrices(model)?;
    validate_resource_ordering(model)?;
    validate_duplicate_resource_ids(model)?;
    validate_multiproperties_references(model)?;
    validate_triangle_properties(model)?;
    validate_production_uuids_required(model, config)?;
    validate_thumbnail_format(model)?;
    validate_mesh_volume(model)?;
    validate_vertex_order(model)?;
    validate_thumbnail_jpeg_colorspace(model)?;
    validate_dtd_declaration(model)?;
    validate_component_properties(model)?;
    validate_duplicate_uuids(model)?;
    validate_component_chain(model)?;

    Ok(())
}

pub(crate) fn validate_required_structure(model: &Model) -> Result<()> {
    // Check if we have objects in resources OR build items with external paths
    let has_local_objects =
        !model.resources.objects.is_empty() || !model.resources.slice_stacks.is_empty();
    let has_external_objects = model
        .build
        .items
        .iter()
        .any(|item| item.production_path.is_some());

    // Model must contain at least one object (either local or external)
    if !has_local_objects && !has_external_objects {
        return Err(Error::InvalidModel(
            "Model must contain at least one object. \
             A valid 3MF file requires either:\n\
             - At least one <object> element within the <resources> section, OR\n\
             - At least one build <item> with a p:path attribute (Production extension) \
             referencing an external file.\n\
             Check that your 3MF file has proper model content."
                .to_string(),
        ));
    }

    // Build section must contain at least one item for main model files
    // However, external slice/resource files may have empty build sections
    // We identify these by: having slice stacks but either no objects or empty build
    let is_external_file = !model.resources.slice_stacks.is_empty()
        && (model.resources.objects.is_empty() || model.build.items.is_empty());

    if model.build.items.is_empty() && !is_external_file {
        return Err(Error::InvalidModel(
            "Build section must contain at least one item. \
             A valid 3MF file requires at least one <item> element within the <build> section. \
             The build section specifies which objects should be printed."
                .to_string(),
        ));
    }

    Ok(())
}

pub(crate) fn validate_required_extensions(model: &Model) -> Result<()> {
    let mut uses_boolean_ops = false;
    let mut objects_with_boolean_and_material_props = Vec::new();

    // Check if model uses boolean operations
    for object in &model.resources.objects {
        if object.boolean_shape.is_some() {
            uses_boolean_ops = true;

            // Per Boolean Operations spec: "producers MUST NOT assign pid or pindex
            // attributes to objects that contain booleanshape"
            if object.pid.is_some() || object.pindex.is_some() {
                objects_with_boolean_and_material_props.push(object.id);
            }
        }
    }

    // Validate Boolean Operations extension requirements
    if uses_boolean_ops {
        // Check if Boolean Operations extension is in required extensions
        let has_bo_extension = model
            .required_extensions
            .contains(&Extension::BooleanOperations);

        if !has_bo_extension {
            return Err(Error::InvalidModel(
                "Model uses boolean operations (<booleanshape>) but does not declare \
                 the Boolean Operations extension in requiredextensions.\n\
                 Per 3MF Boolean Operations spec, you must add 'bo' to the requiredextensions \
                 attribute in the <model> element when using boolean operations.\n\
                 Example: requiredextensions=\"bo\""
                    .to_string(),
            ));
        }
    }

    // Check for objects with both booleanshape and material properties
    if !objects_with_boolean_and_material_props.is_empty() {
        return Err(Error::InvalidModel(format!(
            "Objects {:?} contain both <booleanshape> and pid/pindex attributes.\n\
             Per 3MF Boolean Operations spec section 2 (Object Resources):\n\
             'producers MUST NOT assign pid or pindex attributes to objects that contain booleanshape.'\n\
             Remove the pid/pindex attributes from these objects or remove the boolean shape.",
            objects_with_boolean_and_material_props
        )));
    }

    Ok(())
}

pub(crate) fn validate_object_ids(model: &Model) -> Result<()> {
    let mut seen_ids = HashSet::new();

    for object in &model.resources.objects {
        // Object IDs must be positive (non-zero)
        if object.id == 0 {
            return Err(Error::InvalidModel(
                "Object ID must be a positive integer (greater than 0). \
                 Per the 3MF specification, object IDs must be positive integers. \
                 Found object with ID = 0, which is invalid."
                    .to_string(),
            ));
        }

        // Check for duplicate IDs
        if !seen_ids.insert(object.id) {
            return Err(Error::InvalidModel(format!(
                "Duplicate object ID found: {}. \
                 Each object in the resources section must have a unique ID attribute. \
                 Check your model for multiple objects with the same ID value.",
                object.id
            )));
        }
    }

    Ok(())
}

/// Validate mesh geometry (vertex indices and triangle degeneracy)

pub(crate) fn validate_color_formats(model: &Model) -> Result<()> {
    // Colors are already validated during parsing (stored as (u8, u8, u8, u8) tuples)
    // This function is a placeholder for any additional color validation needs

    // Validate that color groups have at least one color
    for color_group in &model.resources.color_groups {
        if color_group.colors.is_empty() {
            return Err(Error::InvalidModel(format!(
                "Color group {}: Must contain at least one color.\n\
                 A color group without colors is invalid.",
                color_group.id
            )));
        }
    }

    Ok(())
}

/// Validate UUID format per RFC 4122
///
/// UUIDs must follow the format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
/// where x is a hexadecimal digit (0-9, a-f, A-F).

pub(crate) fn validate_uuid_formats(model: &Model) -> Result<()> {
    // Helper function to validate a single UUID
    let validate_uuid = |uuid: &str, context: &str| -> Result<()> {
        // UUID format: 8-4-4-4-12 hexadecimal digits separated by hyphens
        // Example: 550e8400-e29b-41d4-a716-446655440000

        // Check length (36 characters including hyphens)
        if uuid.len() != 36 {
            return Err(Error::InvalidModel(format!(
                "{}: Invalid UUID '{}' - must be 36 characters in format xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
                context, uuid
            )));
        }

        // Check hyphen positions (at indices 8, 13, 18, 23)
        if uuid.chars().nth(8) != Some('-')
            || uuid.chars().nth(13) != Some('-')
            || uuid.chars().nth(18) != Some('-')
            || uuid.chars().nth(23) != Some('-')
        {
            return Err(Error::InvalidModel(format!(
                "{}: Invalid UUID '{}' - hyphens must be at positions 8, 13, 18, and 23",
                context, uuid
            )));
        }

        // Check that all other characters are hexadecimal digits
        for (idx, ch) in uuid.chars().enumerate() {
            if idx == 8 || idx == 13 || idx == 18 || idx == 23 {
                continue; // Skip hyphens
            }
            if !ch.is_ascii_hexdigit() {
                return Err(Error::InvalidModel(format!(
                    "{}: Invalid UUID '{}' - character '{}' at position {} is not a hexadecimal digit",
                    context, uuid, ch, idx
                )));
            }
        }

        Ok(())
    };

    // Validate build UUID
    if let Some(ref uuid) = model.build.production_uuid {
        validate_uuid(uuid, "Build")?;
    }

    // Validate build item UUIDs
    for (idx, item) in model.build.items.iter().enumerate() {
        if let Some(ref uuid) = item.production_uuid {
            validate_uuid(uuid, &format!("Build item {}", idx))?;
        }
    }

    // Validate object UUIDs
    for object in &model.resources.objects {
        if let Some(ref prod_info) = object.production {
            if let Some(ref uuid) = prod_info.uuid {
                validate_uuid(uuid, &format!("Object {}", object.id))?;
            }
        }

        // Validate component UUIDs
        for (idx, component) in object.components.iter().enumerate() {
            if let Some(ref prod_info) = component.production {
                if let Some(ref uuid) = prod_info.uuid {
                    validate_uuid(uuid, &format!("Object {}, Component {}", object.id, idx))?;
                }
            }
        }
    }

    Ok(())
}

/// Validate production paths don't reference OPC internal files
///
/// Per 3MF Production Extension spec, production paths must not reference
/// OPC package internal files like /.rels or /[Content_Types].xml.

pub(crate) fn validate_transform_matrices(model: &Model) -> Result<()> {
    // Build a set of object IDs that have slicestacks
    let sliced_object_ids: std::collections::HashSet<usize> = model
        .resources
        .objects
        .iter()
        .filter_map(|obj| obj.slicestackid.map(|_| obj.id))
        .collect();

    for (idx, item) in model.build.items.iter().enumerate() {
        // Skip validation for build items that reference sliced objects
        // Per 3MF Slice Extension spec, sliced objects have different transform
        // restrictions (planar transforms) which are validated in validate_slice_extension
        if sliced_object_ids.contains(&item.objectid) {
            continue;
        }

        if let Some(ref transform) = item.transform {
            // Calculate the determinant of the 3x3 rotation/scale portion
            // Transform is stored as 12 values: [m00 m01 m02 m10 m11 m12 m20 m21 m22 tx ty tz]
            let m00 = transform[0];
            let m01 = transform[1];
            let m02 = transform[2];
            let m10 = transform[3];
            let m11 = transform[4];
            let m12 = transform[5];
            let m20 = transform[6];
            let m21 = transform[7];
            let m22 = transform[8];

            // Determinant = m00*(m11*m22 - m12*m21) - m01*(m10*m22 - m12*m20) + m02*(m10*m21 - m11*m20)
            let det = m00 * (m11 * m22 - m12 * m21) - m01 * (m10 * m22 - m12 * m20)
                + m02 * (m10 * m21 - m11 * m20);

            // Check for zero determinant (singular matrix) - DPX 3314_07
            const DET_EPSILON: f64 = 1e-10;
            if det.abs() < DET_EPSILON {
                return Err(Error::InvalidModel(format!(
                    "Build item {}: Transform matrix has zero determinant ({:.6}), indicating a singular (non-invertible) transformation.\n\
                     Transform: [{} {} {} {} {} {} {} {} {} {} {} {}]\n\
                     Hint: Check that the transform matrix is valid and non-degenerate.",
                    idx,
                    det,
                    transform[0],
                    transform[1],
                    transform[2],
                    transform[3],
                    transform[4],
                    transform[5],
                    transform[6],
                    transform[7],
                    transform[8],
                    transform[9],
                    transform[10],
                    transform[11]
                )));
            }

            if det < 0.0 {
                return Err(Error::InvalidModel(format!(
                    "Build item {}: Transform matrix has negative determinant ({:.6}).\n\
                     Per 3MF spec, transforms with negative determinants (mirror transformations) \
                     are not allowed as they would invert the object's orientation.\n\
                     Transform: [{} {} {} {} {} {} {} {} {} {} {} {}]",
                    idx,
                    det,
                    transform[0],
                    transform[1],
                    transform[2],
                    transform[3],
                    transform[4],
                    transform[5],
                    transform[6],
                    transform[7],
                    transform[8],
                    transform[9],
                    transform[10],
                    transform[11]
                )));
            }
        }
    }

    Ok(())
}

/// Validate resource ordering
///
/// Per 3MF spec, resources must be defined before they are referenced.
/// For example, texture2d must be defined before texture2dgroup that references it.
/// This validation checks for forward references using parse order.

pub(crate) fn validate_resource_ordering(model: &Model) -> Result<()> {
    // N_XXM_0606_01: Texture2dgroup must not reference texture2d that appears later in XML
    for tex_group in &model.resources.texture2d_groups {
        if let Some(tex2d) = model
            .resources
            .texture2d_resources
            .iter()
            .find(|t| t.id == tex_group.texid)
        {
            if tex_group.parse_order < tex2d.parse_order {
                return Err(Error::InvalidModel(format!(
                    "Texture2DGroup {}: Forward reference to texture2d {} which appears later in the resources.\n\
                     Per 3MF Material Extension spec, texture2d resources must be defined before \
                     texture2dgroups that reference them.\n\
                     Move the texture2d element before the texture2dgroup element in the resources section.",
                    tex_group.id, tex_group.texid
                )));
            }
        } else {
            return Err(Error::InvalidModel(format!(
                "Texture2DGroup {}: References texture2d with ID {} which is not defined.\n\
                 Per 3MF spec, texture2d resources must be defined before texture2dgroups that reference them.\n\
                 Ensure texture2d with ID {} exists in the <resources> section before this texture2dgroup.",
                tex_group.id, tex_group.texid, tex_group.texid
            )));
        }
    }

    // N_XXM_0606_02, N_XXM_0606_03, N_XXM_0607_01: Multiproperties must not have forward references
    for multi_props in &model.resources.multi_properties {
        for &pid in &multi_props.pids {
            // Check if PID references a texture2dgroup
            if let Some(tex_group) = model
                .resources
                .texture2d_groups
                .iter()
                .find(|t| t.id == pid)
            {
                if multi_props.parse_order < tex_group.parse_order {
                    return Err(Error::InvalidModel(format!(
                        "MultiProperties {}: Forward reference to texture2dgroup {} which appears later in the resources.\n\
                         Per 3MF Material Extension spec, property resources must be defined before \
                         multiproperties that reference them.\n\
                         Move the texture2dgroup element before the multiproperties element in the resources section.",
                        multi_props.id, pid
                    )));
                }
            }

            // Check if PID references a colorgroup
            if let Some(color_group) = model.resources.color_groups.iter().find(|c| c.id == pid) {
                if multi_props.parse_order < color_group.parse_order {
                    return Err(Error::InvalidModel(format!(
                        "MultiProperties {}: Forward reference to colorgroup {} which appears later in the resources.\n\
                         Per 3MF Material Extension spec, property resources must be defined before \
                         multiproperties that reference them.\n\
                         Move the colorgroup element before the multiproperties element in the resources section.",
                        multi_props.id, pid
                    )));
                }
            }

            // Check if PID references a basematerials group
            if let Some(base_mat) = model
                .resources
                .base_material_groups
                .iter()
                .find(|b| b.id == pid)
            {
                if multi_props.parse_order < base_mat.parse_order {
                    return Err(Error::InvalidModel(format!(
                        "MultiProperties {}: Forward reference to basematerials group {} which appears later in the resources.\n\
                         Per 3MF Material Extension spec, property resources must be defined before \
                         multiproperties that reference them.\n\
                         Move the basematerials element before the multiproperties element in the resources section.",
                        multi_props.id, pid
                    )));
                }
            }

            // Check if PID references a compositematerials group
            if let Some(composite) = model
                .resources
                .composite_materials
                .iter()
                .find(|c| c.id == pid)
            {
                if multi_props.parse_order < composite.parse_order {
                    return Err(Error::InvalidModel(format!(
                        "MultiProperties {}: Forward reference to compositematerials group {} which appears later in the resources.\n\
                         Per 3MF Material Extension spec, property resources must be defined before \
                         multiproperties that reference them.\n\
                         Move the compositematerials element before the multiproperties element in the resources section.",
                        multi_props.id, pid
                    )));
                }
            }
        }
    }

    // N_XPM_0607_01: Objects must not be intermingled with property resources
    // Per 3MF spec, the resources section should have a consistent ordering:
    // either all property resources first then all objects, or vice versa.
    // Intermingling objects between property resources is invalid.

    // Get parse orders for all property resources
    let mut property_resource_orders = Vec::new();

    for tex2d in &model.resources.texture2d_resources {
        property_resource_orders.push(("Texture2D", tex2d.id, tex2d.parse_order));
    }
    for tex_group in &model.resources.texture2d_groups {
        property_resource_orders.push(("Texture2DGroup", tex_group.id, tex_group.parse_order));
    }
    for color_group in &model.resources.color_groups {
        property_resource_orders.push(("ColorGroup", color_group.id, color_group.parse_order));
    }
    for base_mat in &model.resources.base_material_groups {
        property_resource_orders.push(("BaseMaterials", base_mat.id, base_mat.parse_order));
    }
    for composite in &model.resources.composite_materials {
        property_resource_orders.push(("CompositeMaterials", composite.id, composite.parse_order));
    }
    for multi_props in &model.resources.multi_properties {
        property_resource_orders.push(("MultiProperties", multi_props.id, multi_props.parse_order));
    }

    // Get parse orders for all objects
    let mut object_orders = Vec::new();
    for obj in &model.resources.objects {
        object_orders.push((obj.id, obj.parse_order));
    }

    // Check if there are objects intermingled with property resources
    // This is only an issue if we have both objects and property resources
    if !property_resource_orders.is_empty() && !object_orders.is_empty() {
        // Find min and max parse order for property resources
        let min_prop_order = property_resource_orders
            .iter()
            .map(|(_, _, order)| order)
            .min()
            .unwrap();
        let max_prop_order = property_resource_orders
            .iter()
            .map(|(_, _, order)| order)
            .max()
            .unwrap();

        // If property resources and objects have overlapping ranges, they're intermingled
        // Valid: all properties [0-10], all objects [11-20] OR all objects [0-10], all properties [11-20]
        // Invalid: properties [0-5], objects [6-10], properties [11-15] (intermingled)

        // Check if there's an object between two property resources
        for (prop_type, prop_id, prop_order) in &property_resource_orders {
            for (obj_id, obj_order) in &object_orders {
                // If an object appears between the min and max property resource orders,
                // and there are property resources both before and after it
                if *obj_order > *min_prop_order && *obj_order < *max_prop_order {
                    // Find a property resource that comes after this object
                    if let Some((later_prop_type, later_prop_id, later_prop_order)) =
                        property_resource_orders
                            .iter()
                            .find(|(_, _, order)| *order > *obj_order)
                    {
                        return Err(Error::InvalidModel(format!(
                            "Invalid resource ordering: Object {} appears between property resources.\n\
                             The object is at position {}, between {} {} (position {}) and {} {} (position {}).\n\
                             Per 3MF specification, objects must not be intermingled with property resources.\n\
                             Either place all objects after all property resources, or all property resources after all objects.",
                            obj_id, obj_order, prop_type, prop_id, prop_order,
                            later_prop_type, later_prop_id, later_prop_order
                        )));
                    }
                }
            }
        }
    }

    Ok(())
}

/// Validate that resource IDs are unique within their namespaces
///
/// Per 3MF spec:
/// - Object IDs must be unique among objects
/// - Property resource IDs (basematerials, colorgroups, texture2d, texture2dgroups,
///   compositematerials, multiproperties) must be unique among property resources
/// - Objects and property resources have SEPARATE ID namespaces and can reuse IDs

pub(crate) fn validate_duplicate_resource_ids(model: &Model) -> Result<()> {
    // Check object IDs for duplicates (separate namespace)
    let mut seen_object_ids: HashSet<usize> = HashSet::new();
    for obj in &model.resources.objects {
        if !seen_object_ids.insert(obj.id) {
            return Err(Error::InvalidModel(format!(
                "Duplicate object ID {}: Multiple objects use the same ID.\n\
                 Per 3MF spec, each object must have a unique ID within the objects namespace.\n\
                 Change the ID to a unique value.",
                obj.id
            )));
        }
    }

    // Check property resource IDs for duplicates (separate namespace from objects)
    // Property resources include: basematerials, colorgroups, texture2d, texture2dgroups,
    // compositematerials, and multiproperties
    let mut seen_property_ids: HashSet<usize> = HashSet::new();

    // Helper to check and add property resource ID
    let mut check_property_id = |id: usize, resource_type: &str| -> Result<()> {
        if !seen_property_ids.insert(id) {
            return Err(Error::InvalidModel(format!(
                "Duplicate property resource ID {}: {} resource uses an ID that is already in use by another property resource.\n\
                 Per 3MF spec, property resource IDs must be unique among all property resources \
                 (basematerials, colorgroups, texture2d, texture2dgroups, compositematerials, multiproperties).\n\
                 Note: Objects have a separate ID namespace and can reuse property resource IDs.",
                id, resource_type
            )));
        }
        Ok(())
    };

    // Check all property resource types
    for base_mat in &model.resources.base_material_groups {
        check_property_id(base_mat.id, "BaseMaterials")?;
    }

    for color_group in &model.resources.color_groups {
        check_property_id(color_group.id, "ColorGroup")?;
    }

    for texture in &model.resources.texture2d_resources {
        check_property_id(texture.id, "Texture2D")?;
    }

    for tex_group in &model.resources.texture2d_groups {
        check_property_id(tex_group.id, "Texture2DGroup")?;
    }

    for composite in &model.resources.composite_materials {
        check_property_id(composite.id, "CompositeMaterials")?;
    }

    for multi in &model.resources.multi_properties {
        check_property_id(multi.id, "MultiProperties")?;
    }

    // Check slice stack IDs (Slice Extension)
    // Slicestacks are extension resources but share the property resource ID namespace
    for slice_stack in &model.resources.slice_stacks {
        check_property_id(slice_stack.id, "SliceStack")?;
    }

    Ok(())
}

/// Validate multiproperties references
///
/// Per 3MF Material Extension spec:
/// - All PIDs in multiproperties.pids must reference valid resources
/// - MultiProperties cannot reference the same basematerials group multiple times in pids

pub(crate) fn validate_mesh_volume(model: &Model) -> Result<()> {
    for object in &model.resources.objects {
        // Skip mesh volume validation for sliced objects
        // Per 3MF Slice Extension spec, when an object has a slicestack,
        // the mesh is not used for printing (slices are used instead),
        // so mesh orientation doesn't matter
        if object.slicestackid.is_some() {
            continue;
        }

        if let Some(ref mesh) = object.mesh {
            // Use signed volume to detect inverted meshes
            let volume = mesh_ops::compute_mesh_signed_volume(mesh)?;

            // Use small epsilon for floating-point comparison
            const EPSILON: f64 = 1e-10;
            if volume < -EPSILON {
                return Err(Error::InvalidModel(format!(
                    "Object {}: Mesh has negative volume ({}), indicating inverted or incorrectly oriented triangles",
                    object.id, volume
                )));
            }
        }
    }
    Ok(())
}

/// N_XPX_0418_01: Validate triangle vertex order (normals should point outwards)
///
/// Validates that mesh triangles have correct vertex ordering (counter-clockwise
/// when viewed from outside) by checking the signed volume of each mesh.
///
/// A mesh with inward-pointing normals (reversed vertex order) will have negative
/// signed volume. This validation catches meshes where all or most triangles are
/// inverted.
///
/// Note: This check may not catch partially inverted meshes or complex non-convex
/// geometries where some triangles are legitimately oriented differently.
/// N_XPX_0418_01: Validate triangle vertex order (normals should point outwards)
///
/// **Note: This validation is intentionally disabled.**
///
/// Detecting reversed vertex order reliably requires sophisticated mesh analysis
/// algorithms that are computationally expensive and have reliability issues with
/// certain mesh geometries (e.g., non-convex shapes, complex topology). The simple
/// heuristic of checking if normals point away from the centroid fails for many
/// valid meshes and can cause false positives.
///
/// A proper implementation would require:
/// - Ray casting or winding number algorithms
/// - Topological mesh analysis
/// - Consideration of non-manifold geometries
///
/// For now, we rely on other validators like volume calculation to catch some
/// cases of inverted meshes. Additionally, build item transforms with negative
/// determinants (which would invert normals) are rejected by validate_transform_matrices().

pub(crate) fn validate_vertex_order(_model: &Model) -> Result<()> {
    Ok(())
}

/// N_XPX_0419_01: Validate JPEG thumbnail colorspace (must be RGB, not CMYK)
///
/// **Note: Partial validation implemented in OPC layer.**
///
/// JPEG CMYK validation is performed in `opc::Package::get_thumbnail_metadata()`
/// where the actual thumbnail file data is available. This placeholder exists
/// for documentation and to maintain the validation function signature.

pub(crate) fn validate_thumbnail_jpeg_colorspace(_model: &Model) -> Result<()> {
    Ok(())
}

/// N_XPX_0420_01: Validate no DTD declaration in XML (security risk)
///
/// **Note: Validation implemented in parser.**
///
/// DTD validation is handled during XML parsing in `parser::parse_model_xml()`
/// where the parser rejects `Event::DocType` to prevent XXE (XML External Entity)
/// attacks. This placeholder exists for documentation and to maintain the
/// validation function signature.

pub(crate) fn validate_dtd_declaration(_model: &Model) -> Result<()> {
    Ok(())
}

/// N_XPX_0424_01: Validate objects with components don't have pid/pindex attributes

pub(crate) fn validate_duplicate_uuids(model: &Model) -> Result<()> {
    let mut uuids = std::collections::HashSet::new();

    // Check build UUID
    if let Some(ref uuid) = model.build.production_uuid {
        if !uuids.insert(uuid.clone()) {
            return Err(Error::InvalidModel(format!(
                "Duplicate UUID '{}' found in build",
                uuid
            )));
        }
    }

    // Check build item UUIDs
    for (idx, item) in model.build.items.iter().enumerate() {
        if let Some(ref uuid) = item.production_uuid {
            if !uuids.insert(uuid.clone()) {
                return Err(Error::InvalidModel(format!(
                    "Duplicate UUID '{}' found in build item {}",
                    uuid, idx
                )));
            }
        }
    }

    // Check object UUIDs
    for object in &model.resources.objects {
        if let Some(ref production) = object.production {
            if let Some(ref uuid) = production.uuid {
                if !uuids.insert(uuid.clone()) {
                    return Err(Error::InvalidModel(format!(
                        "Duplicate UUID '{}' found on object {}",
                        uuid, object.id
                    )));
                }
            }
        }

        // Check component UUIDs within each object
        for (comp_idx, component) in object.components.iter().enumerate() {
            if let Some(ref production) = component.production {
                if let Some(ref uuid) = production.uuid {
                    if !uuids.insert(uuid.clone()) {
                        return Err(Error::InvalidModel(format!(
                            "Duplicate UUID '{}' found in object {} component {}",
                            uuid, object.id, comp_idx
                        )));
                    }
                }
            }
        }
    }
    Ok(())
}

/// N_XPX_0803_01: Validate no component reference chains across multiple model parts
///
/// **Note: This validation is intentionally disabled.**
///
/// Detecting component reference chains requires parsing and analyzing external
/// model files referenced via `p:path`. Since the parser only loads the root model
/// file, we cannot reliably detect multi-level chains.
///
/// A full implementation would require:
/// 1. Loading all referenced external model files
/// 2. Building a dependency graph across files
/// 3. Detecting cycles or chains longer than allowed depth
///
/// This is beyond the scope of single-file validation and would require
/// significant architectural changes to support multi-file analysis.

pub(crate) fn validate_component_chain(_model: &Model) -> Result<()> {
    // N_XPM_0803_01: Component reference chain validation
    //
    // The validation for components with p:path referencing local objects
    // is complex and requires more investigation of the 3MF Production Extension spec.
    // The current understanding is insufficient to implement this correctly.
    Ok(())
}

/// Validate thumbnail format
///
/// Per 3MF spec, thumbnails must be PNG or JPEG format, and JPEG must be RGB (not CMYK).
/// Note: Object.has_thumbnail_attribute is a boolean that tracks if thumbnail was present,
/// but the actual path is not stored (deprecated attribute).

pub(crate) fn validate_thumbnail_format(_model: &Model) -> Result<()> {
    // Thumbnail validation is limited because the thumbnail path is not stored in the model
    // The parser only tracks whether the attribute was present via has_thumbnail_attribute
    // Full validation would require parsing the thumbnail file itself

    // For now, this is a placeholder for future thumbnail validation
    // The parser already handles the thumbnail attribute appropriately

    Ok(())
}

#[cfg(test)]
mod tests {

    #[cfg(test)]
    mod tests {
        use crate::model::{
            BuildItem, Mesh, Model, Multi, MultiProperties, Object, Triangle, Vertex,
        };
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
}
