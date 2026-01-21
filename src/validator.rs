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
use crate::model::{Model, ParserConfig};
use std::collections::HashSet;

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
pub fn validate_model(model: &Model) -> Result<()> {
    validate_required_structure(model)?;
    validate_object_ids(model)?;
    validate_mesh_geometry(model)?;
    validate_build_references(model)?;
    validate_material_references(model)?;
    validate_boolean_operations(model)?;
    validate_component_references(model)?;
    Ok(())
}

/// Validate a parsed 3MF model with custom extension validation
///
/// This function performs the same validation as `validate_model` and additionally
/// invokes any custom validation handlers registered in the parser configuration.
#[allow(dead_code)] // Currently called during parsing; may be exposed publicly in future
pub fn validate_model_with_config(model: &Model, config: &ParserConfig) -> Result<()> {
    // Standard validation
    validate_model(model)?;

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
    let has_local_objects = !model.resources.objects.is_empty();
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

    // Build section must contain at least one item
    if model.build.items.is_empty() {
        return Err(Error::InvalidModel(
            "Build section must contain at least one item. \
             A valid 3MF file requires at least one <item> element within the <build> section. \
             The build section specifies which objects should be printed."
                .to_string(),
        ));
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

    // Validate that pid and basematerialid references point to existing color groups or base material groups

    // Collect valid color group IDs
    let valid_colorgroup_ids: HashSet<usize> = model
        .resources
        .color_groups
        .iter()
        .map(|cg| cg.id)
        .collect();

    // Collect valid base material group IDs
    let valid_basematerial_ids: HashSet<usize> = model
        .resources
        .base_material_groups
        .iter()
        .map(|bg| bg.id)
        .collect();

    for object in &model.resources.objects {
        if let Some(pid) = object.pid {
            // If object has a pid, it should reference a valid color group or base material group
            let is_valid =
                valid_colorgroup_ids.contains(&pid) || valid_basematerial_ids.contains(&pid);

            // Only validate if there are material groups defined, otherwise pid might be unused
            let has_materials =
                !valid_colorgroup_ids.is_empty() || !valid_basematerial_ids.is_empty();

            if has_materials && !is_valid {
                return Err(Error::InvalidModel(format!(
                    "Object {} references non-existent color group or base material ID: {}",
                    object.id, pid
                )));
            }
        }

        // Validate basematerialid references
        if let Some(basematerialid) = object.basematerialid {
            // basematerialid should reference a valid base material group
            if !valid_basematerial_ids.contains(&basematerialid) {
                return Err(Error::InvalidModel(format!(
                    "Object {} references non-existent base material group ID: {}. \
                     Check that a basematerials group with this ID exists in the resources section.",
                    object.id, basematerialid
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
                        return Err(Error::InvalidModel(format!(
                            "Object {}: pindex {} is out of bounds (color group {} has {} colors)",
                            object.id,
                            pindex,
                            obj_pid,
                            colorgroup.colors.len()
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
                        return Err(Error::InvalidModel(format!(
                            "Object {}: pindex {} is out of bounds (base material group {} has {} materials)",
                            object.id,
                            pindex,
                            obj_pid,
                            basematerialgroup.materials.len()
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
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} pindex {} is out of bounds (color group {} has {} colors)",
                                    object.id, tri_idx, pindex, pid, num_colors
                                )));
                            }
                        }

                        // Validate per-vertex property indices (p1, p2, p3)
                        if let Some(p1) = triangle.p1 {
                            if p1 >= num_colors {
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p1 {} is out of bounds (color group {} has {} colors)",
                                    object.id, tri_idx, p1, pid, num_colors
                                )));
                            }
                        }

                        if let Some(p2) = triangle.p2 {
                            if p2 >= num_colors {
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p2 {} is out of bounds (color group {} has {} colors)",
                                    object.id, tri_idx, p2, pid, num_colors
                                )));
                            }
                        }

                        if let Some(p3) = triangle.p3 {
                            if p3 >= num_colors {
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p3 {} is out of bounds (color group {} has {} colors)",
                                    object.id, tri_idx, p3, pid, num_colors
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
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} pindex {} is out of bounds (base material group {} has {} materials)",
                                    object.id, tri_idx, pindex, pid, num_materials
                                )));
                            }
                        }

                        // Validate per-vertex property indices (p1, p2, p3)
                        if let Some(p1) = triangle.p1 {
                            if p1 >= num_materials {
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p1 {} is out of bounds (base material group {} has {} materials)",
                                    object.id, tri_idx, p1, pid, num_materials
                                )));
                            }
                        }

                        if let Some(p2) = triangle.p2 {
                            if p2 >= num_materials {
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p2 {} is out of bounds (base material group {} has {} materials)",
                                    object.id, tri_idx, p2, pid, num_materials
                                )));
                            }
                        }

                        if let Some(p3) = triangle.p3 {
                            if p3 >= num_materials {
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p3 {} is out of bounds (base material group {} has {} materials)",
                                    object.id, tri_idx, p3, pid, num_materials
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
/// Checks that:
/// - Objects referenced in boolean operations exist (unless they're external via path attribute)
/// - Objects don't have more than one booleanshape element (checked during parsing)
/// - Boolean operand objects exist (unless they're external via path attribute)
fn validate_boolean_operations(model: &Model) -> Result<()> {
    // Build a set of valid object IDs for quick lookup
    let valid_object_ids: HashSet<usize> = model.resources.objects.iter().map(|o| o.id).collect();

    for object in &model.resources.objects {
        if let Some(ref boolean_shape) = object.boolean_shape {
            // Validate the base object ID exists (skip if it has a path to an external file)
            if boolean_shape.path.is_none() && !valid_object_ids.contains(&boolean_shape.objectid) {
                return Err(Error::InvalidModel(format!(
                    "Object {}: Boolean shape references non-existent object ID {}",
                    object.id, boolean_shape.objectid
                )));
            }

            // Validate all operand object IDs exist (skip external references)
            for operand in &boolean_shape.operands {
                if operand.path.is_none() && !valid_object_ids.contains(&operand.objectid) {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Boolean operand references non-existent object ID {}",
                        object.id, operand.objectid
                    )));
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
                return Err(Error::InvalidModel(format!(
                    "Object {}: Component references non-existent object ID {}",
                    object.id, component.objectid
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
}
