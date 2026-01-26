//! Package reading and file access operations

use crate::error::{Error, Result};
use crate::opc::{CONTENT_TYPES_PATH, RELS_PATH};
use std::io::Read;
use urlencoding::decode;
use zip::ZipArchive;

use super::content_types::{validate_content_types, validate_keystore_content_type};
use super::relationships::{
    discover_keystore_path, discover_model_path, has_relationship_to_target,
    validate_all_relationships, validate_keystore_relationship, validate_model_relationship,
};
use super::thumbnail::{get_thumbnail_metadata, validate_no_model_level_thumbnails};

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
            return Err(Error::invalid_format_context(
                "OPC package structure",
                &format!(
                    "Missing required file '{}'. \
                     This file defines content types for the package and is required by the OPC specification. \
                     The 3MF file may be corrupt or improperly formatted.",
                    CONTENT_TYPES_PATH
                )
            ));
        }

        if !self.has_file(RELS_PATH) {
            return Err(Error::invalid_format_context(
                "OPC package structure",
                &format!(
                    "Missing required file '{}'. \
                     This file defines package relationships and is required by the OPC specification. \
                     The 3MF file may be corrupt or improperly formatted.",
                    RELS_PATH
                )
            ));
        }

        // Validate Content Types
        validate_content_types(&mut self.archive)?;

        // Validate that model relationship exists and points to valid file
        validate_model_relationship(&mut self.archive)?;

        // Validate all relationships point to existing files
        validate_all_relationships(&mut self.archive)?;

        Ok(())
    }

    /// Get the main 3D model file content
    pub fn get_model(&mut self) -> Result<String> {
        // Discover model path from relationships (validation already done in open())
        let model_path = discover_model_path(&mut self.archive)?;

        // Determine which path to use: try the original first, then decoded
        let path_to_use = if self.has_file(&model_path) {
            model_path.clone()
        } else {
            // If the direct path fails, try URL-decoding
            if let Ok(decoded) = decode(&model_path) {
                let decoded_path = decoded.into_owned();
                if decoded_path != model_path && self.has_file(&decoded_path) {
                    decoded_path
                } else {
                    return Err(Error::MissingFile(model_path));
                }
            } else {
                return Err(Error::MissingFile(model_path));
            }
        };

        // Now read the file
        let mut file = self
            .archive
            .by_name(&path_to_use)
            .map_err(|_| Error::MissingFile(path_to_use.clone()))?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        Ok(content)
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
        get_thumbnail_metadata(&mut self.archive)
    }

    /// Validate that thumbnails are not defined ONLY in model-level relationship files
    ///
    /// Per 3MF Core Specification and OPC (Open Packaging Conventions),
    /// if a thumbnail relationship is defined at the part/model level
    /// (e.g., 3D/_rels/3dmodel.model.rels), there MUST also be a thumbnail
    /// relationship at the package level (_rels/.rels).
    ///
    /// Test cases: N_SPX_0417_01, N_SPX_0419_01
    pub fn validate_no_model_level_thumbnails(&mut self) -> Result<()> {
        validate_no_model_level_thumbnails(&mut self.archive)
    }

    /// Discover keystore file path from package relationships
    ///
    /// Returns the path to the keystore file if one exists, or None if no keystore is found.
    /// The keystore is identified by relationship type for the Secure Content extension.
    pub fn discover_keystore_path(&mut self) -> Result<Option<String>> {
        discover_keystore_path(&mut self.archive)
    }

    /// Check if a target file has a relationship of a specific type
    ///
    /// This method searches for relationships that reference the specified target file.
    /// It checks relationship files that could contain such relationships:
    /// - If source_file is specified: checks that file's corresponding .rels file
    /// - If source_file is None: checks ALL .rels files in the package
    ///
    /// # Arguments
    /// * `target_path` - The path to the target file (e.g., "/3D/3dmodel_encrypted.model")
    /// * `relationship_type` - The relationship type to look for (e.g., ENCRYPTEDFILE_REL_TYPE)
    /// * `source_file` - Optional source file that should have the relationship (e.g., "3D/3dmodel.model")
    ///
    /// # Returns
    /// `true` if a relationship of the specified type targeting the file exists, `false` otherwise
    pub fn has_relationship_to_target(
        &mut self,
        target_path: &str,
        relationship_type: &str,
        source_file: Option<&str>,
    ) -> Result<bool> {
        has_relationship_to_target(&mut self.archive, target_path, relationship_type, source_file)
    }

    /// Validate that a keystore file has the correct relationship type in root .rels
    ///
    /// EPX-2606 validation: If a keystore file exists, it must have a proper keystore
    /// relationship (not just mustpreserve or other generic relationships).
    ///
    /// # Arguments
    /// * `keystore_path` - The path to the keystore file (e.g., "Secure/keystore.xml")
    ///
    /// # Returns
    /// `Ok(())` if validation passes, `Err` if the keystore relationship is missing or invalid
    pub fn validate_keystore_relationship(&mut self, keystore_path: &str) -> Result<()> {
        validate_keystore_relationship(&mut self.archive, keystore_path)
    }

    /// Validate that a keystore file has the correct content type override
    ///
    /// EPX-2606 validation: If a keystore file exists, it must have a content type
    /// defined in [Content_Types].xml, either as an Override for the specific file
    /// or as a Default for the .xml extension.
    ///
    /// # Arguments
    /// * `keystore_path` - The path to the keystore file (e.g., "Secure/keystore.xml")
    ///
    /// # Returns
    /// `Ok(())` if validation passes, `Err` if the content type is not properly defined
    pub fn validate_keystore_content_type(&mut self, keystore_path: &str) -> Result<()> {
        validate_keystore_content_type(&mut self.archive, keystore_path)
    }

}
