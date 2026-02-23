//! OPC (Open Packaging Conventions) handling for 3MF files
//!
//! 3MF files are ZIP archives following the OPC standard, containing
//! various parts including the main 3D model file and relationships.

mod content_types;
mod reader;
mod relationships;
mod thumbnail;
mod writer;

use crate::error::Result;
use std::io::Read;
use zip::ZipArchive;

// Re-export public API
pub use writer::{create_package, create_package_with_thumbnail};

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

/// 3D model relationship type
pub const MODEL_REL_TYPE: &str = "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel";

/// Thumbnail relationship type (OPC standard)
pub const THUMBNAIL_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail";

/// Keystore relationship type (Secure Content extension) - 2019/04 namespace
/// Note: The namespace changed from 2019/04 to 2019/07, but both are valid
pub const KEYSTORE_REL_TYPE_2019_04: &str =
    "http://schemas.microsoft.com/3dmanufacturing/2019/04/keystore";

/// Keystore relationship type (Secure Content extension) - 2019/07 namespace
pub const KEYSTORE_REL_TYPE_2019_07: &str =
    "http://schemas.microsoft.com/3dmanufacturing/2019/07/keystore";

/// EncryptedFile relationship type (OPC standard for encrypted files)
/// Per 3MF SecureContent spec, encrypted files must have this relationship type
pub const ENCRYPTEDFILE_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/package/2006/relationships/encryptedfile";

/// 3D Texture relationship type (Materials extension)
/// Per 3MF Materials Extension spec, texture resources must have this relationship type
pub const TEXTURE_REL_TYPE: &str = "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dtexture";

/// Represents an OPC package (3MF file)
pub struct Package<R: Read> {
    archive: ZipArchive<R>,
}

impl<R: Read + std::io::Seek> Package<R> {
    /// Open a 3MF package from a reader
    pub fn open(reader: R) -> Result<Self> {
        reader::open(reader)
    }

    /// Get the main 3D model file content
    pub fn get_model(&mut self) -> Result<String> {
        reader::get_model(self)
    }

