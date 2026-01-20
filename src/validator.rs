//! Validation logic for 3MF models
//!
//! This module contains functions to validate 3MF models according to the
//! 3MF Core Specification requirements. Validation ensures that:
//! - All object IDs are unique and positive
//! - Triangle vertex indices reference valid vertices
//! - Triangles are not degenerate (all three vertices must be distinct)
//! - Build items reference existing objects
//! - Material and color group references are valid

use crate::error::{Error, Result};
use crate::model::Model;
use std::collections::HashSet;

/// Validate a parsed 3MF model
///
/// Performs comprehensive validation of the model structure, including:
/// - Required model structure (objects and build items)
/// - Object ID uniqueness
/// - Triangle vertex bounds and degeneracy checks
/// - Build item object references
/// - Material and color group references
/// - Mesh requirements (must have vertices)
pub fn validate_model(model: &Model) -> Result<()> {
    validate_required_structure(model)?;
    validate_object_ids(model)?;
    validate_mesh_geometry(model)?;
    validate_build_references(model)?;
    validate_material_references(model)?;
    Ok(())
}

/// Validate that the model has required structure
///
/// Per 3MF Core spec, a valid model must have:
/// - At least one object in resources
/// - At least one build item
fn validate_required_structure(model: &Model) -> Result<()> {
    // Model must contain at least one object
    if model.resources.objects.is_empty() {
        return Err(Error::InvalidModel(
            "Model must contain at least one object in resources".to_string(),
        ));
    }

    // Build section must contain at least one item
    if model.build.items.is_empty() {
        return Err(Error::InvalidModel(
            "Build section must contain at least one item".to_string(),
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
                "Object ID must be a positive integer".to_string(),
            ));
        }

        // Check for duplicate IDs
        if !seen_ids.insert(object.id) {
            return Err(Error::InvalidModel(format!(
                "Duplicate object ID found: {}",
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
                    "Object {}: Mesh has triangles but no vertices",
                    object.id
                )));
            }

            let num_vertices = mesh.vertices.len();

            for (tri_idx, triangle) in mesh.triangles.iter().enumerate() {
                // Validate vertex indices are within bounds
                if triangle.v1 >= num_vertices {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Triangle {} vertex v1={} is out of bounds (have {} vertices)",
                        object.id, tri_idx, triangle.v1, num_vertices
                    )));
                }
                if triangle.v2 >= num_vertices {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Triangle {} vertex v2={} is out of bounds (have {} vertices)",
                        object.id, tri_idx, triangle.v2, num_vertices
                    )));
                }
                if triangle.v3 >= num_vertices {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Triangle {} vertex v3={} is out of bounds (have {} vertices)",
                        object.id, tri_idx, triangle.v3, num_vertices
                    )));
                }

                // Check for degenerate triangles (two or more vertices are the same)
                if triangle.v1 == triangle.v2
                    || triangle.v2 == triangle.v3
                    || triangle.v1 == triangle.v3
                {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Triangle {} is degenerate (v1={}, v2={}, v3={})",
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
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();

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
                "Object {}: Non-manifold edge ({}, {}) is shared by {} triangles (max 2 allowed)",
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
        if !valid_object_ids.contains(&item.objectid) {
            return Err(Error::InvalidModel(format!(
                "Build item {} references non-existent object ID: {}",
                item_idx, item.objectid
            )));
        }
    }

    Ok(())
}

/// Validate material and color group references
fn validate_material_references(model: &Model) -> Result<()> {
    // For now, just validate that pid references point to existing color groups or materials
    // Full validation would require checking basematerialid attributes on objects

    // Collect valid color group IDs
    let valid_colorgroup_ids: HashSet<usize> = model
        .resources
        .color_groups
        .iter()
        .map(|cg| cg.id)
        .collect();

    for object in &model.resources.objects {
        if let Some(pid) = object.pid {
            // If object has a pid, it should reference a valid color group or material
            // For now we just check color groups
            // TODO: Also validate basematerials references
            if !valid_colorgroup_ids.is_empty() && !valid_colorgroup_ids.contains(&pid) {
                // Only validate if there are color groups defined
                // Empty color groups means we might be using basematerials instead
                return Err(Error::InvalidModel(format!(
                    "Object {} references non-existent color group ID: {}",
                    object.id, pid
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
        }

        // Validate triangle property index references for color groups
        if let Some(ref mesh) = object.mesh {
            for (tri_idx, triangle) in mesh.triangles.iter().enumerate() {
                // Determine which color group to use for validation
                let pid_to_check = triangle.pid.or(object.pid);

                if let Some(pid) = pid_to_check {
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
                }
            }
        }
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
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("has triangles but no vertices"));
    }
}
