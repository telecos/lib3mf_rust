//! Triangle mesh operations using parry3d
//!
//! This module provides geometric operations on triangle meshes including:
//! - Volume computation
//! - Bounding box calculation
//! - Affine transformations
//! - Mesh subdivision (midpoint and Loop algorithms)
//!
//! These operations are used for validating build items and mesh properties.

use crate::error::{Error, Result};
use crate::model::{Mesh, Model, Triangle, Vertex};
use nalgebra::Point3;
use parry3d::shape::{Shape, TriMesh as ParryTriMesh};
use std::collections::HashMap;

/// A 3D point represented as (x, y, z)
pub type Point3d = (f64, f64, f64);

/// An axis-aligned bounding box represented as (min_point, max_point)
pub type BoundingBox = (Point3d, Point3d);

/// Compute the signed volume of a mesh using the divergence theorem
///
/// Returns the signed volume in cubic units. For a watertight mesh with correct winding order,
/// the volume should be positive. Negative volume indicates inverted triangles.
///
/// This uses the classical divergence theorem approach rather than parry3d's mass properties
/// because we need the signed volume to detect mesh orientation issues.
///
/// # Arguments
/// * `mesh` - The mesh to compute volume for
///
/// # Returns
/// The signed volume of the mesh
pub fn compute_mesh_signed_volume(mesh: &Mesh) -> Result<f64> {
    if mesh.vertices.is_empty() || mesh.triangles.is_empty() {
        return Ok(0.0);
    }

    // Calculate signed volume using divergence theorem
    let mut volume = 0.0_f64;
    for triangle in &mesh.triangles {
        if triangle.v1 >= mesh.vertices.len()
            || triangle.v2 >= mesh.vertices.len()
            || triangle.v3 >= mesh.vertices.len()
        {
            continue; // Skip invalid triangles (caught by other validation)
        }

        let v1 = &mesh.vertices[triangle.v1];
        let v2 = &mesh.vertices[triangle.v2];
        let v3 = &mesh.vertices[triangle.v3];

        // Signed volume contribution of this triangle
        volume += v1.x * (v2.y * v3.z - v2.z * v3.y)
            + v2.x * (v3.y * v1.z - v3.z * v1.y)
            + v3.x * (v1.y * v2.z - v1.z * v2.y);
    }
    volume /= 6.0;

    Ok(volume)
}

/// Compute the unsigned volume of a mesh using parry3d
///
/// Returns the absolute volume in cubic units. This is useful for computing
/// the actual volume of a mesh without regard to orientation.
///
/// # Arguments
/// * `mesh` - The mesh to compute volume for
///
/// # Returns
/// The absolute volume of the mesh, or an error if the mesh is invalid
pub fn compute_mesh_volume(mesh: &Mesh) -> Result<f64> {
    if mesh.vertices.is_empty() || mesh.triangles.is_empty() {
        return Ok(0.0);
    }

    // Convert mesh to parry3d format
    let vertices: Vec<Point3<f32>> = mesh
        .vertices
        .iter()
        .map(|v| Point3::new(v.x as f32, v.y as f32, v.z as f32))
        .collect();

    let indices: Vec<[u32; 3]> = mesh
        .triangles
        .iter()
        .map(|t| [t.v1 as u32, t.v2 as u32, t.v3 as u32])
        .collect();

    // Create parry3d TriMesh
    let trimesh = ParryTriMesh::new(vertices, indices);

    // Compute mass properties with density 1.0
    let mass_props = trimesh.mass_properties(1.0);

    // Volume is the mass when density is 1.0
    Ok(mass_props.mass() as f64)
}

