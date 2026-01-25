//! Validation logic for 3MF models
//!
//! This module contains functions to validate 3MF models according to the
//! 3MF Core Specification requirements. Validation ensures that:
//! - All object IDs are unique and positive
//! - Triangle vertex indices reference valid vertices
//! - Triangles are not degenerate (all three vertices must be distinct)
//! - Build items reference existing objects
//! - Material, color group, and base material references are valid

use crate::error::{Error, Result};
use crate::mesh_ops;
use crate::model::{Extension, Model, ObjectType, ParserConfig};
use std::collections::{HashMap, HashSet};

/// Helper function to convert a HashSet of IDs to a sorted Vec for error messages
fn sorted_ids_from_set(ids: &HashSet<usize>) -> Vec<usize> {
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

/// Validate that the model has required structure
///
/// Per 3MF Core spec, a valid model must have:
/// - At least one object in resources OR at least one build item with p:path (external reference)
/// - At least one build item
fn validate_required_structure(model: &Model) -> Result<()> {
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

/// Validate that required extensions are declared when using extension-specific features
///
/// Per 3MF spec, when a model uses features from an extension, that extension
/// must be listed in the requiredextensions attribute (unless the extension is optional).
///
/// Specifically:
/// - Boolean Operations extension must be declared when using booleanshape elements
/// - Objects with pid/pindex must NOT also have booleanshape (per Boolean Ops spec)
fn validate_required_extensions(model: &Model) -> Result<()> {
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

/// Validate that all object IDs are unique and positive
fn validate_object_ids(model: &Model) -> Result<()> {
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
fn validate_mesh_geometry(model: &Model) -> Result<()> {
    for object in &model.resources.objects {
        if let Some(ref mesh) = object.mesh {
            // If mesh has triangles, it must have vertices
            // Note: Meshes with vertices but no triangles can be valid for extensions
            // like beam lattice, so we don't require triangles to be present
            if !mesh.triangles.is_empty() && mesh.vertices.is_empty() {
                return Err(Error::InvalidModel(format!(
                    "Object {}: Mesh has {} triangle(s) but no vertices. \
                     A mesh with triangles must also have vertex data. \
                     Check that the <vertices> element contains <vertex> elements.",
                    object.id,
                    mesh.triangles.len()
                )));
            }

            let num_vertices = mesh.vertices.len();

            for (tri_idx, triangle) in mesh.triangles.iter().enumerate() {
                // Validate vertex indices are within bounds
                if triangle.v1 >= num_vertices {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Triangle {} vertex v1={} is out of bounds (mesh has {} vertices, valid indices: 0-{}). \
                         Vertex indices must reference valid vertices in the mesh. \
                         Check that all triangle vertex indices are less than the vertex count.",
                        object.id, tri_idx, triangle.v1, num_vertices, num_vertices - 1
                    )));
                }
                if triangle.v2 >= num_vertices {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Triangle {} vertex v2={} is out of bounds (mesh has {} vertices, valid indices: 0-{}). \
                         Vertex indices must reference valid vertices in the mesh. \
                         Check that all triangle vertex indices are less than the vertex count.",
                        object.id, tri_idx, triangle.v2, num_vertices, num_vertices - 1
                    )));
                }
                if triangle.v3 >= num_vertices {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Triangle {} vertex v3={} is out of bounds (mesh has {} vertices, valid indices: 0-{}). \
                         Vertex indices must reference valid vertices in the mesh. \
                         Check that all triangle vertex indices are less than the vertex count.",
                        object.id, tri_idx, triangle.v3, num_vertices, num_vertices - 1
                    )));
                }

                // Check for degenerate triangles (two or more vertices are the same)
                if triangle.v1 == triangle.v2
                    || triangle.v2 == triangle.v3
                    || triangle.v1 == triangle.v3
                {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Triangle {} is degenerate (v1={}, v2={}, v3={}). \
                         All three vertices of a triangle must be distinct. \
                         Degenerate triangles with repeated vertices are not allowed in 3MF models.",
                        object.id, tri_idx, triangle.v1, triangle.v2, triangle.v3
                    )));
                }
            }

            // Validate mesh manifold topology - each edge should be shared by at most 2 triangles
            // An edge shared by more than 2 triangles is non-manifold
            if mesh.triangles.len() >= 2 {
                validate_mesh_manifold(object.id, mesh)?;
            }
        }
    }

    Ok(())
}

/// Validate that the mesh forms a valid manifold (no edges shared by more than 2 triangles)
fn validate_mesh_manifold(object_id: usize, mesh: &crate::model::Mesh) -> Result<()> {
    use std::collections::HashMap;

    // Count how many times each edge appears
    // Edge is represented as (min_vertex, max_vertex) to be direction-independent
    // Pre-allocate capacity: each triangle has 3 edges, but adjacent triangles share edges.
    // For typical manifold meshes, we expect roughly 1.5 edges per triangle.
    // We use a conservative estimate of 2 edges per triangle to avoid reallocation.
    let mut edge_count: HashMap<(usize, usize), usize> =
        HashMap::with_capacity(mesh.triangles.len() * 2);

    for triangle in &mesh.triangles {
        // Add the three edges of this triangle
        let edges = [
            (triangle.v1.min(triangle.v2), triangle.v1.max(triangle.v2)),
            (triangle.v2.min(triangle.v3), triangle.v2.max(triangle.v3)),
            (triangle.v3.min(triangle.v1), triangle.v3.max(triangle.v1)),
        ];

        for edge in &edges {
            *edge_count.entry(*edge).or_insert(0) += 1;
        }
    }

    // Check if any edge is shared by more than 2 triangles (non-manifold)
    for (edge, count) in edge_count {
        if count > 2 {
            return Err(Error::InvalidModel(format!(
                "Object {}: Non-manifold edge (vertices {}-{}) is shared by {} triangles (maximum 2 allowed). \
                 Manifold meshes require each edge to be shared by at most 2 triangles. \
                 This is often caused by T-junctions or overlapping faces. \
                 Use mesh repair tools to fix non-manifold geometry.",
                object_id, edge.0, edge.1, count
            )));
        }
    }

    Ok(())
}

/// Validate that all build items reference existing objects
fn validate_build_references(model: &Model) -> Result<()> {
    // Collect all valid object IDs
    let valid_object_ids: HashSet<usize> =
        model.resources.objects.iter().map(|obj| obj.id).collect();

    // Check each build item references a valid object
    for (item_idx, item) in model.build.items.iter().enumerate() {
        // Skip validation for build items that reference external files (Production extension)
        // When a build item has a p:path attribute, the referenced object is in an external
        // file (potentially encrypted in Secure Content scenarios) and doesn't need to exist
        // in the current model's resources
        if item.production_path.is_some() {
            continue;
        }

        if !valid_object_ids.contains(&item.objectid) {
            return Err(Error::InvalidModel(format!(
                "Build item {} references non-existent object ID: {}. \
                 All build items must reference objects defined in the resources section. \
                 Available object IDs: {:?}",
                item_idx, item.objectid, valid_object_ids
            )));
        }
    }

    Ok(())
}

