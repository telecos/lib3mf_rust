//! Triangle mesh operations using parry3d
//!
//! This module provides geometric operations on triangle meshes including:
//! - Volume computation
//! - Bounding box calculation
//! - Affine transformations
//! - Mesh subdivision (midpoint and Loop algorithms)
//! - Vertex normal calculation
//! - Mesh-plane slicing with contour extraction
//!
//! These operations are used for validating build items and mesh properties.

use crate::error::{Error, Result};
use crate::model::{Mesh, Model, Triangle, Vertex};
use nalgebra::Point3;
use parry3d::shape::{Shape, TriMesh as ParryTriMesh};
use std::collections::HashMap;

/// A 3D point represented as (x, y, z)
pub type Point3d = (f64, f64, f64);

/// A 3D vector represented as (x, y, z)
pub type Vector3 = (f64, f64, f64);

/// An axis-aligned bounding box represented as (min_point, max_point)
pub type BoundingBox = (Point3d, Point3d);

/// A 2D point represented as (x, y)
pub type Point2D = (f64, f64);

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

/// Find the line segment where a triangle intersects a horizontal Z plane
///
/// This function computes the intersection of a triangle with a plane at a given Z height.
/// If the triangle crosses the plane, it returns a 2D line segment (in XY coordinates).
///
/// # Arguments
/// * `v0` - First vertex of the triangle
/// * `v1` - Second vertex of the triangle  
/// * `v2` - Third vertex of the triangle
/// * `z` - The Z height of the slicing plane
///
/// # Returns
/// An optional tuple of two 2D points representing the intersection line segment,
/// or None if the triangle doesn't intersect the plane
///
/// # Example
/// ```
/// use lib3mf::{Vertex, mesh_ops::triangle_plane_intersection};
///
/// let v0 = Vertex::new(0.0, 0.0, 0.0);
/// let v1 = Vertex::new(10.0, 0.0, 5.0);
/// let v2 = Vertex::new(5.0, 10.0, 0.0);
///
/// if let Some((p1, p2)) = triangle_plane_intersection(&v0, &v1, &v2, 2.5) {
///     println!("Intersection segment: {:?} to {:?}", p1, p2);
/// }
/// ```
pub fn triangle_plane_intersection(
    v0: &Vertex,
    v1: &Vertex,
    v2: &Vertex,
    z: f64,
) -> Option<(Point2D, Point2D)> {
    let vertices = [v0, v1, v2];
    let mut intersections = Vec::with_capacity(2);

    // Check each edge of the triangle
    for i in 0..3 {
        let va = vertices[i];
        let vb = vertices[(i + 1) % 3];

        // Check if edge crosses the plane
        let za = va.z;
        let zb = vb.z;

        // Skip if both vertices are on the same side or on the plane
        if (za - z) * (zb - z) > 0.0 {
            continue;
        }

        // Handle edge exactly on the plane
        if (za - z).abs() < 1e-10 && (zb - z).abs() < 1e-10 {
            // Both vertices on plane - this is a degenerate case
            // We'll include both points but this will be handled in contour assembly
            intersections.push((va.x, va.y));
            intersections.push((vb.x, vb.y));
            break;
        }

        // Handle single vertex on plane
        if (za - z).abs() < 1e-10 {
            intersections.push((va.x, va.y));
            continue;
        }
        if (zb - z).abs() < 1e-10 {
            intersections.push((vb.x, vb.y));
            continue;
        }

        // Compute intersection point via linear interpolation
        let t = (z - za) / (zb - za);
        let x = va.x + t * (vb.x - va.x);
        let y = va.y + t * (vb.y - va.y);
        intersections.push((x, y));
    }

    // We need exactly 2 intersection points to form a line segment
    if intersections.len() >= 2 {
        // Remove duplicates (vertices on the plane might be counted multiple times)
        if intersections.len() > 2 {
            intersections.sort_by(|a, b| {
                a.0.partial_cmp(&b.0)
                    .unwrap()
                    .then(a.1.partial_cmp(&b.1).unwrap())
            });
            intersections.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-10 && (a.1 - b.1).abs() < 1e-10);
        }

        if intersections.len() >= 2 {
            Some((intersections[0], intersections[1]))
        } else {
            None
        }
    } else {
        None
    }
}