/// Compute the axis-aligned bounding box (AABB) of a mesh
///
/// Returns the minimum and maximum corners of the bounding box.
///
/// # Arguments
/// * `mesh` - The mesh to compute the bounding box for
///
/// # Returns
/// A tuple of (min_point, max_point) where each point is (x, y, z)
pub fn compute_mesh_aabb(mesh: &Mesh) -> Result<BoundingBox> {
    if mesh.vertices.is_empty() {
        return Err(Error::InvalidFormat(
            "Cannot compute bounding box of empty mesh".to_string(),
        ));
    }

    // Check for triangles - parry3d requires at least one triangle
    if mesh.triangles.is_empty() {
        return Err(Error::InvalidFormat(
            "Cannot compute bounding box of mesh with no triangles".to_string(),
        ));
    }

    // Convert mesh to parry3d format
    let vertices: Vec<Point3<f32>> = mesh
        .vertices
        .iter()
        .map(|v| Point3::new(v.x as f32, v.y as f32, v.z as f32))
        .collect();

    let indices: Vec<[u32; 3]> = mesh
        .triangles
        .iter()
        .map(|t| [t.v1 as u32, t.v2 as u32, t.v3 as u32])
        .collect();

    // Create parry3d TriMesh
    let trimesh = ParryTriMesh::new(vertices, indices);

    // Get the local AABB
    let aabb = trimesh.local_aabb();

    Ok((
        (aabb.mins.x as f64, aabb.mins.y as f64, aabb.mins.z as f64),
        (aabb.maxs.x as f64, aabb.maxs.y as f64, aabb.maxs.z as f64),
    ))
}

/// Apply an affine transformation matrix to a point
///
/// # Arguments
/// * `point` - The point to transform (x, y, z)
/// * `transform` - 4x3 affine transformation matrix in row-major order
///
/// # Returns
/// The transformed point (x, y, z)
pub fn apply_transform(point: Point3d, transform: &[f64; 12]) -> Point3d {
    let (x, y, z) = point;

    let tx = transform[0] * x + transform[1] * y + transform[2] * z + transform[3];
    let ty = transform[4] * x + transform[5] * y + transform[6] * z + transform[7];
    let tz = transform[8] * x + transform[9] * y + transform[10] * z + transform[11];

    (tx, ty, tz)
}

/// Compute the transformed bounding box of a mesh with an affine transformation
///
/// This applies the transformation to all 8 corners of the original AABB and
/// computes a new AABB that bounds all transformed corners.
///
/// # Arguments
/// * `mesh` - The mesh to compute the bounding box for
/// * `transform` - Optional 4x3 affine transformation matrix
///
/// # Returns
/// A tuple of (min_point, max_point) where each point is (x, y, z)
pub fn compute_transformed_aabb(mesh: &Mesh, transform: Option<&[f64; 12]>) -> Result<BoundingBox> {
    // Get the original AABB
    let (min, max) = compute_mesh_aabb(mesh)?;

    // If no transform, return the original AABB
    let Some(transform) = transform else {
        return Ok((min, max));
    };

    // Transform all 8 corners of the AABB
    let corners = [
        (min.0, min.1, min.2),
        (min.0, min.1, max.2),
        (min.0, max.1, min.2),
        (min.0, max.1, max.2),
        (max.0, min.1, min.2),
        (max.0, min.1, max.2),
        (max.0, max.1, min.2),
        (max.0, max.1, max.2),
    ];

    let transformed_corners: Vec<_> = corners
        .iter()
        .map(|&corner| apply_transform(corner, transform))
        .collect();

    // Find min and max of transformed corners
    let mut result_min = transformed_corners[0];
    let mut result_max = transformed_corners[0];

    for &(x, y, z) in &transformed_corners[1..] {
        result_min.0 = result_min.0.min(x);
        result_min.1 = result_min.1.min(y);
        result_min.2 = result_min.2.min(z);
        result_max.0 = result_max.0.max(x);
        result_max.1 = result_max.1.max(y);
        result_max.2 = result_max.2.max(z);
    }

    Ok((result_min, result_max))
}

