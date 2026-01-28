//! Triangle mesh operations using parry3d
//!
//! This module provides geometric operations on triangle meshes including:
//! - Volume computation
//! - Bounding box calculation
//! - Affine transformations
//! - Vertex normal calculation
//!
//! These operations are used for validating build items and mesh properties.

use crate::error::{Error, Result};
use crate::model::{Mesh, Model, Vertex};
use nalgebra::Point3;
use parry3d::shape::{Shape, TriMesh as ParryTriMesh};

/// A 3D point represented as (x, y, z)
pub type Point3d = (f64, f64, f64);

/// A 3D vector represented as (x, y, z)
pub type Vector3 = (f64, f64, f64);

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

/// Helper function to calculate the cross product of two 3D vectors
///
/// Returns the cross product v1 × v2
#[inline]
fn cross_product(v1: (f64, f64, f64), v2: (f64, f64, f64)) -> Vector3 {
    (
        v1.1 * v2.2 - v1.2 * v2.1,
        v1.2 * v2.0 - v1.0 * v2.2,
        v1.0 * v2.1 - v1.1 * v2.0,
    )
}

/// Calculate the normal vector for a single triangle face
///
/// The normal is computed using the cross product of two edges of the triangle.
/// The result is normalized to unit length. If the triangle is degenerate
/// (zero area), returns a zero vector.
///
/// # Arguments
/// * `v0` - First vertex of the triangle
/// * `v1` - Second vertex of the triangle
/// * `v2` - Third vertex of the triangle
///
/// # Returns
/// A normalized vector perpendicular to the triangle face, or (0, 0, 0) for degenerate triangles
///
/// # Example
/// ```
/// use lib3mf::{Vertex, mesh_ops::calculate_face_normal};
///
/// let v0 = Vertex::new(0.0, 0.0, 0.0);
/// let v1 = Vertex::new(1.0, 0.0, 0.0);
/// let v2 = Vertex::new(0.0, 1.0, 0.0);
///
/// let normal = calculate_face_normal(&v0, &v1, &v2);
/// // Normal should point in +Z direction: (0, 0, 1)
/// ```
pub fn calculate_face_normal(v0: &Vertex, v1: &Vertex, v2: &Vertex) -> Vector3 {
    // Calculate edges
    let edge1 = (v1.x - v0.x, v1.y - v0.y, v1.z - v0.z);
    let edge2 = (v2.x - v0.x, v2.y - v0.y, v2.z - v0.z);

    // Calculate cross product: edge1 × edge2
    let cross = cross_product(edge1, edge2);

    // Calculate magnitude
    let magnitude = (cross.0 * cross.0 + cross.1 * cross.1 + cross.2 * cross.2).sqrt();

    // Normalize (return zero vector if degenerate)
    if magnitude > 0.0 {
        (cross.0 / magnitude, cross.1 / magnitude, cross.2 / magnitude)
    } else {
        (0.0, 0.0, 0.0)
    }
}