/// Collect all intersection segments from a mesh at a given Z height
///
/// This function slices a mesh at a specified Z plane and returns all line segments
/// where triangles intersect the plane.
///
/// # Arguments
/// * `mesh` - The mesh to slice
/// * `z` - The Z height of the slicing plane
///
/// # Returns
/// A vector of 2D line segments representing the intersection
///
/// # Example
/// ```
/// use lib3mf::{Mesh, Vertex, Triangle, mesh_ops::collect_intersection_segments};
///
/// let mut mesh = Mesh::new();
/// // ... add vertices and triangles ...
///
/// let segments = collect_intersection_segments(&mesh, 5.0);
/// println!("Found {} intersection segments", segments.len());
/// ```
pub fn collect_intersection_segments(mesh: &Mesh, z: f64) -> Vec<(Point2D, Point2D)> {
    mesh.triangles
        .iter()
        .filter_map(|tri| {
            // Validate triangle indices
            if tri.v1 >= mesh.vertices.len()
                || tri.v2 >= mesh.vertices.len()
                || tri.v3 >= mesh.vertices.len()
            {
                return None;
            }

            let v0 = &mesh.vertices[tri.v1];
            let v1 = &mesh.vertices[tri.v2];
            let v2 = &mesh.vertices[tri.v3];
            triangle_plane_intersection(v0, v1, v2, z)
        })
        .collect()
}

/// Assemble line segments into closed contour loops
///
/// This function takes a collection of line segments and connects them into
/// closed polygonal contours. Each contour represents a closed loop suitable
/// for filled 2D rendering.
///
/// # Arguments
/// * `segments` - Vector of 2D line segments to assemble
/// * `tolerance` - Distance tolerance for connecting segment endpoints
///
/// # Returns
/// A vector of contours, where each contour is a vector of 2D points
///
/// # Example
/// ```
/// use lib3mf::mesh_ops::assemble_contours;
///
/// let segments = vec![
///     ((0.0, 0.0), (1.0, 0.0)),
///     ((1.0, 0.0), (1.0, 1.0)),
///     ((1.0, 1.0), (0.0, 1.0)),
///     ((0.0, 1.0), (0.0, 0.0)),
/// ];
///
/// let contours = assemble_contours(segments, 1e-6);
/// assert_eq!(contours.len(), 1); // One closed square
/// assert_eq!(contours[0].len(), 4); // Four vertices
/// ```
pub fn assemble_contours(segments: Vec<(Point2D, Point2D)>, tolerance: f64) -> Vec<Vec<Point2D>> {
    if segments.is_empty() {
        return Vec::new();
    }

    let mut remaining_segments: Vec<(Point2D, Point2D)> = segments;
    let mut contours: Vec<Vec<Point2D>> = Vec::new();

    while !remaining_segments.is_empty() {
        // Start a new contour with the first remaining segment
        let first_segment = remaining_segments.remove(0);
        let mut contour = vec![first_segment.0, first_segment.1];
        let start_point = first_segment.0;
        let mut current_point = first_segment.1;

        // Keep trying to extend the contour
        let mut found_connection = true;
        while found_connection && !remaining_segments.is_empty() {
            found_connection = false;

            // Find a segment that connects to the current endpoint
            for i in 0..remaining_segments.len() {
                let segment = remaining_segments[i];
                let dist_to_start = point_distance(current_point, segment.0);
                let dist_to_end = point_distance(current_point, segment.1);

                if dist_to_start <= tolerance {
                    // Connect via segment.0 -> segment.1
                    current_point = segment.1;
                    contour.push(current_point);
                    remaining_segments.remove(i);
                    found_connection = true;
                    break;
                } else if dist_to_end <= tolerance {
                    // Connect via segment.1 -> segment.0 (reversed)
                    current_point = segment.0;
                    contour.push(current_point);
                    remaining_segments.remove(i);
                    found_connection = true;
                    break;
                }
            }

            // Check if we've closed the loop
            if point_distance(current_point, start_point) <= tolerance {
                // Remove the duplicate end point (same as start)
                contour.pop();
                break;
            }
        }

        // Only add the contour if it's closed (or if it's the last one)
        if !contour.is_empty() {
            contours.push(contour);
        }
    }

    contours
}