/// Validate material, color group, and base material references
fn validate_material_references(model: &Model) -> Result<()> {
    // Validate that all property group IDs are unique across all property group types
    // Property groups include: color groups, base material groups, multiproperties,
    // texture2d groups, and composite materials
    let mut seen_property_group_ids: HashMap<usize, String> = HashMap::new();

    // Check color group IDs
    for colorgroup in &model.resources.color_groups {
        if let Some(existing_type) =
            seen_property_group_ids.insert(colorgroup.id, "colorgroup".to_string())
        {
            return Err(Error::InvalidModel(format!(
                "Duplicate resource ID: {}. \
                 This ID is used by both a {} and a colorgroup. \
                 Each resource must have a unique id attribute. \
                 Check your material definitions for duplicate IDs.",
                colorgroup.id, existing_type
            )));
        }
    }

    // Check base material group IDs
    for basematerialgroup in &model.resources.base_material_groups {
        if let Some(existing_type) =
            seen_property_group_ids.insert(basematerialgroup.id, "basematerials".to_string())
        {
            return Err(Error::InvalidModel(format!(
                "Duplicate resource ID: {}. \
                 This ID is used by both a {} and a basematerials group. \
                 Each resource must have a unique id attribute. \
                 Check your material definitions for duplicate IDs.",
                basematerialgroup.id, existing_type
            )));
        }
    }

    // Check multiproperties IDs
    for multiprop in &model.resources.multi_properties {
        if let Some(existing_type) =
            seen_property_group_ids.insert(multiprop.id, "multiproperties".to_string())
        {
            return Err(Error::InvalidModel(format!(
                "Duplicate resource ID: {}. \
                 This ID is used by both a {} and a multiproperties group. \
                 Each resource must have a unique id attribute. \
                 Check your material definitions for duplicate IDs.",
                multiprop.id, existing_type
            )));
        }
    }

    // Check texture2d group IDs
    for tex2dgroup in &model.resources.texture2d_groups {
        if let Some(existing_type) =
            seen_property_group_ids.insert(tex2dgroup.id, "texture2dgroup".to_string())
        {
            return Err(Error::InvalidModel(format!(
                "Duplicate resource ID: {}. \
                 This ID is used by both a {} and a texture2dgroup. \
                 Each resource must have a unique id attribute. \
                 Check your material definitions for duplicate IDs.",
                tex2dgroup.id, existing_type
            )));
        }
    }

    // Check composite materials IDs
    for composite in &model.resources.composite_materials {
        if let Some(existing_type) =
            seen_property_group_ids.insert(composite.id, "compositematerials".to_string())
        {
            return Err(Error::InvalidModel(format!(
                "Duplicate resource ID: {}. \
                 This ID is used by both a {} and a compositematerials group. \
                 Each resource must have a unique id attribute. \
                 Check your material definitions for duplicate IDs.",
                composite.id, existing_type
            )));
        }
    }

    // Keep separate base material IDs set for basematerialid validation
    let valid_basematerial_ids: HashSet<usize> = model
        .resources
        .base_material_groups
        .iter()
        .map(|bg| bg.id)
        .collect();

    // Validate multiproperties: each multi element's pindices must be valid for the referenced property groups
    for multiprop in &model.resources.multi_properties {
        for (multi_idx, multi) in multiprop.multis.iter().enumerate() {
            // Validate each pindex against the corresponding property group
            // Note: pindices.len() can be less than pids.len() - unspecified indices default to 0
            for (layer_idx, (&pid, &pindex)) in
                multiprop.pids.iter().zip(multi.pindices.iter()).enumerate()
            {
                // Check if it's a color group
                if let Some(colorgroup) =
                    model.resources.color_groups.iter().find(|cg| cg.id == pid)
                {
                    if pindex >= colorgroup.colors.len() {
                        let max_index = colorgroup.colors.len().saturating_sub(1);
                        return Err(Error::InvalidModel(format!(
                            "MultiProperties group {}: Multi element {} layer {} references pindex {} which is out of bounds.\n\
                             Color group {} has {} colors (valid indices: 0-{}).\n\
                             Hint: Each pindex in a multi element must be less than the number of items in the corresponding property group.",
                            multiprop.id,
                            multi_idx,
                            layer_idx,
                            pindex,
                            pid,
                            colorgroup.colors.len(),
                            max_index
                        )));
                    }
                }
                // Check if it's a base material group
                else if let Some(basematerialgroup) = model
                    .resources
                    .base_material_groups
                    .iter()
                    .find(|bg| bg.id == pid)
                {
                    if pindex >= basematerialgroup.materials.len() {
                        let max_index = basematerialgroup.materials.len().saturating_sub(1);
                        return Err(Error::InvalidModel(format!(
                            "MultiProperties group {}: Multi element {} layer {} references pindex {} which is out of bounds.\n\
                             Base material group {} has {} materials (valid indices: 0-{}).\n\
                             Hint: Each pindex in a multi element must be less than the number of items in the corresponding property group.",
                            multiprop.id,
                            multi_idx,
                            layer_idx,
                            pindex,
                            pid,
                            basematerialgroup.materials.len(),
                            max_index
                        )));
                    }
                }
                // Check if it's a texture2d group
                else if let Some(tex2dgroup) = model
                    .resources
                    .texture2d_groups
                    .iter()
                    .find(|tg| tg.id == pid)
                {
                    if pindex >= tex2dgroup.tex2coords.len() {
                        let max_index = tex2dgroup.tex2coords.len().saturating_sub(1);
                        return Err(Error::InvalidModel(format!(
                            "MultiProperties group {}: Multi element {} layer {} references pindex {} which is out of bounds.\n\
                             Texture2D group {} has {} texture coordinates (valid indices: 0-{}).\n\
                             Hint: Each pindex in a multi element must be less than the number of items in the corresponding property group.",
                            multiprop.id,
                            multi_idx,
                            layer_idx,
                            pindex,
                            pid,
                            tex2dgroup.tex2coords.len(),
                            max_index
                        )));
                    }
                }
                // Check if it's a composite materials group
                else if let Some(composite) = model
                    .resources
                    .composite_materials
                    .iter()
                    .find(|cm| cm.id == pid)
                {
                    if pindex >= composite.composites.len() {
                        let max_index = composite.composites.len().saturating_sub(1);
                        return Err(Error::InvalidModel(format!(
                            "MultiProperties group {}: Multi element {} layer {} references pindex {} which is out of bounds.\n\
                             Composite materials group {} has {} composite elements (valid indices: 0-{}).\n\
                             Hint: Each pindex in a multi element must be less than the number of items in the corresponding property group.",
                            multiprop.id,
                            multi_idx,
                            layer_idx,
                            pindex,
                            pid,
                            composite.composites.len(),
                            max_index
                        )));
                    }
                }
                // If the pid is another multiproperties, we don't need to validate here
                // as nested multiproperties would be validated separately
            }
        }
    }

    for object in &model.resources.objects {
        if let Some(pid) = object.pid {
            // If object has a pid, it should reference a valid property group
            // Only validate if there are property groups defined, otherwise pid might be unused
            if !seen_property_group_ids.is_empty() && !seen_property_group_ids.contains_key(&pid) {
                let available_ids: Vec<usize> = {
                    let mut ids: Vec<usize> = seen_property_group_ids.keys().copied().collect();
                    ids.sort();
                    ids
                };
                return Err(Error::InvalidModel(format!(
                    "Object {} references non-existent property group ID: {}.\n\
                     Available property group IDs: {:?}\n\
                     Hint: Check that all referenced property groups are defined in the <resources> section.",
                    object.id, pid, available_ids
                )));
            }
        }

        // Validate basematerialid references
        if let Some(basematerialid) = object.basematerialid {
            // basematerialid should reference a valid base material group
            if !valid_basematerial_ids.contains(&basematerialid) {
                let available_ids = sorted_ids_from_set(&valid_basematerial_ids);
                return Err(Error::InvalidModel(format!(
                    "Object {} references non-existent base material group ID: {}.\n\
                     Available base material group IDs: {:?}\n\
                     Hint: Check that a basematerials group with this ID exists in the <resources> section.",
                    object.id, basematerialid, available_ids
                )));
            }
        }

        // Validate object pindex references for color groups
        if let Some(obj_pid) = object.pid {
            if let Some(colorgroup) = model
                .resources
                .color_groups
                .iter()
                .find(|cg| cg.id == obj_pid)
            {
                // Validate object-level pindex
                if let Some(pindex) = object.pindex {
                    if pindex >= colorgroup.colors.len() {
                        let max_index = colorgroup.colors.len().saturating_sub(1);
                        return Err(Error::InvalidModel(format!(
                            "Object {}: pindex {} is out of bounds.\n\
                             Color group {} has {} colors (valid indices: 0-{}).\n\
                             Hint: pindex must be less than the number of colors in the color group.",
                            object.id,
                            pindex,
                            obj_pid,
                            colorgroup.colors.len(),
                            max_index
                        )));
                    }
                }
            }
            // Validate object pindex references for base material groups
            else if let Some(basematerialgroup) = model
                .resources
                .base_material_groups
                .iter()
                .find(|bg| bg.id == obj_pid)
            {
                // Validate object-level pindex
                if let Some(pindex) = object.pindex {
                    if pindex >= basematerialgroup.materials.len() {
                        let max_index = basematerialgroup.materials.len().saturating_sub(1);
                        return Err(Error::InvalidModel(format!(
                            "Object {}: pindex {} is out of bounds.\n\
                             Base material group {} has {} materials (valid indices: 0-{}).\n\
                             Hint: pindex must be less than the number of materials in the base material group.",
                            object.id,
                            pindex,
                            obj_pid,
                            basematerialgroup.materials.len(),
                            max_index
                        )));
                    }
                }
            }
            // Validate object pindex references for texture2d groups
            else if let Some(tex2dgroup) = model
                .resources
                .texture2d_groups
                .iter()
                .find(|tg| tg.id == obj_pid)
            {
                // Validate object-level pindex
                if let Some(pindex) = object.pindex {
                    if pindex >= tex2dgroup.tex2coords.len() {
                        let max_index = tex2dgroup.tex2coords.len().saturating_sub(1);
                        return Err(Error::InvalidModel(format!(
                            "Object {}: pindex {} is out of bounds.\n\
                             Texture2D group {} has {} texture coordinates (valid indices: 0-{}).\n\
                             Hint: pindex must be less than the number of texture coordinates in the texture2d group.",
                            object.id,
                            pindex,
                            obj_pid,
                            tex2dgroup.tex2coords.len(),
                            max_index
                        )));
                    }
                }
            }
            // Validate object pindex references for multiproperties
            else if let Some(multiprop) = model
                .resources
                .multi_properties
                .iter()
                .find(|mp| mp.id == obj_pid)
            {
                // Validate object-level pindex
                if let Some(pindex) = object.pindex {
                    if pindex >= multiprop.multis.len() {
                        let max_index = multiprop.multis.len().saturating_sub(1);
                        return Err(Error::InvalidModel(format!(
                            "Object {}: pindex {} is out of bounds.\n\
                             MultiProperties group {} has {} multi elements (valid indices: 0-{}).\n\
                             Hint: pindex must be less than the number of multi elements in the multiproperties group.",
                            object.id,
                            pindex,
                            obj_pid,
                            multiprop.multis.len(),
                            max_index
                        )));
                    }
                }
            }
            // Validate object pindex references for composite materials
            else if let Some(composite) = model
                .resources
                .composite_materials
                .iter()
                .find(|cm| cm.id == obj_pid)
            {
                // Validate object-level pindex
                if let Some(pindex) = object.pindex {
                    if pindex >= composite.composites.len() {
                        let max_index = composite.composites.len().saturating_sub(1);
                        return Err(Error::InvalidModel(format!(
                            "Object {}: pindex {} is out of bounds.\n\
                             Composite materials group {} has {} composite elements (valid indices: 0-{}).\n\
                             Hint: pindex must be less than the number of composite elements in the composite materials group.",
                            object.id,
                            pindex,
                            obj_pid,
                            composite.composites.len(),
                            max_index
                        )));
                    }
                }
            }
        }

        // Validate triangle property index references for color groups and base materials
        if let Some(ref mesh) = object.mesh {
            for (tri_idx, triangle) in mesh.triangles.iter().enumerate() {
                // Determine which color group or base material to use for validation
                let pid_to_check = triangle.pid.or(object.pid);

                if let Some(pid) = pid_to_check {
                    // Check if it's a color group
                    if let Some(colorgroup) =
                        model.resources.color_groups.iter().find(|cg| cg.id == pid)
                    {
                        let num_colors = colorgroup.colors.len();

                        // Validate triangle-level pindex
                        if let Some(pindex) = triangle.pindex {
                            if pindex >= num_colors {
                                let max_index = num_colors.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} pindex {} is out of bounds.\n\
                                     Color group {} has {} colors (valid indices: 0-{}).\n\
                                     Hint: pindex must be less than the number of colors in the color group.",
                                    object.id, tri_idx, pindex, pid, num_colors, max_index
                                )));
                            }
                        }

                        // Validate per-vertex property indices (p1, p2, p3)
                        if let Some(p1) = triangle.p1 {
                            if p1 >= num_colors {
                                let max_index = num_colors.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p1 {} is out of bounds.\n\
                                     Color group {} has {} colors (valid indices: 0-{}).\n\
                                     Hint: p1 must be less than the number of colors in the color group.",
                                    object.id, tri_idx, p1, pid, num_colors, max_index
                                )));
                            }
                        }

                        if let Some(p2) = triangle.p2 {
                            if p2 >= num_colors {
                                let max_index = num_colors.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p2 {} is out of bounds.\n\
                                     Color group {} has {} colors (valid indices: 0-{}).\n\
                                     Hint: p2 must be less than the number of colors in the color group.",
                                    object.id, tri_idx, p2, pid, num_colors, max_index
                                )));
                            }
                        }

                        if let Some(p3) = triangle.p3 {
                            if p3 >= num_colors {
                                let max_index = num_colors.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p3 {} is out of bounds.\n\
                                     Color group {} has {} colors (valid indices: 0-{}).\n\
                                     Hint: p3 must be less than the number of colors in the color group.",
                                    object.id, tri_idx, p3, pid, num_colors, max_index
                                )));
                            }
                        }
                    }
                    // Check if it's a base material group
                    else if let Some(basematerialgroup) = model
                        .resources
                        .base_material_groups
                        .iter()
                        .find(|bg| bg.id == pid)
                    {
                        let num_materials = basematerialgroup.materials.len();

                        // Validate triangle-level pindex
                        if let Some(pindex) = triangle.pindex {
                            if pindex >= num_materials {
                                let max_index = num_materials.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} pindex {} is out of bounds.\n\
                                     Base material group {} has {} materials (valid indices: 0-{}).\n\
                                     Hint: pindex must be less than the number of materials in the base material group.",
                                    object.id, tri_idx, pindex, pid, num_materials, max_index
                                )));
                            }
                        }

                        // Validate per-vertex property indices (p1, p2, p3)
                        if let Some(p1) = triangle.p1 {
                            if p1 >= num_materials {
                                let max_index = num_materials.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p1 {} is out of bounds.\n\
                                     Base material group {} has {} materials (valid indices: 0-{}).\n\
                                     Hint: p1 must be less than the number of materials in the base material group.",
                                    object.id, tri_idx, p1, pid, num_materials, max_index
                                )));
                            }
                        }

                        if let Some(p2) = triangle.p2 {
                            if p2 >= num_materials {
                                let max_index = num_materials.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p2 {} is out of bounds.\n\
                                     Base material group {} has {} materials (valid indices: 0-{}).\n\
                                     Hint: p2 must be less than the number of materials in the base material group.",
                                    object.id, tri_idx, p2, pid, num_materials, max_index
                                )));
                            }
                        }

                        if let Some(p3) = triangle.p3 {
                            if p3 >= num_materials {
                                let max_index = num_materials.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p3 {} is out of bounds.\n\
                                     Base material group {} has {} materials (valid indices: 0-{}).\n\
                                     Hint: p3 must be less than the number of materials in the base material group.",
                                    object.id, tri_idx, p3, pid, num_materials, max_index
                                )));
                            }
                        }
                    }
                    // Check if it's a texture2d group
                    else if let Some(tex2dgroup) = model
                        .resources
                        .texture2d_groups
                        .iter()
                        .find(|tg| tg.id == pid)
                    {
                        let num_coords = tex2dgroup.tex2coords.len();

                        // Validate triangle-level pindex
                        if let Some(pindex) = triangle.pindex {
                            if pindex >= num_coords {
                                let max_index = num_coords.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} pindex {} is out of bounds.\n\
                                     Texture2D group {} has {} texture coordinates (valid indices: 0-{}).\n\
                                     Hint: pindex must be less than the number of texture coordinates in the texture2d group.",
                                    object.id, tri_idx, pindex, pid, num_coords, max_index
                                )));
                            }
                        }

                        // Validate per-vertex property indices (p1, p2, p3)
                        if let Some(p1) = triangle.p1 {
                            if p1 >= num_coords {
                                let max_index = num_coords.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p1 {} is out of bounds.\n\
                                     Texture2D group {} has {} texture coordinates (valid indices: 0-{}).\n\
                                     Hint: p1 must be less than the number of texture coordinates in the texture2d group.",
                                    object.id, tri_idx, p1, pid, num_coords, max_index
                                )));
                            }
                        }

                        if let Some(p2) = triangle.p2 {
                            if p2 >= num_coords {
                                let max_index = num_coords.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p2 {} is out of bounds.\n\
                                     Texture2D group {} has {} texture coordinates (valid indices: 0-{}).\n\
                                     Hint: p2 must be less than the number of texture coordinates in the texture2d group.",
                                    object.id, tri_idx, p2, pid, num_coords, max_index
                                )));
                            }
                        }

                        if let Some(p3) = triangle.p3 {
                            if p3 >= num_coords {
                                let max_index = num_coords.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p3 {} is out of bounds.\n\
                                     Texture2D group {} has {} texture coordinates (valid indices: 0-{}).\n\
                                     Hint: p3 must be less than the number of texture coordinates in the texture2d group.",
                                    object.id, tri_idx, p3, pid, num_coords, max_index
                                )));
                            }
                        }
                    }
                    // Check if it's a multiproperties group
                    else if let Some(multiprop) = model
                        .resources
                        .multi_properties
                        .iter()
                        .find(|mp| mp.id == pid)
                    {
                        let num_multis = multiprop.multis.len();

                        // Validate triangle-level pindex
                        if let Some(pindex) = triangle.pindex {
                            if pindex >= num_multis {
                                let max_index = num_multis.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} pindex {} is out of bounds.\n\
                                     MultiProperties group {} has {} multi elements (valid indices: 0-{}).\n\
                                     Hint: pindex must be less than the number of multi elements in the multiproperties group.",
                                    object.id, tri_idx, pindex, pid, num_multis, max_index
                                )));
                            }
                        }

                        // Validate per-vertex property indices (p1, p2, p3)
                        if let Some(p1) = triangle.p1 {
                            if p1 >= num_multis {
                                let max_index = num_multis.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p1 {} is out of bounds.\n\
                                     MultiProperties group {} has {} multi elements (valid indices: 0-{}).\n\
                                     Hint: p1 must be less than the number of multi elements in the multiproperties group.",
                                    object.id, tri_idx, p1, pid, num_multis, max_index
                                )));
                            }
                        }

                        if let Some(p2) = triangle.p2 {
                            if p2 >= num_multis {
                                let max_index = num_multis.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p2 {} is out of bounds.\n\
                                     MultiProperties group {} has {} multi elements (valid indices: 0-{}).\n\
                                     Hint: p2 must be less than the number of multi elements in the multiproperties group.",
                                    object.id, tri_idx, p2, pid, num_multis, max_index
                                )));
                            }
                        }

                        if let Some(p3) = triangle.p3 {
                            if p3 >= num_multis {
                                let max_index = num_multis.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p3 {} is out of bounds.\n\
                                     MultiProperties group {} has {} multi elements (valid indices: 0-{}).\n\
                                     Hint: p3 must be less than the number of multi elements in the multiproperties group.",
                                    object.id, tri_idx, p3, pid, num_multis, max_index
                                )));
                            }
                        }
                    }
                    // Check if it's a composite materials group
                    else if let Some(composite) = model
                        .resources
                        .composite_materials
                        .iter()
                        .find(|cm| cm.id == pid)
                    {
                        let num_composites = composite.composites.len();

                        // Validate triangle-level pindex
                        if let Some(pindex) = triangle.pindex {
                            if pindex >= num_composites {
                                let max_index = num_composites.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} pindex {} is out of bounds.\n\
                                     Composite materials group {} has {} composite elements (valid indices: 0-{}).\n\
                                     Hint: pindex must be less than the number of composite elements in the composite materials group.",
                                    object.id, tri_idx, pindex, pid, num_composites, max_index
                                )));
                            }
                        }

                        // Validate per-vertex property indices (p1, p2, p3)
                        if let Some(p1) = triangle.p1 {
                            if p1 >= num_composites {
                                let max_index = num_composites.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p1 {} is out of bounds.\n\
                                     Composite materials group {} has {} composite elements (valid indices: 0-{}).\n\
                                     Hint: p1 must be less than the number of composite elements in the composite materials group.",
                                    object.id, tri_idx, p1, pid, num_composites, max_index
                                )));
                            }
                        }

                        if let Some(p2) = triangle.p2 {
                            if p2 >= num_composites {
                                let max_index = num_composites.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p2 {} is out of bounds.\n\
                                     Composite materials group {} has {} composite elements (valid indices: 0-{}).\n\
                                     Hint: p2 must be less than the number of composite elements in the composite materials group.",
                                    object.id, tri_idx, p2, pid, num_composites, max_index
                                )));
                            }
                        }

                        if let Some(p3) = triangle.p3 {
                            if p3 >= num_composites {
                                let max_index = num_composites.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p3 {} is out of bounds.\n\
                                     Composite materials group {} has {} composite elements (valid indices: 0-{}).\n\
                                     Hint: p3 must be less than the number of composite elements in the composite materials group.",
                                    object.id, tri_idx, p3, pid, num_composites, max_index
                                )));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Validate boolean operation references
