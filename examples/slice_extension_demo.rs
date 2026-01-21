// Example to demonstrate slice extension support
use lib3mf::Model;
use std::fs::File;

fn main() {
    // Load the box_sliced.3mf test file
    let file = File::open("test_files/slices/box_sliced.3mf")
        .expect("Failed to open box_sliced.3mf test file");

    let model = Model::from_reader(file).expect("Failed to parse box_sliced.3mf");

    println!("Slice Extension Demo");
    println!("====================\n");

    // Display slice stacks
    println!("Slice Stacks: {}", model.resources.slice_stacks.len());
    for slice_stack in &model.resources.slice_stacks {
        println!("\nSliceStack ID: {}", slice_stack.id);
        println!("  Z Bottom: {}", slice_stack.zbottom);
        println!("  Slices: {}", slice_stack.slices.len());
        println!("  Slice References: {}", slice_stack.slice_refs.len());

        for slice_ref in &slice_stack.slice_refs {
            println!(
                "    - SliceStackID: {}, Path: {}",
                slice_ref.slicestackid, slice_ref.slicepath
            );
        }

        // Display details for first few slices
        if !slice_stack.slices.is_empty() {
            println!("\n  First 3 slices (showing layer details):");
            for (i, slice) in slice_stack.slices.iter().take(3).enumerate() {
                println!("    Slice {}: ztop={:.3}mm", i, slice.ztop);
                println!("      Vertices: {}", slice.vertices.len());
                println!("      Polygons: {}", slice.polygons.len());

                if let Some(polygon) = slice.polygons.first() {
                    println!(
                        "        First polygon: startv={}, segments={}",
                        polygon.startv,
                        polygon.segments.len()
                    );
                }
            }
        }
    }

    // Display objects with slice references
    println!("\nObjects with Slice References:");
    for object in &model.resources.objects {
        if let Some(slicestackid) = object.slicestackid {
            println!(
                "  Object ID: {}, Name: {:?}, SliceStackID: {}",
                object.id, object.name, slicestackid
            );
        }
    }

    println!("\n✅ Slice extension data successfully extracted!");
}
