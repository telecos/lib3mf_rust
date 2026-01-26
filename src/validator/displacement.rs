//! Displacement extension validation

use crate::error::{Error, Result};
use crate::model::{Extension, Model};
use std::collections::{HashMap, HashSet};

use super::sorted_ids_from_set;

pub fn validate_displacement_extension(model: &Model) -> Result<()> {
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
