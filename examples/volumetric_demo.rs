//! Volumetric extension demo
//!
//! Demonstrates creating, writing, and reading 3MF files with volumetric data
//! (voxel grids, boundaries, implicit volumes, and property groups).
//!
//! Usage:
//!   cargo run --example volumetric_demo
//!   cargo run --example volumetric_demo test_files/volumetric/simple_volumetric.3mf

use lib3mf::{
    BuildItem, Extension, ImplicitVolume, Mesh, Model, Object, Triangle, Vertex,
    VolumetricBoundary, VolumetricData, VolumetricProperty, VolumetricPropertyGroup, Voxel,
    VoxelGrid,
};
use std::io::Cursor;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        // Read mode: parse an existing 3MF file and display volumetric data
        read_volumetric(&args[1]);
    } else {
        // Create mode: build a model with volumetric data, write it, then read it back
        create_and_roundtrip();
    }
}

/// Read a 3MF file and display its volumetric data
fn read_volumetric(path: &str) {
    println!("=== Reading volumetric data from: {} ===\n", path);

    let file = std::fs::File::open(path).expect("Failed to open file");
    let model = Model::from_reader(file).expect("Failed to parse 3MF file");

    // Display volumetric property groups
    if model.resources.volumetric_property_groups.is_empty() {
        println!("No volumetric property groups found.");
    } else {
        println!(
            "Volumetric property groups: {}",
            model.resources.volumetric_property_groups.len()
        );
        for group in &model.resources.volumetric_property_groups {
            println!("  Property Group ID={}", group.id);
            for prop in &group.properties {
                println!("    index={} value=\"{}\"", prop.index, prop.value);
            }
        }
    }

    println!();

    // Display volumetric data resources
    if model.resources.volumetric_data.is_empty() {
        println!("No volumetric data resources found.");
    } else {
        println!(
            "Volumetric data resources: {}",
            model.resources.volumetric_data.len()
        );
        for vol in &model.resources.volumetric_data {
            println!("  VolumetricData ID={}", vol.id);

            if let Some(ref boundary) = vol.boundary {
                println!(
                    "    Boundary: min=({}, {}, {}) max=({}, {}, {})",
                    boundary.min.0,
                    boundary.min.1,
                    boundary.min.2,
                    boundary.max.0,
                    boundary.max.1,
                    boundary.max.2
                );
            }

            if let Some(ref grid) = vol.voxels {
                println!(
                    "    Voxel Grid: dimensions={}x{}x{}, {} voxels",
                    grid.dimensions.0,
                    grid.dimensions.1,
                    grid.dimensions.2,
                    grid.voxels.len()
                );
                if let Some(spacing) = grid.spacing {
                    println!(
                        "      spacing=({}, {}, {})",
                        spacing.0, spacing.1, spacing.2
                    );
                }
                for (i, voxel) in grid.voxels.iter().enumerate().take(10) {
                    print!(
                        "      voxel[{}]: ({}, {}, {})",
                        i, voxel.position.0, voxel.position.1, voxel.position.2
                    );
                    if let Some(pid) = voxel.property_id {
                        print!(" property={}", pid);
                    }
                    println!();
                }
                if grid.voxels.len() > 10 {
                    println!("      ... and {} more voxels", grid.voxels.len() - 10);
                }
            }

            if let Some(ref implicit) = vol.implicit {
                println!("    Implicit Volume: type=\"{}\"", implicit.implicit_type);
                for (k, v) in &implicit.parameters {
                    println!("      {}=\"{}\"", k, v);
                }
            }
        }
    }
}

/// Create a model with volumetric data, write to 3MF, and read it back
fn create_and_roundtrip() {
    println!("=== Creating model with volumetric data ===\n");

    let mut model = Model::new();
    model.required_extensions.push(Extension::Volumetric);

    // 1. Create a volumetric property group
    let mut prop_group = VolumetricPropertyGroup::new(1);
    prop_group
        .properties
        .push(VolumetricProperty::new(0, "metal".to_string()));
    prop_group
        .properties
        .push(VolumetricProperty::new(1, "plastic".to_string()));
    model.resources.volumetric_property_groups.push(prop_group);
    println!("Added property group (id=1) with 2 properties");

    // 2. Create volumetric data with a voxel grid
    let mut vol_data = VolumetricData::new(2);
    vol_data.boundary = Some(VolumetricBoundary::new((0.0, 0.0, 0.0), (20.0, 20.0, 20.0)));

    let mut grid = VoxelGrid::new((4, 4, 4));
    grid.spacing = Some((5.0, 5.0, 5.0));
    grid.origin = Some((0.0, 0.0, 0.0));

    // Fill some voxels
    let mut voxel = Voxel::new((1, 1, 1));
    voxel.property_id = Some(1); // metal
    grid.voxels.push(voxel);

    let mut voxel = Voxel::new((2, 2, 2));
    voxel.property_id = Some(1); // plastic
    grid.voxels.push(voxel);

    grid.voxels.push(Voxel::new((3, 3, 3)));

    vol_data.voxels = Some(grid);
    model.resources.volumetric_data.push(vol_data);
    println!("Added volumetric data (id=2) with 4x4x4 voxel grid, 3 voxels");

    // 3. Create volumetric data with an implicit volume
    let mut vol_implicit = VolumetricData::new(3);
    let mut implicit = ImplicitVolume::new("sdf".to_string());
    implicit
        .parameters
        .push(("radius".to_string(), "10.0".to_string()));
    implicit
        .parameters
        .push(("center".to_string(), "0 0 0".to_string()));
    vol_implicit.implicit = Some(implicit);
    model.resources.volumetric_data.push(vol_implicit);
    println!("Added volumetric data (id=3) with implicit SDF volume");

    // 4. Add a simple mesh object (required for a valid 3MF)
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(20.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(10.0, 20.0, 0.0));
    mesh.vertices.push(Vertex::new(10.0, 10.0, 20.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));
    mesh.triangles.push(Triangle::new(0, 1, 3));
    mesh.triangles.push(Triangle::new(1, 2, 3));
    mesh.triangles.push(Triangle::new(0, 2, 3));
    let mut obj = Object::new(4);
    obj.mesh = Some(mesh);
    model.resources.objects.push(obj);
    model.build.items.push(BuildItem::new(4));

    // Write to memory
    let mut buf = Vec::new();
    model
        .to_writer(Cursor::new(&mut buf))
        .expect("Failed to write 3MF");

    println!("\nWritten 3MF to memory ({} bytes)\n", buf.len());

    // Read it back
    println!("=== Re-reading the written 3MF ===\n");
    let model2 = Model::from_reader(Cursor::new(&buf)).expect("Failed to re-parse");

    println!(
        "Property groups: {}",
        model2.resources.volumetric_property_groups.len()
    );
    println!(
        "Volumetric data resources: {}",
        model2.resources.volumetric_data.len()
    );
    for vol in &model2.resources.volumetric_data {
        print!("  id={}", vol.id);
        if vol.voxels.is_some() {
            print!(" (voxel grid)");
        }
        if vol.implicit.is_some() {
            print!(" (implicit)");
        }
        println!();
    }

    println!("\nRound-trip successful!");
}
