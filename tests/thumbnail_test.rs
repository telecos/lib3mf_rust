//! Tests for thumbnail validation and extraction

use lib3mf::parser::{parse_3mf, read_thumbnail};
use std::fs::File;
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Create a 3MF file with a thumbnail
fn create_3mf_with_thumbnail() -> Vec<u8> {
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    let mut zip = ZipWriter::new(cursor);

    let options = SimpleFileOptions::default();

    // Add [Content_Types].xml with PNG content type
    let content_types = r##"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
  <Default Extension="png" ContentType="image/png"/>
</Types>"##;

    zip.start_file("[Content_Types].xml", options).unwrap();
    zip.write_all(content_types.as_bytes()).unwrap();

    // Add _rels/.rels with thumbnail relationship
    let rels = r##"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Target="/3D/3dmodel.model" Id="rel0" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
  <Relationship Target="/Metadata/thumbnail.png" Id="rel-thumbnail" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail"/>
</Relationships>"##;

    zip.start_file("_rels/.rels", options).unwrap();
    zip.write_all(rels.as_bytes()).unwrap();

    // Add 3D/3dmodel.model
    let model = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <resources>
    <object id="1" type="model">
      <mesh>
        <vertices>
          <vertex x="0.0" y="0.0" z="0.0"/>
          <vertex x="10.0" y="0.0" z="0.0"/>
          <vertex x="5.0" y="10.0" z="0.0"/>
        </vertices>
        <triangles>
          <triangle v1="0" v2="1" v3="2"/>
        </triangles>
      </mesh>
    </object>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"##;

    zip.start_file("3D/3dmodel.model", options).unwrap();
    zip.write_all(model.as_bytes()).unwrap();

    // Add Metadata/thumbnail.png (1x1 red pixel PNG - raw bytes)
    // This is a minimal valid PNG file
    let png_data: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
        0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41,
        0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0xF0,
        0x1F, 0x00, 0x05, 0x05, 0x02, 0x00, 0x5F, 0xC8,
        0xF1, 0xD2, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
        0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    zip.start_file("Metadata/thumbnail.png", options).unwrap();
    zip.write_all(&png_data).unwrap();

    zip.finish().unwrap();
    buffer
}

/// Create a 3MF file without a thumbnail
fn create_3mf_without_thumbnail() -> Vec<u8> {
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    let mut zip = ZipWriter::new(cursor);

    let options = SimpleFileOptions::default();

    // Add [Content_Types].xml
    let content_types = r##"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
</Types>"##;

    zip.start_file("[Content_Types].xml", options).unwrap();
    zip.write_all(content_types.as_bytes()).unwrap();

    // Add _rels/.rels without thumbnail relationship
    let rels = r##"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Target="/3D/3dmodel.model" Id="rel0" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"##;

    zip.start_file("_rels/.rels", options).unwrap();
    zip.write_all(rels.as_bytes()).unwrap();

    // Add 3D/3dmodel.model
    let model = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <resources>
    <object id="1" type="model">
      <mesh>
        <vertices>
          <vertex x="0.0" y="0.0" z="0.0"/>
          <vertex x="10.0" y="0.0" z="0.0"/>
          <vertex x="5.0" y="10.0" z="0.0"/>
        </vertices>
        <triangles>
          <triangle v1="0" v2="1" v3="2"/>
        </triangles>
      </mesh>
    </object>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"##;

    zip.start_file("3D/3dmodel.model", options).unwrap();
    zip.write_all(model.as_bytes()).unwrap();

    zip.finish().unwrap();
    buffer
}

#[test]
fn test_parse_3mf_with_thumbnail() {
    let data = create_3mf_with_thumbnail();
    let cursor = Cursor::new(data);
    let model = parse_3mf(cursor).expect("Failed to parse 3MF with thumbnail");

    // Check that thumbnail was extracted
    assert!(model.thumbnail.is_some(), "Thumbnail should be present");

    let thumbnail = model.thumbnail.unwrap();
    assert_eq!(thumbnail.path, "Metadata/thumbnail.png");
    assert_eq!(thumbnail.content_type, "image/png");
}

