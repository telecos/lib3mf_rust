#![no_main]

use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // Fuzz 3MF parsing with all extensions enabled
    // This tests parsing with Material, Production, Slice, BeamLattice, BooleanOps, Displacement, and SecureContent
    let cursor = Cursor::new(data);
    
    // Create a parser config with all extensions enabled
    let config = lib3mf::ParserConfig::with_all_extensions();
    
    // Try parsing with the config
    let _ = lib3mf::Model::from_reader_with_config(cursor, config);
});