/// Compute the overall build volume bounding box
///
/// This computes a bounding box that encompasses all build items in the model,
/// taking into account their transformations.
///
/// # Arguments
/// * `model` - The 3MF model
///
/// # Returns
/// A tuple of (min_point, max_point) representing the overall build volume,
/// or None if there are no build items or meshes
pub fn compute_build_volume(model: &Model) -> Option<BoundingBox> {
    let mut overall_min: Option<Point3d> = None;
    let mut overall_max: Option<Point3d> = None;

    for item in &model.build.items {
        // Find the object for this build item
        let Some(object) = model
            .resources
            .objects
            .iter()
            .find(|obj| obj.id == item.objectid)
        else {
            continue;
        };

        // Get the mesh
        let Some(mesh) = &object.mesh else {
            continue;
        };

        // Compute transformed AABB
        let Ok((min, max)) = compute_transformed_aabb(mesh, item.transform.as_ref()) else {
            continue;
        };

        // Update overall bounds
        match (overall_min, overall_max) {
            (None, None) => {
                overall_min = Some(min);
                overall_max = Some(max);
            }
            (Some(cur_min), Some(cur_max)) => {
                overall_min = Some((
                    cur_min.0.min(min.0),
                    cur_min.1.min(min.1),
                    cur_min.2.min(min.2),
                ));
                overall_max = Some((
                    cur_max.0.max(max.0),
                    cur_max.1.max(max.1),
                    cur_max.2.max(max.2),
                ));
            }
            // This should never happen as we initialize both to None together
            // and update both together, but we handle it gracefully just in case
            _ => {
                overall_min = Some(min);
                overall_max = Some(max);
            }
        }
    }

    match (overall_min, overall_max) {
        (Some(min), Some(max)) => Some((min, max)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Mesh, Triangle, Vertex};

    #[test]
    fn test_compute_mesh_volume_cube() {
        // Create a simple unit cube
        let mut mesh = Mesh::new();

        // Vertices of a unit cube
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0)); // 0
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0)); // 1
        mesh.vertices.push(Vertex::new(1.0, 1.0, 0.0)); // 2
        mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0)); // 3
        mesh.vertices.push(Vertex::new(0.0, 0.0, 1.0)); // 4
        mesh.vertices.push(Vertex::new(1.0, 0.0, 1.0)); // 5
        mesh.vertices.push(Vertex::new(1.0, 1.0, 1.0)); // 6
        mesh.vertices.push(Vertex::new(0.0, 1.0, 1.0)); // 7

        // Triangles forming the cube faces (counter-clockwise winding)
        // Bottom face (z=0)
        mesh.triangles.push(Triangle::new(0, 2, 1));
        mesh.triangles.push(Triangle::new(0, 3, 2));
        // Top face (z=1)
        mesh.triangles.push(Triangle::new(4, 5, 6));
        mesh.triangles.push(Triangle::new(4, 6, 7));
        // Front face (y=0)
        mesh.triangles.push(Triangle::new(0, 1, 5));
        mesh.triangles.push(Triangle::new(0, 5, 4));
        // Back face (y=1)
        mesh.triangles.push(Triangle::new(3, 7, 6));
        mesh.triangles.push(Triangle::new(3, 6, 2));
        // Left face (x=0)
        mesh.triangles.push(Triangle::new(0, 4, 7));
        mesh.triangles.push(Triangle::new(0, 7, 3));
        // Right face (x=1)
        mesh.triangles.push(Triangle::new(1, 2, 6));
        mesh.triangles.push(Triangle::new(1, 6, 5));

        // Test signed volume
        let signed_volume = compute_mesh_signed_volume(&mesh).unwrap();
        assert!(signed_volume > 0.0, "Signed volume should be positive");
        assert!(
            (signed_volume - 1.0).abs() < 0.01,
            "Signed volume: {}",
            signed_volume
        );

        // Test unsigned volume
        let volume = compute_mesh_volume(&mesh).unwrap();
        // Volume of a unit cube should be approximately 1.0
        assert!((volume - 1.0).abs() < 0.01, "Volume: {}", volume);
    }

    #[test]
    fn test_compute_mesh_signed_volume_inverted() {
        // Create a cube with inverted triangles
        let mut mesh = Mesh::new();

        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0)); // 0
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0)); // 1
        mesh.vertices.push(Vertex::new(1.0, 1.0, 0.0)); // 2
        mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0)); // 3
        mesh.vertices.push(Vertex::new(0.0, 0.0, 1.0)); // 4
        mesh.vertices.push(Vertex::new(1.0, 0.0, 1.0)); // 5
        mesh.vertices.push(Vertex::new(1.0, 1.0, 1.0)); // 6
        mesh.vertices.push(Vertex::new(0.0, 1.0, 1.0)); // 7

        // Inverted triangles (clockwise winding) - swap first and third vertices
        mesh.triangles.push(Triangle::new(1, 2, 0)); // inverted
        mesh.triangles.push(Triangle::new(2, 3, 0)); // inverted
        mesh.triangles.push(Triangle::new(6, 5, 4)); // inverted
        mesh.triangles.push(Triangle::new(7, 6, 4)); // inverted
        mesh.triangles.push(Triangle::new(5, 1, 0)); // inverted
        mesh.triangles.push(Triangle::new(4, 5, 0)); // inverted
        mesh.triangles.push(Triangle::new(6, 7, 3)); // inverted
        mesh.triangles.push(Triangle::new(2, 6, 3)); // inverted
        mesh.triangles.push(Triangle::new(7, 4, 0)); // inverted
        mesh.triangles.push(Triangle::new(3, 7, 0)); // inverted
        mesh.triangles.push(Triangle::new(6, 2, 1)); // inverted
        mesh.triangles.push(Triangle::new(5, 6, 1)); // inverted

        let signed_volume = compute_mesh_signed_volume(&mesh).unwrap();
        assert!(
            signed_volume < 0.0,
            "Inverted mesh should have negative signed volume, got {}",
            signed_volume
        );
    }

    #[test]
    fn test_compute_mesh_aabb() {
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(10.0, 5.0, 3.0));
        mesh.vertices.push(Vertex::new(-2.0, 8.0, 1.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));

        let (min, max) = compute_mesh_aabb(&mesh).unwrap();

        assert_eq!(min, (-2.0, 0.0, 0.0));
        assert_eq!(max, (10.0, 8.0, 3.0));
    }

    #[test]
    fn test_apply_transform_identity() {
        let point = (1.0, 2.0, 3.0);
        // Identity transform
        let transform = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];

        let result = apply_transform(point, &transform);
        assert_eq!(result, point);
    }

    #[test]
    fn test_apply_transform_translation() {
        let point = (1.0, 2.0, 3.0);
        // Translation by (5, 10, 15)
        let transform = [1.0, 0.0, 0.0, 5.0, 0.0, 1.0, 0.0, 10.0, 0.0, 0.0, 1.0, 15.0];

        let result = apply_transform(point, &transform);
        assert_eq!(result, (6.0, 12.0, 18.0));
    }

    #[test]
    fn test_compute_transformed_aabb() {
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(10.0, 10.0, 10.0));
        mesh.triangles.push(Triangle::new(0, 1, 0));

        // Translation by (5, 5, 5)
        let transform = [1.0, 0.0, 0.0, 5.0, 0.0, 1.0, 0.0, 5.0, 0.0, 0.0, 1.0, 5.0];

        let (min, max) = compute_transformed_aabb(&mesh, Some(&transform)).unwrap();

        assert_eq!(min, (5.0, 5.0, 5.0));
        assert_eq!(max, (15.0, 15.0, 15.0));
    }

    #[test]
    fn test_empty_mesh_volume() {
        let mesh = Mesh::new();
        let volume = compute_mesh_volume(&mesh).unwrap();
        assert_eq!(volume, 0.0);
    }

    #[test]
    fn test_empty_mesh_aabb() {
        let mesh = Mesh::new();
        let result = compute_mesh_aabb(&mesh);
        assert!(result.is_err());
    }

    #[test]
    fn test_mesh_with_no_triangles_aabb() {
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(10.0, 10.0, 10.0));
        // No triangles added
        let result = compute_mesh_aabb(&mesh);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no triangles"));
    }
}