///
/// Per 3MF Boolean Operations Extension spec:
/// - Objects referenced in boolean operations exist (unless they're external via path attribute)
/// - Objects don't have more than one booleanshape element (checked during parsing)
/// - Boolean operand objects exist (unless they're external via path attribute)
/// - Base object and operand objects must be of type "model" (not support, solidsupport, etc.)
/// - Referenced objects must be defined before the object containing the booleanshape (forward reference rule)
/// - Base object must define a shape (mesh or booleanshape), not components
/// - Operand objects must be triangle meshes only
fn validate_boolean_operations(model: &Model) -> Result<()> {
    // Build a set of valid object IDs for quick lookup
    let valid_object_ids: HashSet<usize> = model.resources.objects.iter().map(|o| o.id).collect();

    // Build a map of object ID to its index in the objects vector (for forward reference check)
    let object_indices: HashMap<usize, usize> = model
        .resources
        .objects
        .iter()
        .enumerate()
        .map(|(idx, obj)| (obj.id, idx))
        .collect();

    // Build a map of object ID to the object itself for type checking
    let object_map: HashMap<usize, &crate::model::Object> = model
        .resources
        .objects
        .iter()
        .map(|obj| (obj.id, obj))
        .collect();

    for (current_idx, object) in model.resources.objects.iter().enumerate() {
        if let Some(ref boolean_shape) = object.boolean_shape {
            // Per 3MF spec, objects containing booleanshape should be of type "model"
            // While the spec focuses on the BASE and OPERAND objects being type "model",
            // it's also implied that the containing object should be type="model" since
            // other types (support, solidsupport, surface, other) are for specific purposes
            if object.object_type != crate::model::ObjectType::Model {
                return Err(Error::InvalidModel(format!(
                    "Object {}: Object containing booleanshape must be of type \"model\", but is type \"{:?}\".\n\
                     Objects with boolean shapes should be model objects, not support, surface, or other types.",
                    object.id, object.object_type
                )));
            }

            // Per 3MF Boolean Operations Extension spec, an object can have
            // EITHER a mesh, components, OR booleanshape, but not multiple
            // Check that object with booleanshape doesn't also have components or mesh
            if !object.components.is_empty() {
                return Err(Error::InvalidModel(format!(
                    "Object {}: Contains both <booleanshape> and <components>.\n\
                     Per 3MF Boolean Operations spec, an object can contain either a mesh, \
                     components, or booleanshape, but not multiple of these.\n\
                     Remove either the booleanshape or the components from this object.",
                    object.id
                )));
            }

            if object.mesh.is_some() {
                return Err(Error::InvalidModel(format!(
                    "Object {}: Contains both <booleanshape> and <mesh>.\n\
                     Per 3MF Boolean Operations spec, an object can contain either a mesh, \
                     components, or booleanshape, but not multiple of these.\n\
                     Remove either the booleanshape or the mesh from this object.",
                    object.id
                )));
            }

            // Check that booleanshape has at least one operand
            if boolean_shape.operands.is_empty() {
                return Err(Error::InvalidModel(format!(
                    "Object {}: Boolean shape has no operands.\n\
                     Per 3MF Boolean Operations spec, <booleanshape> must contain \
                     one or more <boolean> elements to define the operands.\n\
                     Add at least one <boolean> element inside the <booleanshape>.",
                    object.id
                )));
            }

            // Validate the base object ID exists (skip if it has a path to an external file)
            if boolean_shape.path.is_none() {
                if !valid_object_ids.contains(&boolean_shape.objectid) {
                    let available_ids = sorted_ids_from_set(&valid_object_ids);
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Boolean shape references non-existent object ID {}.\n\
                         Available object IDs: {:?}\n\
                         Hint: Ensure the referenced object exists in the <resources> section.",
                        object.id, boolean_shape.objectid, available_ids
                    )));
                }

                // Check forward reference: base object must be defined before this object
                if let Some(&base_idx) = object_indices.get(&boolean_shape.objectid) {
                    if base_idx >= current_idx {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Boolean shape references object {} which is defined after it.\n\
                             Per 3MF spec, objects must be defined before they are referenced.\n\
                             Move object {} definition before object {} in the file.",
                            object.id, boolean_shape.objectid, boolean_shape.objectid, object.id
                        )));
                    }
                }

                // Check that base object is of type "model"
                if let Some(base_obj) = object_map.get(&boolean_shape.objectid) {
                    if base_obj.object_type != crate::model::ObjectType::Model {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Boolean shape base object {} must be of type \"model\", but is type \"{:?}\".\n\
                             Per 3MF Boolean Operations spec, only model objects can be used as base objects in boolean operations.\n\
                             Change the base object's type attribute to \"model\" or reference a different object.",
                            object.id, boolean_shape.objectid, base_obj.object_type
                        )));
                    }

                    // Check that base object has a shape (mesh, booleanshape, or extension shapes), not just components
                    // Per Boolean Operations spec, the base object "MUST NOT reference a components object"
                    // An object without mesh/boolean_shape but also without components is assumed to have extension shapes
                    let has_shape = base_obj.mesh.is_some()
                        || base_obj.boolean_shape.is_some()
                        || base_obj.has_extension_shapes
                        || (base_obj.components.is_empty()); // If no mesh/boolean and no components, assume extension shape

                    if !has_shape {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Boolean shape base object {} does not define a shape.\n\
                             Per 3MF Boolean Operations spec, the base object must define a shape \
                             (mesh, displacementmesh, booleanshape, or shapes from other extensions), not just an assembly of components.",
                            object.id, boolean_shape.objectid
                        )));
                    }
                }
            }

            // Validate all operand object IDs exist (skip external references)
            for operand in &boolean_shape.operands {
                if operand.path.is_none() {
                    if !valid_object_ids.contains(&operand.objectid) {
                        let available_ids = sorted_ids_from_set(&valid_object_ids);
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Boolean operand references non-existent object ID {}.\n\
                             Available object IDs: {:?}\n\
                             Hint: Ensure the referenced object exists in the <resources> section.",
                            object.id, operand.objectid, available_ids
                        )));
                    }

                    // Check forward reference: operand object must be defined before this object
                    if let Some(&operand_idx) = object_indices.get(&operand.objectid) {
                        if operand_idx >= current_idx {
                            return Err(Error::InvalidModel(format!(
                                "Object {}: Boolean operand references object {} which is defined after it.\n\
                                 Per 3MF spec, objects must be defined before they are referenced.\n\
                                 Move object {} definition before object {} in the file.",
                                object.id, operand.objectid, operand.objectid, object.id
                            )));
                        }
                    }

                    // Check that operand object is of type "model" and is a mesh
                    if let Some(operand_obj) = object_map.get(&operand.objectid) {
                        if operand_obj.object_type != crate::model::ObjectType::Model {
                            return Err(Error::InvalidModel(format!(
                                "Object {}: Boolean operand object {} must be of type \"model\", but is type \"{:?}\".\n\
                                 Per 3MF Boolean Operations spec, only model objects can be used in boolean operations.\n\
                                 Change the operand object's type attribute to \"model\" or reference a different object.",
                                object.id, operand.objectid, operand_obj.object_type
                            )));
                        }

                        // Per spec, operand must be a triangle mesh object only
                        // It MUST NOT contain shapes defined in any other extension
                        if operand_obj.mesh.is_none() {
                            return Err(Error::InvalidModel(format!(
                                "Object {}: Boolean operand object {} must be a triangle mesh.\n\
                                 Per 3MF Boolean Operations spec, operands must be mesh objects only.",
                                object.id, operand.objectid
                            )));
                        }

                        // Check that operand doesn't have booleanshape or components
                        // Per spec: "MUST be only a triangle mesh object" and
                        // "MUST NOT contain shapes defined in any other extension"
                        if operand_obj.boolean_shape.is_some() {
                            return Err(Error::InvalidModel(format!(
                                "Object {}: Boolean operand object {} has a boolean shape.\n\
                                 Per 3MF Boolean Operations spec, operands must be simple triangle meshes only.",
                                object.id, operand.objectid
                            )));
                        }

                        if !operand_obj.components.is_empty() {
                            return Err(Error::InvalidModel(format!(
                                "Object {}: Boolean operand object {} has components.\n\
                                 Per 3MF Boolean Operations spec, operands must be simple triangle meshes only.",
                                object.id, operand.objectid
                            )));
                        }

                        // Check for extension shape elements (beamlattice, displacement, etc.)
                        if operand_obj.has_extension_shapes {
                            return Err(Error::InvalidModel(format!(
                                "Object {}: Boolean operand object {} contains extension shape elements (e.g., beamlattice).\n\
                                 Per 3MF Boolean Operations spec, operands MUST be only triangle mesh objects \
                                 and MUST NOT contain shapes defined in any other extension.",
                                object.id, operand.objectid
                            )));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Validate component references
///
/// Per 3MF Core spec:
/// - Component objectid must reference an existing object in resources
/// - Components must not create circular dependencies (no cycles in the component graph)
fn validate_component_references(model: &Model) -> Result<()> {
    // Build a set of valid object IDs for quick lookup
    let valid_object_ids: HashSet<usize> = model.resources.objects.iter().map(|o| o.id).collect();

    // Get list of encrypted file paths from SecureContent metadata
    let encrypted_paths: HashSet<&str> = if let Some(ref sc_info) = model.secure_content {
        sc_info.encrypted_files.iter().map(|s| s.as_str()).collect()
    } else {
        HashSet::new()
    };

    // Validate that all component object references exist
    for object in &model.resources.objects {
        for component in &object.components {
            // Skip validation for components referencing encrypted files
            // These files cannot be loaded/parsed, so their objects won't exist in resources
            // Only skip if BOTH conditions are true:
            // 1. Component has a path (references external file)
            // 2. That path is in the encrypted files list
            let is_encrypted_reference = if let Some(ref path) = component.path {
                encrypted_paths.contains(path.as_str())
            } else {
                false
            };

            if is_encrypted_reference {
                // This component references an encrypted file - skip validation
                continue;
            }

            // Skip validation for components that reference external files (Production extension)
            // When a component has a p:path attribute, the referenced object is in an external
            // file (potentially encrypted in Secure Content scenarios) and doesn't need to exist
            // in the current model's resources
            if component
                .production
                .as_ref()
                .is_some_and(|p| p.path.is_some())
            {
                continue;
            }

            if !valid_object_ids.contains(&component.objectid) {
                let available_ids = sorted_ids_from_set(&valid_object_ids);
                return Err(Error::InvalidModel(format!(
                    "Object {}: Component references non-existent object ID {}.\n\
                     Available object IDs: {:?}\n\
                     Hint: Ensure the referenced object exists in the <resources> section.",
                    object.id, component.objectid, available_ids
                )));
            }
        }
    }

    // Detect circular component references using depth-first search
    // We need to detect if following component references creates a cycle
    for object in &model.resources.objects {
        if !object.components.is_empty() {
            let mut visited = HashSet::new();
            let mut path = Vec::new();
            if let Some(cycle_path) =
                detect_circular_components(object.id, model, &mut visited, &mut path)?
            {
                return Err(Error::InvalidModel(format!(
                    "Circular component reference: {}",
                    cycle_path
                        .iter()
                        .map(|id| id.to_string())
                        .collect::<Vec<_>>()
                        .join(" → ")
                )));
            }
        }
    }

    Ok(())
}

/// Recursive helper function to detect circular dependencies in component graph
///
/// Uses depth-first search with path tracking to detect cycles.
/// Returns Some(path) with the circular path if a cycle is detected, None otherwise.
fn detect_circular_components(
    object_id: usize,
    model: &Model,
    visited: &mut HashSet<usize>,
    path: &mut Vec<usize>,
) -> Result<Option<Vec<usize>>> {
    // If this object is already in the current path, we have a cycle
    if let Some(cycle_start) = path.iter().position(|&id| id == object_id) {
        // Return the circular portion of the path plus the repeated node
        let mut cycle_path = path[cycle_start..].to_vec();
        cycle_path.push(object_id);
        return Ok(Some(cycle_path));
    }

    // If we've already fully processed this object, no cycle here
    if visited.contains(&object_id) {
        return Ok(None);
    }

    // Mark as being processed and add to path
    visited.insert(object_id);
    path.push(object_id);

    // Find the object and check its components
    if let Some(object) = model.resources.objects.iter().find(|o| o.id == object_id) {
        for component in &object.components {
            // Skip circular reference check for components with external production paths
            // When a component has p:path, it references an object in an external file,
            // so it doesn't create a circular reference within the current model
            let has_external_path = component
                .production
                .as_ref()
                .is_some_and(|p| p.path.is_some());

            if has_external_path {
                continue;
            }

            if let Some(cycle) =
                detect_circular_components(component.objectid, model, visited, path)?
            {
                return Ok(Some(cycle));
            }
        }
    }

    // Done processing this object, remove from path and visited set
    // We need to remove from visited to allow the node to be visited from other paths
    // This is necessary for proper cycle detection when the same node can be reached
    // via different paths in the component graph (e.g., checking if A→B→C→A forms a cycle)
    path.pop();
    visited.remove(&object_id);
    Ok(None)
}

/// Validate production extension requirements
///
/// Checks that:
/// - p:path attributes have valid format (must start with /, cannot contain .., cannot end with /)
/// - Build items with p:path must have valid paths
///
/// Note: This is the legacy validation function that doesn't consider parser config.
/// Prefer using `validate_production_extension_with_config` for more flexible validation.
#[allow(dead_code)] // Kept for backward compatibility and testing
fn validate_production_extension(model: &Model) -> Result<()> {
    // Helper function to validate p:path format
    let validate_path = |path: &str, context: &str| -> Result<()> {
        // Per 3MF Production Extension spec:
        // - Path MUST start with / (absolute path within the package)
        // - Path MUST NOT contain .. (no parent directory references)
        // - Path MUST NOT end with / (must reference a file, not a directory)
        // - Filename MUST NOT start with . (hidden files not allowed)

        if !path.starts_with('/') {
            return Err(Error::InvalidModel(format!(
                "{}: Production path '{}' must start with / (absolute path required)",
                context, path
            )));
        }

        if path.contains("..") {
            return Err(Error::InvalidModel(format!(
                "{}: Production path '{}' must not contain .. (parent directory traversal not allowed)",
                context, path
            )));
        }

        if path.ends_with('/') {
            return Err(Error::InvalidModel(format!(
                "{}: Production path '{}' must not end with / (must reference a file)",
                context, path
            )));
        }

        // Check for hidden files (filename starting with .)
        if let Some(filename) = path.rsplit('/').next() {
            if filename.starts_with('.') {
                return Err(Error::InvalidModel(format!(
                    "{}: Production path '{}' references a hidden file (filename cannot start with .)",
                    context, path
                )));
            }
        }

        // Path should reference a .model file
        if !path.ends_with(".model") {
            return Err(Error::InvalidModel(format!(
                "{}: Production path '{}' must reference a .model file",
                context, path
            )));
        }

        Ok(())
    };

    // Check all objects to validate production paths
    for object in &model.resources.objects {
        // Note: The thumbnail attribute is deprecated in 3MF v1.4+ when production extension is used,
        // but deprecation doesn't make it invalid. Per the official 3MF test suite, files with
        // thumbnail attributes and production extension should still parse successfully.
        // Therefore, we do not reject files with thumbnail attributes.

        // Validate production extension usage
        if let Some(ref prod_info) = object.production {
            // If object has production path, validate it
            if let Some(ref path) = prod_info.path {
                validate_path(path, &format!("Object {}", object.id))?;
            }
        }

        // Check components
        for (idx, component) in object.components.iter().enumerate() {
            if let Some(ref prod_info) = component.production {
                // Validate production path format if present
                // Note: component.path is set from prod_info.path during parsing
                // Per 3MF Production Extension spec:
                // - p:UUID can be used on components to uniquely identify them
                // - p:path is only required when referencing external objects (not in current file)
                // - A component with p:UUID but no p:path references a local object
                if let Some(ref path) = prod_info.path {
                    validate_path(path, &format!("Object {}, Component {}", object.id, idx))?;
                }
            }
        }
    }

    // Check build items for production path validation
    for (idx, item) in model.build.items.iter().enumerate() {
        if let Some(ref path) = item.production_path {
            validate_path(path, &format!("Build Item {}", idx))?;
        }
    }

    // Note: We don't validate that production attributes require the production extension
    // to be in requiredextensions, because per the 3MF spec, extensions can be declared
    // in namespaces (xmlns:p) without being in requiredextensions - they are then optional
    // extensions. The parser already validates that the production namespace is declared
    // when production attributes are used.

    Ok(())
}

/// Validate production extension requirements with parser configuration
///
/// This is a variant of `validate_production_extension` that accepts a parser config.
/// When the parser config explicitly supports the production extension, we allow
/// production attributes to be used even if the file doesn't declare the production
/// extension in requiredextensions. This is useful for backward compatibility and
/// for files that use production attributes but were created before strict validation.
fn validate_production_extension_with_config(model: &Model, config: &ParserConfig) -> Result<()> {
    // Check if production extension is required in the file
    let has_production = model.required_extensions.contains(&Extension::Production);

    // Check if the parser config explicitly supports production extension
    let config_supports_production = config.supports(&Extension::Production);

    // Track whether any production attributes are used (for validation later)
    let mut has_production_attrs = false;

    // Helper function to validate p:path format
    let validate_path = |path: &str, context: &str| -> Result<()> {
        // Per 3MF Production Extension spec:
        // - Path MUST start with / (absolute path within the package)
        // - Path MUST NOT contain .. (no parent directory references)
        // - Path MUST NOT end with / (must reference a file, not a directory)
        // - Filename MUST NOT start with . (hidden files not allowed)

        if !path.starts_with('/') {
            return Err(Error::InvalidModel(format!(
                "{}: Production path '{}' must start with / (absolute path required)",
                context, path
            )));
        }

        if path.contains("..") {
            return Err(Error::InvalidModel(format!(
                "{}: Production path '{}' must not contain .. (parent directory traversal not allowed)",
                context, path
            )));
        }

        if path.ends_with('/') {
            return Err(Error::InvalidModel(format!(
                "{}: Production path '{}' must not end with / (must reference a file)",
                context, path
            )));
        }

        // Check for hidden files (filename starting with .)
        if let Some(filename) = path.rsplit('/').next() {
            if filename.starts_with('.') {
                return Err(Error::InvalidModel(format!(
                    "{}: Production path '{}' references a hidden file (filename cannot start with .)",
                    context, path
                )));
            }
        }

        // Path should reference a .model file
        if !path.ends_with(".model") {
            return Err(Error::InvalidModel(format!(
                "{}: Production path '{}' must reference a .model file",
                context, path
            )));
        }

        Ok(())
    };

    // Check all objects to validate production paths
    for object in &model.resources.objects {
        // Note: The thumbnail attribute is deprecated in 3MF v1.4+ when production extension is used,
        // but deprecation doesn't make it invalid. Per the official 3MF test suite, files with
        // thumbnail attributes and production extension should still parse successfully.
        // Therefore, we do not reject files with thumbnail attributes.

        // Validate production extension usage and track attributes
        if let Some(ref prod_info) = object.production {
            has_production_attrs = true;

            // If object has production path, validate it
            if let Some(ref path) = prod_info.path {
                validate_path(path, &format!("Object {}", object.id))?;
            }
        }

        // Check components
        for (idx, component) in object.components.iter().enumerate() {
            if let Some(ref prod_info) = component.production {
                has_production_attrs = true;

                // Per 3MF Production Extension spec:
                // - p:UUID can be used on components to uniquely identify them
                // - p:path is only required when referencing external objects (not in current file)
                // - A component with p:UUID but no p:path references a local object
                // - When p:path is used (external reference), p:UUID is REQUIRED to identify the object

                // Validate that p:UUID is present when p:path is used
                if prod_info.path.is_some() && prod_info.uuid.is_none() {
                    return Err(Error::InvalidModel(format!(
                        "Object {}, Component {}: Component has p:path but missing required p:UUID.\n\
                         Per 3MF Production Extension spec, components with external references (p:path) \
                         must have p:UUID to identify the referenced object.\n\
                         Add p:UUID attribute to the component element.",
                        object.id, idx
                    )));
                }

                // Validate production path format if present
                // Note: component.path is set from prod_info.path during parsing
                if let Some(ref path) = prod_info.path {
                    validate_path(path, &format!("Object {}, Component {}", object.id, idx))?;
                }
            }
        }
    }

    // Check build items for production path validation
    for (idx, item) in model.build.items.iter().enumerate() {
        if item.production_uuid.is_some() || item.production_path.is_some() {
            has_production_attrs = true;
        }

        if let Some(ref path) = item.production_path {
            validate_path(path, &format!("Build Item {}", idx))?;
        }
    }

    // Check build production UUID
    if model.build.production_uuid.is_some() {
        has_production_attrs = true;
    }

    // Validate that production attributes are only used when production extension is declared
    // UNLESS the parser config explicitly supports production extension (for backward compatibility)
    if has_production_attrs && !has_production && !config_supports_production {
        return Err(Error::InvalidModel(
            "Production extension attributes (p:UUID, p:path) are used but production extension \
             is not declared in requiredextensions.\n\
             Per 3MF Production Extension specification, when using production attributes, \
             you must add 'p' to the requiredextensions attribute in the <model> element.\n\
             Example: requiredextensions=\"p\" or requiredextensions=\"m p\" for materials and production."
                .to_string(),
        ));
    }

    Ok(())
}

/// Validate displacement extension usage
///
/// Per Displacement Extension spec:
/// - Displacement2D resources must reference existing texture files in the package
/// - Disp2DGroup must reference existing Displacement2D and NormVectorGroup resources
/// - Disp2DCoord must reference valid normvector indices
/// - NormVectors must be normalized (unit length)
/// - DisplacementTriangle did must reference existing Disp2DGroup resources
/// - DisplacementTriangle d1, d2, d3 must reference valid displacement coordinates
fn validate_displacement_extension(model: &Model) -> Result<()> {
    // Check if displacement resources/elements are used (DPX 3312)
    let has_displacement_resources = !model.resources.displacement_maps.is_empty()
        || !model.resources.norm_vector_groups.is_empty()
        || !model.resources.disp2d_groups.is_empty()
        || model
            .resources
            .objects
            .iter()
            .any(|obj| obj.displacement_mesh.is_some());

    if has_displacement_resources {
        // Check if displacement extension is declared in requiredextensions
        let has_displacement_required = model
            .required_extensions
            .iter()
            .any(|ext| matches!(ext, Extension::Displacement))
            || model.required_custom_extensions.iter().any(|ns| {
                ns.contains("displacement/2022/07") || ns.contains("displacement/2023/10")
            });

        if !has_displacement_required {
            return Err(Error::InvalidModel(
                "Model contains displacement extension elements (displacement2d, normvectorgroup, disp2dgroup, or displacementmesh) \
                 but displacement extension is not declared in requiredextensions attribute.\n\
                 Per 3MF Displacement Extension spec, files using displacement elements MUST declare the displacement extension \
                 as a required extension in the <model> element's requiredextensions attribute.\n\
                 Add 'd' to requiredextensions and declare xmlns:d=\"http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07\"."
                    .to_string(),
            ));
        }
    }

    // Validate Displacement2D path requirements (DPX 3300)
    // Per 3MF Displacement Extension spec 3.1: displacement texture paths must be in /3D/Textures/
    // Per OPC spec: paths must contain only ASCII characters
    for disp_map in &model.resources.displacement_maps {
        // Check that the path contains only ASCII characters
        if !disp_map.path.is_ascii() {
            return Err(Error::InvalidModel(format!(
                "Displacement2D resource {}: Path '{}' contains non-ASCII characters.\n\
                 Per OPC specification, all 3MF package paths must contain only ASCII characters.\n\
                 Hint: Remove Unicode or special characters from the displacement texture path.",
                disp_map.id, disp_map.path
            )));
        }

        // Check if this displacement map is encrypted (Secure Content extension)
        // For encrypted files, skip strict path validation as they may use non-standard paths
        let is_encrypted = model
            .secure_content
            .as_ref()
            .map(|sc| {
                sc.encrypted_files.iter().any(|encrypted_path| {
                    // Compare normalized paths (both without leading slash)
                    let disp_normalized = disp_map.path.trim_start_matches('/');
                    let enc_normalized = encrypted_path.trim_start_matches('/');
                    enc_normalized == disp_normalized
                })
            })
            .unwrap_or(false);

        // Per 3MF Displacement Extension spec 3.1, displacement texture paths should be in /3D/Textures/
        // Skip this check for encrypted files as they may use non-standard paths
        // Use case-insensitive comparison as 3MF paths are case-insensitive per OPC spec
        if !is_encrypted && !disp_map.path.to_lowercase().starts_with("/3d/textures/") {
            return Err(Error::InvalidModel(format!(
                "Displacement2D resource {}: Path '{}' is not in /3D/Textures/ directory (case-insensitive).\n\
                 Per 3MF Displacement Extension spec 3.1, displacement texture files must be stored in /3D/Textures/ \
                 (any case variation like /3D/textures/ is also accepted).\n\
                 Move the displacement texture file to the appropriate directory and update the path.",
                disp_map.id, disp_map.path
            )));
        }

        // Validate file extension matches expected image type (DPX 3314_08)
        // Displacement textures should be PNG files
        let path_lower = disp_map.path.to_lowercase();
        if !path_lower.ends_with(".png") {
            return Err(Error::InvalidModel(format!(
                "Displacement2D resource {}: Path '{}' does not end with .png extension.\n\
                 Per 3MF Displacement Extension spec 3.1, displacement textures should be PNG files.\n\
                 Hint: Ensure the displacement texture file has a .png extension and correct content type.",
                disp_map.id, disp_map.path
            )));
        }
    }

    // Build sets of valid IDs for quick lookup
    let displacement_map_ids: HashSet<usize> = model
        .resources
        .displacement_maps
        .iter()
        .map(|d| d.id)
        .collect();

    let norm_vector_group_ids: HashSet<usize> = model
        .resources
        .norm_vector_groups
        .iter()
        .map(|n| n.id)
        .collect();

    let disp2d_group_ids: HashSet<usize> =
        model.resources.disp2d_groups.iter().map(|d| d.id).collect();

    // Validate Disp2DGroup references
    for disp2d_group in &model.resources.disp2d_groups {
        // Validate dispid reference
        if !displacement_map_ids.contains(&disp2d_group.dispid) {
            let available_ids = sorted_ids_from_set(&displacement_map_ids);
            return Err(Error::InvalidModel(format!(
                "Disp2DGroup {}: References non-existent Displacement2D resource with ID {}.\n\
                 Available Displacement2D IDs: {:?}\n\
                 Hint: Ensure the referenced displacement2d resource exists in the <resources> section.",
                disp2d_group.id, disp2d_group.dispid, available_ids
            )));
        }

        // Validate nid reference
        if !norm_vector_group_ids.contains(&disp2d_group.nid) {
            let available_ids = sorted_ids_from_set(&norm_vector_group_ids);
            return Err(Error::InvalidModel(format!(
                "Disp2DGroup {}: References non-existent NormVectorGroup with ID {}.\n\
                 Available NormVectorGroup IDs: {:?}\n\
                 Hint: Ensure the referenced normvectorgroup resource exists in the <resources> section.",
                disp2d_group.id, disp2d_group.nid, available_ids
            )));
        }

        // Validate displacement coordinate normvector indices
        if let Some(norm_group) = model
            .resources
            .norm_vector_groups
            .iter()
            .find(|n| n.id == disp2d_group.nid)
        {
            for (coord_idx, coord) in disp2d_group.coords.iter().enumerate() {
                if coord.n >= norm_group.vectors.len() {
                    let max_index = if !norm_group.vectors.is_empty() {
                        norm_group.vectors.len() - 1
                    } else {
                        0
                    };
                    return Err(Error::InvalidModel(format!(
                        "Disp2DGroup {}: Displacement coordinate {} references normvector index {} \
                         but NormVectorGroup {} only contains {} normvectors.\n\
                         Hint: Normvector indices must be in range [0, {}].",
                        disp2d_group.id, coord_idx, coord.n, disp2d_group.nid,
                        norm_group.vectors.len(), max_index
                    )));
                }
            }
        }
    }

    // Validate NormVectorGroup - vectors must point outward
    // Per DPX 3302: Normalized displacement vectors MUST point to the outer hemisphere of the triangle
    // The scalar product of a normalized displacement vector to the triangle normal MUST be greater than 0

    // Epsilon for zero-length vector detection
    const NORMVECTOR_ZERO_EPSILON: f64 = 0.000001;

    for norm_group in &model.resources.norm_vector_groups {
        for (idx, norm_vec) in norm_group.vectors.iter().enumerate() {
            // Calculate the magnitude of the vector
            let length_squared =
                norm_vec.x * norm_vec.x + norm_vec.y * norm_vec.y + norm_vec.z * norm_vec.z;

            // Check if vector has zero length
            if length_squared < NORMVECTOR_ZERO_EPSILON {
                return Err(Error::InvalidModel(format!(
                    "NormVectorGroup {}: Normvector {} has near-zero length (x={}, y={}, z={}). \
                     Normal vectors must have non-zero length.",
                    norm_group.id, idx, norm_vec.x, norm_vec.y, norm_vec.z
                )));
            }

            // Note: Full validation of scalar product with triangle normal requires knowing
            // which triangles use which normvectors, which is complex cross-referencing.
            // The parser and validator together ensure proper usage.
        }
    }

    // NOTE: Normalization validation is commented out because official test suite positive tests
    // include non-normalized vectors (e.g., P_DPX_3204_03.3mf has z=0.9, P_DPX_3204_06.3mf has x=y=z=1)
    // The spec may allow non-normalized vectors to be automatically normalized by the renderer.
    // If strict validation is needed, it can be re-enabled, but this would fail valid test cases.
    //
    // // Validate NormVectorGroup - all vectors must be normalized (unit length)
    // for norm_group in &model.resources.norm_vector_groups {
    //     for (idx, norm_vec) in norm_group.vectors.iter().enumerate() {
    //         let length_squared = norm_vec.x * norm_vec.x + norm_vec.y * norm_vec.y + norm_vec.z * norm_vec.z;
    //         let length = length_squared.sqrt();
    //
    //         // Allow a small tolerance for floating point errors (0.01%)
    //         if (length - 1.0).abs() > 0.0001 {
    //             return Err(Error::InvalidModel(format!(
    //                 "NormVectorGroup {}: Normvector {} is not normalized (length = {:.6}, expected 1.0).\n\
    //                  Vector components: x={}, y={}, z={}\n\
    //                  Hint: Normal vectors must be unit length. Normalize the vector by dividing each component by its length.",
    //                 norm_group.id, idx, length, norm_vec.x, norm_vec.y, norm_vec.z
    //             )));
    //         }
    //     }
    // }

    // Validate displacement meshes in objects
    for object in &model.resources.objects {
        if let Some(ref disp_mesh) = object.displacement_mesh {
            // Validate that displacement mesh has at least 4 triangles (minimum for closed volume)
            // Per DPX 3308: A valid 3D mesh must have at least 4 triangles to form a tetrahedron
            if disp_mesh.triangles.len() < 4 {
                return Err(Error::InvalidModel(format!(
                    "Object {}: Displacement mesh has only {} triangles. \
                     A valid 3D mesh must have at least 4 triangles to form a closed volume.",
                    object.id,
                    disp_mesh.triangles.len()
                )));
            }

            // Validate mesh volume (DPX 3314_02)
            // Calculate signed volume to detect negative volume (inverted meshes)
            let mut volume = 0.0_f64;
            for triangle in &disp_mesh.triangles {
                if triangle.v1 >= disp_mesh.vertices.len()
                    || triangle.v2 >= disp_mesh.vertices.len()
                    || triangle.v3 >= disp_mesh.vertices.len()
                {
                    continue; // Skip invalid triangles (caught by other validation)
                }

                let v1 = &disp_mesh.vertices[triangle.v1];
                let v2 = &disp_mesh.vertices[triangle.v2];
                let v3 = &disp_mesh.vertices[triangle.v3];

                // Signed volume contribution of this triangle
                volume += v1.x * (v2.y * v3.z - v2.z * v3.y)
                    + v2.x * (v3.y * v1.z - v3.z * v1.y)
                    + v3.x * (v1.y * v2.z - v1.z * v2.y);
            }
            volume /= 6.0;

            // Use small epsilon for floating-point comparison
            const EPSILON: f64 = 1e-10;
            // DPX 3314_02, 3314_05: Reject negative volumes (inverted/reversed triangles)
            // Use a very small negative threshold to catch reversed triangles
            if volume < EPSILON {
                if volume < 0.0 {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Displacement mesh has negative volume ({:.10}), indicating inverted or incorrectly oriented triangles.\n\
                         Hint: Check triangle vertex winding order - vertices should be ordered counter-clockwise when viewed from outside.",
                        object.id, volume
                    )));
                }
                // Also reject very small positive volumes as they indicate nearly flat meshes
                return Err(Error::InvalidModel(format!(
                    "Object {}: Displacement mesh has near-zero volume ({:.10}), indicating a degenerate or flat mesh.\n\
                     Hint: Ensure the mesh encloses a non-zero 3D volume.",
                    object.id, volume
                )));
            }

            // Validate manifold mesh and check for duplicate vertices (DPX 3314_06)
            // Check for duplicate vertices (exact same position)
            for i in 0..disp_mesh.vertices.len() {
                for j in (i + 1)..disp_mesh.vertices.len() {
                    let v1 = &disp_mesh.vertices[i];
                    let v2 = &disp_mesh.vertices[j];
                    let dist_sq =
                        (v1.x - v2.x).powi(2) + (v1.y - v2.y).powi(2) + (v1.z - v2.z).powi(2);
                    if dist_sq < EPSILON {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Displacement mesh has duplicate vertices at indices {} and {} \
                             with same position ({}, {}, {}).\n\
                             Hint: Remove duplicate vertices or merge them properly.",
                            object.id, i, j, v1.x, v1.y, v1.z
                        )));
                    }
                }
            }

            // Check for manifold mesh and consistent triangle orientation (DPX 3314_05, 3314_06)
            // For a properly oriented closed mesh:
            // - Each edge should be shared by exactly 2 triangles (manifold property)
            // - The two triangles should traverse the edge in opposite directions (consistent winding)
            // Build edge map: directed_edge -> count
            let mut edge_to_triangles: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
            for (tri_idx, triangle) in disp_mesh.triangles.iter().enumerate() {
                // Add all three directed edges (v1->v2, v2->v3, v3->v1)
                let edges = [
                    (triangle.v1, triangle.v2),
                    (triangle.v2, triangle.v3),
                    (triangle.v3, triangle.v1),
                ];
                for edge in &edges {
                    edge_to_triangles.entry(*edge).or_default().push(tri_idx);
                }
            }

            // Check for consistent edge orientation and manifold property
            // For each directed edge, check if its reverse also exists with exactly one triangle
            let mut checked_edges = HashSet::new();
            for ((v1, v2), tris) in &edge_to_triangles {
                if checked_edges.contains(&(*v1, *v2)) {
                    continue;
                }
                checked_edges.insert((*v1, *v2));
                checked_edges.insert((*v2, *v1));

                let reverse_edge = (*v2, *v1);
                let reverse_tris = edge_to_triangles.get(&reverse_edge);

                match (tris.len(), reverse_tris) {
                    (1, Some(rev_tris)) if rev_tris.len() == 1 => {
                        // Perfect: edge traversed once in each direction - consistent orientation
                    }
                    (1, None) => {
                        // Edge only traversed in one direction - boundary edge (non-manifold)
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Displacement mesh is non-manifold. \
                             Edge from vertex {} to vertex {} is only used by one triangle (should be two).\n\
                             Hint: Ensure the mesh is a closed, watertight surface with no holes or dangling edges.",
                            object.id, v1, v2
                        )));
                    }
                    (1, Some(rev_tris)) if rev_tris.len() > 1 => {
                        // Reverse edge used by multiple triangles - non-manifold
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Displacement mesh is non-manifold. \
                             Edge between vertices {} and {} is used by {} triangles (should be exactly 2).\n\
                             Hint: Ensure the mesh is a closed, watertight surface with no holes or dangling edges.",
                            object.id, v1.min(v2), v1.max(v2), tris.len() + rev_tris.len()
                        )));
                    }
                    (count, _) if count > 1 => {
                        // DPX 3314_05: Edge traversed multiple times in the same direction
                        // This indicates reversed/inconsistent triangle winding
                        // In a properly oriented mesh, each directed edge should appear exactly once
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Displacement mesh has inconsistent triangle winding.\n\
                             Edge from vertex {} to vertex {} is traversed {} times in the same direction.\n\
                             This indicates some triangles have reversed vertex order (normals pointing inward).\n\
                             Hint: Check triangle vertex winding order - vertices should be ordered counter-clockwise when viewed from outside.",
                            object.id, v1, v2, count
                        )));
                    }
                    _ => {
                        // Should not reach here
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Displacement mesh has an unexpected edge configuration for vertices {} and {}.",
                            object.id, v1, v2
                        )));
                    }
                }
            }

            // Validate that displacement triangles have valid vertex references
            for (tri_idx, triangle) in disp_mesh.triangles.iter().enumerate() {
                // Check vertex indices
                if triangle.v1 >= disp_mesh.vertices.len() {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Displacement triangle {} has invalid vertex index v1={} \
                         (mesh only has {} vertices).",
                        object.id,
                        tri_idx,
                        triangle.v1,
                        disp_mesh.vertices.len()
                    )));
                }
                if triangle.v2 >= disp_mesh.vertices.len() {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Displacement triangle {} has invalid vertex index v2={} \
                         (mesh only has {} vertices).",
                        object.id,
                        tri_idx,
                        triangle.v2,
                        disp_mesh.vertices.len()
                    )));
                }
                if triangle.v3 >= disp_mesh.vertices.len() {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Displacement triangle {} has invalid vertex index v3={} \
                         (mesh only has {} vertices).",
                        object.id,
                        tri_idx,
                        triangle.v3,
                        disp_mesh.vertices.len()
                    )));
                }

                // Check for degenerate triangles (DPX 3310)
                // All three vertices must be distinct
                if triangle.v1 == triangle.v2
                    || triangle.v2 == triangle.v3
                    || triangle.v1 == triangle.v3
                {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Displacement triangle {} is degenerate (v1={}, v2={}, v3={}). \
                         All three vertex indices must be distinct.",
                        object.id, tri_idx, triangle.v1, triangle.v2, triangle.v3
                    )));
                }

                // Check for zero-area triangles - collinear vertices (DPX 3314_07, 3314_08)
                // Even if indices are distinct, vertices might be at same position or collinear
                let v1 = &disp_mesh.vertices[triangle.v1];
                let v2 = &disp_mesh.vertices[triangle.v2];
                let v3 = &disp_mesh.vertices[triangle.v3];

                let edge1_x = v2.x - v1.x;
                let edge1_y = v2.y - v1.y;
                let edge1_z = v2.z - v1.z;
                let edge2_x = v3.x - v1.x;
                let edge2_y = v3.y - v1.y;
                let edge2_z = v3.z - v1.z;

                // Cross product magnitude squared = (2 * area)^2
                let cross_x = edge1_y * edge2_z - edge1_z * edge2_y;
                let cross_y = edge1_z * edge2_x - edge1_x * edge2_z;
                let cross_z = edge1_x * edge2_y - edge1_y * edge2_x;
                let cross_mag_sq = cross_x * cross_x + cross_y * cross_y + cross_z * cross_z;

                const AREA_EPSILON: f64 = 1e-20;
                if cross_mag_sq < AREA_EPSILON {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Displacement triangle {} has zero or near-zero area (vertices are collinear).\n\
                         Vertices: v1=({:.6}, {:.6}, {:.6}), v2=({:.6}, {:.6}, {:.6}), v3=({:.6}, {:.6}, {:.6})\n\
                         Hint: Ensure triangle vertices form a non-degenerate triangle with non-zero area.",
                        object.id,
                        tri_idx,
                        v1.x,
                        v1.y,
                        v1.z,
                        v2.x,
                        v2.y,
                        v2.z,
                        v3.x,
                        v3.y,
                        v3.z
                    )));
                }

                // Validate did reference
                if let Some(did) = triangle.did {
                    if !disp2d_group_ids.contains(&did) {
                        let available_ids = sorted_ids_from_set(&disp2d_group_ids);
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Displacement triangle {} references non-existent Disp2DGroup with ID {}.\n\
                             Available Disp2DGroup IDs: {:?}\n\
                             Hint: Ensure the referenced disp2dgroup resource exists in the <resources> section.",
                            object.id, tri_idx, did, available_ids
                        )));
                    }

                    // Validate displacement coordinate indices (d1, d2, d3)
                    if let Some(disp_group) =
                        model.resources.disp2d_groups.iter().find(|d| d.id == did)
                    {
                        let max_coord_index = if !disp_group.coords.is_empty() {
                            disp_group.coords.len() - 1
                        } else {
                            0
                        };

                        if let Some(d1) = triangle.d1 {
                            if d1 >= disp_group.coords.len() {
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Displacement triangle {} has invalid d1 index {} \
                                     (Disp2DGroup {} only has {} coordinates).\n\
                                     Hint: Displacement coordinate indices must be in range [0, {}].",
                                    object.id, tri_idx, d1, did, disp_group.coords.len(),
                                    max_coord_index
                                )));
                            }
                        }

                        if let Some(d2) = triangle.d2 {
                            if d2 >= disp_group.coords.len() {
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Displacement triangle {} has invalid d2 index {} \
                                     (Disp2DGroup {} only has {} coordinates).\n\
                                     Hint: Displacement coordinate indices must be in range [0, {}].",
                                    object.id, tri_idx, d2, did, disp_group.coords.len(),
                                    max_coord_index
                                )));
                            }
                        }

                        if let Some(d3) = triangle.d3 {
                            if d3 >= disp_group.coords.len() {
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Displacement triangle {} has invalid d3 index {} \
                                     (Disp2DGroup {} only has {} coordinates).\n\
                                     Hint: Displacement coordinate indices must be in range [0, {}].",
                                    object.id, tri_idx, d3, did, disp_group.coords.len(),
                                    max_coord_index
                                )));
                            }
                        }

                        // Validate that normvectors point outward relative to triangle normal (DPX 3302)
                        // Calculate triangle normal and check scalar product with displacement vectors
                        let v1 = &disp_mesh.vertices[triangle.v1];
                        let v2 = &disp_mesh.vertices[triangle.v2];
                        let v3 = &disp_mesh.vertices[triangle.v3];

                        // Calculate triangle normal using cross product
                        let edge1_x = v2.x - v1.x;
                        let edge1_y = v2.y - v1.y;
                        let edge1_z = v2.z - v1.z;
                        let edge2_x = v3.x - v1.x;
                        let edge2_y = v3.y - v1.y;
                        let edge2_z = v3.z - v1.z;

                        let normal_x = edge1_y * edge2_z - edge1_z * edge2_y;
                        let normal_y = edge1_z * edge2_x - edge1_x * edge2_z;
                        let normal_z = edge1_x * edge2_y - edge1_y * edge2_x;

                        // Get the normvectorgroup
                        if let Some(norm_group) = model
                            .resources
                            .norm_vector_groups
                            .iter()
                            .find(|n| n.id == disp_group.nid)
                        {
                            // Check normvectors for each displacement coordinate used
                            for (_coord_idx, disp_coord_idx) in
                                [(1, triangle.d1), (2, triangle.d2), (3, triangle.d3)].iter()
                            {
                                if let Some(d_idx) = disp_coord_idx {
                                    if *d_idx < disp_group.coords.len() {
                                        let coord = &disp_group.coords[*d_idx];
                                        if coord.n < norm_group.vectors.len() {
                                            let norm_vec = &norm_group.vectors[coord.n];

                                            // Calculate scalar (dot) product
                                            let dot_product = normal_x * norm_vec.x
                                                + normal_y * norm_vec.y
                                                + normal_z * norm_vec.z;

                                            // Per DPX spec: scalar product must be > 0
                                            // Use epsilon for floating-point comparison
                                            const DOT_PRODUCT_EPSILON: f64 = 1e-10;
                                            if dot_product <= DOT_PRODUCT_EPSILON {
                                                return Err(Error::InvalidModel(format!(
                                                    "Object {}: Displacement triangle {} uses normvector {} from group {} \
                                                     that points inward (scalar product with triangle normal = {:.6} <= 0).\n\
                                                     Per 3MF Displacement spec, normalized displacement vectors MUST point to the outer hemisphere.\n\
                                                     Hint: Reverse the normvector direction or fix the triangle vertex order.",
                                                    object.id, tri_idx, coord.n, disp_group.nid, dot_product
                                                )));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Validate slice stacks and their slices
///
/// Per 3MF Slice Extension spec, validates that:
/// - Slices must contain at least one polygon with vertices (non-empty slices)
/// - Polygon vertex indices (startv and v2) must reference valid vertices in the slice
fn validate_slices(model: &Model) -> Result<()> {
    // Validate all slice stacks in resources
    for slice_stack in &model.resources.slice_stacks {
        // N_SPX_1606_01: Validate ztop values are >= zbottom
        // N_SPX_1607_01: Validate ztop values are strictly increasing
        let mut prev_ztop: Option<f64> = None;

        for (slice_idx, slice) in slice_stack.slices.iter().enumerate() {
            // Check ztop >= zbottom
            if slice.ztop < slice_stack.zbottom {
                return Err(Error::InvalidModel(format!(
                    "SliceStack {}: Slice {} has ztop={} which is less than zbottom={}.\n\
                     Per 3MF Slice Extension spec, each slice's ztop must be >= the slicestack's zbottom.",
                    slice_stack.id, slice_idx, slice.ztop, slice_stack.zbottom
                )));
            }

            // Check ztop values are strictly increasing
            if let Some(prev) = prev_ztop {
                if slice.ztop <= prev {
                    return Err(Error::InvalidModel(format!(
                        "SliceStack {}: Slice {} has ztop={} which is not greater than the previous slice's ztop={}.\n\
                         Per 3MF Slice Extension spec, ztop values must be strictly increasing within a slicestack.",
                        slice_stack.id, slice_idx, slice.ztop, prev
                    )));
                }
            }
            prev_ztop = Some(slice.ztop);

            validate_slice(slice_stack.id, slice_idx, slice)?;
        }
    }

    Ok(())
}

/// Validate slice extension requirements
///
/// Per 3MF Slice Extension spec v1.0.2:
/// - SliceRef slicepath must point to /2D/ folder (not /3D/ or other directories)
/// - When an object references a slicestack, transforms must be planar (no Z-axis rotation/shear)
/// - SliceStack must contain either slices OR slicerefs, not both
fn validate_slice_extension(model: &Model) -> Result<()> {
    // Check if model uses slice extension
    if model.resources.slice_stacks.is_empty() {
        return Ok(());
    }

    // Validate slicerefs in all slicestacks
    for stack in &model.resources.slice_stacks {
        // Rule: SliceStack must contain either slices OR slicerefs, not both
        if !stack.slices.is_empty() && !stack.slice_refs.is_empty() {
            return Err(Error::InvalidModel(format!(
                "SliceStack {}: Contains both <slice> and <sliceref> elements.\n\
                 Per 3MF Slice Extension spec, a slicestack MUST contain either \
                 <slice> elements or <sliceref> elements, but MUST NOT contain both element types concurrently.",
                stack.id
            )));
        }

        // Note: SliceRef validation happens during loading in parser.rs::load_slice_references()
        // because slice_refs are cleared after loading external files are resolved.
        // Validation performed during loading includes:
        // - SliceRef slicepath must start with "/2D/"
        // - Referenced slicestackid must exist in external file
        // - SliceStack cannot contain both <slice> and <sliceref> elements (mixed elements)
    }

    // Build a set of valid slicestack IDs for reference validation
    let valid_slicestack_ids: std::collections::HashSet<usize> = model
        .resources
        .slice_stacks
        .iter()
        .map(|stack| stack.id)
        .collect();

    // Validate that objects reference existing slicestacks
    for object in &model.resources.objects {
        if let Some(slicestackid) = object.slicestackid {
            if !valid_slicestack_ids.contains(&slicestackid) {
                let available_ids = sorted_ids_from_set(&valid_slicestack_ids);
                return Err(Error::InvalidModel(format!(
                    "Object {}: References non-existent slicestackid {}.\n\
                     Per 3MF Slice Extension spec, the slicestackid attribute must reference \
                     a valid <slicestack> resource defined in the model.\n\
                     Available slicestack IDs: {:?}",
                    object.id, slicestackid, available_ids
                )));
            }
        }
    }

    // Find all objects that reference slicestacks
    let mut objects_with_slices: Vec<&crate::model::Object> = Vec::new();
    for object in &model.resources.objects {
        if object.slicestackid.is_some() {
            objects_with_slices.push(object);
        }
    }

    // If no objects reference slicestacks, we're done
    if objects_with_slices.is_empty() {
        return Ok(());
    }

    // Validate transforms for build items that reference objects with slicestacks
    for item in &model.build.items {
        // Check if this build item references an object with a slicestack
        let object_has_slicestack = objects_with_slices
            .iter()
            .any(|obj| obj.id == item.objectid);

        if !object_has_slicestack {
            continue;
        }

        // If object has slicestack, validate that transform is planar
        if let Some(ref transform) = item.transform {
            validate_planar_transform(
                transform,
                &format!("Build Item referencing object {}", item.objectid),
            )?;
        }
    }

    // Also validate transforms in components that reference objects with slicestacks
    for object in &model.resources.objects {
        for component in &object.components {
            // Check if this component references an object with a slicestack
            let component_has_slicestack = objects_with_slices
                .iter()
                .any(|obj| obj.id == component.objectid);

            if !component_has_slicestack {
                continue;
            }

            // If component references object with slicestack, validate transform
            if let Some(ref transform) = component.transform {
                validate_planar_transform(
                    transform,
                    &format!(
                        "Object {}, Component referencing object {}",
                        object.id, component.objectid
                    ),
                )?;
            }
        }
    }

    Ok(())
}

/// Validate a single slice
fn validate_slice(
    slice_stack_id: usize,
    slice_idx: usize,
    slice: &crate::model::Slice,
) -> Result<()> {
    // Per 3MF Slice Extension spec and official test suite:
    // Empty slices (no polygons) are allowed - they can represent empty layers
    // or boundaries of the sliced object. However, if a slice has polygons,
    // it must have vertices.

    // If slice is empty (no polygons), it's valid - skip further validation
    if slice.polygons.is_empty() {
        return Ok(());
    }

    // If there are polygons, there must be vertices
    if slice.vertices.is_empty() {
        return Err(Error::InvalidModel(format!(
            "SliceStack {}: Slice {} (ztop={}) has {} polygon(s) but no vertices. \
             Per 3MF Slice Extension spec, slices with polygons must have vertex data. \
             Add vertices to the slice.",
            slice_stack_id,
            slice_idx,
            slice.ztop,
            slice.polygons.len()
        )));
    }

    let num_vertices = slice.vertices.len();

    // Validate polygon vertex indices
    for (poly_idx, polygon) in slice.polygons.iter().enumerate() {
        // Validate startv index
        if polygon.startv >= num_vertices {
            return Err(Error::InvalidModel(format!(
                "SliceStack {}: Slice {} (ztop={}), Polygon {} has invalid startv={} \
                 (slice has {} vertices, valid indices: 0-{}). \
                 Vertex indices must reference valid vertices in the slice.",
                slice_stack_id,
                slice_idx,
                slice.ztop,
                poly_idx,
                polygon.startv,
                num_vertices,
                num_vertices - 1
            )));
        }

        // N_SPX_1609_01: Validate polygon has at least 2 segments (not a single point)
        // A valid polygon needs at least 2 segments to form a shape
        if polygon.segments.len() < 2 {
            return Err(Error::InvalidModel(format!(
                "SliceStack {}: Slice {} (ztop={}), Polygon {} has only {} segment(s).\n\
                 Per 3MF Slice Extension spec, a polygon must have at least 2 segments to form a valid shape.",
                slice_stack_id,
                slice_idx,
                slice.ztop,
                poly_idx,
                polygon.segments.len()
            )));
        }

        // Validate segment v2 indices and check for duplicates
        let mut prev_v2: Option<usize> = None;
        for (seg_idx, segment) in polygon.segments.iter().enumerate() {
            if segment.v2 >= num_vertices {
                return Err(Error::InvalidModel(format!(
                    "SliceStack {}: Slice {} (ztop={}), Polygon {}, Segment {} has invalid v2={} \
                     (slice has {} vertices, valid indices: 0-{}). \
                     Vertex indices must reference valid vertices in the slice.",
                    slice_stack_id,
                    slice_idx,
                    slice.ztop,
                    poly_idx,
                    seg_idx,
                    segment.v2,
                    num_vertices,
                    num_vertices - 1
                )));
            }

            // N_SPX_1608_01: Check for duplicate v2 in consecutive segments
            if let Some(prev) = prev_v2 {
                if segment.v2 == prev {
                    return Err(Error::InvalidModel(format!(
                        "SliceStack {}: Slice {} (ztop={}), Polygon {}, Segments {} and {} have the same v2={}.\n\
                         Per 3MF Slice Extension spec, consecutive segments cannot reference the same vertex.",
                        slice_stack_id,
                        slice_idx,
                        slice.ztop,
                        poly_idx,
                        seg_idx - 1,
                        seg_idx,
                        segment.v2
                    )));
                }
            }
            prev_v2 = Some(segment.v2);
        }

        // N_SPX_1609_02: Validate polygon is closed (last segment v2 == startv)
        if let Some(last_segment) = polygon.segments.last() {
            if last_segment.v2 != polygon.startv {
                return Err(Error::InvalidModel(format!(
                    "SliceStack {}: Slice {} (ztop={}), Polygon {} is not closed.\n\
                     Last segment v2={} does not equal startv={}.\n\
                     Per 3MF Slice Extension spec, polygons must be closed (last segment must connect back to start vertex).",
                    slice_stack_id,
                    slice_idx,
                    slice.ztop,
                    poly_idx,
                    last_segment.v2,
                    polygon.startv
                )));
            }
        }
    }

    Ok(())
}

/// Validate that a transform is planar (no Z-axis rotation or shear)
///
/// Per 3MF Slice Extension spec:
/// When an object references slice model data, the 3D transform matrices in <build><item>
/// and <component> elements are limited to those that do not impact the slicing orientation
/// (planar transformations). Therefore, any transform applied (directly or indirectly) to an
/// object that references a <slicestack> MUST have m02, m12, m20, and m21 equal to zero and
/// m22 equal to one.
///
/// Transform matrix layout (3x3 rotation + translation, stored in row-major order as 12 elements):
/// ```text
/// Matrix representation:
/// [m00, m01, m02, tx,
///  m10, m11, m12, ty,
///  m20, m21, m22, tz]
///
/// Array indices:
/// [0:m00, 1:m01, 2:m02, 3:tx,
///  4:m10, 5:m11, 6:m12, 7:ty,
///  8:m20, 9:m21, 10:m22, 11:tz]
/// ```
///
/// For planar transforms:
/// - m02 (index 2), m12 (index 5), m20 (index 6), m21 (index 7) must be exactly 0.0
/// - m22 (index 8) must be exactly 1.0
fn validate_planar_transform(transform: &[f64; 12], context: &str) -> Result<()> {
    // Check m02 (index 2)
    if transform[2] != 0.0 {
        return Err(Error::InvalidModel(format!(
            "{}: Transform is not planar. Matrix element m02 = {} (must be 0.0).\n\
             Per 3MF Slice Extension spec, when an object references a slicestack, \
             transforms must be planar (no Z-axis rotation or shear). Elements m02, m12, m20, m21 \
             must be 0.0 and m22 must be 1.0.",
            context, transform[2]
        )));
    }

    // Check m12 (index 5)
    if transform[5] != 0.0 {
        return Err(Error::InvalidModel(format!(
            "{}: Transform is not planar. Matrix element m12 = {} (must be 0.0).\n\
             Per 3MF Slice Extension spec, when an object references a slicestack, \
             transforms must be planar (no Z-axis rotation or shear). Elements m02, m12, m20, m21 \
             must be 0.0 and m22 must be 1.0.",
            context, transform[5]
        )));
    }

    // Check m20 (index 6)
    if transform[6] != 0.0 {
        return Err(Error::InvalidModel(format!(
            "{}: Transform is not planar. Matrix element m20 = {} (must be 0.0).\n\
             Per 3MF Slice Extension spec, when an object references a slicestack, \
             transforms must be planar (no Z-axis rotation or shear). Elements m02, m12, m20, m21 \
             must be 0.0 and m22 must be 1.0.",
            context, transform[6]
        )));
    }

    // Check m21 (index 7)
    if transform[7] != 0.0 {
        return Err(Error::InvalidModel(format!(
            "{}: Transform is not planar. Matrix element m21 = {} (must be 0.0).\n\
             Per 3MF Slice Extension spec, when an object references a slicestack, \
             transforms must be planar (no Z-axis rotation or shear). Elements m02, m12, m20, m21 \
             must be 0.0 and m22 must be 1.0.",
            context, transform[7]
        )));
    }

    // Check m22 (index 8)
    if transform[8] != 1.0 {
        return Err(Error::InvalidModel(format!(
            "{}: Transform is not planar. Matrix element m22 = {} (must be 1.0).\n\
             Per 3MF Slice Extension spec, when an object references a slicestack, \
             transforms must be planar (no Z-axis rotation or shear). Elements m02, m12, m20, m21 \
             must be 0.0 and m22 must be 1.0.",
            context, transform[8]
        )));
    }

    Ok(())
}

/// Validate beam lattice extension requirements
///
/// Performs validation specific to beam lattice objects according to the
/// Beam Lattice Extension specification, including:
/// - Beam vertex indices must reference valid vertices in the mesh
/// - Clipping mesh references must be valid
/// - Representation mesh references must be valid and not self-referencing
/// - Clipping mode must have an associated clipping mesh
/// - Ball mode is detected (currently unsupported)
/// - Material/property references from beams and beamsets are valid
fn validate_beam_lattice(model: &Model) -> Result<()> {
    // Collect all valid resource IDs (objects, property groups, etc.)
    let mut valid_resource_ids = HashSet::new();

    for obj in &model.resources.objects {
        valid_resource_ids.insert(obj.id as u32);
    }
    for cg in &model.resources.color_groups {
        valid_resource_ids.insert(cg.id as u32);
    }
    for bg in &model.resources.base_material_groups {
        valid_resource_ids.insert(bg.id as u32);
    }
    for tg in &model.resources.texture2d_groups {
        valid_resource_ids.insert(tg.id as u32);
    }
    for c2d in &model.resources.composite_materials {
        valid_resource_ids.insert(c2d.id as u32);
    }
    for mg in &model.resources.multi_properties {
        valid_resource_ids.insert(mg.id as u32);
    }

    // Validate each object with beam lattice
    for (obj_position, object) in model.resources.objects.iter().enumerate() {
        if let Some(ref mesh) = object.mesh {
            if let Some(ref beamset) = mesh.beamset {
                // Validate object type
                // Per spec: "A beamlattice MUST only be added to a mesh object of type 'model' or 'solidsupport'"
                if object.object_type != ObjectType::Model
                    && object.object_type != ObjectType::SolidSupport
                {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: BeamLattice can only be added to objects of type 'model' or 'solidsupport'. \
                         This object has type '{:?}'. Per the Beam Lattice spec, types like 'support' or 'other' are not allowed.",
                        object.id, object.object_type
                    )));
                }

                let vertex_count = mesh.vertices.len();

                // Validate beam vertex indices
                for (beam_idx, beam) in beamset.beams.iter().enumerate() {
                    if beam.v1 >= vertex_count {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Beam {} references invalid vertex index v1={} \
                             (mesh has {} vertices). Beam vertex indices must be less than \
                             the number of vertices in the mesh.",
                            object.id, beam_idx, beam.v1, vertex_count
                        )));
                    }
                    if beam.v2 >= vertex_count {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Beam {} references invalid vertex index v2={} \
                             (mesh has {} vertices). Beam vertex indices must be less than \
                             the number of vertices in the mesh.",
                            object.id, beam_idx, beam.v2, vertex_count
                        )));
                    }

                    // Validate that beam is not self-referencing (v1 != v2)
                    // A beam must connect two different vertices
                    if beam.v1 == beam.v2 {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Beam {} is self-referencing (v1=v2={}). \
                             A beam must connect two different vertices.",
                            object.id, beam_idx, beam.v1
                        )));
                    }

                    // Validate beam material references
                    if let Some(pid) = beam.property_id {
                        if !valid_resource_ids.contains(&pid) {
                            return Err(Error::InvalidModel(format!(
                                "Object {}: Beam {} references non-existent property group ID {}. \
                                 Property group IDs must reference existing color groups, base material groups, \
                                 texture groups, composite materials, or multi-property groups.",
                                object.id, beam_idx, pid
                            )));
                        }
                    }
                }

                // Validate no duplicate beams
                // Two beams are considered duplicates if they connect the same pair of vertices
                // (regardless of order: beam(v1,v2) equals beam(v2,v1))
                let mut seen_beams = HashSet::new();
                for (beam_idx, beam) in beamset.beams.iter().enumerate() {
                    // Normalize beam to sorted order so (1,2) and (2,1) are treated as the same
                    let normalized = if beam.v1 < beam.v2 {
                        (beam.v1, beam.v2)
                    } else {
                        (beam.v2, beam.v1)
                    };

                    if !seen_beams.insert(normalized) {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Beam {} is a duplicate (connects vertices {} and {}). \
                             Each pair of vertices can only be connected by one beam.",
                            object.id, beam_idx, beam.v1, beam.v2
                        )));
                    }
                }

                // Validate that if beamlattice has pid, object must also have pid
                // Per spec requirement: when beamlattice specifies pid, object level pid is required
                if beamset.property_id.is_some() && object.pid.is_none() {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: BeamLattice specifies pid but object does not have pid attribute. \
                         When beamlattice has pid, the object must also specify pid.",
                        object.id
                    )));
                }

                // Validate that if beams or balls have property assignments,
                // then beamlattice or object must have a default pid
                // Per spec: "If this beam lattice contains any beam or ball with assigned properties,
                // the beam lattice or object MUST specify pid and pindex"
                let beams_have_properties = beamset
                    .beams
                    .iter()
                    .any(|b| b.property_id.is_some() || b.p1.is_some() || b.p2.is_some());

                let balls_have_properties = beamset
                    .balls
                    .iter()
                    .any(|b| b.property_id.is_some() || b.property_index.is_some());

                if beams_have_properties || balls_have_properties {
                    let has_default_pid = beamset.property_id.is_some() || object.pid.is_some();
                    if !has_default_pid {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: BeamLattice contains beams or balls with property assignments \
                             but neither the beamlattice nor the object specifies a default pid. \
                             Per the Beam Lattice spec, when beams or balls have assigned properties, \
                             the beamlattice or object MUST specify pid and pindex to act as default values.",
                            object.id
                        )));
                    }
                }

                // Validate beamset references (if any)
                // Beamset refs are indices into the beams array and must be within bounds
                for ref_index in &beamset.beam_set_refs {
                    if *ref_index >= beamset.beams.len() {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: BeamSet reference index {} is out of bounds. \
                             The beamlattice has {} beams (valid indices: 0-{}).",
                            object.id,
                            ref_index,
                            beamset.beams.len(),
                            beamset.beams.len().saturating_sub(1)
                        )));
                    }
                }

                // Validate ball set references (if any)
                // Ball set refs are indices into the balls array and must be within bounds
                for ref_index in &beamset.ball_set_refs {
                    if *ref_index >= beamset.balls.len() {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: BallSet reference index {} is out of bounds. \
                             The beamlattice has {} balls (valid indices: 0-{}).",
                            object.id,
                            ref_index,
                            beamset.balls.len(),
                            beamset.balls.len().saturating_sub(1)
                        )));
                    }
                }

                // Validate balls (from balls sub-extension)
                // First, build set of beam endpoint vertices
                let mut beam_endpoints: HashSet<usize> = HashSet::new();
                for beam in &beamset.beams {
                    beam_endpoints.insert(beam.v1);
                    beam_endpoints.insert(beam.v2);
                }

                for (ball_idx, ball) in beamset.balls.iter().enumerate() {
                    // Validate ball vertex index
                    if ball.vindex >= vertex_count {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Ball {} references invalid vertex index {} \
                             (mesh has {} vertices). Ball vertex indices must be less than \
                             the number of vertices in the mesh.",
                            object.id, ball_idx, ball.vindex, vertex_count
                        )));
                    }

                    // Validate that ball vindex is at a beam endpoint
                    // Per spec requirement: balls must be placed at beam endpoints
                    if !beam_endpoints.contains(&ball.vindex) {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Ball {} at vertex {} is not at a beam endpoint. \
                             Balls must be placed at vertices that are endpoints of beams.",
                            object.id, ball_idx, ball.vindex
                        )));
                    }

                    // Validate ball material references
                    if let Some(ball_pid) = ball.property_id {
                        if !valid_resource_ids.contains(&ball_pid) {
                            return Err(Error::InvalidModel(format!(
                                "Object {}: Ball {} references non-existent property group ID {}. \
                                 Property group IDs must reference existing color groups, base material groups, \
                                 texture groups, composite materials, or multi-property groups.",
                                object.id, ball_idx, ball_pid
                            )));
                        }

                        // Validate ball property index if present
                        if let Some(ball_p) = ball.property_index {
                            // Check if it's a color group
                            if let Some(colorgroup) = model
                                .resources
                                .color_groups
                                .iter()
                                .find(|cg| cg.id as u32 == ball_pid)
                            {
                                if ball_p as usize >= colorgroup.colors.len() {
                                    let max_index = colorgroup.colors.len().saturating_sub(1);
                                    return Err(Error::InvalidModel(format!(
                                        "Object {}: Ball {} property index {} is out of bounds.\n\
                                         Color group {} has {} colors (valid indices: 0-{}).",
                                        object.id,
                                        ball_idx,
                                        ball_p,
                                        ball_pid,
                                        colorgroup.colors.len(),
                                        max_index
                                    )));
                                }
                            }
                            // Check if it's a base material group
                            else if let Some(basematerialgroup) = model
                                .resources
                                .base_material_groups
                                .iter()
                                .find(|bg| bg.id as u32 == ball_pid)
                            {
                                if ball_p as usize >= basematerialgroup.materials.len() {
                                    let max_index =
                                        basematerialgroup.materials.len().saturating_sub(1);
                                    return Err(Error::InvalidModel(format!(
                                        "Object {}: Ball {} property index {} is out of bounds.\n\
                                         Base material group {} has {} materials (valid indices: 0-{}).",
                                        object.id, ball_idx, ball_p, ball_pid, basematerialgroup.materials.len(), max_index
                                    )));
                                }
                            }
                        }
                    }
                }

                // Validate clipping mesh reference
                if let Some(clip_id) = beamset.clipping_mesh_id {
                    if !valid_resource_ids.contains(&clip_id) {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: BeamLattice references non-existent clippingmesh ID {}. \
                             The clippingmesh attribute must reference a valid object resource.",
                            object.id, clip_id
                        )));
                    }

                    // Check for self-reference (clipping mesh cannot be the same object)
                    if clip_id == object.id as u32 {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: BeamLattice clippingmesh references itself. \
                             The clippingmesh cannot be the same object that contains the beamlattice.",
                            object.id
                        )));
                    }

                    // Per spec: "The clippingmesh attribute MUST reference an object id earlier in the file"
                    // This means clippingmesh must be a backward reference (earlier position in objects vector)
                    if let Some(clip_obj_position) = model
                        .resources
                        .objects
                        .iter()
                        .position(|o| o.id as u32 == clip_id)
                    {
                        if clip_obj_position >= obj_position {
                            return Err(Error::InvalidModel(format!(
                                "Object {}: BeamLattice clippingmesh={} is not declared earlier in the file. \
                                 Per the Beam Lattice spec, clippingmesh MUST reference an object that appears earlier \
                                 in the resources section of the 3MF file.",
                                object.id, clip_id
                            )));
                        }
                    }

                    // Check that the referenced object is a mesh object (not a component-only object)
                    // and does not contain a beamlattice
                    if let Some(clip_obj) = model
                        .resources
                        .objects
                        .iter()
                        .find(|o| o.id as u32 == clip_id)
                    {
                        // Object must have a mesh, not just components
                        if clip_obj.mesh.is_none() && !clip_obj.components.is_empty() {
                            return Err(Error::InvalidModel(format!(
                                "Object {}: BeamLattice clippingmesh references object {} which is a component object (no mesh). \
                                 The clippingmesh must reference an object that contains a mesh.",
                                object.id, clip_id
                            )));
                        }

                        // Clipping mesh MUST NOT contain a beamlattice
                        if let Some(ref clip_mesh) = clip_obj.mesh {
                            if clip_mesh.beamset.is_some() {
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: BeamLattice clippingmesh references object {} which contains a beamlattice. \
                                     Per the Beam Lattice spec, clippingmesh objects MUST NOT contain a beamlattice.",
                                    object.id, clip_id
                                )));
                            }
                        }
                    }
                }

                // Validate representation mesh reference
                if let Some(rep_id) = beamset.representation_mesh_id {
                    if !valid_resource_ids.contains(&rep_id) {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: BeamLattice references non-existent representationmesh ID {}. \
                             The representationmesh attribute must reference a valid object resource.",
                            object.id, rep_id
                        )));
                    }

                    // Check for self-reference (representation mesh cannot be the same object)
                    if rep_id == object.id as u32 {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: BeamLattice representationmesh references itself. \
                             The representationmesh cannot be the same object that contains the beamlattice.",
                            object.id
                        )));
                    }

                    // Per spec: "The representationmesh attribute MUST reference an object id earlier in the file"
                    // This means representationmesh must be a backward reference (earlier position in objects vector)
                    if let Some(rep_obj_position) = model
                        .resources
                        .objects
                        .iter()
                        .position(|o| o.id as u32 == rep_id)
                    {
                        if rep_obj_position >= obj_position {
                            return Err(Error::InvalidModel(format!(
                                "Object {}: BeamLattice representationmesh={} is not declared earlier in the file. \
                                 Per the Beam Lattice spec, representationmesh MUST reference an object that appears earlier \
                                 in the resources section of the 3MF file.",
                                object.id, rep_id
                            )));
                        }
                    }

                    // Check that the referenced object is a mesh object (not a component-only object)
                    // and does not contain a beamlattice
                    if let Some(rep_obj) = model
                        .resources
                        .objects
                        .iter()
                        .find(|o| o.id as u32 == rep_id)
                    {
                        // Object must have a mesh, not just components
                        if rep_obj.mesh.is_none() && !rep_obj.components.is_empty() {
                            return Err(Error::InvalidModel(format!(
                                "Object {}: BeamLattice representationmesh references object {} which is a component object (no mesh). \
                                 The representationmesh must reference an object that contains a mesh.",
                                object.id, rep_id
                            )));
                        }

                        // Representation mesh MUST NOT contain a beamlattice
                        if let Some(ref rep_mesh) = rep_obj.mesh {
                            if rep_mesh.beamset.is_some() {
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: BeamLattice representationmesh references object {} which contains a beamlattice. \
                                     Per the Beam Lattice spec, representationmesh objects MUST NOT contain a beamlattice.",
                                    object.id, rep_id
                                )));
                            }
                        }
                    }
                }

                // Validate clipping mode
                if let Some(ref clip_mode) = beamset.clipping_mode {
                    // Check that clipping mode has valid value
                    if clip_mode != "none" && clip_mode != "inside" && clip_mode != "outside" {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: BeamLattice has invalid clippingmode '{}'. \
                             Valid values are: 'none', 'inside', 'outside'.",
                            object.id, clip_mode
                        )));
                    }

                    // If clipping mode is specified (and not 'none'), must have clipping mesh
                    if clip_mode != "none" && beamset.clipping_mesh_id.is_none() {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: BeamLattice has clippingmode='{}' but no clippingmesh attribute. \
                             When clippingmode is specified (other than 'none'), a clippingmesh must be provided.",
                            object.id, clip_mode
                        )));
                    }
                }

                // Validate ball mode - only check if value is valid
                // Valid values are: "none", "all", "mixed"
                // Per Beam Lattice Balls sub-extension spec
                if let Some(ref ball_mode) = beamset.ball_mode {
                    if ball_mode != "none" && ball_mode != "all" && ball_mode != "mixed" {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: BeamLattice has invalid ballmode '{}'. \
                             Valid values are: 'none', 'all', 'mixed'.",
                            object.id, ball_mode
                        )));
                    }

                    // If ballmode is 'all' or 'mixed', ballradius must be specified
                    // Per Beam Lattice Balls sub-extension spec
                    if (ball_mode == "all" || ball_mode == "mixed") && beamset.ball_radius.is_none()
                    {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: BeamLattice has ballmode='{}' but no ballradius attribute. \
                             When ballmode is 'all' or 'mixed', ballradius must be specified.",
                            object.id, ball_mode
                        )));
                    }
                }

                // Validate beamset material reference and property index
                if let Some(pid) = beamset.property_id {
                    if !valid_resource_ids.contains(&pid) {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: BeamLattice references non-existent property group ID {}. \
                             Property group IDs must reference existing color groups, base material groups, \
                             texture groups, composite materials, or multi-property groups.",
                            object.id, pid
                        )));
                    }

                    // Validate beamset pindex if present
                    if let Some(pindex) = beamset.property_index {
                        // Check if it's a color group
                        if let Some(colorgroup) = model
                            .resources
                            .color_groups
                            .iter()
                            .find(|cg| cg.id as u32 == pid)
                        {
                            if pindex as usize >= colorgroup.colors.len() {
                                let max_index = colorgroup.colors.len().saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: BeamLattice pindex {} is out of bounds.\n\
                                     Color group {} has {} colors (valid indices: 0-{}).",
                                    object.id,
                                    pindex,
                                    pid,
                                    colorgroup.colors.len(),
                                    max_index
                                )));
                            }
                        }
                        // Check if it's a base material group
                        else if let Some(basematerialgroup) = model
                            .resources
                            .base_material_groups
                            .iter()
                            .find(|bg| bg.id as u32 == pid)
                        {
                            if pindex as usize >= basematerialgroup.materials.len() {
                                let max_index = basematerialgroup.materials.len().saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: BeamLattice pindex {} is out of bounds.\n\
                                     Base material group {} has {} materials (valid indices: 0-{}).",
                                    object.id, pindex, pid, basematerialgroup.materials.len(), max_index
                                )));
                            }
                        }
                    }
                }

                // Validate beam-level property indices (p1, p2)
                for (beam_idx, beam) in beamset.beams.iter().enumerate() {
                    // Determine which property group to use for validation
                    let pid_to_check = beam.property_id.or(beamset.property_id);

                    if let Some(pid) = pid_to_check {
                        // Check if it's a color group
                        if let Some(colorgroup) = model
                            .resources
                            .color_groups
                            .iter()
                            .find(|cg| cg.id as u32 == pid)
                        {
                            let num_colors = colorgroup.colors.len();

                            // Validate p1
                            if let Some(p1) = beam.p1 {
                                if p1 as usize >= num_colors {
                                    let max_index = num_colors.saturating_sub(1);
                                    return Err(Error::InvalidModel(format!(
                                        "Object {}: Beam {} p1 {} is out of bounds.\n\
                                         Color group {} has {} colors (valid indices: 0-{}).",
                                        object.id, beam_idx, p1, pid, num_colors, max_index
                                    )));
                                }
                            }

                            // Validate p2
                            if let Some(p2) = beam.p2 {
                                if p2 as usize >= num_colors {
                                    let max_index = num_colors.saturating_sub(1);
                                    return Err(Error::InvalidModel(format!(
                                        "Object {}: Beam {} p2 {} is out of bounds.\n\
                                         Color group {} has {} colors (valid indices: 0-{}).",
                                        object.id, beam_idx, p2, pid, num_colors, max_index
                                    )));
                                }
                            }
                        }
                        // Check if it's a base material group
                        else if let Some(basematerialgroup) = model
                            .resources
                            .base_material_groups
                            .iter()
                            .find(|bg| bg.id as u32 == pid)
                        {
                            let num_materials = basematerialgroup.materials.len();

                            // Validate p1
                            if let Some(p1) = beam.p1 {
                                if p1 as usize >= num_materials {
                                    let max_index = num_materials.saturating_sub(1);
                                    return Err(Error::InvalidModel(format!(
                                        "Object {}: Beam {} p1 {} is out of bounds.\n\
                                         Base material group {} has {} materials (valid indices: 0-{}).",
                                        object.id, beam_idx, p1, pid, num_materials, max_index
                                    )));
                                }
                            }

                            // Validate p2
                            if let Some(p2) = beam.p2 {
                                if p2 as usize >= num_materials {
                                    let max_index = num_materials.saturating_sub(1);
                                    return Err(Error::InvalidModel(format!(
                                        "Object {}: Beam {} p2 {} is out of bounds.\n\
                                         Base material group {} has {} materials (valid indices: 0-{}).",
                                        object.id, beam_idx, p2, pid, num_materials, max_index
                                    )));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Validate texture paths contain only valid ASCII characters
///
/// Per 3MF Material Extension spec, texture paths must contain only valid ASCII characters.
/// Non-ASCII characters (like Unicode) in texture paths are not allowed.
fn validate_texture_paths(model: &Model) -> Result<()> {
    // Get list of encrypted files to skip validation for them
    let encrypted_files: Vec<String> = model
        .secure_content
        .as_ref()
        .map(|sc| sc.encrypted_files.clone())
        .unwrap_or_default();

    for texture in &model.resources.texture2d_resources {
        // Skip validation for encrypted files (they may not follow standard paths)
        if encrypted_files.contains(&texture.path) {
            continue;
        }

        // N_XXM_0610_01: Check for empty or invalid texture paths
        if texture.path.is_empty() {
            return Err(Error::InvalidModel(format!(
                "Texture2D resource {}: Path is empty.\n\
                 Per 3MF Material Extension spec, texture path must reference a valid file in the package.",
                texture.id
            )));
        }

        // Check for obviously invalid path patterns (e.g., paths with null bytes, backslashes)
        if texture.path.contains('\0') {
            return Err(Error::InvalidModel(format!(
                "Texture2D resource {}: Path '{}' contains null bytes.\n\
                 Per 3MF Material Extension spec, texture paths must be valid OPC part names.",
                texture.id, texture.path
            )));
        }

        // Per OPC spec, part names should use forward slashes, not backslashes
        if texture.path.contains('\\') {
            return Err(Error::InvalidModel(format!(
                "Texture2D resource {}: Path '{}' contains backslashes.\n\
                 Per OPC specification, part names must use forward slashes ('/') as path separators, not backslashes ('\\').",
                texture.id, texture.path
            )));
        }

        // Check that the path contains only ASCII characters
        if !texture.path.is_ascii() {
            return Err(Error::InvalidModel(format!(
                "Texture2D resource {}: Path '{}' contains non-ASCII characters.\n\
                 Per 3MF Material Extension specification, texture paths must contain only ASCII characters.\n\
                 Hint: Remove Unicode or special characters from the texture path.",
                texture.id, texture.path
            )));
        }

        // Note: The 3MF Materials Extension spec does NOT require texture paths to be in
        // /3D/Texture/ or /3D/Textures/ directories. The spec only requires that:
        // 1. The path attribute specifies the part name of the texture data
        // 2. The texture must be the target of a 3D Texture relationship from the 3D Model part
        // Therefore, we do not validate the directory path here.

        // Validate content type
        let valid_content_types = ["image/png", "image/jpeg"];
        if !valid_content_types.contains(&texture.contenttype.as_str()) {
            return Err(Error::InvalidModel(format!(
                "Texture2D resource {}: Invalid contenttype '{}'.\n\
                 Per 3MF Material Extension spec, texture content type must be 'image/png' or 'image/jpeg'.\n\
                 Update the contenttype attribute to one of the supported values.",
                texture.id, texture.contenttype
            )));
        }
    }
    Ok(())
}

/// Validate color formats in color groups
///
/// Per 3MF Material Extension spec, colors are stored as RGBA tuples (u8, u8, u8, u8).
/// The parser already validates format during parsing, but this provides an additional check.
fn validate_color_formats(model: &Model) -> Result<()> {
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
fn validate_uuid_formats(model: &Model) -> Result<()> {
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
fn validate_production_paths(model: &Model) -> Result<()> {
    // Helper function to validate that a path doesn't reference OPC internal files
    let validate_not_opc_internal = |path: &str, context: &str| -> Result<()> {
        // OPC internal paths that should not be referenced:
        // - /_rels/.rels or any path starting with /_rels/
        // - /[Content_Types].xml

        if path.starts_with("/_rels/") || path == "/_rels" {
            return Err(Error::InvalidModel(format!(
                "{}: Production path '{}' references OPC internal relationships directory.\n\
                 Production paths must not reference package internal files.",
                context, path
            )));
        }

        if path == "/[Content_Types].xml" {
            return Err(Error::InvalidModel(format!(
                "{}: Production path '{}' references OPC content types file.\n\
                 Production paths must not reference package internal files.",
                context, path
            )));
        }

        Ok(())
    };

    // Check all objects
    for object in &model.resources.objects {
        if let Some(ref prod_info) = object.production {
            if let Some(ref path) = prod_info.path {
                validate_not_opc_internal(path, &format!("Object {}", object.id))?;
            }
        }

        // Check components
        for (idx, component) in object.components.iter().enumerate() {
            if let Some(ref prod_info) = component.production {
                if let Some(ref path) = prod_info.path {
                    validate_not_opc_internal(
                        path,
                        &format!("Object {}, Component {}", object.id, idx),
                    )?;
                }
            }
        }
    }

    // Check build items - validate p:path doesn't reference OPC internal files
    for (idx, item) in model.build.items.iter().enumerate() {
        if let Some(ref path) = item.production_path {
            validate_not_opc_internal(path, &format!("Build item {}", idx))?;
        }
    }

    Ok(())
}

/// Validate transform matrices for build items
///
/// Per 3MF spec, transform matrices must have a non-negative determinant.
/// A negative determinant indicates a mirror transformation which would
/// invert the object's orientation (inside-out).
///
/// Exception: For sliced objects (objects with slicestackid), the transform
/// restrictions are different per the 3MF Slice Extension spec. Sliced objects
/// must have planar transforms (validated separately in validate_slice_extension),
/// but can have negative determinants (mirror transformations).
fn validate_transform_matrices(model: &Model) -> Result<()> {
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
fn validate_resource_ordering(model: &Model) -> Result<()> {
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

    Ok(())
}

/// Validate that resource IDs are unique within their namespaces
///
/// Per 3MF spec:
/// - Object IDs must be unique among objects
/// - Property resource IDs (basematerials, colorgroups, texture2d, texture2dgroups,
///   compositematerials, multiproperties) must be unique among property resources
/// - Objects and property resources have SEPARATE ID namespaces and can reuse IDs
fn validate_duplicate_resource_ids(model: &Model) -> Result<()> {
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
fn validate_multiproperties_references(model: &Model) -> Result<()> {
    // Build sets of valid resource IDs
    let base_mat_ids: HashSet<usize> = model
        .resources
        .base_material_groups
        .iter()
        .map(|b| b.id)
        .collect();

    let color_group_ids: HashSet<usize> =
        model.resources.color_groups.iter().map(|c| c.id).collect();

    let tex_group_ids: HashSet<usize> = model
        .resources
        .texture2d_groups
        .iter()
        .map(|t| t.id)
        .collect();

    let composite_ids: HashSet<usize> = model
        .resources
        .composite_materials
        .iter()
        .map(|c| c.id)
        .collect();

    // Validate each multiproperties group
    for multi_props in &model.resources.multi_properties {
        // Track basematerials and colorgroup IDs to detect duplicates
        let mut base_mat_count: HashMap<usize, usize> = HashMap::new();
        let mut color_group_count: HashMap<usize, usize> = HashMap::new();

        for (idx, &pid) in multi_props.pids.iter().enumerate() {
            // Check if PID references a valid resource
            let is_valid = base_mat_ids.contains(&pid)
                || color_group_ids.contains(&pid)
                || tex_group_ids.contains(&pid)
                || composite_ids.contains(&pid);

            if !is_valid {
                return Err(Error::InvalidModel(format!(
                    "MultiProperties {}: PID {} at index {} does not reference a valid resource.\n\
                     Per 3MF spec, multiproperties pids must reference existing basematerials, \
                     colorgroup, texture2dgroup, or compositematerials resources.\n\
                     Ensure resource with ID {} exists in the <resources> section.",
                    multi_props.id, pid, idx, pid
                )));
            }

            // Track basematerials references
            if base_mat_ids.contains(&pid) {
                *base_mat_count.entry(pid).or_insert(0) += 1;

                // N_XXM_0604_03: basematerials MUST be at layer 0 (first position) if included
                // Per 3MF Material Extension spec Chapter 5: "A material, if included, MUST be 
                // positioned as the first element in the list forming the first layer"
                if idx != 0 {
                    return Err(Error::InvalidModel(format!(
                        "MultiProperties {}: basematerials group {} referenced at layer {}.\n\
                         Per 3MF Material Extension spec, basematerials MUST be positioned as the first element \
                         (layer 0) in multiproperties pids when included.\n\
                         Move the basematerials reference to layer 0.",
                        multi_props.id, pid, idx
                    )));
                }
            }

            // Track colorgroup references
            if color_group_ids.contains(&pid) {
                *color_group_count.entry(pid).or_insert(0) += 1;
            }
        }

        // N_XXM_0604_01: Check that at most one colorgroup is referenced
        // Per 3MF Material Extension spec Chapter 5: "The pids list MUST NOT contain 
        // more than one reference to a colorgroup"
        if color_group_count.len() > 1 {
            let color_ids: Vec<usize> = color_group_count.keys().copied().collect();
            return Err(Error::InvalidModel(format!(
                "MultiProperties {}: References multiple colorgroups {:?} in pids.\n\
                 Per 3MF Material Extension spec, multiproperties pids list MUST NOT contain \
                 more than one reference to a colorgroup.\n\
                 Remove all but one colorgroup reference from the pids list.",
                multi_props.id, color_ids
            )));
        }

        // Also check for duplicate references to the same colorgroup
        for (&color_id, &count) in &color_group_count {
            if count > 1 {
                return Err(Error::InvalidModel(format!(
                    "MultiProperties {}: References colorgroup {} multiple times in pids.\n\
                     Per 3MF Material Extension spec, multiproperties cannot reference the same colorgroup \
                     more than once in the pids list.",
                    multi_props.id, color_id
                )));
            }
        }

        // Check for duplicate basematerials references
        for (&base_id, &count) in &base_mat_count {
            if count > 1 {
                return Err(Error::InvalidModel(format!(
                    "MultiProperties {}: References basematerials group {} multiple times in pids.\n\
                     Per 3MF spec, multiproperties cannot reference the same basematerials group \
                     more than once in the pids list.",
                    multi_props.id, base_id
                )));
            }
        }
    }

    Ok(())
}

/// Validate triangle property attributes
///
/// Per 3MF Materials Extension spec section 4.1.1 (Triangle Properties):
/// - Triangles can have per-vertex properties (p1/p2/p3) to specify different properties for each vertex
/// - Partial specification (e.g., only p1 or only p1 and p2) is allowed and commonly used
/// - When unspecified, vertices inherit the default property from pid/pindex or object-level properties
///
/// Real-world usage: Files like kinect_scan.3mf use partial specification extensively (8,682 triangles
/// with only p1 specified), demonstrating this is valid and intentional usage per the spec.
///
/// Note: Earlier interpretation that ALL THREE must be specified was too strict and rejected
/// valid real-world files.
fn validate_triangle_properties(model: &Model) -> Result<()> {
    // Per 3MF Materials Extension spec:
    // - Triangles can have triangle-level properties (pid and/or pindex)
    // - Triangles can have per-vertex properties (p1, p2, p3) which work WITH pid for interpolation
    // - Having both pid and p1/p2/p3 is ALLOWED and is used for per-vertex material interpolation
    //
    // NOTE: After testing against positive test cases, we found that partial per-vertex
    // properties are actually allowed in some scenarios. The validation has been relaxed.

    // Validate object-level properties
    for object in &model.resources.objects {
        // Validate object-level pid/pindex
        if let (Some(pid), Some(pindex)) = (object.pid, object.pindex) {
            // Get the size of the property resource
            let property_size = get_property_resource_size(model, pid)?;

            // Validate pindex is within bounds
            if pindex >= property_size {
                return Err(Error::InvalidModel(format!(
                    "Object {} has pindex {} which is out of bounds. \
                     Property resource {} has only {} elements (valid indices: 0-{}).",
                    object.id,
                    pindex,
                    pid,
                    property_size,
                    property_size - 1
                )));
            }
        }

        // Validate triangle-level properties
        if let Some(ref mesh) = object.mesh {
            for triangle in &mesh.triangles {
                // N_XXM_0601_01: If triangle has per-vertex material properties (p1/p2/p3)
                // but no triangle-level pid AND object has no default pid, this is invalid
                // Per 3MF Material Extension spec, per-vertex properties need a pid context,
                // either from the triangle itself or from the object's default
                let has_per_vertex_properties =
                    triangle.p1.is_some() || triangle.p2.is_some() || triangle.p3.is_some();

                if has_per_vertex_properties && triangle.pid.is_none() && object.pid.is_none() {
                    return Err(Error::InvalidModel(format!(
                        "Triangle in object {} has per-vertex material properties (p1/p2/p3) \
                         but neither the triangle nor the object has a pid to provide material context.\n\
                         Per 3MF Material Extension spec, per-vertex properties require a pid, \
                         either on the triangle or as a default on the object.\n\
                         Add a pid attribute to either the triangle or object {}.",
                        object.id, object.id
                    )));
                }

                // Validate triangle pindex is within bounds for multi-properties
                if let (Some(pid), Some(pindex)) = (triangle.pid, triangle.pindex) {
                    // Check if pid references a multiproperties resource
                    if let Some(multi_props) = model
                        .resources
                        .multi_properties
                        .iter()
                        .find(|m| m.id == pid)
                    {
                        // pindex must be within bounds of the multiproperties entries
                        if pindex >= multi_props.multis.len() {
                            return Err(Error::InvalidModel(format!(
                                "Triangle in object {} has pindex {} which is out of bounds. \
                                 MultiProperties resource {} has only {} entries (valid indices: 0-{}).",
                                object.id, pindex, pid, multi_props.multis.len(), multi_props.multis.len() - 1
                            )));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Helper function to get the size of a property resource (number of entries)
fn get_property_resource_size(model: &Model, resource_id: usize) -> Result<usize> {
    // Check colorgroup
    if let Some(color_group) = model
        .resources
        .color_groups
        .iter()
        .find(|c| c.id == resource_id)
    {
        if color_group.colors.is_empty() {
            return Err(Error::InvalidModel(format!(
                "ColorGroup {} has no colors. Per 3MF Materials Extension spec, \
                 color groups must contain at least one color element.",
                resource_id
            )));
        }
        return Ok(color_group.colors.len());
    }

    // Check texture2dgroup
    if let Some(tex_group) = model
        .resources
        .texture2d_groups
        .iter()
        .find(|t| t.id == resource_id)
    {
        if tex_group.tex2coords.is_empty() {
            return Err(Error::InvalidModel(format!(
                "Texture2DGroup {} has no texture coordinates. Per 3MF Materials Extension spec, \
                 texture2dgroup must contain at least one tex2coord element.",
                resource_id
            )));
        }
        return Ok(tex_group.tex2coords.len());
    }

    // Check compositematerials
    if let Some(composite) = model
        .resources
        .composite_materials
        .iter()
        .find(|c| c.id == resource_id)
    {
        if composite.composites.is_empty() {
            return Err(Error::InvalidModel(format!(
                "CompositeMaterials {} has no composite elements. Per 3MF Materials Extension spec, \
                 compositematerials must contain at least one composite element.",
                resource_id
            )));
        }
        return Ok(composite.composites.len());
    }

    // Check basematerials
    if let Some(base_mat) = model
        .resources
        .base_material_groups
        .iter()
        .find(|b| b.id == resource_id)
    {
        if base_mat.materials.is_empty() {
            return Err(Error::InvalidModel(format!(
                "BaseMaterials {} has no base material elements. Per 3MF spec, \
                 basematerials must contain at least one base element.",
                resource_id
            )));
        }
        return Ok(base_mat.materials.len());
    }

    // Check multiproperties
    if let Some(multi_props) = model
        .resources
        .multi_properties
        .iter()
        .find(|m| m.id == resource_id)
    {
        if multi_props.multis.is_empty() {
            return Err(Error::InvalidModel(format!(
                "MultiProperties {} has no multi elements. Per 3MF Materials Extension spec, \
                 multiproperties must contain at least one multi element.",
                resource_id
            )));
        }
        return Ok(multi_props.multis.len());
    }

    // Resource not found or not a property resource
    Err(Error::InvalidModel(format!(
        "Property resource {} not found or is not a valid property resource type",
        resource_id
    )))
}

/// Validate production extension UUID usage
///
/// N_XPX_0802_01 and N_XPX_0802_05: Per 3MF Production Extension spec Chapter 4:
/// - Build MUST have p:UUID when production extension is required
/// - Build items MUST have p:UUID when production extension is required
/// - Objects MUST have p:UUID when production extension is required
///
/// Note: The validation for missing UUIDs applies only when production extension
/// is declared as "required" in the model's requiredextensions attribute.
fn validate_production_uuids_required(model: &Model, _config: &ParserConfig) -> Result<()> {
    // Only validate if production extension is explicitly required in the model
    // The config.supports() tells us what the parser accepts, but we need to check
    // what the model file actually requires
    let production_required = model.required_extensions.contains(&Extension::Production);

    if !production_required {
        return Ok(());
    }

    // When production extension is required:
    // 1. Build MUST have UUID (Chapter 4.1) if it has items
    // Per spec, the build UUID is required to identify builds across devices/jobs
    if !model.build.items.is_empty() && model.build.production_uuid.is_none() {
        return Err(Error::InvalidModel(
            "Production extension requires build to have p:UUID attribute when build items are present".to_string(),
        ));
    }

    // 2. Build items MUST have UUID (Chapter 4.1.1)
    for (idx, item) in model.build.items.iter().enumerate() {
        if item.production_uuid.is_none() {
            return Err(Error::InvalidModel(format!(
                "Production extension requires build item {} to have p:UUID attribute",
                idx
            )));
        }
    }

    // 3. Objects MUST have UUID (Chapter 4.2)
    for object in &model.resources.objects {
        // Check if object has production info with UUID
        let has_uuid = object
            .production
            .as_ref()
            .and_then(|p| p.uuid.as_ref())
            .is_some();

        if !has_uuid {
            return Err(Error::InvalidModel(format!(
                "Production extension requires object {} to have p:UUID attribute",
                object.id
            )));
        }
    }

    Ok(())
}

/// N_XPX_0416_01: Validate mesh has positive volume
fn validate_mesh_volume(model: &Model) -> Result<()> {
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
/// cases of inverted meshes.
fn validate_vertex_order(_model: &Model) -> Result<()> {
    Ok(())
}

/// N_XPX_0419_01: Validate JPEG thumbnail colorspace (must be RGB, not CMYK)
///
/// **Note: Partial validation implemented in OPC layer.**
///
/// JPEG CMYK validation is performed in `opc::Package::get_thumbnail_metadata()`
/// where the actual thumbnail file data is available. This placeholder exists
/// for documentation and to maintain the validation function signature.
fn validate_thumbnail_jpeg_colorspace(_model: &Model) -> Result<()> {
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
fn validate_dtd_declaration(_model: &Model) -> Result<()> {
    Ok(())
}

/// N_XPX_0424_01: Validate objects with components don't have pid/pindex attributes
fn validate_component_properties(model: &Model) -> Result<()> {
    // Per 3MF spec, objects that contain components (assemblies) cannot have pid/pindex
    // because assemblies don't have their own material properties
    for object in &model.resources.objects {
        if !object.components.is_empty() {
            if object.pid.is_some() {
                return Err(Error::InvalidModel(format!(
                    "Object {} contains components and cannot have pid attribute",
                    object.id
                )));
            }
            if object.pindex.is_some() {
                return Err(Error::InvalidModel(format!(
                    "Object {} contains components and cannot have pindex attribute",
                    object.id
                )));
            }
        }
    }
    Ok(())
}

/// N_XPX_0802_04 and N_XPX_0802_05: Validate no duplicate UUIDs across all scopes
///
/// Per the 3MF Production Extension specification, UUIDs must be globally unique
/// across all elements in a 3MF package including:
/// - Build element
/// - Build item elements
/// - Object elements
/// - Component elements
fn validate_duplicate_uuids(model: &Model) -> Result<()> {
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
fn validate_component_chain(_model: &Model) -> Result<()> {
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
fn validate_thumbnail_format(_model: &Model) -> Result<()> {
    // Thumbnail validation is limited because the thumbnail path is not stored in the model
    // The parser only tracks whether the attribute was present via has_thumbnail_attribute
    // Full validation would require parsing the thumbnail file itself

    // For now, this is a placeholder for future thumbnail validation
    // The parser already handles the thumbnail attribute appropriately

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BuildItem, Mesh, Multi, MultiProperties, Object, Triangle, Vertex};

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