/// Calculate area-weighted vertex normals for an entire mesh
///
/// For each vertex, computes the average of all adjacent face normals,
/// weighted by the face areas. This produces smooth normals suitable for
/// rendering and displacement mapping.
///
/// The algorithm:
/// 1. For each triangle, calculate its face normal and area
/// 2. Add the area-weighted normal to each of the triangle's vertices
/// 3. Normalize the accumulated normals
///
/// Degenerate triangles (zero area) are skipped. Invalid triangle indices
/// are also skipped. If a vertex is not referenced by any valid triangle,
/// its normal will be (0, 0, 0).
///
/// # Arguments
/// * `mesh` - The mesh to calculate vertex normals for
///
/// # Returns
/// A vector of normalized vertex normals, one per vertex in the mesh.
/// The order matches the mesh's vertex order.
///
/// # Example
/// ```
/// use lib3mf::{Mesh, Vertex, Triangle, mesh_ops::calculate_vertex_normals};
///
/// let mut mesh = Mesh::new();
/// mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
/// mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
/// mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));
/// mesh.triangles.push(Triangle::new(0, 1, 2));
///
/// let normals = calculate_vertex_normals(&mesh);
/// // All three vertices should have similar normals pointing in +Z
/// ```
pub fn calculate_vertex_normals(mesh: &Mesh) -> Vec<Vector3> {
    // Initialize accumulator for each vertex
    let mut normals: Vec<(f64, f64, f64)> = vec![(0.0, 0.0, 0.0); mesh.vertices.len()];

    // Process each triangle
    for triangle in &mesh.triangles {
        // Validate triangle indices
        if triangle.v1 >= mesh.vertices.len()
            || triangle.v2 >= mesh.vertices.len()
            || triangle.v3 >= mesh.vertices.len()
        {
            continue; // Skip invalid triangles
        }

        let v0 = &mesh.vertices[triangle.v1];
        let v1 = &mesh.vertices[triangle.v2];
        let v2 = &mesh.vertices[triangle.v3];

        // Calculate edges
        let edge1 = (v1.x - v0.x, v1.y - v0.y, v1.z - v0.z);
        let edge2 = (v2.x - v0.x, v2.y - v0.y, v2.z - v0.z);

        // Calculate cross product (unnormalized normal)
        // The magnitude of the cross product is 2 * triangle area
        // So we can use the unnormalized cross product directly for area weighting
        let area_weighted_normal = cross_product(edge1, edge2);

        // Skip degenerate triangles
        let magnitude = (area_weighted_normal.0 * area_weighted_normal.0
            + area_weighted_normal.1 * area_weighted_normal.1
            + area_weighted_normal.2 * area_weighted_normal.2)
            .sqrt();

        if magnitude > 0.0 {
            // Add area-weighted normal to each vertex of the triangle
            normals[triangle.v1].0 += area_weighted_normal.0;
            normals[triangle.v1].1 += area_weighted_normal.1;
            normals[triangle.v1].2 += area_weighted_normal.2;

            normals[triangle.v2].0 += area_weighted_normal.0;
            normals[triangle.v2].1 += area_weighted_normal.1;
            normals[triangle.v2].2 += area_weighted_normal.2;

            normals[triangle.v3].0 += area_weighted_normal.0;
            normals[triangle.v3].1 += area_weighted_normal.1;
            normals[triangle.v3].2 += area_weighted_normal.2;
        }
    }

    // Normalize all vertex normals
    normals
        .into_iter()
        .map(|(x, y, z)| {
            let magnitude = (x * x + y * y + z * z).sqrt();
            if magnitude > 0.0 {
                (x / magnitude, y / magnitude, z / magnitude)
            } else {
                (0.0, 0.0, 0.0)
            }
        })
        .collect()
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

    #[test]
    fn test_calculate_face_normal_simple() {
        // Triangle in XY plane with vertices in counter-clockwise order
        let v0 = Vertex::new(0.0, 0.0, 0.0);
        let v1 = Vertex::new(1.0, 0.0, 0.0);
        let v2 = Vertex::new(0.0, 1.0, 0.0);

        let normal = calculate_face_normal(&v0, &v1, &v2);

        // Normal should point in +Z direction
        assert!((normal.0 - 0.0).abs() < 1e-10, "X component: {}", normal.0);
        assert!((normal.1 - 0.0).abs() < 1e-10, "Y component: {}", normal.1);
        assert!((normal.2 - 1.0).abs() < 1e-10, "Z component: {}", normal.2);
    }

    #[test]
    fn test_calculate_face_normal_negative_z() {
        // Triangle in XY plane with vertices in clockwise order (reversed)
        let v0 = Vertex::new(0.0, 0.0, 0.0);
        let v1 = Vertex::new(0.0, 1.0, 0.0);
        let v2 = Vertex::new(1.0, 0.0, 0.0);

        let normal = calculate_face_normal(&v0, &v1, &v2);

        // Normal should point in -Z direction
        assert!((normal.0 - 0.0).abs() < 1e-10);
        assert!((normal.1 - 0.0).abs() < 1e-10);
        assert!((normal.2 - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_calculate_face_normal_arbitrary() {
        // Triangle in 3D space
        let v0 = Vertex::new(1.0, 0.0, 0.0);
        let v1 = Vertex::new(0.0, 1.0, 0.0);
        let v2 = Vertex::new(0.0, 0.0, 1.0);

        let normal = calculate_face_normal(&v0, &v1, &v2);

        // The normal should be normalized
        let magnitude =
            (normal.0 * normal.0 + normal.1 * normal.1 + normal.2 * normal.2).sqrt();
        assert!((magnitude - 1.0).abs() < 1e-10, "Magnitude: {}", magnitude);

        // All components should be equal for this symmetric triangle
        assert!(
            (normal.0 - normal.1).abs() < 1e-10,
            "X: {}, Y: {}",
            normal.0,
            normal.1
        );
        assert!(
            (normal.1 - normal.2).abs() < 1e-10,
            "Y: {}, Z: {}",
            normal.1,
            normal.2
        );
    }

    #[test]
    fn test_calculate_face_normal_degenerate() {
        // Degenerate triangle (all vertices collinear)
        let v0 = Vertex::new(0.0, 0.0, 0.0);
        let v1 = Vertex::new(1.0, 0.0, 0.0);
        let v2 = Vertex::new(2.0, 0.0, 0.0);

        let normal = calculate_face_normal(&v0, &v1, &v2);

        // Should return zero vector
        assert_eq!(normal, (0.0, 0.0, 0.0));
    }

    #[test]
    fn test_calculate_face_normal_zero_area() {
        // Triangle with zero area (duplicate vertices)
        let v0 = Vertex::new(1.0, 2.0, 3.0);
        let v1 = Vertex::new(1.0, 2.0, 3.0);
        let v2 = Vertex::new(4.0, 5.0, 6.0);

        let normal = calculate_face_normal(&v0, &v1, &v2);

        // Should return zero vector
        assert_eq!(normal, (0.0, 0.0, 0.0));
    }

    #[test]
    fn test_calculate_vertex_normals_single_triangle() {
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));

        let normals = calculate_vertex_normals(&mesh);

        assert_eq!(normals.len(), 3);

        // All three vertices should have the same normal pointing in +Z
        for normal in &normals {
            assert!((normal.0 - 0.0).abs() < 1e-10, "X: {}", normal.0);
            assert!((normal.1 - 0.0).abs() < 1e-10, "Y: {}", normal.1);
            assert!((normal.2 - 1.0).abs() < 1e-10, "Z: {}", normal.2);
        }
    }

    #[test]
    fn test_calculate_vertex_normals_cube() {
        // Create a unit cube
        let mut mesh = Mesh::new();

        // Vertices
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0)); // 0
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0)); // 1
        mesh.vertices.push(Vertex::new(1.0, 1.0, 0.0)); // 2
        mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0)); // 3
        mesh.vertices.push(Vertex::new(0.0, 0.0, 1.0)); // 4
        mesh.vertices.push(Vertex::new(1.0, 0.0, 1.0)); // 5
        mesh.vertices.push(Vertex::new(1.0, 1.0, 1.0)); // 6
        mesh.vertices.push(Vertex::new(0.0, 1.0, 1.0)); // 7

        // Triangles (counter-clockwise winding)
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

        let normals = calculate_vertex_normals(&mesh);

        assert_eq!(normals.len(), 8);

        // Each vertex is at the corner of three faces, so the normal should be
        // the normalized average of three perpendicular directions
        // For example, vertex 0 is at (0,0,0) and is part of:
        // - Bottom face (normal: 0, 0, -1)
        // - Front face (normal: 0, -1, 0)
        // - Left face (normal: -1, 0, 0)
        // Average: (-1, -1, -1), normalized: (-1/√3, -1/√3, -1/√3)

        let expected = 1.0 / (3.0_f64).sqrt();

        // Vertex 0: (-1, -1, -1) normalized
        assert!(
            (normals[0].0 - (-expected)).abs() < 1e-10,
            "V0 X: {}",
            normals[0].0
        );
        assert!(
            (normals[0].1 - (-expected)).abs() < 1e-10,
            "V0 Y: {}",
            normals[0].1
        );
        assert!(
            (normals[0].2 - (-expected)).abs() < 1e-10,
            "V0 Z: {}",
            normals[0].2
        );

        // Verify all normals are normalized
        for (i, normal) in normals.iter().enumerate() {
            let magnitude =
                (normal.0 * normal.0 + normal.1 * normal.1 + normal.2 * normal.2).sqrt();
            assert!(
                (magnitude - 1.0).abs() < 1e-10,
                "Vertex {} normal magnitude: {}",
                i,
                magnitude
            );
        }
    }

    #[test]
    fn test_calculate_vertex_normals_empty_mesh() {
        let mesh = Mesh::new();
        let normals = calculate_vertex_normals(&mesh);
        assert_eq!(normals.len(), 0);
    }

    #[test]
    fn test_calculate_vertex_normals_with_degenerate_triangles() {
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));
        mesh.vertices.push(Vertex::new(2.0, 0.0, 0.0)); // collinear with 0,1

        // Valid triangle
        mesh.triangles.push(Triangle::new(0, 1, 2));
        // Degenerate triangle (collinear vertices)
        mesh.triangles.push(Triangle::new(0, 1, 3));

        let normals = calculate_vertex_normals(&mesh);

        assert_eq!(normals.len(), 4);

        // Vertices 0, 1, 2 should have valid normals from the first triangle
        assert!((normals[0].2 - 1.0).abs() < 1e-10);
        assert!((normals[1].2 - 1.0).abs() < 1e-10);
        assert!((normals[2].2 - 1.0).abs() < 1e-10);

        // Vertex 3 is only in the degenerate triangle, should have zero normal
        assert_eq!(normals[3], (0.0, 0.0, 0.0));
    }

    #[test]
    fn test_calculate_vertex_normals_with_invalid_indices() {
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));

        // Valid triangle
        mesh.triangles.push(Triangle::new(0, 1, 2));
        // Triangle with out-of-bounds index (should be skipped)
        mesh.triangles.push(Triangle::new(0, 1, 10));

        let normals = calculate_vertex_normals(&mesh);

        assert_eq!(normals.len(), 3);

        // All three vertices should have normals from only the first triangle
        for normal in &normals {
            assert!((normal.2 - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_calculate_vertex_normals_area_weighting() {
        let mut mesh = Mesh::new();

        // Create a vertex shared by two triangles of different sizes
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0)); // 0 - shared vertex
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0)); // 1
        mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0)); // 2
        mesh.vertices.push(Vertex::new(2.0, 0.0, 0.0)); // 3
        mesh.vertices.push(Vertex::new(0.0, 2.0, 0.0)); // 4

        // Small triangle with area 0.5
        mesh.triangles.push(Triangle::new(0, 1, 2));
        // Large triangle with area 2.0
        mesh.triangles.push(Triangle::new(0, 3, 4));

        let normals = calculate_vertex_normals(&mesh);

        // Both triangles have normals pointing in +Z
        // Vertex 0 normal should still point in +Z (weighted average of same direction)
        assert!((normals[0].0 - 0.0).abs() < 1e-10);
        assert!((normals[0].1 - 0.0).abs() < 1e-10);
        assert!((normals[0].2 - 1.0).abs() < 1e-10);

        // The normal should be normalized
        let magnitude =
            (normals[0].0 * normals[0].0 + normals[0].1 * normals[0].1 + normals[0].2 * normals[0].2)
                .sqrt();
        assert!((magnitude - 1.0).abs() < 1e-10);
    }
}
