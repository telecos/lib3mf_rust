//! OPC (Open Packaging Conventions) handling for 3MF files
//!
//! 3MF files are ZIP archives following the OPC standard, containing
//! various parts including the main 3D model file and relationships.

use crate::error::{Error, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::Read;
use zip::ZipArchive;

/// Main 3D model file path within the 3MF archive
pub const MODEL_PATH: &str = "3D/3dmodel.model";

/// Alternative model path (some implementations use this)
pub const MODEL_PATH_ALT: &str = "/3D/3dmodel.model";

/// Content types file path
pub const CONTENT_TYPES_PATH: &str = "[Content_Types].xml";

/// Relationships file path
pub const RELS_PATH: &str = "_rels/.rels";

/// 3D model relationship type
pub const MODEL_REL_TYPE: &str = "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel";

/// Thumbnail relationship type (OPC standard)
pub const THUMBNAIL_REL_TYPE: &str =
    "http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail";

/// Represents an OPC package (3MF file)
pub struct Package<R: Read> {
    archive: ZipArchive<R>,
}

impl<R: Read + std::io::Seek> Package<R> {
    /// Open a 3MF package from a reader
    pub fn open(reader: R) -> Result<Self> {
        let archive = ZipArchive::new(reader)?;
        let mut package = Self { archive };

        // Validate required OPC structure
        package.validate_opc_structure()?;

        Ok(package)
    }

    /// Validate OPC package structure according to 3MF spec
    fn validate_opc_structure(&mut self) -> Result<()> {
        // Validate required files exist
        if !self.has_file(CONTENT_TYPES_PATH) {
            return Err(Error::InvalidFormat(format!(
                "Missing required file: {}",
                CONTENT_TYPES_PATH
            )));
        }

        if !self.has_file(RELS_PATH) {
            return Err(Error::InvalidFormat(format!(
                "Missing required file: {}",
                RELS_PATH
            )));
        }

        // Validate Content Types
        self.validate_content_types()?;

        // Validate that model relationship exists and points to valid file
        self.validate_model_relationship()?;

        // Validate all relationships point to existing files
        self.validate_all_relationships()?;

        Ok(())
    }

    /// Validate [Content_Types].xml structure
    fn validate_content_types(&mut self) -> Result<()> {
        let content = self.get_file(CONTENT_TYPES_PATH)?;
        let mut reader = Reader::from_str(&content);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();

        let mut found_rels = false;
        let mut found_model = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let name_str = std::str::from_utf8(name.as_ref())
                        .map_err(|e| Error::InvalidXml(e.to_string()))?;

                    if name_str.ends_with("Default") {
                        let mut extension = None;
                        let mut content_type = None;

                        for attr in e.attributes() {
                            let attr = attr?;
                            let key = std::str::from_utf8(attr.key.as_ref())
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;
                            let value = std::str::from_utf8(&attr.value)
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;

                            match key {
                                "Extension" => extension = Some(value.to_string()),
                                "ContentType" => content_type = Some(value.to_string()),
                                _ => {}
                            }
                        }

                        if let (Some(ext), Some(ct)) = (extension, content_type) {
                            // Check for required content types
                            if ext.eq_ignore_ascii_case("rels")
                                && ct == "application/vnd.openxmlformats-package.relationships+xml"
                            {
                                found_rels = true;
                            }
                            // Validate 3dmodel content type mapping
                            if ct == "application/vnd.ms-package.3dmanufacturing-3dmodel+xml" {
                                // Per 3MF spec, the extension for 3D model files must be "model"
                                if !ext.eq_ignore_ascii_case("model") {
                                    return Err(Error::InvalidFormat(format!(
                                        "Content type '{}' must use Extension='model', not Extension='{}'",
                                        ct, ext
                                    )));
                                }
                                found_model = true;
                            }
                        }
                    } else if name_str.ends_with("Override") {
                        // Override elements can also define model content type
                        let mut content_type = None;

                        for attr in e.attributes() {
                            let attr = attr?;
                            let key = std::str::from_utf8(attr.key.as_ref())
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;
                            let value = std::str::from_utf8(&attr.value)
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;

                            if key == "ContentType" {
                                content_type = Some(value.to_string());
                            }
                        }

                        if let Some(ct) = content_type {
                            if ct == "application/vnd.ms-package.3dmanufacturing-3dmodel+xml" {
                                found_model = true;
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }

        if !found_rels {
            return Err(Error::InvalidFormat(
                "Content Types missing required 'rels' extension definition".to_string(),
            ));
        }

        if !found_model {
            return Err(Error::InvalidFormat(
                "Content Types missing required model content type (Default or Override)"
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// Validate model relationship exists and points to a valid file
    fn validate_model_relationship(&mut self) -> Result<()> {
        let model_path = self.discover_model_path()?;

        // Verify the model file actually exists
        if !self.has_file(&model_path) {
            return Err(Error::InvalidFormat(format!(
                "Model relationship points to non-existent file: {}",
                model_path
            )));
        }

        Ok(())
    }

    /// Validate all relationships point to existing files
    fn validate_all_relationships(&mut self) -> Result<()> {
        let rels_content = self.get_file(RELS_PATH)?;
        let mut reader = Reader::from_str(&rels_content);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let name_str = std::str::from_utf8(name.as_ref())
                        .map_err(|e| Error::InvalidXml(e.to_string()))?;

                    if name_str.ends_with("Relationship") {
                        let mut target = None;
                        let mut rel_type = None;

                        for attr in e.attributes() {
                            let attr = attr?;
                            let key = std::str::from_utf8(attr.key.as_ref())
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;
                            let value = std::str::from_utf8(&attr.value)
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;

                            if key == "Target" {
                                target = Some(value.to_string());
                            } else if key == "Type" {
                                rel_type = Some(value.to_string());
                            }
                        }

                        // Validate relationship Type - must not contain query strings or fragments
                        if let Some(t) = &rel_type {
                            if t.contains('?') {
                                return Err(Error::InvalidFormat(format!(
                                    "Relationship Type cannot contain query string: {}",
                                    t
                                )));
                            }
                            if t.contains('#') {
                                return Err(Error::InvalidFormat(format!(
                                    "Relationship Type cannot contain fragment identifier: {}",
                                    t
                                )));
                            }
                        }

                        if let Some(t) = target {
                            // Validate the target is a valid OPC part name
                            Self::validate_opc_part_name(&t)?;

                            // Remove leading slash if present
                            let path = if let Some(stripped) = t.strip_prefix('/') {
                                stripped.to_string()
                            } else {
                                t
                            };

                            // Verify the target file exists
                            if !self.has_file(&path) {
                                return Err(Error::InvalidFormat(format!(
                                    "Relationship points to non-existent file: {}",
                                    path
                                )));
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(())
    }

    /// Validate OPC part name according to OPC specification
    ///
    /// Part names must not contain:
    /// - Fragment identifiers (#)
    /// - Query strings (?)
    /// - Path segments that are "." or ".."
    /// - Empty path segments (consecutive slashes)
    /// - Segments ending with "." (like "3D.")
    fn validate_opc_part_name(part_name: &str) -> Result<()> {
        // Check for fragment identifiers
        if part_name.contains('#') {
            return Err(Error::InvalidFormat(format!(
                "Part name cannot contain fragment identifier: {}",
                part_name
            )));
        }

        // Check for query strings
        if part_name.contains('?') {
            return Err(Error::InvalidFormat(format!(
                "Part name cannot contain query string: {}",
                part_name
            )));
        }

        // Split into path segments and validate each
        let segments: Vec<&str> = part_name.split('/').collect();

        for (idx, segment) in segments.iter().enumerate() {
            // Check for empty segments (consecutive slashes)
            if segment.is_empty() {
                // Allow leading slash (which creates empty first segment)
                if idx == 0 && part_name.starts_with('/') {
                    continue;
                }
                return Err(Error::InvalidFormat(format!(
                    "Part name cannot contain empty path segments (consecutive slashes): {}",
                    part_name
                )));
            }

            // Check for "." or ".." segments
            if *segment == "." || *segment == ".." {
                return Err(Error::InvalidFormat(format!(
                    "Part name cannot contain '.' or '..' segments: {}",
                    part_name
                )));
            }

            // Check for segments ending with "."
            if segment.ends_with('.') {
                return Err(Error::InvalidFormat(format!(
                    "Part name segments cannot end with '.': {}",
                    part_name
                )));
            }
        }

        Ok(())
    }

    /// Get the main 3D model file content
    pub fn get_model(&mut self) -> Result<String> {
        // Discover model path from relationships (validation already done in open())
        let model_path = self.discover_model_path()?;

        // Read the model file
        let mut file = self
            .archive
            .by_name(&model_path)
            .map_err(|_| Error::MissingFile(model_path.clone()))?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        Ok(content)
    }

    /// Discover the model file path from the relationships file
    fn discover_model_path(&mut self) -> Result<String> {
        // Read the _rels/.rels file
        let rels_content = self.get_file(RELS_PATH)?;

        // Parse the XML to find the model relationship
        let mut reader = Reader::from_str(&rels_content);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let name_str = std::str::from_utf8(name.as_ref())
                        .map_err(|e| Error::InvalidXml(e.to_string()))?;

                    if name_str.ends_with("Relationship") {
                        let mut target = None;
                        let mut rel_type = None;

                        for attr in e.attributes() {
                            let attr = attr?;
                            let key = std::str::from_utf8(attr.key.as_ref())
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;
                            let value = std::str::from_utf8(&attr.value)
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;

                            match key {
                                "Target" => target = Some(value.to_string()),
                                "Type" => rel_type = Some(value.to_string()),
                                _ => {}
                            }
                        }

                        // Check if this is the 3D model relationship
                        if let (Some(t), Some(rt)) = (target, rel_type) {
                            if rt == MODEL_REL_TYPE {
                                // Remove leading slash if present
                                let path = if let Some(stripped) = t.strip_prefix('/') {
                                    stripped.to_string()
                                } else {
                                    t
                                };
                                return Ok(path);
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::InvalidXml(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        Err(Error::MissingFile(
            "3D model relationship not found".to_string(),
        ))
    }

    /// Get a file by name from the archive
    pub fn get_file(&mut self, name: &str) -> Result<String> {
        let mut file = self
            .archive
            .by_name(name)
            .map_err(|_| Error::MissingFile(name.to_string()))?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Ok(content)
    }

    /// Check if a file exists in the archive
    pub fn has_file(&mut self, name: &str) -> bool {
        self.archive.by_name(name).is_ok()
    }

    /// Get the number of files in the archive
    pub fn len(&self) -> usize {
        self.archive.len()
    }

    /// Check if the archive is empty
    pub fn is_empty(&self) -> bool {
        self.archive.len() == 0
    }

    /// List all file names in the archive
    pub fn file_names(&mut self) -> Vec<String> {
        (0..self.archive.len())
            .filter_map(|i| self.archive.by_index(i).ok().map(|f| f.name().to_string()))
            .collect()
    }

    /// Get a file as binary data from the archive
    pub fn get_file_binary(&mut self, name: &str) -> Result<Vec<u8>> {
        let mut file = self
            .archive
            .by_name(name)
            .map_err(|_| Error::MissingFile(name.to_string()))?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        Ok(content)
    }

    /// Extract thumbnail metadata from package relationships
    ///
    /// Returns thumbnail path and content type if a thumbnail relationship exists.
    /// The thumbnail is validated to exist in the package and have a valid content type.
    pub fn get_thumbnail_metadata(&mut self) -> Result<Option<crate::model::Thumbnail>> {
        // Check if relationships file exists
        if !self.has_file(RELS_PATH) {
            return Ok(None);
        }

        // Parse relationships to find thumbnail
        let rels_content = self.get_file(RELS_PATH)?;
        let mut reader = Reader::from_str(&rels_content);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();

        let mut thumbnail_path: Option<String> = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let name_str = std::str::from_utf8(name.as_ref())
                        .map_err(|e| Error::InvalidXml(e.to_string()))?;

                    if name_str.ends_with("Relationship") {
                        let mut target = None;
                        let mut rel_type = None;

                        for attr in e.attributes() {
                            let attr = attr?;
                            let key = std::str::from_utf8(attr.key.as_ref())
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;
                            let value = std::str::from_utf8(&attr.value)
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;

                            match key {
                                "Target" => target = Some(value.to_string()),
                                "Type" => rel_type = Some(value.to_string()),
                                _ => {}
                            }
                        }

                        // Check if this is a thumbnail relationship
                        if let (Some(t), Some(rt)) = (target, rel_type) {
                            if rt == THUMBNAIL_REL_TYPE {
                                // Remove leading slash if present
                                let path = if let Some(stripped) = t.strip_prefix('/') {
                                    stripped.to_string()
                                } else {
                                    t
                                };
                                thumbnail_path = Some(path);
                                break;
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }

        // If no thumbnail relationship found, return None
        let thumb_path = match thumbnail_path {
            Some(p) => p,
            None => return Ok(None),
        };

        // Validate thumbnail file exists
        if !self.has_file(&thumb_path) {
            return Err(Error::InvalidFormat(format!(
                "Thumbnail relationship points to non-existent file: {}",
                thumb_path
            )));
        }

        // Get content type from [Content_Types].xml
        let content_type = self.get_content_type(&thumb_path)?;

        // Validate it's an image content type
        if !content_type.starts_with("image/") {
            return Err(Error::InvalidFormat(format!(
                "Thumbnail has invalid content type: {} (expected image/*)",
                content_type
            )));
        }

        Ok(Some(crate::model::Thumbnail::new(
            thumb_path,
            content_type,
        )))
    }

    /// Get content type for a file from [Content_Types].xml
    fn get_content_type(&mut self, path: &str) -> Result<String> {
        let content = self.get_file(CONTENT_TYPES_PATH)?;
        let mut reader = Reader::from_str(&content);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();

        // First check for Override elements (specific path matches)
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let name_str = std::str::from_utf8(name.as_ref())
                        .map_err(|e| Error::InvalidXml(e.to_string()))?;

                    if name_str.ends_with("Override") {
                        let mut part_name = None;
                        let mut content_type = None;

                        for attr in e.attributes() {
                            let attr = attr?;
                            let key = std::str::from_utf8(attr.key.as_ref())
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;
                            let value = std::str::from_utf8(&attr.value)
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;

                            match key {
                                "PartName" => part_name = Some(value.to_string()),
                                "ContentType" => content_type = Some(value.to_string()),
                                _ => {}
                            }
                        }

                        if let (Some(pn), Some(ct)) = (part_name, content_type) {
                            // Match with or without leading slash
                            let pn_normalized = pn.strip_prefix('/').unwrap_or(&pn);
                            let path_normalized = path.strip_prefix('/').unwrap_or(path);
                            if pn_normalized == path_normalized {
                                return Ok(ct);
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }

        // If no Override found, check Default elements based on extension
        if let Some(extension) = path.rsplit('.').next() {
            let content = self.get_file(CONTENT_TYPES_PATH)?;
            let mut reader = Reader::from_str(&content);
            reader.config_mut().trim_text(true);
            let mut buf = Vec::new();

            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                        let name = e.name();
                        let name_str = std::str::from_utf8(name.as_ref())
                            .map_err(|e| Error::InvalidXml(e.to_string()))?;

                        if name_str.ends_with("Default") {
                            let mut ext = None;
                            let mut content_type = None;

                            for attr in e.attributes() {
                                let attr = attr?;
                                let key = std::str::from_utf8(attr.key.as_ref())
                                    .map_err(|e| Error::InvalidXml(e.to_string()))?;
                                let value = std::str::from_utf8(&attr.value)
                                    .map_err(|e| Error::InvalidXml(e.to_string()))?;

                                match key {
                                    "Extension" => ext = Some(value.to_string()),
                                    "ContentType" => content_type = Some(value.to_string()),
                                    _ => {}
                                }
                            }

                            if let (Some(e), Some(ct)) = (ext, content_type) {
                                if e.eq_ignore_ascii_case(extension) {
                                    return Ok(ct);
                                }
                            }
                        }
                    }
                    Ok(Event::Eof) => break,
                    Err(e) => return Err(Error::Xml(e)),
                    _ => {}
                }
                buf.clear();
            }
        }

        Err(Error::InvalidFormat(format!(
            "No content type found for file: {}",
            path
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

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
        let zip = zip::ZipWriter::new(cursor);
        let cursor = zip.finish().unwrap();

        // Should fail validation because it's missing required files
        let result = Package::open(cursor);
        assert!(
            result.is_err(),
            "Expected package validation to fail for empty ZIP"
        );
    }
}