#[test]
fn test_parse_3mf_without_thumbnail() {
    let data = create_3mf_without_thumbnail();
    let cursor = Cursor::new(data);
    let model = parse_3mf(cursor).expect("Failed to parse 3MF without thumbnail");

    // Check that thumbnail is not present
    assert!(model.thumbnail.is_none(), "Thumbnail should not be present");
}

#[test]
fn test_parse_real_file_with_thumbnail() {
    // Test with the real test file we created
    let file = File::open("test_files/test_thumbnail.3mf");
    if let Ok(f) = file {
        let model = parse_3mf(f).expect("Failed to parse test_thumbnail.3mf");
        
        assert!(model.thumbnail.is_some(), "Thumbnail should be present in test file");
        let thumbnail = model.thumbnail.unwrap();
        assert_eq!(thumbnail.path, "Metadata/thumbnail.png");
        assert_eq!(thumbnail.content_type, "image/png");
    }
}

#[test]
fn test_thumbnail_validation_missing_file() {
    // Create a 3MF with thumbnail relationship but missing file
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    let mut zip = ZipWriter::new(cursor);

    let options = SimpleFileOptions::default();

    // Add [Content_Types].xml
    let content_types = r##"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
  <Default Extension="png" ContentType="image/png"/>
</Types>"##;

    zip.start_file("[Content_Types].xml", options).unwrap();
    zip.write_all(content_types.as_bytes()).unwrap();

    // Add _rels/.rels with thumbnail relationship
    let rels = r##"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Target="/3D/3dmodel.model" Id="rel0" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
  <Relationship Target="/Metadata/thumbnail.png" Id="rel-thumbnail" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail"/>
</Relationships>"##;

    zip.start_file("_rels/.rels", options).unwrap();
    zip.write_all(rels.as_bytes()).unwrap();

    // Add 3D/3dmodel.model
    let model = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <resources>
    <object id="1" type="model">
      <mesh>
        <vertices>
          <vertex x="0.0" y="0.0" z="0.0"/>
          <vertex x="10.0" y="0.0" z="0.0"/>
          <vertex x="5.0" y="10.0" z="0.0"/>
        </vertices>
        <triangles>
          <triangle v1="0" v2="1" v3="2"/>
        </triangles>
      </mesh>
    </object>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"##;

    zip.start_file("3D/3dmodel.model", options).unwrap();
    zip.write_all(model.as_bytes()).unwrap();

    // NOTE: Not adding thumbnail file - this should cause an error

    zip.finish().unwrap();

    let cursor = Cursor::new(buffer);
    let result = parse_3mf(cursor);

    // Should fail because thumbnail file is missing
    assert!(result.is_err(), "Should fail when thumbnail file is missing");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("non-existent file"),
        "Error should mention missing file"
    );
}

#[test]
fn test_read_thumbnail_data() {
    // Test reading thumbnail binary data
    let data = create_3mf_with_thumbnail();
    let cursor = Cursor::new(&data);
    
    let thumbnail_data = read_thumbnail(cursor).expect("Failed to read thumbnail");
    assert!(thumbnail_data.is_some(), "Thumbnail data should be present");
    
    let thumb = thumbnail_data.unwrap();
    assert!(!thumb.is_empty(), "Thumbnail data should not be empty");
    
    // Verify it's a valid PNG by checking signature
    assert_eq!(&thumb[0..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
}

#[test]
fn test_read_thumbnail_data_none() {
    // Test reading thumbnail when none exists
    let data = create_3mf_without_thumbnail();
    let cursor = Cursor::new(data);
    
    let thumbnail_data = read_thumbnail(cursor).expect("Failed to read thumbnail");
    assert!(thumbnail_data.is_none(), "Thumbnail data should not be present");
}