/// Mesh subdivision method
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubdivisionMethod {
    /// Simple midpoint subdivision (fast, no smoothing)
    /// Each triangle is split into 4 by adding midpoint vertices
    Midpoint,
    /// Loop subdivision (planned - currently same as Midpoint)
    /// TODO: Implement proper Loop subdivision with edge vertex averaging
    Loop,
    /// Catmull-Clark subdivision (for quad-dominant meshes)
    /// Note: Currently not implemented
    CatmullClark,
}

/// Options for mesh subdivision
#[derive(Clone, Debug)]
pub struct SubdivisionOptions {
    /// Subdivision method to use
    pub method: SubdivisionMethod,
    /// Number of subdivision levels to apply
    pub levels: u32,
    /// Whether to preserve boundary edges
    /// Note: Not yet implemented - currently has no effect
    pub preserve_boundaries: bool,
    /// Whether to interpolate UV coordinates
    /// Note: Not yet implemented - currently has no effect
    pub interpolate_uvs: bool,
}

impl Default for SubdivisionOptions {
    fn default() -> Self {
        Self {
            method: SubdivisionMethod::Midpoint,
            levels: 1,
            preserve_boundaries: true,
            interpolate_uvs: true,
        }
    }
}

/// Subdivide a mesh according to the specified options
///
/// # Arguments
/// * `mesh` - The mesh to subdivide
/// * `options` - Subdivision options
///
/// # Returns
/// A new subdivided mesh
///
/// # Example
/// ```
/// use lib3mf::mesh_ops::{subdivide, SubdivisionOptions, SubdivisionMethod};
/// use lib3mf::{Mesh, Vertex, Triangle};
///
/// let mut mesh = Mesh::new();
/// mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
/// mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
/// mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
/// mesh.triangles.push(Triangle::new(0, 1, 2));
///
/// let options = SubdivisionOptions {
///     method: SubdivisionMethod::Midpoint,
///     levels: 1,
///     ..Default::default()
/// };
///
/// let subdivided = subdivide(&mesh, &options);
/// assert_eq!(subdivided.triangles.len(), 4);
/// ```
pub fn subdivide(mesh: &Mesh, options: &SubdivisionOptions) -> Mesh {
    let mut result = mesh.clone();
    for _ in 0..options.levels {
        result = match options.method {
            SubdivisionMethod::Midpoint => subdivide_midpoint(&result),
            SubdivisionMethod::Loop => subdivide_loop(&result),
            SubdivisionMethod::CatmullClark => {
                // Not implemented yet, fall back to midpoint
                subdivide_midpoint(&result)
            }
        };
    }
    result
}

