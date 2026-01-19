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

/// Represents an OPC package (3MF file)
pub struct Package<R: Read> {
    archive: ZipArchive<R>,
}

impl<R: Read + std::io::Seek> Package<R> {
    /// Open a 3MF package from a reader
    pub fn open(reader: R) -> Result<Self> {
        let archive = ZipArchive::new(reader)?;
        Ok(Self { archive })
    }

    /// Get the main 3D model file content
    pub fn get_model(&mut self) -> Result<String> {
        // First, try to discover the model path from relationships
        if let Ok(model_path) = self.discover_model_path() {
            if let Ok(mut file) = self.archive.by_name(&model_path) {
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                return Ok(content);
            }
        }

        // Fallback: Try primary path
        if let Ok(mut file) = self.archive.by_name(MODEL_PATH) {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            return Ok(content);
        }

        // Try alternative path
        if let Ok(mut file) = self.archive.by_name(MODEL_PATH_ALT) {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            return Ok(content);
        }

        Err(Error::MissingFile(MODEL_PATH.to_string()))
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

        let package = Package::open(cursor).unwrap();
        assert_eq!(package.len(), 0);
        assert!(package.is_empty());
    }
}
