//! Core validation functions for 3MF models

use crate::error::{Error, Result};
use crate::mesh_ops;
use crate::model::{Extension, Model};
use std::collections::HashSet;

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
                        object.id,
                        tri_idx,
                        triangle.v1,
                        num_vertices,
                        num_vertices - 1
                    )));
                }
                if triangle.v2 >= num_vertices {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Triangle {} vertex v2={} is out of bounds (mesh has {} vertices, valid indices: 0-{}). \
                         Vertex indices must reference valid vertices in the mesh. \
                         Check that all triangle vertex indices are less than the vertex count.",
                        object.id,
                        tri_idx,
                        triangle.v2,
                        num_vertices,
                        num_vertices - 1
                    )));
                }
                if triangle.v3 >= num_vertices {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Triangle {} vertex v3={} is out of bounds (mesh has {} vertices, valid indices: 0-{}). \
                         Vertex indices must reference valid vertices in the mesh. \
                         Check that all triangle vertex indices are less than the vertex count.",
                        object.id,
                        tri_idx,
                        triangle.v3,
                        num_vertices,
                        num_vertices - 1
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

/// Helper function to convert a HashSet of IDs to a sorted Vec for error messages
pub(crate) fn sorted_ids_from_set(ids: &HashSet<usize>) -> Vec<usize> {
    let mut sorted: Vec<usize> = ids.iter().copied().collect();
    sorted.sort();
    sorted
}

/// Validates the required structure of a 3MF model
///
/// Ensures the model contains:
/// - At least one object (either local or external via production path)
/// - At least one build item (unless it's an external file with slice stacks)
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

/// Validates that required extensions are properly declared
///
/// Checks that models using extension-specific features have the corresponding
/// extension in their requiredextensions attribute.
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

/// Validates that object IDs are unique and positive
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

/// Validates transform matrices for build items
///
/// Ensures that:
/// - Transform matrices are non-singular (non-zero determinant)
/// - Transform matrices don't have negative determinants (no mirroring)
///
/// Note: Sliced objects have different transform restrictions validated separately
pub(crate) fn validate_transform_matrices(model: &Model) -> Result<()> {
    // Build a set of object IDs that have slicestacks
    let sliced_object_ids: HashSet<usize> = model
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

/// Validates mesh volume is positive (not inverted)
///
/// Uses signed volume calculation to detect inverted meshes.
/// Skips validation for sliced objects where mesh orientation doesn't matter.
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