    /// Get a streaming reader for the main 3D model file
    ///
    /// Returns a reader that decompresses the model file on-the-fly from the ZIP
    /// archive, avoiding loading the entire file into memory. The returned reader
    /// implements `Read` and borrows the package for its lifetime.
    pub fn get_model_reader(&mut self) -> Result<impl Read + '_> {
        reader::get_model_reader(self)
    }

    /// Get a file from the package by name
    pub fn get_file(&mut self, name: &str) -> Result<String> {
        reader::get_file(self, name)
    }

    /// Check if a file exists in the package
    pub fn has_file(&mut self, name: &str) -> bool {
        reader::has_file(self, name)
    }

    /// Get the number of files in the package
    pub fn len(&self) -> usize {
        reader::len(self)
    }

    /// Check if the package is empty
    pub fn is_empty(&self) -> bool {
        reader::is_empty(self)
    }

    /// Get a list of all file names in the package
    pub fn file_names(&mut self) -> Vec<String> {
        reader::file_names(self)
    }

    /// Get a file as binary data
    pub fn get_file_binary(&mut self, name: &str) -> Result<Vec<u8>> {
        reader::get_file_binary(self, name)
    }

    /// Get thumbnail metadata from the package
    pub fn get_thumbnail_metadata(&mut self) -> Result<Option<crate::model::Thumbnail>> {
        thumbnail::get_thumbnail_metadata(self)
    }

    /// Validate no model-level thumbnails exist
    pub fn validate_no_model_level_thumbnails(&mut self) -> Result<()> {
        thumbnail::validate_no_model_level_thumbnails(self)
    }

    /// Discover keystore file path from package relationships
    pub fn discover_keystore_path(&mut self) -> Result<Option<String>> {
        relationships::discover_keystore_path(self)
    }

    /// Check if a target file has a relationship of a specific type
    pub fn has_relationship_to_target(
        &mut self,
        target_path: &str,
        relationship_type: &str,
        source_file: Option<&str>,
    ) -> Result<bool> {
        relationships::has_relationship_to_target(self, target_path, relationship_type, source_file)
    }

    /// Validate keystore relationship
    pub fn validate_keystore_relationship(&mut self, keystore_path: &str) -> Result<()> {
        relationships::validate_keystore_relationship(self, keystore_path)
    }

    /// Validate keystore content type
    pub fn validate_keystore_content_type(&mut self, keystore_path: &str) -> Result<()> {
        content_types::validate_keystore_content_type(self, keystore_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::io::Read;
    use std::io::Write;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    // -----------------------------------------------------------------------
    // Test helpers
    // -----------------------------------------------------------------------

    /// Create a ZIP archive from a slice of (filename, data) pairs.
    fn make_zip(files: &[(&str, &[u8])]) -> Cursor<Vec<u8>> {
        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default();
        for (name, data) in files {
            zip.start_file(*name, options).unwrap();
            zip.write_all(data).unwrap();
        }
        zip.finish().unwrap()
    }

    const MINIMAL_CONTENT_TYPES: &[u8] = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
  <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
  <Default Extension=\"model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\
</Types>";

    const MINIMAL_RELS: &[u8] = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
</Relationships>";

    const MINIMAL_MODEL: &[u8] =
        b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<model unit=\"millimeter\" xml:lang=\"en-US\" \
xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\">\
  <resources/><build/></model>";

    /// Create the smallest valid 3MF package.
    fn minimal_3mf() -> Cursor<Vec<u8>> {
        make_zip(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES),
            ("_rels/.rels", MINIMAL_RELS),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ])
    }

    /// Create a valid 3MF package that also includes a PNG thumbnail.
    fn minimal_3mf_with_thumbnail() -> Cursor<Vec<u8>> {
        let content_types = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
  <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
  <Default Extension=\"model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\
  <Default Extension=\"png\" ContentType=\"image/png\"/>\
</Types>";
        let rels = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
  <Relationship Target=\"/Metadata/thumbnail.png\" Id=\"rel1\" Type=\"http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail\"/>\
</Relationships>";
        // Minimal valid 1x1 RGB PNG bytes
        let png: &[u8] = &[
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG magic
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00,
            0x90, 0x77, 0x53, 0xDE, // IHDR data + CRC
            0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, // IDAT chunk
            0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0xE2,
            0x21, 0xBC, 0x33, // IDAT data + CRC
            0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82, // IEND
        ];
        make_zip(&[
            ("[Content_Types].xml", content_types),
            ("_rels/.rels", rels),
            ("3D/3dmodel.model", MINIMAL_MODEL),
            ("Metadata/thumbnail.png", png),
        ])
    }

    /// Create a valid 3MF package that includes a keystore file.
    fn minimal_3mf_with_keystore() -> Cursor<Vec<u8>> {
        let content_types = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
  <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
  <Default Extension=\"model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\
  <Default Extension=\"xml\" ContentType=\"application/vnd.ms-package.3dmanufacturing-keystore+xml\"/>\
</Types>";
        let rels = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
  <Relationship Target=\"/Metadata/keystore.xml\" Id=\"rel1\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2019/07/keystore\"/>\
</Relationships>";
        make_zip(&[
            ("[Content_Types].xml", content_types),
            ("_rels/.rels", rels),
            ("3D/3dmodel.model", MINIMAL_MODEL),
            ("Metadata/keystore.xml", b"<?xml version=\"1.0\"?><keystore/>"),
        ])
    }

    // -----------------------------------------------------------------------
    // Package method tests (happy-path, require a valid package)
    // -----------------------------------------------------------------------

    #[test]
    fn test_package_get_model() {
        let mut pkg = Package::open(minimal_3mf()).unwrap();
        let model = pkg.get_model().unwrap();
        assert!(model.contains("<model"), "get_model should return model XML");
    }

    #[test]
    fn test_package_get_model_reader() {
        let mut pkg = Package::open(minimal_3mf()).unwrap();
        let mut reader = pkg.get_model_reader().unwrap();
        let mut content = String::new();
        reader.read_to_string(&mut content).unwrap();
        assert!(content.contains("<model"), "get_model_reader should stream model XML");
    }

    #[test]
    fn test_package_get_file() {
        let mut pkg = Package::open(minimal_3mf()).unwrap();
        let content = pkg.get_file(RELS_PATH).unwrap();
        assert!(
            content.contains("Relationships"),
            "get_file should return rels XML"
        );
    }

    #[test]
    fn test_package_has_file_existing_and_missing() {
        let mut pkg = Package::open(minimal_3mf()).unwrap();
        assert!(pkg.has_file(MODEL_PATH), "has_file should return true for existing file");
        assert!(!pkg.has_file("nonexistent.bin"), "has_file should return false for missing file");
    }

    #[test]
    fn test_package_len_and_is_empty() {
        let pkg = Package::open(minimal_3mf()).unwrap();
        assert_eq!(pkg.len(), 3, "minimal 3MF should have 3 files");
        assert!(!pkg.is_empty(), "non-empty package should not be is_empty");
    }

    #[test]
    fn test_package_file_names() {
        let mut pkg = Package::open(minimal_3mf()).unwrap();
        let names = pkg.file_names();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&CONTENT_TYPES_PATH.to_string()));
        assert!(names.contains(&RELS_PATH.to_string()));
        assert!(names.contains(&MODEL_PATH.to_string()));
    }

    #[test]
    fn test_package_get_file_binary() {
        let mut pkg = Package::open(minimal_3mf()).unwrap();
        let data = pkg.get_file_binary(MODEL_PATH).unwrap();
        assert!(!data.is_empty(), "get_file_binary should return non-empty data");
        // Model XML starts with <?xml
        assert_eq!(&data[..5], b"<?xml");
    }

    #[test]
    fn test_package_get_file_missing_returns_error() {
        let mut pkg = Package::open(minimal_3mf()).unwrap();
        assert!(pkg.get_file("does_not_exist.xml").is_err());
        assert!(pkg.get_file_binary("does_not_exist.bin").is_err());
    }

    // -----------------------------------------------------------------------
    // Missing required files
    // -----------------------------------------------------------------------

    #[test]
    fn test_open_missing_rels_file() {
        let cursor = make_zip(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err(), "Package without _rels/.rels should fail to open");
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("_rels/.rels") || err.contains("rels"),
            "Error should mention missing rels file, got: {err}"
        );
    }

    // -----------------------------------------------------------------------
    // Content types validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_content_types_missing_rels_extension() {
        // Content_Types.xml with no Default for "rels" extension
        let ct = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
  <Default Extension=\"model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\
</Types>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", ct),
            ("_rels/.rels", MINIMAL_RELS),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("rels"),
            "Error should mention missing rels extension, got: {err}"
        );
    }

    #[test]
    fn test_content_types_missing_model_type() {
        // Content_Types.xml with no model content type
        let ct = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
  <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
</Types>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", ct),
            ("_rels/.rels", MINIMAL_RELS),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("model"),
            "Error should mention missing model content type, got: {err}"
        );
    }

    #[test]
    fn test_content_types_empty_extension() {
        let ct = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
  <Default Extension=\"\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
  <Default Extension=\"model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\
</Types>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", ct),
            ("_rels/.rels", MINIMAL_RELS),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("empty"),
            "Error should mention empty Extension, got: {err}"
        );
    }

    #[test]
    fn test_content_types_duplicate_extension() {
        let ct = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
  <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
  <Default Extension=\"model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\
  <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
</Types>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", ct),
            ("_rels/.rels", MINIMAL_RELS),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("Duplicate"),
            "Error should mention duplicate extension, got: {err}"
        );
    }

    #[test]
    fn test_content_types_invalid_png_content_type() {
        let ct = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
  <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
  <Default Extension=\"model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\
  <Default Extension=\"png\" ContentType=\"image/jpeg\"/>\
</Types>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", ct),
            ("_rels/.rels", MINIMAL_RELS),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("image/png"),
            "Error should mention correct PNG content type, got: {err}"
        );
    }

    #[test]
    fn test_content_types_wrong_model_extension() {
        // Model content type assigned to an extension that is neither "model" nor "part"
        let ct = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
  <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
  <Default Extension=\"xyz\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\
</Types>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", ct),
            ("_rels/.rels", MINIMAL_RELS),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("Extension"),
            "Error should mention Extension requirement, got: {err}"
        );
    }

    #[test]
    fn test_content_types_model_via_override_succeeds() {
        // Override element for the model file is valid (found_model via Override)
        let ct = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
  <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
  <Override PartName=\"/3D/3dmodel.model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\
</Types>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", ct),
            ("_rels/.rels", MINIMAL_RELS),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ]);
        assert!(
            Package::open(cursor).is_ok(),
            "Model content type via Override should be accepted"
        );
    }

    #[test]
    fn test_content_types_empty_partname() {
        let ct = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
  <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
  <Default Extension=\"model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\
  <Override PartName=\"\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\
</Types>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", ct),
            ("_rels/.rels", MINIMAL_RELS),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("PartName"),
            "Error should mention empty PartName, got: {err}"
        );
    }

    #[test]
    fn test_content_types_duplicate_override() {
        let ct = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
  <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
  <Default Extension=\"model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\
  <Override PartName=\"/3D/3dmodel.model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\
  <Override PartName=\"/3D/3dmodel.model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\
</Types>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", ct),
            ("_rels/.rels", MINIMAL_RELS),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("Duplicate"),
            "Error should mention duplicate Override, got: {err}"
        );
    }

    // -----------------------------------------------------------------------
    // Model relationship / filename validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_model_filename_dot_prefix() {
        // Relationship points to a file whose name starts with '.'
        let rels = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/3D/.3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
