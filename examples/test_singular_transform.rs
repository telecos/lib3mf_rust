use lib3mf::Model;
use std::fs::File;

fn main() {
    let file_path = std::env::args().nth(1).expect("Need file path");
    println!("Testing: {}", file_path);
    
    match File::open(&file_path) {
        Ok(file) => match Model::from_reader(file) {
            Ok(model) => {
                println!("  ✓ Successfully parsed!");
                println!("  Objects: {}", model.resources.objects.len());
                println!("  Build items: {}", model.build.items.len());
                for (i, item) in model.build.items.iter().enumerate() {
                    if let Some(ref transform) = item.transform {
                        println!("  Build item {}: transform = {:?}", i, transform);
                    }
                }
            },
            Err(e) => {
                println!("  ✗ FAILED: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("  Cannot open file: {}", e);
            std::process::exit(1);
        }
    }
}
