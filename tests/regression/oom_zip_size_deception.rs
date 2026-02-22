//! Regression tests for Out-of-Memory (OOM) via ZIP size-deception.
//!
//! A crafted ZIP file can declare a very large `uncompressed_size` in its local
//! file header while containing only a tiny compressed payload. Previously the
//! parser called `String::with_capacity(file.size() as usize)`, which blindly
//! trusted that attacker-controlled header field and would attempt to allocate
//! gigabytes of memory before reading a single byte.
//!
//! After the fix, the pre-allocation hint is capped at 64 KB so a lied header
//! field never triggers a large upfront allocation. The buffer still grows
//! as actual bytes are read, so legitimate large files are unaffected.
//!
//! References:
//! - fuzzing artifact `oom-c064722642ee5d1624dfa84e1f5c2ff38ec086b0` (lied model uncompressed size)
//! - fuzzing artifact `oom-b005ceb96c1d2f53cb537a0e9816f45213ef77e8` (hash `1540c29df25a1778`,
//!   lied `[Content_Types].xml` compressed+uncompressed size = near u32::MAX)

use lib3mf::Model;
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

/// Helper that builds a minimal valid 3MF ZIP archive whose model XML is
/// replaced by `model_content`. The ZIP entry is stored with
/// `declared_uncompressed_size` reported in the local file header (only used
/// for the `file.size()` hint; the actual bytes written are the real content).
///
/// We use `zip::ZipWriter` with `SimpleFileOptions` and then patch the
/// `uncompressed_size` field in the resulting bytes to simulate the attack.
/// Because we need to lie about the size we write the archive in *store* mode
/// (no compression), then manually overwrite the 32-bit uncompressed-size
/// field in the local file header to an arbitrarily large value.
fn make_3mf_with_lied_size(actual_xml: &[u8], lied_size: u32) -> Vec<u8> {
    let buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buf);
    let opts = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .large_file(false);

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

    // 3D/3dmodel.model  (STORED – no compression, size fields are meaningful)
    zip.start_file("3D/3dmodel.model", opts).unwrap();
    zip.write_all(actual_xml).unwrap();

    let cursor = zip.finish().unwrap();
    let mut bytes = cursor.into_inner();

    // Patch the local file header of 3D/3dmodel.model.
    // ZIP local file header layout (offsets from start of header signature):
    //   0x00  4  signature  PK\x03\x04
    //   0x04  2  version needed
    //   0x06  2  general purpose bit flag
    //   0x08  2  compression method
    //   0x0A  2  last mod file time
    //   0x0C  2  last mod file date
    //   0x0E  4  crc-32
    //   0x12  4  compressed size     <-- offset +0x12 from signature
    //   0x16  4  uncompressed size   <-- offset +0x16 from signature  *** LIE HERE ***
    //   0x1A  2  file name length
    //   0x1C  2  extra field length
    //   ...      file name
    //   ...      extra field
    //   ...      file data
    //
    // We need to find the third local-file-header signature (PK\x03\x04) for
    // the "3D/3dmodel.model" entry and overwrite its uncompressed-size field.
    let sig = b"PK\x03\x04";
    let mut count = 0;
    let mut header_offset = None;
    let mut i = 0;
    while i + 4 <= bytes.len() {
        if bytes[i..i + 4] == *sig {
            count += 1;
            if count == 3 {
                header_offset = Some(i);
                break;
            }
        }
        i += 1;
    }

    if let Some(off) = header_offset {
        let uncompressed_size_offset = off + 0x16;
        let le = lied_size.to_le_bytes();
        bytes[uncompressed_size_offset..uncompressed_size_offset + 4].copy_from_slice(&le);
    }

    bytes
}

/// A minimal but valid 3MF model XML for use in tests.
const MINIMAL_MODEL_XML: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
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
        </vertices>
        <triangles>
          <triangle v1="0" v2="2" v3="1"/>
          <triangle v1="0" v2="1" v3="3"/>
          <triangle v1="0" v2="3" v3="2"/>
          <triangle v1="1" v2="2" v3="3"/>
        </triangles>
      </mesh>
    </object>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"#;

/// Verify that a normal 3MF (honest size in header) still parses correctly
/// after the fix.
#[test]
fn test_normal_3mf_still_parses() {
    let bytes = make_3mf_with_lied_size(MINIMAL_MODEL_XML, MINIMAL_MODEL_XML.len() as u32);
    let result = Model::from_reader(Cursor::new(bytes));
    assert!(
        result.is_ok(),
        "A valid 3MF with honest size header should parse successfully; got: {:?}",
        result.err()
    );
}

