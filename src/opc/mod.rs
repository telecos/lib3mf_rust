//! OPC (Open Packaging Conventions) handling for 3MF files
//!
//! 3MF files are ZIP archives following the OPC standard, containing
//! various parts including the main 3D model file and relationships.

mod content_types;
mod reader;
mod relationships;
mod thumbnail;
mod validation;
mod writer;

// Re-export public types and functions
pub use reader::Package;
pub use writer::{create_package, create_package_with_thumbnail};

// Re-export constants for external use
pub use relationships::{
    ENCRYPTEDFILE_REL_TYPE, KEYSTORE_REL_TYPE_2019_04, KEYSTORE_REL_TYPE_2019_07,
    MODEL_REL_TYPE, TEXTURE_REL_TYPE, THUMBNAIL_REL_TYPE,
};

/// Main 3D model file path within the 3MF archive
pub const MODEL_PATH: &str = "3D/3dmodel.model";

/// Alternative model path (some implementations use this)
pub const MODEL_PATH_ALT: &str = "/3D/3dmodel.model";

/// Content types file path
pub const CONTENT_TYPES_PATH: &str = "[Content_Types].xml";

/// Relationships file path
pub const RELS_PATH: &str = "_rels/.rels";

/// Model relationships file path
pub const MODEL_RELS_PATH: &str = "3D/_rels/3dmodel.model.rels";

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    #[test]
    fn test_package_constants() {
        assert_eq!(MODEL_PATH, "3D/3dmodel.model");
        assert_eq!(CONTENT_TYPES_PATH, "[Content_Types].xml");
    }

    #[test]
    fn test_package_from_empty_zip() {
        // Create an empty ZIP archive
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let zip = ZipWriter::new(cursor);
        let cursor = zip.finish().unwrap();

        // Should fail validation because it's missing required files
        let result = Package::open(cursor);
        assert!(
            result.is_err(),
            "Expected package validation to fail for empty ZIP"
        );
    }

    #[test]
    fn test_percent_encoded_part_names() {
        // Create a 3MF file with percent-encoded part name in XML relationships
        // and UTF-8 character in ZIP file name (correct per OPC spec)
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default();

        // [Content_Types].xml
        zip.start_file("[Content_Types].xml", options).unwrap();
        zip.write_all(
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">
  <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>
  <Default Extension=\"model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>
</Types>",
        )
        .unwrap();

        // _rels/.rels with percent-encoded target (%C3%86 = Æ)
        zip.start_file("_rels/.rels", options).unwrap();
        zip.write_all(
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">
  <Relationship Target=\"/2D/test%C3%86file.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>
</Relationships>",
        )
        .unwrap();

        // Actual ZIP file with UTF-8 character (Æ)
        zip.start_file("2D/testÆfile.model", options).unwrap();
        zip.write_all(
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<model unit=\"millimeter\" xml:lang=\"en-US\" xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\">
  <resources>
    <object id=\"1\" type=\"model\">
      <mesh>
        <vertices>
          <vertex x=\"0\" y=\"0\" z=\"0\"/>
          <vertex x=\"100\" y=\"0\" z=\"0\"/>
          <vertex x=\"0\" y=\"100\" z=\"0\"/>
        </vertices>
        <triangles>
          <triangle v1=\"0\" v2=\"1\" v3=\"2\"/>
        </triangles>
      </mesh>
    </object>
  </resources>
  <build>
    <item objectid=\"1\"/>
  </build>
</model>",
        )
        .unwrap();

        let cursor = zip.finish().unwrap();

        // This should succeed: percent-encoded in XML, UTF-8 in ZIP
        let result = Package::open(cursor);
        assert!(
            result.is_ok(),
            "Package with percent-encoded part names should open successfully"
        );
    }

    #[test]
    fn test_utf8_in_xml_accepted_for_compatibility() {
        // Per OPC spec, non-ASCII should be percent-encoded in XML Target attributes.
        // However, for compatibility with real-world files (including official test suites),
        // we accept UTF-8 characters directly in the Target attribute.
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default();

        zip.start_file("[Content_Types].xml", options).unwrap();
        zip.write_all(
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">
  <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>
  <Default Extension=\"model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>
</Types>",
        )
        .unwrap();

        zip.start_file("_rels/.rels", options).unwrap();
        let rels = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">
  <Relationship Target=\"/2D/testÆfile.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>
</Relationships>";
        zip.write_all(rels.as_bytes()).unwrap();

        zip.start_file("2D/testÆfile.model", options).unwrap();
        zip.write_all(
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<model unit=\"millimeter\" xml:lang=\"en-US\" xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\">
  <resources>
    <object id=\"1\" type=\"model\">
      <mesh>
        <vertices>
          <vertex x=\"0\" y=\"0\" z=\"0\"/>
          <vertex x=\"100\" y=\"0\" z=\"0\"/>
          <vertex x=\"0\" y=\"100\" z=\"0\"/>
        </vertices>
        <triangles>
          <triangle v1=\"0\" v2=\"1\" v3=\"2\"/>
        </triangles>
      </mesh>
    </object>
  </resources>
  <build>
    <item objectid=\"1\"/>
  </build>
</model>",
        )
        .unwrap();

        let cursor = zip.finish().unwrap();

        // This should now succeed for compatibility
        let result = Package::open(cursor);
        assert!(
            result.is_ok(),
            "Package with UTF-8 characters in XML should be accepted for compatibility"
        );
    }
}
