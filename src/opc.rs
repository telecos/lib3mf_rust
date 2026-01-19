//! OPC (Open Packaging Conventions) handling for 3MF files
//!
//! 3MF files are ZIP archives following the OPC standard, containing
//! various parts including the main 3D model file and relationships.

use crate::error::{Error, Result};
use std::io::Read;
use zip::ZipArchive;

/// Main 3D model file path within the 3MF archive
pub const MODEL_PATH: &str = "3D/3dmodel.model";

/// Alternative model path (some implementations use this)
pub const MODEL_PATH_ALT: &str = "/3D/3dmodel.model";

/// Content types file path
pub const CONTENT_TYPES_PATH: &str = "[Content_Types].xml";

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
        // Try primary path first
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
            .filter_map(|i| {
                self.archive
                    .by_index(i)
                    .ok()
                    .map(|f| f.name().to_string())
            })
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