/// Quick midpoint subdivision with specified number of levels
///
/// This is a convenience function for simple midpoint subdivision.
///
/// # Arguments
/// * `mesh` - The mesh to subdivide
/// * `levels` - Number of subdivision levels (0 = no subdivision)
///
/// # Returns
/// A new subdivided mesh
///
/// # Example
/// ```
/// use lib3mf::mesh_ops::subdivide_simple;
/// use lib3mf::{Mesh, Vertex, Triangle};
///
/// let mut mesh = Mesh::new();
/// mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
/// mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
/// mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
/// mesh.triangles.push(Triangle::new(0, 1, 2));
///
/// let subdivided = subdivide_simple(&mesh, 2);
/// assert_eq!(subdivided.triangles.len(), 16); // 1 -> 4 -> 16
/// ```
pub fn subdivide_simple(mesh: &Mesh, levels: u32) -> Mesh {
    let options = SubdivisionOptions {
        method: SubdivisionMethod::Midpoint,
        levels,
        ..Default::default()
    };
    subdivide(mesh, &options)
}

/// Perform simple midpoint subdivision
///
/// Each triangle is split into 4 triangles by adding midpoint vertices:
/// ```text
///     v0
///     /\
///   m2  m0
///   /____\
/// v2  m1  v1
///
/// Becomes 4 triangles:
/// (v0, m0, m2), (m0, v1, m1), (m2, m1, v2), (m0, m1, m2)
/// ```
///
/// Triangle properties (pid, p1, p2, p3) are preserved by duplicating
/// the parent triangle's properties to all child triangles.
///
/// # Arguments
/// * `mesh` - The mesh to subdivide
///
/// # Returns
/// A new mesh with each triangle subdivided into 4 triangles
pub fn subdivide_midpoint(mesh: &Mesh) -> Mesh {
    if mesh.triangles.is_empty() {
        return mesh.clone();
    }

    // Pre-allocate capacity for new mesh
    // Estimate: Original vertices + ~1.5 edges per triangle (Euler's formula for closed meshes)
    // Each edge adds one midpoint vertex
    let estimated_new_vertices = mesh.vertices.len() + (mesh.triangles.len() * 3 / 2);
    let new_triangle_count = mesh.triangles.len() * 4;

    let mut result = Mesh::with_capacity(estimated_new_vertices, new_triangle_count);

    // Copy original vertices
    result.vertices.extend_from_slice(&mesh.vertices);

    // Map to track edge midpoints to avoid duplicates
    // Key: (min_vertex_index, max_vertex_index), Value: midpoint_vertex_index
    let mut edge_midpoints: HashMap<(usize, usize), usize> = HashMap::new();

    for triangle in &mesh.triangles {
        // Get or create midpoint vertices for each edge
        let m0 = get_or_create_midpoint(
            &mut result.vertices,
            &mut edge_midpoints,
            &mesh.vertices,
            triangle.v1,
            triangle.v2,
        );
        let m1 = get_or_create_midpoint(
            &mut result.vertices,
            &mut edge_midpoints,
            &mesh.vertices,
            triangle.v2,
            triangle.v3,
        );
        let m2 = get_or_create_midpoint(
            &mut result.vertices,
            &mut edge_midpoints,
            &mesh.vertices,
            triangle.v3,
            triangle.v1,
        );

        // Create 4 new triangles preserving winding order
        // Corner triangles
        let mut t0 = Triangle::new(triangle.v1, m0, m2);
        let mut t1 = Triangle::new(m0, triangle.v2, m1);
        let mut t2 = Triangle::new(m2, m1, triangle.v3);
        // Center triangle
        let mut t3 = Triangle::new(m0, m1, m2);

        // Preserve triangle properties
        // All child triangles inherit the parent's properties
        for t in [&mut t0, &mut t1, &mut t2, &mut t3] {
            t.pid = triangle.pid;
            t.pindex = triangle.pindex;
            // For vertex-specific properties, we could interpolate
            // but for now we just inherit the parent's properties
            t.p1 = triangle.p1;
            t.p2 = triangle.p2;
            t.p3 = triangle.p3;
        }

        result.triangles.push(t0);
        result.triangles.push(t1);
        result.triangles.push(t2);
        result.triangles.push(t3);
    }

    // Preserve beam lattice if present
    result.beamset = mesh.beamset.clone();

    result
}

