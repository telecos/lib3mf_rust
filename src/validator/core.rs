//! Core validation functions for 3MF models

use crate::error::{Error, Result};
use crate::model::Model;
use std::collections::HashSet;

use super::sorted_ids_from_set;

/// Validates mesh geometry for all objects in the model
pub fn validate_mesh_geometry(model: &Model) -> Result<()> {
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

/// Validates that mesh edges are manifold (each edge shared by at most 2 triangles)
pub fn validate_mesh_manifold(object_id: usize, mesh: &crate::model::Mesh) -> Result<()> {
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

/// Validates that all build items reference valid objects
pub fn validate_build_references(model: &Model) -> Result<()> {
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

/// Validates that all component references are valid and non-circular
pub fn validate_component_references(model: &Model) -> Result<()> {
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
            // in the current model's resources.
            //
            // The external file validation is done separately in validate_production_external_paths
            // which checks that:
            // 1. The external file exists
            // 2. The referenced object exists in that file
            // 3. Non-root model files don't have components with p:path (N_XPM_0803_01)
            if component
                .production
                .as_ref()
                .is_some_and(|p| p.path.is_some())
            {
                continue;
            }

            // For local component references (no p:path), verify the object exists
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

/// Detects circular component references using depth-first search
pub fn detect_circular_components(
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

/// Validates that objects with components don't have pid/pindex attributes
pub fn validate_component_properties(model: &Model) -> Result<()> {
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
