//! Create a sample 3MF file with component hierarchy for testing the slicer
//!
//! This example creates a 3MF file with:
//! - A base cube object (10x10x10mm)
//! - A sphere object (radius 3mm)
//! - A component object that references the sphere with transforms
//! - A build item that references the component object with additional transform

use lib3mf::{BuildItem, Component, Mesh, Model, Object, Triangle, Vertex};
use std::f64::consts::PI;

fn create_cube_mesh(size: f64) -> Mesh {
    let half = size / 2.0;

    let vertices = vec![
        // Bottom face
        Vertex::new(-half, -half, -half),
        Vertex::new(half, -half, -half),
        Vertex::new(half, half, -half),
        Vertex::new(-half, half, -half),
        // Top face
        Vertex::new(-half, -half, half),
        Vertex::new(half, -half, half),
        Vertex::new(half, half, half),
        Vertex::new(-half, half, half),
    ];

    let triangles = vec![
        // Bottom face (z = -half, normal pointing down -Z)
        Triangle::new(0, 2, 1),
        Triangle::new(0, 3, 2),
        // Top face (z = +half, normal pointing up +Z)
        Triangle::new(4, 5, 6),
        Triangle::new(4, 6, 7),
        // Front face (y = -half, normal pointing -Y)
        Triangle::new(0, 1, 5),
        Triangle::new(0, 5, 4),
        // Back face (y = +half, normal pointing +Y)
        Triangle::new(2, 3, 7),
        Triangle::new(2, 7, 6),
        // Left face (x = -half, normal pointing -X)
        Triangle::new(0, 4, 7),
        Triangle::new(0, 7, 3),
        // Right face (x = +half, normal pointing +X)
        Triangle::new(1, 2, 6),
        Triangle::new(1, 6, 5),
    ];

    Mesh {
        vertices,
        triangles,
        beamset: None,
    }
}

fn create_sphere_mesh(radius: f64, segments: usize, rings: usize) -> Mesh {
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();

    // Top vertex
    vertices.push(Vertex::new(0.0, 0.0, radius));

    // Generate vertices for each ring
    for ring in 1..rings {
        let phi = PI * (ring as f64) / (rings as f64);
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();

        for seg in 0..segments {
            let theta = 2.0 * PI * (seg as f64) / (segments as f64);
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            let x = radius * sin_phi * cos_theta;
            let y = radius * sin_phi * sin_theta;
            let z = radius * cos_phi;

            vertices.push(Vertex::new(x, y, z));
        }
    }

    // Bottom vertex
    vertices.push(Vertex::new(0.0, 0.0, -radius));

    // Top cap triangles
    for seg in 0..segments {
        let next = (seg + 1) % segments;
        triangles.push(Triangle::new(0, seg + 1, next + 1));
    }

    // Middle triangles
    for ring in 0..(rings - 2) {
        let ring_start = 1 + ring * segments;
        let next_ring_start = ring_start + segments;

        for seg in 0..segments {
            let next = (seg + 1) % segments;

            let v0 = ring_start + seg;
            let v1 = next_ring_start + seg;
            let v2 = next_ring_start + next;
            let v3 = ring_start + next;

            triangles.push(Triangle::new(v0, v1, v2));
            triangles.push(Triangle::new(v0, v2, v3));
        }
    }

    // Bottom cap triangles
    let last_ring_start = 1 + (rings - 2) * segments;
    let bottom_vertex = vertices.len() - 1;
    for seg in 0..segments {
        let next = (seg + 1) % segments;
        triangles.push(Triangle::new(
            last_ring_start + seg,
            bottom_vertex,
            last_ring_start + next,
        ));
    }

    Mesh {
        vertices,
        triangles,
        beamset: None,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut model = Model::new();

    // Object 1: Base cube (10x10x10mm)
    let mut cube_obj = Object::new(1);
    cube_obj.name = Some("Base Cube".to_string());
    cube_obj.mesh = Some(create_cube_mesh(10.0));

    // Object 2: Sphere (radius 3mm)
    let mut sphere_obj = Object::new(2);
    sphere_obj.name = Some("Sphere".to_string());
    sphere_obj.mesh = Some(create_sphere_mesh(3.0, 16, 12));

    // Object 3: Component assembly - contains 4 spheres positioned at cube corners
    let mut component_obj = Object::new(3);
    component_obj.name = Some("Sphere Assembly".to_string());

    // Add 4 components (spheres at different positions)
    let positions = vec![
        (5.0, 5.0, 5.0),   // Top-right-front
        (-5.0, 5.0, 5.0),  // Top-left-front
        (5.0, -5.0, 5.0),  // Top-right-back
        (-5.0, -5.0, 5.0), // Top-left-back
    ];

    for (x, y, z) in positions {
        // Create transform: identity matrix with translation
        let transform = [
            1.0, 0.0, 0.0, // Row 0
            0.0, 1.0, 0.0, // Row 1
            0.0, 0.0, 1.0, // Row 2
            x, y, z, // Translation
        ];

        component_obj
            .components
            .push(Component::with_transform(2, transform));
    }

    // Add objects to model
    model.resources.objects.push(cube_obj);
    model.resources.objects.push(sphere_obj);
    model.resources.objects.push(component_obj);

    // Add build item - references the component object with a transform that moves it up
    let build_transform = [
        1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0,
        20.0, // Move entire assembly up by 20mm
    ];

    let mut build_item = BuildItem::new(3);
    build_item.transform = Some(build_transform);
    model.build.items.push(build_item);

    // Write to file
    let output_path = "tools/slicer/samples/components/components.3mf";
    model.write_to_file(output_path)?;

    println!("Created component sample 3MF file: {}", output_path);
    println!("  - 1 cube object (10x10x10mm)");
    println!("  - 1 sphere object (radius 3mm)");
    println!("  - 1 component object with 4 sphere instances");
    println!("  - Build item with transform (moved up 20mm)");
    println!("  - Final Z range: approximately 17mm to 29mm");

    Ok(())
}