/// Get or create a midpoint vertex between two vertices
///
/// Uses a hashmap to avoid creating duplicate vertices for shared edges.
///
/// # Safety
/// This function assumes `v1` and `v2` are valid indices into `original_vertices`.
/// The caller must ensure the mesh has been validated before calling subdivision.
fn get_or_create_midpoint(
    vertices: &mut Vec<Vertex>,
    edge_midpoints: &mut HashMap<(usize, usize), usize>,
    original_vertices: &[Vertex],
    v1: usize,
    v2: usize,
) -> usize {
    // Create a canonical edge key (smaller index first)
    let edge_key = if v1 < v2 { (v1, v2) } else { (v2, v1) };

    // Return existing midpoint if already created
    if let Some(&midpoint_idx) = edge_midpoints.get(&edge_key) {
        return midpoint_idx;
    }

    // Create new midpoint vertex
    let vert1 = &original_vertices[v1];
    let vert2 = &original_vertices[v2];
    let midpoint = Vertex::new(
        (vert1.x + vert2.x) / 2.0,
        (vert1.y + vert2.y) / 2.0,
        (vert1.z + vert2.z) / 2.0,
    );

    let midpoint_idx = vertices.len();
    vertices.push(midpoint);
    edge_midpoints.insert(edge_key, midpoint_idx);

    midpoint_idx
}

/// Perform Loop subdivision for smoother surfaces
///
/// **Note: Loop subdivision is not yet fully implemented.**
/// This function currently performs the same midpoint subdivision as `subdivide_midpoint()`.
/// Proper Loop subdivision would use weighted averaging of neighboring vertices
/// to produce smoother surfaces.
///
/// A full Loop subdivision implementation would require:
/// 1. Build edge-vertex connectivity
/// 2. Compute valence for each vertex
/// 3. Apply Loop weights (beta for old vertices, 3/8, 1/8 for edge vertices)
///
/// For now, this serves as a placeholder for future implementation.
/// Use `SubdivisionMethod::Midpoint` for the same behavior with clearer intent.
///
/// # Arguments
/// * `mesh` - The mesh to subdivide
///
/// # Returns
/// A new subdivided mesh (using midpoint subdivision)
pub fn subdivide_loop(mesh: &Mesh) -> Mesh {
    if mesh.triangles.is_empty() {
        return mesh.clone();
    }

    // TODO: Implement full Loop subdivision with proper vertex weighting
    // For now, use midpoint subdivision
    subdivide_midpoint(mesh)
}

#[cfg(test)]
mod subdivision_tests {
    use super::*;