</Relationships>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES),
            ("_rels/.rels", rels),
            ("3D/.3dmodel.model", MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("dot"),
            "Error should mention dot-prefix filename, got: {err}"
        );
    }

    #[test]
    fn test_model_filename_non_ascii_prefix() {
        // Relationship points to a file whose name has a non-ASCII prefix before "3dmodel".
        // U+00C6 (Æ, Latin Capital Letter Ae) is used as a representative non-ASCII character
        // that visually resembles an ASCII letter and could be used to spoof a model filename.
        let rels = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/3D/\u{00C6}3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
</Relationships>";
        let model_name = "3D/\u{00C6}3dmodel.model";
        let cursor = make_zip(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES),
            ("_rels/.rels", rels.as_bytes()),
            (model_name, MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("non-ASCII"),
            "Error should mention non-ASCII prefix, got: {err}"
        );
    }

    #[test]
    fn test_model_file_not_found_in_zip() {
        // Relationship points to a file that doesn't exist in the archive
        let rels = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/3D/missing.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
</Relationships>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES),
            ("_rels/.rels", rels),
            // Note: "3D/missing.model" is deliberately absent
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("missing") || err.contains("non-existent") || err.contains("exist"),
            "Error should indicate file not found, got: {err}"
        );
    }

    // -----------------------------------------------------------------------
    // All-relationships validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_duplicate_relationship_ids() {
        // Two relationships share the same Id in _rels/.rels
        let rels = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
