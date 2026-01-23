//! Triangle mesh operations using parry3d
//!
//! This module provides geometric operations on triangle meshes including:
//! - Volume computation
//! - Bounding box calculation
//! - Affine transformations
//!
//! These operations are used for validating build items and mesh properties.

use crate::error::{Error, Result};
use crate::model::{Mesh, Model};
use nalgebra::Point3;
use parry3d::shape::{Shape, TriMesh as ParryTriMesh};

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
}
