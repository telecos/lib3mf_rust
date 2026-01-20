use lib3mf::Model;
use std::fs::File;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path-to-3mf-file>", args[0]);
        std::process::exit(1);
    }
    
    let path = &args[1];
    println!("Testing: {}", path);
    
    let file = File::open(path).expect("Failed to open file");
    match Model::from_reader(file) {
        Ok(model) => {
            println!("✓ Parsed successfully!");
            println!("  Objects: {}", model.resources.objects.len());
            println!("  Build items: {}", model.build.items.len());
        }
        Err(e) => {
            println!("✗ Failed to parse: {}", e);
        }
    }
}
