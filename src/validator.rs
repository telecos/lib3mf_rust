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
use crate::model::{Extension, Model, ParserConfig};
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
    validate_slice_extension(model)?;

    // Custom extension validation
    for ext_info in config.custom_extensions().values() {
        if let Some(validator) = &ext_info.validation_handler {
            validator(model)
                .map_err(|e| Error::InvalidModel(format!("Custom validation failed: {}", e)))?;
        }
    }

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
    // Validate that color group IDs are unique
    let mut seen_colorgroup_ids = HashSet::new();
    for colorgroup in &model.resources.color_groups {
        if !seen_colorgroup_ids.insert(colorgroup.id) {
            return Err(Error::InvalidModel(format!(
                "Duplicate color group ID: {}. \
                 Each color group must have a unique id attribute. \
                 Check your material definitions for duplicate IDs.",
                colorgroup.id
            )));
        }
    }

    // Validate that pid and basematerialid references point to existing property groups
    // Valid property groups include: color groups, base material groups, multiproperties,
    // texture2d groups, and composite materials

    // Collect all valid property group IDs into a single HashSet for efficient lookup
    let mut valid_property_group_ids: HashSet<usize> = HashSet::new();

    // Add color group IDs
    for cg in &model.resources.color_groups {
        valid_property_group_ids.insert(cg.id);
    }

    // Add base material group IDs
    for bg in &model.resources.base_material_groups {
        valid_property_group_ids.insert(bg.id);
    }

    // Add multiproperties IDs
    for mp in &model.resources.multi_properties {
        valid_property_group_ids.insert(mp.id);
    }

    // Add texture2d group IDs
    for tg in &model.resources.texture2d_groups {
        valid_property_group_ids.insert(tg.id);
    }

    // Add composite materials IDs
    for cm in &model.resources.composite_materials {
        valid_property_group_ids.insert(cm.id);
    }

    // Keep separate base material IDs set for basematerialid validation
    let valid_basematerial_ids: HashSet<usize> = model
        .resources
        .base_material_groups
        .iter()
        .map(|bg| bg.id)
        .collect();

    for object in &model.resources.objects {
        if let Some(pid) = object.pid {
            // If object has a pid, it should reference a valid property group
            // Only validate if there are property groups defined, otherwise pid might be unused
            if !valid_property_group_ids.is_empty() && !valid_property_group_ids.contains(&pid) {
                let available_ids = sorted_ids_from_set(&valid_property_group_ids);
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

                    // Check that base object has a shape (mesh or booleanshape), not just components
                    // Per Boolean Operations spec, the base object "MUST NOT reference a components object"
                    if base_obj.mesh.is_none() && base_obj.boolean_shape.is_none() {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Boolean shape base object {} does not define a shape.\n\
                             Per 3MF Boolean Operations spec, the base object must define a shape \
                             (mesh, booleanshape, or shapes from other extensions), not just an assembly of components.",
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

    // Done processing this object, remove from path
    path.pop();
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
                // Therefore, we do NOT require p:path when p:UUID is present

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
            "Production extension attributes (p:UUID, p:path) are used but production extension is not declared in requiredextensions"
                .to_string(),
        ));
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

        // Validate each sliceref
        for sliceref in &stack.slice_refs {
            // Rule: SliceRef slicepath must be in /2D/ folder
            // Per spec: "For package readability and organization, slice models SHOULD be stored
            // in the 2D folder UNLESS they are part of the root model part."
            // We enforce this as a MUST for external slice files to catch common packaging errors.
            if !sliceref.slicepath.starts_with("/2D/") {
                return Err(Error::InvalidModel(format!(
                    "SliceStack {}: SliceRef references invalid path '{}'.\n\
                     Per 3MF Slice Extension spec, external slice models must be stored in the /2D/ folder. \
                     Slicepath must start with '/2D/'.",
                    stack.id, sliceref.slicepath
                )));
            }

            // Note: We cannot validate that the referenced slicestack doesn't contain slicerefs
            // because the referenced slicestack is in an external file that was loaded separately.
            // The parser already handles loading external slice files, so we trust that validation
            // is performed during parsing of those files.
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

/// Validate that a transform is planar (no Z-axis rotation or shear)
///
/// Per 3MF Slice Extension spec:
/// When an object references slice model data, the 3D transform matrices in <build><item>
/// and <component> elements are limited to those that do not impact the slicing orientation
/// (planar transformations). Therefore, any transform applied (directly or indirectly) to an
/// object that references a <slicestack> MUST have m02, m12, m20, and m21 equal to zero and
/// m22 equal to one.
///
/// Transform matrix layout (row-major, 4x3):
/// ```
/// [m00, m01, m02, m03,
///  m10, m11, m12, m13,
///  m20, m21, m22, m23]
/// ```
///
/// For planar transforms:
/// - m02 (index 2), m12 (index 6), m20 (index 8), m21 (index 9) must be exactly 0.0
/// - m22 (index 10) must be exactly 1.0
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

    // Check m12 (index 6)
    if transform[6] != 0.0 {
        return Err(Error::InvalidModel(format!(
            "{}: Transform is not planar. Matrix element m12 = {} (must be 0.0).\n\
             Per 3MF Slice Extension spec, when an object references a slicestack, \
             transforms must be planar (no Z-axis rotation or shear). Elements m02, m12, m20, m21 \
             must be 0.0 and m22 must be 1.0.",
            context, transform[6]
        )));
    }

    // Check m20 (index 8)
    if transform[8] != 0.0 {
        return Err(Error::InvalidModel(format!(
            "{}: Transform is not planar. Matrix element m20 = {} (must be 0.0).\n\
             Per 3MF Slice Extension spec, when an object references a slicestack, \
             transforms must be planar (no Z-axis rotation or shear). Elements m02, m12, m20, m21 \
             must be 0.0 and m22 must be 1.0.",
            context, transform[8]
        )));
    }

    // Check m21 (index 9)
    if transform[9] != 0.0 {
        return Err(Error::InvalidModel(format!(
            "{}: Transform is not planar. Matrix element m21 = {} (must be 0.0).\n\
             Per 3MF Slice Extension spec, when an object references a slicestack, \
             transforms must be planar (no Z-axis rotation or shear). Elements m02, m12, m20, m21 \
             must be 0.0 and m22 must be 1.0.",
            context, transform[9]
        )));
    }

    // Check m22 (index 10)
    if transform[10] != 1.0 {
        return Err(Error::InvalidModel(format!(
            "{}: Transform is not planar. Matrix element m22 = {} (must be 1.0).\n\
             Per 3MF Slice Extension spec, when an object references a slicestack, \
             transforms must be planar (no Z-axis rotation or shear). Elements m02, m12, m20, m21 \
             must be 0.0 and m22 must be 1.0.",
            context, transform[10]
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BuildItem, Mesh, Object, Triangle, Vertex};

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
}