/// Helper function to compute Euclidean distance between two 2D points
#[inline]
fn point_distance(p1: Point2D, p2: Point2D) -> f64 {
    let dx = p1.0 - p2.0;
    let dy = p1.1 - p2.1;
    (dx * dx + dy * dy).sqrt()
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
        (
            cross.0 / magnitude,
            cross.1 / magnitude,
            cross.2 / magnitude,
        )
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
        let magnitude = (normal.0 * normal.0 + normal.1 * normal.1 + normal.2 * normal.2).sqrt();
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
        let magnitude = (normals[0].0 * normals[0].0
            + normals[0].1 * normals[0].1
            + normals[0].2 * normals[0].2)
            .sqrt();
        assert!((magnitude - 1.0).abs() < 1e-10);
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

    #[test]
    fn test_triangle_plane_intersection_simple() {
        // Triangle that crosses the Z=5 plane
        let v0 = Vertex::new(0.0, 0.0, 0.0);
        let v1 = Vertex::new(10.0, 0.0, 10.0);
        let v2 = Vertex::new(5.0, 10.0, 0.0);

        let result = triangle_plane_intersection(&v0, &v1, &v2, 5.0);
        assert!(result.is_some());

        let (p1, p2) = result.unwrap();
        // Should intersect edge v0-v1 at (5.0, 0.0) and edge v1-v2 at (7.5, 5.0)
        // Verify both points have reasonable values
        assert!(p1.0 >= 0.0 && p1.0 <= 10.0);
        assert!(p1.1 >= 0.0 && p1.1 <= 10.0);
        assert!(p2.0 >= 0.0 && p2.0 <= 10.0);
        assert!(p2.1 >= 0.0 && p2.1 <= 10.0);
    }

    #[test]
    fn test_triangle_plane_intersection_no_intersection() {
        // Triangle completely above the plane
        let v0 = Vertex::new(0.0, 0.0, 10.0);
        let v1 = Vertex::new(10.0, 0.0, 15.0);
        let v2 = Vertex::new(5.0, 10.0, 12.0);

        let result = triangle_plane_intersection(&v0, &v1, &v2, 5.0);
        assert!(result.is_none());

        // Triangle completely below the plane
        let v0 = Vertex::new(0.0, 0.0, 0.0);
        let v1 = Vertex::new(10.0, 0.0, 2.0);
        let v2 = Vertex::new(5.0, 10.0, 1.0);

        let result = triangle_plane_intersection(&v0, &v1, &v2, 5.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_triangle_plane_intersection_vertex_on_plane() {
        // Triangle with one vertex exactly on the plane
        let v0 = Vertex::new(0.0, 0.0, 5.0); // On plane
        let v1 = Vertex::new(10.0, 0.0, 0.0); // Below
        let v2 = Vertex::new(5.0, 10.0, 10.0); // Above

        let result = triangle_plane_intersection(&v0, &v1, &v2, 5.0);
        assert!(result.is_some());
    }

    #[test]
    fn test_collect_intersection_segments() {
        let mut mesh = Mesh::new();

        // Create a simple pyramid
        // Base vertices (Z=0)
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0)); // 0
        mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0)); // 1
        mesh.vertices.push(Vertex::new(10.0, 10.0, 0.0)); // 2
        mesh.vertices.push(Vertex::new(0.0, 10.0, 0.0)); // 3
        // Apex (Z=10)
        mesh.vertices.push(Vertex::new(5.0, 5.0, 10.0)); // 4

        // Base triangles
        mesh.triangles.push(Triangle::new(0, 2, 1));
        mesh.triangles.push(Triangle::new(0, 3, 2));
        // Side triangles
        mesh.triangles.push(Triangle::new(0, 1, 4));
        mesh.triangles.push(Triangle::new(1, 2, 4));
        mesh.triangles.push(Triangle::new(2, 3, 4));
        mesh.triangles.push(Triangle::new(3, 0, 4));

        // Slice at Z=5 (halfway up)
        let segments = collect_intersection_segments(&mesh, 5.0);

        // Should get 4 segments (one per side face of pyramid)
        assert_eq!(segments.len(), 4);
    }

    #[test]
    fn test_assemble_contours_simple_square() {
        // Create a square contour from 4 segments
        let segments = vec![
            ((0.0, 0.0), (1.0, 0.0)),
            ((1.0, 0.0), (1.0, 1.0)),
            ((1.0, 1.0), (0.0, 1.0)),
            ((0.0, 1.0), (0.0, 0.0)),
        ];

        let contours = assemble_contours(segments, 1e-6);

        assert_eq!(contours.len(), 1);
        assert_eq!(contours[0].len(), 4);
    }

    #[test]
    fn test_assemble_contours_unordered_segments() {
        // Segments in random order should still assemble
        let segments = vec![
            ((1.0, 1.0), (0.0, 1.0)),
            ((0.0, 0.0), (1.0, 0.0)),
            ((0.0, 1.0), (0.0, 0.0)),
            ((1.0, 0.0), (1.0, 1.0)),
        ];

        let contours = assemble_contours(segments, 1e-6);

        assert_eq!(contours.len(), 1);
        assert_eq!(contours[0].len(), 4);
    }

    #[test]
    fn test_assemble_contours_multiple_loops() {
        // Two separate squares
        let segments = vec![
            // First square
            ((0.0, 0.0), (1.0, 0.0)),
            ((1.0, 0.0), (1.0, 1.0)),
            ((1.0, 1.0), (0.0, 1.0)),
            ((0.0, 1.0), (0.0, 0.0)),
            // Second square (offset)
            ((5.0, 5.0), (6.0, 5.0)),
            ((6.0, 5.0), (6.0, 6.0)),
            ((6.0, 6.0), (5.0, 6.0)),
            ((5.0, 6.0), (5.0, 5.0)),
        ];

        let contours = assemble_contours(segments, 1e-6);

        assert_eq!(contours.len(), 2);
        for contour in &contours {
            assert_eq!(contour.len(), 4);
        }
    }

    #[test]
    fn test_point_distance() {
        let p1 = (0.0, 0.0);
        let p2 = (3.0, 4.0);
        let dist = point_distance(p1, p2);
        assert!((dist - 5.0).abs() < 1e-10); // 3-4-5 triangle
    }
}
