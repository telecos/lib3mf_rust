use lib3mf::{Extension, Model, ParserConfig};
use std::env;
use std::fs::File;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path-to-3mf-file>", args[0]);
        return;
    }

    let file_path = &args[1];
    let config = ParserConfig::new().with_extension(Extension::Production);

    println!("Testing: {}", file_path);
    match File::open(file_path) {
        Ok(file) => match Model::from_reader_with_config(file, config) {
            Ok(_) => println!("  ✗ SUCCEEDED (should have failed)"),
            Err(e) => println!("  ✓ FAILED as expected: {}", e),
        },
        Err(e) => eprintln!("  Cannot open file: {}", e),
    }
}
