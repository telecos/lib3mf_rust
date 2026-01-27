//! Inspect color data in a 3MF file

use lib3mf::Model;
use std::env;
use std::fs::File;
use std::process;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <3mf-file>", args[0]);
        process::exit(1);
    }

    let file = File::open(&args[1])?;
    let model = Model::from_reader(file)?;
    
    println!("Objects: {}", model.resources.objects.len());
    for obj in &model.resources.objects {
        println!("Object {}: pid={:?}, pindex={:?}", obj.id, obj.pid, obj.pindex);
        if let Some(ref mesh) = obj.mesh {
            println!("  Triangles: {}", mesh.triangles.len());
            // Check first 10 triangles
            for (i, tri) in mesh.triangles.iter().take(10).enumerate() {
                println!("    Tri {}: pid={:?}, pindex={:?}, p1={:?}, p2={:?}, p3={:?}", 
                    i, tri.pid, tri.pindex, tri.p1, tri.p2, tri.p3);
            }
        }
    }
    
    println!("\nColor Groups: {}", model.resources.color_groups.len());
    for cg in &model.resources.color_groups {
        println!("ColorGroup {}: {} colors", cg.id, cg.colors.len());
    }
    
    Ok(())
}