/// Regression test: a ZIP whose `uncompressed_size` header field claims 2 GiB
/// but whose actual payload is tiny must not cause an OOM.
///
/// Before the fix `String::with_capacity(2_147_483_648)` was called, which
/// immediately exhausted available memory on most systems. After the fix the
/// pre-allocation is capped at 64 KB and the parse succeeds because the
/// actual payload is only a few hundred bytes.
#[test]
fn test_lied_uncompressed_size_does_not_cause_oom() {
    // Claim 2 GiB uncompressed while actual content is only a few hundred bytes.
    let lied_size: u32 = 2u32 * 1024 * 1024 * 1024; // 2 GiB
    let bytes = make_3mf_with_lied_size(MINIMAL_MODEL_XML, lied_size);

    // The actual XML is valid; the lied header field only affected the capacity
    // hint, which is now safely capped.  The parse must succeed without OOM.
    let result = Model::from_reader(Cursor::new(bytes));
    assert!(
        result.is_ok(),
        "Parser should succeed even when the ZIP header lies about uncompressed size; got: {:?}",
        result.err()
    );
}

/// Regression test: a ZIP whose `uncompressed_size` field is set to u32::MAX
/// (4 GiB - 1) must not cause an OOM pre-allocation.
#[test]
fn test_max_lied_uncompressed_size_does_not_cause_oom() {
    let bytes = make_3mf_with_lied_size(MINIMAL_MODEL_XML, u32::MAX);

    // Must complete without panic/OOM.
    let result = Model::from_reader(Cursor::new(bytes));
    assert!(
        result.is_ok(),
        "Parser should succeed with u32::MAX lied size; got: {:?}",
        result.err()
    );
}

/// Build a 3MF ZIP archive where the FIRST entry (`[Content_Types].xml`) has
/// both its `compressed_size` and `uncompressed_size` local-header fields
/// patched to `lied_size`, while the actual data is a normal short XML.
///
/// This simulates the fuzzing artifact `oom-b005ceb96c1d2f53cb537a0e9816f45213ef77e8`
/// (hash `1540c29df25a1778`) where `[Content_Types].xml` declared both sizes as
/// ~4 GB in the local file header.
fn make_3mf_with_lied_content_types_size(lied_size: u32) -> Vec<u8> {
    let buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buf);
    let opts = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .large_file(false);

    // [Content_Types].xml (first entry – this is the one we will patch)
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

    // 3D/3dmodel.model
    zip.start_file("3D/3dmodel.model", opts).unwrap();
    zip.write_all(MINIMAL_MODEL_XML).unwrap();

    let cursor = zip.finish().unwrap();
    let mut bytes = cursor.into_inner();

    // Patch the FIRST local file header (for `[Content_Types].xml`).
    // The first entry starts at offset 0 with the PK\x03\x04 signature.
    // ZIP local file header layout (offsets from start of header):
    //   0x12  4  compressed size
    //   0x16  4  uncompressed size
    let sig = b"PK\x03\x04";
    if bytes.starts_with(sig) {
        let lied = lied_size.to_le_bytes();
        bytes[0x12..0x16].copy_from_slice(&lied); // compressed size
        bytes[0x16..0x1A].copy_from_slice(&lied); // uncompressed size
    }

    bytes
}

/// Regression test for fuzzing artifact `oom-b005ceb96c1d2f53cb537a0e9816f45213ef77e8`
/// (hash `1540c29df25a1778`).
///
/// A ZIP where `[Content_Types].xml` claims both `compressed_size` and
/// `uncompressed_size` equal to u32::MAX (~4 GB) must not cause an OOM.
///
/// Before the fix, `get_file()` called `String::with_capacity(file.size() as usize)`
/// where `file.size()` returned u32::MAX, immediately exhausting memory. After the
/// fix the pre-allocation hint is capped at 64 KB.
#[test]
fn test_lied_content_types_sizes_does_not_cause_oom() {
    let bytes = make_3mf_with_lied_content_types_size(u32::MAX);

    // The actual XML payload is valid; only the size fields in the local header
    // are inflated. The parser must not OOM and must complete (with either
    // a success or a graceful error – the lied compressed_size makes the stored
    // entry corrupt from the zip crate's perspective).
    let _result = Model::from_reader(Cursor::new(bytes));
    // We only assert that execution reached this point without panic/OOM.
}