    #[test]
    fn test_subdivide_simple_single_triangle() {
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));

        let subdivided = subdivide_simple(&mesh, 1);

        // Should have 4 triangles after 1 level of subdivision
        assert_eq!(subdivided.triangles.len(), 4);
        // Original 3 vertices + 3 midpoints = 6 vertices
        assert_eq!(subdivided.vertices.len(), 6);
    }

    #[test]
    fn test_subdivide_midpoint_vertex_count() {
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));

        let subdivided = subdivide_midpoint(&mesh);

        // Check midpoint positions
        // Midpoint of (0,0,0) and (1,0,0) should be (0.5, 0, 0)
        let m0 = &subdivided.vertices[3];
        assert!((m0.x - 0.5).abs() < 1e-10);
        assert!((m0.y - 0.0).abs() < 1e-10);
        assert!((m0.z - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_subdivide_multiple_levels() {
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));

        // Test multiple levels: 1 -> 4 -> 16
        let subdivided = subdivide_simple(&mesh, 2);
        assert_eq!(subdivided.triangles.len(), 16);

        // Test 3 levels: 1 -> 4 -> 16 -> 64
        let subdivided = subdivide_simple(&mesh, 3);
        assert_eq!(subdivided.triangles.len(), 64);
    }

    #[test]
    fn test_subdivide_preserves_properties() {
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));

        let mut triangle = Triangle::new(0, 1, 2);
        triangle.pid = Some(5);
        triangle.p1 = Some(1);
        triangle.p2 = Some(2);
        triangle.p3 = Some(3);
        mesh.triangles.push(triangle);

        let subdivided = subdivide_midpoint(&mesh);

        // All 4 child triangles should inherit parent properties
        for tri in &subdivided.triangles {
            assert_eq!(tri.pid, Some(5));
            assert_eq!(tri.p1, Some(1));
            assert_eq!(tri.p2, Some(2));
            assert_eq!(tri.p3, Some(3));
        }
    }

    #[test]
    fn test_subdivide_empty_mesh() {
        let mesh = Mesh::new();
        let subdivided = subdivide_simple(&mesh, 1);

        assert_eq!(subdivided.vertices.len(), 0);
        assert_eq!(subdivided.triangles.len(), 0);
    }

    #[test]
    fn test_subdivide_two_triangles_shared_edge() {
        let mut mesh = Mesh::new();
        // Create two triangles sharing an edge
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0)); // 0
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0)); // 1
        mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0)); // 2
        mesh.vertices.push(Vertex::new(0.5, -1.0, 0.0)); // 3

        mesh.triangles.push(Triangle::new(0, 1, 2));
        mesh.triangles.push(Triangle::new(0, 3, 1));

        let subdivided = subdivide_midpoint(&mesh);

        // 2 triangles become 8 triangles
        assert_eq!(subdivided.triangles.len(), 8);

        // Should reuse midpoint on shared edge (0,1)
        // Original: 4 vertices
        // Triangle 1 adds: 3 midpoints
        // Triangle 2 adds: 2 midpoints (reuses edge 0-1)
        // Total: 4 + 3 + 2 = 9 vertices
        assert_eq!(subdivided.vertices.len(), 9);
    }

    #[test]
    fn test_subdivide_winding_order() {
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0)); // 0
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0)); // 1
        mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0)); // 2
        mesh.triangles.push(Triangle::new(0, 1, 2));

        let subdivided = subdivide_midpoint(&mesh);

        // Check that all triangles maintain counter-clockwise winding
        // by verifying that the signed area is positive
        for tri in &subdivided.triangles {
            let v0 = &subdivided.vertices[tri.v1];
            let v1 = &subdivided.vertices[tri.v2];
            let v2 = &subdivided.vertices[tri.v3];

            // Compute signed area using cross product
            let area = (v1.x - v0.x) * (v2.y - v0.y) - (v2.x - v0.x) * (v1.y - v0.y);

            // All subdivided triangles should have positive area (CCW winding)
            assert!(area > 0.0, "Triangle winding order not preserved");
        }
    }

    #[test]
    fn test_subdivision_options() {
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));

        let options = SubdivisionOptions {
            method: SubdivisionMethod::Midpoint,
            levels: 2,
            preserve_boundaries: true,
            interpolate_uvs: true,
        };

        let subdivided = subdivide(&mesh, &options);
        assert_eq!(subdivided.triangles.len(), 16);
    }

    #[test]
    fn test_subdivide_loop_basic() {
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));

        let options = SubdivisionOptions {
            method: SubdivisionMethod::Loop,
            levels: 1,
            ..Default::default()
        };

        let subdivided = subdivide(&mesh, &options);

        // Loop subdivision should also produce 4 triangles
        assert_eq!(subdivided.triangles.len(), 4);
    }
}
