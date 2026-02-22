//! Regression tests for Out-of-Memory (OOM) via ZIP decompression bombs.
//!
//! A crafted ZIP file can contain a small compressed payload that decompresses
//! into an extremely large output (a "zip bomb"). Without a read limit, the
//! parser would try to allocate gigabytes of memory as it reads the decompressed
//! data, causing an out-of-memory condition.
//!
//! After the fix, all file reads are bounded by `MAX_FILE_CONTENT_BYTES` (1 GB)
//! using `Read::take()`, so decompression bombs cannot exhaust available memory.
//!
//! References:
//! - fuzzing artifact `oom-781b3b8632ef181d4bbdfc85c468298b95f75a4d` (fuzz_parse_3mf)
//! - fuzzing artifact `oom-decd7af532fee02b2569d0e28017a693ea1938db` (fuzz_parse_with_extensions)

use lib3mf::{Model, ParserConfig};
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

/// Build a 3MF ZIP archive where the model XML is a highly repetitive string
/// that compresses well but decompresses to a large size.
fn make_3mf_with_large_model(decompressed_size_approx: usize) -> Vec<u8> {
    let buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buf);
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    // [Content_Types].xml
    zip.start_file("[Content_Types].xml", opts).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
</Types>"#,
    )
    .unwrap();

    // _rels/.rels
    zip.start_file("_rels/.rels", opts).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rel0" Target="/3D/3dmodel.model" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"#,
    )
    .unwrap();

    // 3D/3dmodel.model with highly repetitive vertex data
    zip.start_file("3D/3dmodel.model", opts).unwrap();

    let header = br#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US"
  xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <resources>
    <object id="1" type="model">
      <mesh>
        <vertices>
          <vertex x="0" y="0" z="0"/>
          <vertex x="10" y="0" z="0"/>
          <vertex x="0" y="10" z="0"/>
          <vertex x="0" y="0" z="10"/>
"#;
    zip.write_all(header).unwrap();

    // Each vertex line is about 50 bytes. Write enough to approach the target.
    let vertex_line = b"          <vertex x=\"1\" y=\"2\" z=\"3\"/>\n";
    let lines_needed = decompressed_size_approx / vertex_line.len();
    for _ in 0..lines_needed {
        zip.write_all(vertex_line).unwrap();
    }

    let footer = br#"        </vertices>
        <triangles>
          <triangle v1="0" v2="1" v3="2"/>
          <triangle v1="0" v2="1" v3="3"/>
          <triangle v1="0" v2="2" v3="3"/>
          <triangle v1="1" v2="2" v3="3"/>
        </triangles>
      </mesh>
    </object>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"#;
    zip.write_all(footer).unwrap();

    let cursor = zip.finish().unwrap();
    cursor.into_inner()
}

/// Verify that a normal-sized model (10 KB decompressed) still parses
/// successfully with the read limit in place.
#[test]
fn test_normal_model_parses_with_read_limit() {
    let bytes = make_3mf_with_large_model(10 * 1024);
    let result = Model::from_reader(Cursor::new(bytes));
    assert!(
        result.is_ok(),
        "A normal-sized 3MF should parse successfully; got: {:?}",
        result.err()
    );
}

/// Verify that a moderately large model (10 MB decompressed) still parses
/// successfully. This ensures the read limit doesn't break legitimate files.
#[test]
fn test_moderately_large_model_parses() {
    let bytes = make_3mf_with_large_model(10 * 1024 * 1024);
    let result = Model::from_reader(Cursor::new(bytes));
    assert!(
        result.is_ok(),
        "A 10 MB 3MF model should parse successfully; got: {:?}",
        result.err()
    );
}

/// Regression test: the `fuzz_parse_with_extensions` target (all extensions enabled)
/// is protected by the same read bound as the basic parser.
///
/// Before the fix, `get_model_reader()` returned an unbounded reader, so a ZIP
/// decompression bomb would exhaust memory inside `parse_model_from_reader`'s
/// `read_to_string` call regardless of the extension config used.
/// After the fix, `get_model_reader()` applies `Read::take(MAX_FILE_CONTENT_BYTES + 1)`,
/// bounding the streaming decompression for all callers.
///
/// Reference: fuzzing artifact `oom-decd7af532fee02b2569d0e28017a693ea1938db` (hash 6cffe6233fa57583)
#[test]
fn test_with_all_extensions_is_protected_by_read_limit() {
    // A 10 MB model should still parse successfully even with all extensions enabled.
    let bytes = make_3mf_with_large_model(10 * 1024 * 1024);
    let config = ParserConfig::with_all_extensions();
    let result = Model::from_reader_with_config(Cursor::new(bytes), config);
    assert!(
        result.is_ok(),
        "A 10 MB 3MF model with all extensions should parse successfully; got: {:?}",
        result.err()
    );
}