</Relationships>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES),
            ("_rels/.rels", rels),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("Duplicate") || err.contains("duplicate"),
            "Error should mention duplicate ID, got: {err}"
        );
    }

    #[test]
    fn test_relationship_id_starts_with_digit_in_root_rels() {
        // Root .rels relationship Id starts with a digit
        let rels = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"1invalid\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
</Relationships>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES),
            ("_rels/.rels", rels),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("digit"),
            "Error should mention ID starting with digit, got: {err}"
        );
    }

    #[test]
    fn test_relationship_missing_id_attribute() {
        // A relationship element has no Id attribute
        let rels = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
  <Relationship Target=\"/3D/3dmodel.model\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
</Relationships>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES),
            ("_rels/.rels", rels),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("Id"),
            "Error should mention missing Id attribute, got: {err}"
        );
    }

    #[test]
    fn test_wrong_relationship_type_for_texture_file() {
        // A PNG texture in 3dmodel.model.rels uses MODEL_REL_TYPE instead of TEXTURE_REL_TYPE
        let ct = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
  <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
  <Default Extension=\"model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\
  <Default Extension=\"png\" ContentType=\"image/png\"/>\
</Types>";
        let model_rels =
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/3D/texture.png\" Id=\"tex0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
</Relationships>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", ct),
            ("_rels/.rels", MINIMAL_RELS),
            ("3D/3dmodel.model", MINIMAL_MODEL),
            ("3D/_rels/3dmodel.model.rels", model_rels),
            // texture.png intentionally absent (error fires before file-existence check)
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("texture") || err.contains("3dtexture"),
            "Error should mention texture relationship type, got: {err}"
        );
    }

    #[test]
    fn test_relationship_type_with_query_string() {
        let rels = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel1\" Type=\"http://example.com/type?query=1\"/>\
</Relationships>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES),
            ("_rels/.rels", rels),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("query"),
            "Error should mention query string, got: {err}"
        );
    }

    #[test]
    fn test_relationship_type_with_fragment() {
        let rels = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel1\" Type=\"http://example.com/type#frag\"/>\
</Relationships>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES),
            ("_rels/.rels", rels),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("fragment"),
            "Error should mention fragment identifier, got: {err}"
        );
    }

    #[test]
    fn test_duplicate_relationship_targets() {
        // Two relationships point to the same target with the same type (different IDs)
        let rels = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel1\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
</Relationships>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES),
            ("_rels/.rels", rels),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("Duplicate") || err.contains("duplicate"),
            "Error should mention duplicate target, got: {err}"
        );
    }

    #[test]
    fn test_part_specific_rels_without_associated_part() {
        // 3D/_rels/orphan.model.rels exists but 3D/orphan.model does not
        let orphan_rels = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
</Relationships>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES),
            ("_rels/.rels", MINIMAL_RELS),
            ("3D/3dmodel.model", MINIMAL_MODEL),
            ("3D/_rels/orphan.model.rels", orphan_rels),
            // "3D/orphan.model" is intentionally absent
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("orphan") || err.contains("exist"),
            "Error should mention missing associated part, got: {err}"
        );
    }

    #[test]
    fn test_invalid_part_name_with_hash() {
        let rels = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
  <Relationship Target=\"/3D/bad#part.model\" Id=\"rel1\" Type=\"http://example.com/other\"/>\
</Relationships>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES),
            ("_rels/.rels", rels),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("fragment") || err.contains('#'),
            "Error should mention fragment in part name, got: {err}"
        );
    }

    #[test]
    fn test_invalid_part_name_with_question_mark() {
        let rels = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
  <Relationship Target=\"/3D/bad?part.model\" Id=\"rel1\" Type=\"http://example.com/other\"/>\
</Relationships>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES),
            ("_rels/.rels", rels),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("query") || err.contains('?'),
            "Error should mention query string in part name, got: {err}"
        );
    }

    #[test]
    fn test_invalid_part_name_with_dotdot_segment() {
        let rels = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
  <Relationship Target=\"/3D/../etc/passwd\" Id=\"rel1\" Type=\"http://example.com/other\"/>\
</Relationships>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES),
            ("_rels/.rels", rels),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains(".."),
            "Error should mention '..' segment, got: {err}"
        );
    }

    #[test]
    fn test_invalid_part_name_with_single_dot_segment() {
        let rels = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
  <Relationship Target=\"/3D/./other.model\" Id=\"rel1\" Type=\"http://example.com/other\"/>\
</Relationships>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES),
            ("_rels/.rels", rels),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("'.'"),
            "Error should mention '.' segment, got: {err}"
        );
    }

    #[test]
    fn test_invalid_part_name_segment_ends_with_dot() {
        let rels = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
  <Relationship Target=\"/3D./other.model\" Id=\"rel1\" Type=\"http://example.com/other\"/>\
</Relationships>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES),
            ("_rels/.rels", rels),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("'.'") || err.contains("end"),
            "Error should mention segment ending with dot, got: {err}"
        );
    }

    #[test]
    fn test_invalid_part_name_empty_path_segment() {
        // Double slash creates an empty path segment
        let rels = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
  <Relationship Target=\"/3D//other.model\" Id=\"rel1\" Type=\"http://example.com/other\"/>\
</Relationships>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", MINIMAL_CONTENT_TYPES),
            ("_rels/.rels", rels),
            ("3D/3dmodel.model", MINIMAL_MODEL),
        ]);
        let result = Package::open(cursor);
        assert!(result.is_err());
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("empty") || err.contains("segment"),
            "Error should mention empty path segment, got: {err}"
        );
    }

    // -----------------------------------------------------------------------
    // Thumbnail tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_get_thumbnail_metadata_returns_none_when_no_thumbnail() {
        let mut pkg = Package::open(minimal_3mf()).unwrap();
        let result = pkg.get_thumbnail_metadata().unwrap();
        assert!(result.is_none(), "Package without thumbnail should return None");
    }

    #[test]
    fn test_get_thumbnail_metadata_png() {
        let mut pkg = Package::open(minimal_3mf_with_thumbnail()).unwrap();
        let thumb = pkg.get_thumbnail_metadata().unwrap();
        assert!(thumb.is_some(), "Package with thumbnail should return Some");
        let thumb = thumb.unwrap();
        assert!(
            thumb.path.contains("thumbnail"),
            "Thumbnail path should contain 'thumbnail'"
        );
        assert_eq!(&thumb.content_type, "image/png");
    }

    #[test]
    fn test_thumbnail_cmyk_jpeg_rejected() {
        // Craft a minimal CMYK JPEG (4 components in SOF0 marker)
        // Layout: FF D8 (SOI), then FF C0 (SOF0) at position 2
        // data[2..]: FF C0 LL LL PP HH HH WW WW CC
        //   where CC = num_components at offset 9 from FF = data[11]
        let cmyk_jpeg: Vec<u8> = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xC0, // SOF0 marker
            0x00, 0x0B, // length = 11
            0x08, // precision
            0x00, 0x01, // height = 1
            0x00, 0x01, // width = 1
            0x04, // num_components = 4 (CMYK)
        ];
        let ct = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
  <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
  <Default Extension=\"model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\
  <Default Extension=\"jpeg\" ContentType=\"image/jpeg\"/>\
</Types>";
        let rels = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
  <Relationship Target=\"/Metadata/thumbnail.jpeg\" Id=\"rel1\" Type=\"http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail\"/>\
</Relationships>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", ct),
            ("_rels/.rels", rels),
            ("3D/3dmodel.model", MINIMAL_MODEL),
            ("Metadata/thumbnail.jpeg", &cmyk_jpeg),
        ]);
        let mut pkg = Package::open(cursor).expect("Package with CMYK JPEG should open");
        let result = pkg.get_thumbnail_metadata();
        assert!(result.is_err(), "CMYK JPEG thumbnail should be rejected");
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("CMYK"),
            "Error should mention CMYK, got: {err}"
        );
    }

    #[test]
    fn test_validate_no_model_level_thumbnail_with_package_thumbnail_ok() {
        // Package has both package-level and model-level thumbnails -> OK
        let ct = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
  <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
  <Default Extension=\"model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\
  <Default Extension=\"png\" ContentType=\"image/png\"/>\
</Types>";
        let pkg_rels = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
  <Relationship Target=\"/Metadata/thumbnail.png\" Id=\"rel1\" Type=\"http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail\"/>\
</Relationships>";
        let model_rels = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/Metadata/thumbnail.png\" Id=\"mrel0\" Type=\"http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail\"/>\
</Relationships>";
        let png = &[0x89u8, 0x50, 0x4E, 0x47]; // first 4 bytes of valid PNG magic number
        let cursor = make_zip(&[
            ("[Content_Types].xml", ct),
            ("_rels/.rels", pkg_rels),
            ("3D/3dmodel.model", MINIMAL_MODEL),
            ("3D/_rels/3dmodel.model.rels", model_rels),
            ("Metadata/thumbnail.png", png),
        ]);
        let mut pkg = Package::open(cursor).expect("Package should open");
        assert!(
            pkg.validate_no_model_level_thumbnails().is_ok(),
            "Model-level thumbnail is allowed when package-level thumbnail also exists"
        );
    }

    #[test]
    fn test_validate_model_level_thumbnail_without_package_level_fails() {
        // Package has model-level thumbnail but NO package-level thumbnail -> should fail
        let ct = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
  <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
  <Default Extension=\"model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\
  <Default Extension=\"png\" ContentType=\"image/png\"/>\
</Types>";
        // No thumbnail in root rels
        let pkg_rels = MINIMAL_RELS;
        let model_rels = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/Metadata/thumbnail.png\" Id=\"mrel0\" Type=\"http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail\"/>\
</Relationships>";
        let png = &[0x89u8, 0x50, 0x4E, 0x47];
        let cursor = make_zip(&[
            ("[Content_Types].xml", ct),
            ("_rels/.rels", pkg_rels),
            ("3D/3dmodel.model", MINIMAL_MODEL),
            ("3D/_rels/3dmodel.model.rels", model_rels),
            ("Metadata/thumbnail.png", png),
        ]);
        let mut pkg = Package::open(cursor).expect("Package should open");
        let result = pkg.validate_no_model_level_thumbnails();
        assert!(
            result.is_err(),
            "Model-level thumbnail without package-level thumbnail should fail"
        );
    }

    // -----------------------------------------------------------------------
    // Keystore / relationship discovery tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_discover_keystore_path_returns_none_when_absent() {
        let mut pkg = Package::open(minimal_3mf()).unwrap();
        let path = pkg.discover_keystore_path().unwrap();
        assert!(path.is_none(), "No keystore relationship should return None");
    }

    #[test]
    fn test_discover_keystore_path_returns_path_when_present() {
        let mut pkg = Package::open(minimal_3mf_with_keystore()).unwrap();
        let path = pkg.discover_keystore_path().unwrap();
        assert_eq!(path, Some("Metadata/keystore.xml".to_string()));
    }

    #[test]
    fn test_has_relationship_to_target_found() {
        let mut pkg = Package::open(minimal_3mf_with_thumbnail()).unwrap();
        let found = pkg
            .has_relationship_to_target(
                "Metadata/thumbnail.png",
                "http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail",
                None,
            )
            .unwrap();
        assert!(found, "Should find the thumbnail relationship");
    }

    #[test]
    fn test_has_relationship_to_target_not_found() {
        let mut pkg = Package::open(minimal_3mf()).unwrap();
        let found = pkg
            .has_relationship_to_target(
                "Metadata/thumbnail.png",
                "http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail",
                None,
            )
            .unwrap();
        assert!(!found, "Should not find thumbnail relationship in package without thumbnail");
    }

    #[test]
    fn test_has_relationship_to_target_with_source_file_not_found() {
        // Source file has no associated .rels -> should return false gracefully
        let mut pkg = Package::open(minimal_3mf()).unwrap();
        let found = pkg
            .has_relationship_to_target(
                "3D/3dmodel.model",
                "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel",
                Some("3D/3dmodel.model"),
            )
            .unwrap();
        assert!(
            !found,
            "Should return false when the associated .rels file does not exist"
        );
    }

    #[test]
    fn test_validate_keystore_relationship_fails_when_absent() {
        let mut pkg = Package::open(minimal_3mf()).unwrap();
        let result = pkg.validate_keystore_relationship("Metadata/keystore.xml");
        assert!(result.is_err(), "Should fail when no keystore relationship exists");
    }

    #[test]
    fn test_validate_keystore_relationship_succeeds_when_present() {
        let mut pkg = Package::open(minimal_3mf_with_keystore()).unwrap();
        assert!(
            pkg.validate_keystore_relationship("Metadata/keystore.xml").is_ok(),
            "Should succeed when keystore relationship is present"
        );
    }

    #[test]
    fn test_validate_keystore_content_type_via_default_extension() {
        // Content types has Default Extension="xml" with keystore content type
        let mut pkg = Package::open(minimal_3mf_with_keystore()).unwrap();
        assert!(
            pkg.validate_keystore_content_type("Metadata/keystore.xml").is_ok(),
            "Should accept keystore content type declared via Default Extension='xml'"
        );
    }

    #[test]
    fn test_validate_keystore_content_type_via_override() {
        let ct = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
  <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
  <Default Extension=\"model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\
  <Override PartName=\"/Metadata/keystore.xml\" ContentType=\"application/vnd.ms-package.3dmanufacturing-keystore+xml\"/>\
</Types>";
        let rels = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\
  <Relationship Target=\"/Metadata/keystore.xml\" Id=\"rel1\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2019/07/keystore\"/>\
</Relationships>";
        let cursor = make_zip(&[
            ("[Content_Types].xml", ct),
            ("_rels/.rels", rels),
            ("3D/3dmodel.model", MINIMAL_MODEL),
            ("Metadata/keystore.xml", b"<?xml version=\"1.0\"?><keystore/>"),
        ]);
        let mut pkg = Package::open(cursor).unwrap();
        assert!(
            pkg.validate_keystore_content_type("Metadata/keystore.xml").is_ok(),
            "Should accept keystore content type declared via Override PartName"
        );
    }

    #[test]
    fn test_validate_keystore_content_type_fails_when_absent() {
        let mut pkg = Package::open(minimal_3mf()).unwrap();
        let result = pkg.validate_keystore_content_type("Metadata/keystore.xml");
        assert!(result.is_err(), "Should fail when no keystore content type exists");
    }

    // -----------------------------------------------------------------------
    // writer tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_create_package_with_thumbnail_no_thumbnail_data() {
        // Call create_package_with_thumbnail with None thumbnail -> should produce valid package
        let model_xml = std::str::from_utf8(MINIMAL_MODEL).unwrap();
        let buf = create_package_with_thumbnail(Cursor::new(Vec::new()), model_xml, None, None)
            .unwrap();
        let mut pkg = Package::open(buf).unwrap();
        assert!(pkg.get_model().is_ok());
    }

    #[test]
    fn test_create_package_with_jpeg_thumbnail() {
        let model_xml = std::str::from_utf8(MINIMAL_MODEL).unwrap();
        let thumb_data = &[0xFFu8, 0xD8, 0xFF, 0xE0]; // first 4 bytes of JPEG SOI + APP0 marker
        let buf = create_package_with_thumbnail(
            Cursor::new(Vec::new()),
            model_xml,
            Some(thumb_data),
            Some("image/jpeg"),
        )
        .unwrap();
        // Package should open successfully
        assert!(Package::open(buf).is_ok());
    }

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
