//! Example demonstrating mesh subdivision for displacement rendering
//!
//! This example shows how to:
//! 1. Create a simple low-poly mesh
//! 2. Subdivide it using different methods and levels
//! 3. Compare the results

use lib3mf::{subdivide, subdivide_simple, Mesh, SubdivisionMethod, SubdivisionOptions, Triangle, Vertex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Mesh Subdivision Example\n");
    println!("========================\n");

    // Create a simple low-poly mesh (single triangle)
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));

    println!("Original mesh:");
    println!("  Vertices: {}", mesh.vertices.len());
    println!("  Triangles: {}", mesh.triangles.len());
    println!();

    // Demonstrate simple subdivision with different levels
    println!("Simple Midpoint Subdivision:");
    println!("----------------------------");
    for level in 1..=4 {
        let subdivided = subdivide_simple(&mesh, level);
        let vertex_count = subdivided.vertices.len();
        let triangle_count = subdivided.triangles.len();
        let multiplier = triangle_count as f64 / mesh.triangles.len() as f64;
        
        println!(
            "  Level {}: {} vertices, {} triangles ({}× increase)",
            level, vertex_count, triangle_count, multiplier as u32
        );
    }
    println!();

    // Create a slightly more complex mesh (two triangles)
    let mut mesh2 = Mesh::new();
    mesh2.vertices.push(Vertex::new(0.0, 0.0, 0.0)); // 0
    mesh2.vertices.push(Vertex::new(10.0, 0.0, 0.0)); // 1
    mesh2.vertices.push(Vertex::new(5.0, 10.0, 0.0)); // 2
    mesh2.vertices.push(Vertex::new(5.0, -10.0, 0.0)); // 3
    mesh2.triangles.push(Triangle::new(0, 1, 2));
    mesh2.triangles.push(Triangle::new(0, 3, 1));

    println!("Two-triangle mesh (sharing an edge):");
    println!("  Original vertices: {}", mesh2.vertices.len());
    println!("  Original triangles: {}", mesh2.triangles.len());
    println!();

    let subdivided2 = subdivide_simple(&mesh2, 1);
    println!("  After 1 level subdivision:");
    println!("    Vertices: {}", subdivided2.vertices.len());
    println!("    Triangles: {}", subdivided2.triangles.len());
    println!("    (Note: Shared edge vertices are reused)");
    println!();

    // Demonstrate subdivision with options
    println!("Using SubdivisionOptions:");
    println!("-------------------------");
    
    let options_midpoint = SubdivisionOptions {
        method: SubdivisionMethod::Midpoint,
        levels: 2,
        preserve_boundaries: true,
        interpolate_uvs: true,
    };
    
    let subdivided_midpoint = subdivide(&mesh, &options_midpoint);
    println!("  Midpoint method (2 levels):");
    println!("    Vertices: {}", subdivided_midpoint.vertices.len());
    println!("    Triangles: {}", subdivided_midpoint.triangles.len());
    println!();

    // Note: Loop subdivision is planned but not yet fully implemented
    // It currently produces the same result as midpoint subdivision
    println!("  Loop method (planned feature, currently same as Midpoint):");
    let options_loop = SubdivisionOptions {
        method: SubdivisionMethod::Loop,
        levels: 1,
        ..Default::default()
    };
    
    let subdivided_loop = subdivide(&mesh, &options_loop);
    println!("    Vertices: {}", subdivided_loop.vertices.len());
    println!("    Triangles: {}", subdivided_loop.triangles.len());
    println!();

    // Demonstrate property preservation
    println!("Property Preservation:");
    println!("---------------------");
    let mut mesh_with_props = Mesh::new();
    mesh_with_props.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh_with_props.vertices.push(Vertex::new(1.0, 0.0, 0.0));
    mesh_with_props.vertices.push(Vertex::new(0.5, 1.0, 0.0));
    
    let mut tri = Triangle::new(0, 1, 2);
    tri.pid = Some(5);
    tri.p1 = Some(1);
    tri.p2 = Some(2);
    tri.p3 = Some(3);
    mesh_with_props.triangles.push(tri);

    println!("  Original triangle properties:");
    println!("    pid: {:?}", mesh_with_props.triangles[0].pid);
    println!("    p1, p2, p3: {:?}, {:?}, {:?}", 
        mesh_with_props.triangles[0].p1,
        mesh_with_props.triangles[0].p2,
        mesh_with_props.triangles[0].p3
    );
    println!();

    let subdivided_props = subdivide_simple(&mesh_with_props, 1);
    println!("  After subdivision (all 4 child triangles):");
    for (i, tri) in subdivided_props.triangles.iter().enumerate() {
        println!("    Triangle {}: pid={:?}, p1={:?}, p2={:?}, p3={:?}",
            i, tri.pid, tri.p1, tri.p2, tri.p3);
    }
    println!();

    // Show growth table
    println!("Vertex Count Growth Table:");
    println!("==========================");
    println!("| Level | Triangles | Multiplier |");
    println!("|-------|-----------|------------|");
    
    let base_triangles = 1000;
    for level in 0..=4 {
        let multiplier = 4_u32.pow(level);
        let triangles = base_triangles * multiplier;
        println!("| {:5} | {:9} | {:10}× |", level, triangles, multiplier);
    }
    println!();

    println!("Use Cases:");
    println!("----------");
    println!("• Displacement map rendering - increase vertex density for detail");
    println!("• Mesh refinement - improve triangle quality before operations");
    println!("• LOD generation - create multiple detail levels");
    println!();
    
    println!("Note:");
    println!("-----");
    println!("• Loop subdivision is planned but not yet fully implemented");
    println!("• It currently produces the same result as midpoint subdivision");
    println!();

    println!("✓ Subdivision demonstration complete!");

    Ok(())
}
