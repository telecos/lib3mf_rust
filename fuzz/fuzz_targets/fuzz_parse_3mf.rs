#![no_main]

use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // Fuzz the main 3MF parsing API
    // This tests the complete parsing pipeline: ZIP extraction -> XML parsing -> model construction
    let cursor = Cursor::new(data);
    
    // Try parsing with default configuration
    let _ = lib3mf::Model::from_reader(cursor);
});
