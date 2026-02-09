#![no_main]

use libfuzzer_sys::fuzz_target;
use libfuzzer_sys::arbitrary::{Arbitrary, Result, Unstructured};

#[derive(Debug)]
struct FuzzMesh {
    vertices: Vec<(f64, f64, f64)>,
    triangles: Vec<(usize, usize, usize)>,
}

impl<'a> Arbitrary<'a> for FuzzMesh {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let vertex_count = u.int_in_range(0..=100)?;
        let mut vertices = Vec::new();
        for _ in 0..vertex_count {
            vertices.push((u.arbitrary()?, u.arbitrary()?, u.arbitrary()?));
        }
        
        // Generate triangles with indices constrained to valid vertex range
        // This avoids generating meaningless out-of-bounds triangles that would be filtered out,
        // and prevents panics in downstream mesh operations unrelated to the target logic.
        let triangle_count = u.int_in_range(0..=50)?;
        let mut triangles = Vec::new();
        if vertex_count > 0 {
            for _ in 0..triangle_count {
                let v1 = u.int_in_range(0..=(vertex_count - 1))?;
                let v2 = u.int_in_range(0..=(vertex_count - 1))?;
                let v3 = u.int_in_range(0..=(vertex_count - 1))?;
                triangles.push((v1, v2, v3));
            }
        }
        
        Ok(FuzzMesh { vertices, triangles })
    }
}

fuzz_target!(|mesh_data: FuzzMesh| {
    // Fuzz mesh validation and operations
    // This tests mesh validation, volume calculation, AABB, etc.
    
    // Convert to actual mesh structure
    let mut mesh = lib3mf::Mesh::new();
    
    // Add vertices
    for (x, y, z) in mesh_data.vertices.iter() {
        // Skip NaN and infinite values
        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            continue;
        }
        mesh.vertices.push(lib3mf::Vertex::new(*x, *y, *z));
    }
    
    // Add triangles with bounds checking
    for (v1, v2, v3) in mesh_data.triangles.iter() {
        if *v1 < mesh.vertices.len() 
            && *v2 < mesh.vertices.len() 
            && *v3 < mesh.vertices.len() {
            mesh.triangles.push(lib3mf::Triangle::new(*v1, *v2, *v3));
        }
    }
    
    // Try various mesh operations
    let _ = lib3mf::mesh_ops::compute_mesh_volume(&mesh);
    let _ = lib3mf::mesh_ops::compute_mesh_aabb(&mesh);
    let _ = lib3mf::mesh_ops::compute_mesh_signed_volume(&mesh);
    let _ = lib3mf::mesh_ops::calculate_vertex_normals(&mesh);
    
    // Try mesh slicing at various Z values
    if !mesh.vertices.is_empty() {
        let _ = lib3mf::mesh_ops::collect_intersection_segments(&mesh, 0.0);
    }
});
