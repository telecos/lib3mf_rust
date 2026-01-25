//! OPC (Open Packaging Conventions) handling for 3MF files
//!
//! 3MF files are ZIP archives following the OPC standard, containing
//! various parts including the main 3D model file and relationships.

use crate::error::{Error, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::Read;
use urlencoding::decode;
use zip::ZipArchive;

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
        let mut default_extensions = std::collections::HashSet::new();
        let mut override_parts = std::collections::HashSet::new();

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
                            // N_XPX_0206_01: Check for empty extension
                            if ext.is_empty() {
                                return Err(Error::InvalidFormat(
                                    "Content type Default element cannot have empty Extension attribute".to_string()
                                ));
                            }

                            // N_XPX_0205_01: Check for duplicate default extensions
                            if !default_extensions.insert(ext.clone()) {
                                return Err(Error::InvalidFormat(format!(
                                    "Duplicate Default content type mapping for extension '{}'",
                                    ext
                                )));
                            }

                            // N_XPX_0404_04: Validate PNG content type
                            if ext.eq_ignore_ascii_case("png") && ct != "image/png" {
                                return Err(Error::InvalidFormat(format!(
                                    "Invalid content type '{}' for PNG extension, must be 'image/png'",
                                    ct
                                )));
                            }

                            // Check for required content types
                            if ext.eq_ignore_ascii_case("rels")
                                && ct == "application/vnd.openxmlformats-package.relationships+xml"
                            {
                                found_rels = true;
                            }
                            // Validate 3dmodel content type mapping
                            if ct == "application/vnd.ms-package.3dmanufacturing-3dmodel+xml" {
                                // Per 3MF spec, the extension for 3D model files is typically "model"
                                // However, "part" is also valid for backward compatibility
                                if !ext.eq_ignore_ascii_case("model")
                                    && !ext.eq_ignore_ascii_case("part")
                                {
                                    return Err(Error::InvalidFormat(format!(
                                        "Content type '{}' must use Extension='model' or 'part', not Extension='{}'",
                                        ct, ext
                                    )));
                                }
                                found_model = true;
                            }
                        }
                    } else if name_str.ends_with("Override") {
                        // Override elements can also define model content type
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

                        // N_XPX_0207_01: Check for empty PartName
                        if let Some(ref pn) = part_name {
                            if pn.is_empty() {
                                return Err(Error::InvalidFormat(
                                    "Content type Override element cannot have empty PartName attribute".to_string()
                                ));
                            }
                        }

                        if let (Some(pn), Some(ct)) = (part_name, content_type) {
                            // N_XPX_0205_02: Check for duplicate override parts
                            if !override_parts.insert(pn.clone()) {
                                return Err(Error::InvalidFormat(format!(
                                    "Duplicate Override content type for part '{}'",
                                    pn
                                )));
                            }

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
            return Err(Error::invalid_format_context(
                "Content Types validation",
                "Missing required 'rels' extension definition in [Content_Types].xml. \
                 The Content Types file must define a Default element for the '.rels' extension. \
                 This is required by the OPC specification.",
            ));
        }

        if !found_model {
            return Err(Error::invalid_format_context(
                "Content Types validation",
                "Missing required model content type definition in [Content_Types].xml. \
                 The file must define either a Default or Override element for the 3D model content type \
                 ('application/vnd.ms-package.3dmanufacturing-3dmodel+xml'). \
                 Check that the model file has a proper content type declaration."
            ));
        }

        Ok(())
    }

    /// Validate model relationship exists and points to a valid file
    fn validate_model_relationship(&mut self) -> Result<()> {
        let model_path = self.discover_model_path()?;

        // N_XXX_0208_01: Validate model filename structure
        // The 3MF spec expects model files to be named "3dmodel.model" or similar with
        // ASCII characters. We reject files that try to masquerade as standard model files
        // by using non-ASCII lookalike characters (e.g., Cyrillic letters that look like Latin).
        // N_SPX_0415_01: Also reject filenames with dot prefix (e.g., ".3dmodel.model")
        if let Some((_dir, filename)) = model_path.rsplit_once('/') {
            // Check for dot prefix (hidden file)
            if filename.starts_with('.') {
                return Err(Error::InvalidFormat(format!(
                    "Model filename '{}' starts with a dot (hidden file). \
                     The 3MF specification requires standard naming for model files without dot prefix.",
                    filename
                )));
            }

            if filename.contains("3dmodel") {
                // If the filename contains "3dmodel", check that any prefix uses ASCII
                // This catches cases like "Ԫ3dmodel.model" (Cyrillic character before "3dmodel")
                if let Some(pos) = filename.find("3dmodel") {
                    let prefix = &filename[..pos];
                    if !prefix.is_empty() && !prefix.is_ascii() {
                        return Err(Error::InvalidFormat(format!(
                            "Model filename '{}' contains non-ASCII characters before '3dmodel'. \
                             The 3MF specification requires standard ASCII naming for model files.",
                            filename
                        )));
                    }
                }
            }
        }

        // Verify the model file actually exists (try both encoded and decoded paths)
        let file_exists = self.has_file(&model_path) || {
            if let Ok(decoded) = decode(&model_path) {
                let decoded_path = decoded.into_owned();
                decoded_path != model_path && self.has_file(&decoded_path)
            } else {
                false
            }
        };

        if !file_exists {
            return Err(Error::InvalidFormat(format!(
                "Model relationship points to non-existent file: {}",
                model_path
            )));
        }

        Ok(())
    }

    /// Validate all relationships point to existing files
    fn validate_all_relationships(&mut self) -> Result<()> {
        // Collect all .rels files in the archive
        let mut rels_files = Vec::new();
        for i in 0..self.archive.len() {
            if let Ok(file) = self.archive.by_index(i) {
                let name = file.name().to_string();
                if name.ends_with(".rels") {
                    rels_files.push(name);
                }
            }
        }

        // Validate each .rels file
        for rels_file in &rels_files {
            // For part-specific .rels files (e.g., 3D/_rels/3dmodel.model.rels),
            // verify the .rels file name matches the part file it references
            if rels_file.contains("/_rels/") && rels_file != RELS_PATH {
                // Extract the part name from the .rels file path
                // Format is: <dir>/_rels/<partname>.<ext>.rels
                let parts: Vec<&str> = rels_file.split("/_rels/").collect();
                if parts.len() == 2 {
                    let dir = parts[0];
                    let rels_filename = parts[1];

                    // Remove .rels suffix to get the part filename
                    if let Some(part_filename) = rels_filename.strip_suffix(".rels") {
                        // Reconstruct the expected part file path
                        let expected_part_path = if dir.is_empty() {
                            part_filename.to_string()
                        } else {
                            format!("{}/{}", dir, part_filename)
                        };

                        // Verify the corresponding part file exists
                        if !self.has_file(&expected_part_path) {
                            return Err(Error::InvalidFormat(format!(
                                "Relationship file '{}' references part '{}' which does not exist in the package.\n\
                                 Per OPC specification, part-specific relationship files must have names matching their associated parts.",
                                rels_file, expected_part_path
                            )));
                        }
                    }
                }
            }

            // Now validate the content of this .rels file
            let rels_content = self.get_file(rels_file)?;
            let mut reader = Reader::from_str(&rels_content);
            reader.config_mut().trim_text(true);
            let mut buf = Vec::new();

            let mut relationship_ids = std::collections::HashSet::new();
            let mut relationship_targets = std::collections::HashMap::new();

            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                        let name = e.name();
                        let name_str = std::str::from_utf8(name.as_ref())
                            .map_err(|e| Error::InvalidXml(e.to_string()))?;

                        if name_str.ends_with("Relationship") {
                            let mut target = None;
                            let mut rel_type = None;
                            let mut rel_id = None;

                            for attr in e.attributes() {
                                let attr = attr?;
                                let key = std::str::from_utf8(attr.key.as_ref())
                                    .map_err(|e| Error::InvalidXml(e.to_string()))?;
                                let value = std::str::from_utf8(&attr.value)
                                    .map_err(|e| Error::InvalidXml(e.to_string()))?;

                                match key {
                                    "Target" => target = Some(value.to_string()),
                                    "Type" => rel_type = Some(value.to_string()),
                                    "Id" => rel_id = Some(value.to_string()),
                                    _ => {}
                                }
                            }

                            // N_XPX_0413_01: Check for duplicate relationship IDs
                            if let Some(ref id) = rel_id {
                                if !relationship_ids.insert(id.clone()) {
                                    return Err(Error::InvalidFormat(format!(
                                        "Duplicate relationship ID '{}' in '{}'",
                                        id, rels_file
                                    )));
                                }

                                // N_XPX_0405_04: Check if ID starts with a digit (only for root .rels)
                                if rels_file == RELS_PATH {
                                    if let Some(first_char) = id.chars().next() {
                                        if first_char.is_ascii_digit() {
                                            return Err(Error::InvalidFormat(format!(
                                                "Relationship ID '{}' in root .rels cannot start with a digit",
                                                id
                                            )));
                                        }
                                    }
                                }
                            } else {
                                return Err(Error::InvalidFormat(format!(
                                    "Relationship missing required Id attribute in '{}'",
                                    rels_file
                                )));
                            }

                            // N_XPX_0405_03 & N_XPX_0405_05: Validate relationship Type values
                            if let Some(ref rt) = rel_type {
                                // N_XPX_0405_03: For 3dmodel.model.rels, check for incorrect model relationship type
                                if rels_file.contains("3dmodel.model.rels")
                                    && rt.contains("3dmodel")
                                    && rt != MODEL_REL_TYPE
                                {
                                    return Err(Error::InvalidFormat(format!(
                                        "Incorrect relationship Type '{}' in 3dmodel.model.rels",
                                        rt
                                    )));
                                }

                                // N_XPX_0405_05: For root .rels, check for incorrect thumbnail relationship type
                                if rels_file == RELS_PATH
                                    && rt.contains("thumbnail")
                                    && rt != THUMBNAIL_REL_TYPE
                                {
                                    return Err(Error::InvalidFormat(format!(
                                        "Incorrect thumbnail relationship Type '{}' in root .rels",
                                        rt
                                    )));
                                }

                                // Validate relationship Type - must not contain query strings or fragments
                                if rt.contains('?') {
                                    return Err(Error::InvalidFormat(format!(
                                        "Relationship Type in '{}' cannot contain query string: {}",
                                        rels_file, rt
                                    )));
                                }
                                if rt.contains('#') {
                                    return Err(Error::InvalidFormat(format!(
                                        "Relationship Type in '{}' cannot contain fragment identifier: {}",
                                        rels_file, rt
                                    )));
                                }
                            }

                            if let Some(t) = target {
                                // N_XPX_0406_01 & N_XPX_0406_02: Check for duplicate targets
                                if let Some(ref rt) = rel_type {
                                    let key = (t.clone(), rt.clone());
                                    if relationship_targets
                                        .insert(key, rel_id.clone().unwrap_or_default())
                                        .is_some()
                                    {
                                        // For root .rels with MODEL_REL_TYPE, this is N_XPX_0406_01
                                        // For other .rels files, this is N_XPX_0406_02
                                        return Err(Error::InvalidFormat(format!(
                                            "Duplicate relationship to same target '{}' with type '{}' in '{}'",
                                            t, rt, rels_file
                                        )));
                                    }
                                }

                                // Validate the target is a valid OPC part name
                                Self::validate_opc_part_name(&t)?;

                                // Remove leading slash if present
                                let path_with_slash = if let Some(stripped) = t.strip_prefix('/') {
                                    stripped.to_string()
                                } else {
                                    t.clone()
                                };

                                // Try to find the file in the ZIP archive.
                                // Per OPC spec, Target attributes should use percent-encoding for non-ASCII,
                                // but in practice, we may encounter:
                                // 1. Percent-encoded in XML, percent-encoded in ZIP (%C3%86 in both)
                                // 2. Percent-encoded in XML, UTF-8 in ZIP (%C3%86 in XML, Æ in ZIP)
                                // 3. UTF-8 in XML, UTF-8 in ZIP (Æ in both)
                                // We try both the original name and the URL-decoded name.
                                let file_exists = if self.has_file(&path_with_slash) {
                                    true
                                } else {
                                    // Try URL-decoding in case ZIP has UTF-8 but XML has percent-encoding
                                    if let Ok(decoded) = decode(&path_with_slash) {
                                        let decoded_path = decoded.into_owned();
                                        if decoded_path != path_with_slash {
                                            self.has_file(&decoded_path)
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    }
                                };

                                if !file_exists {
                                    return Err(Error::InvalidFormat(format!(
                                        "Relationship in '{}' points to non-existent file: {}",
                                        rels_file, path_with_slash
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
    /// - Control characters (newlines, tabs, etc.)
    ///
    /// Note: Per OPC spec (ECMA-376), Target attributes should use percent-encoding
    /// for non-ASCII characters. However, for compatibility with real-world files,
    /// we accept both percent-encoded and UTF-8 characters.
    fn validate_opc_part_name(part_name: &str) -> Result<()> {
        // Note: We don't strictly enforce ASCII-only here for compatibility.
        // Per OPC spec, non-ASCII should be percent-encoded, but many real-world
        // files include UTF-8 characters directly. We accept both and handle
        // URL-decoding when looking up files.

        // Check for control characters (newlines, tabs, etc.)
        // Per OPC spec, these are not allowed in part names
        if part_name.chars().any(|c| c.is_control()) {
            return Err(Error::InvalidFormat(format!(
                "Part name cannot contain control characters (newlines, tabs, etc.): {}",
                part_name.escape_debug()
            )));
        }

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

                                // Return the path as-is. The caller will handle trying both
                                // percent-encoded and decoded versions when accessing the file.
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
                                let path = Self::normalize_path(&t).to_string();
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

        // N_XPX_0419_01: Validate JPEG thumbnails are not CMYK
        if content_type.starts_with("image/jpeg") || content_type.starts_with("image/jpg") {
            let data = self.get_file_binary(&thumb_path)?;
            // Check if it's a JPEG (starts with FF D8 FF)
            if data.len() >= 3 && data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
                // Look for ALL SOF (Start of Frame) markers to determine color space
                // Note: JPEG files may have embedded thumbnails in EXIF data with different
                // color spaces than the main image, so we must check all SOF markers
                let mut i = 2;
                while i + 1 < data.len() {
                    if data[i] == 0xFF {
                        let marker = data[i + 1];
                        // SOF markers: 0xC0-0xCF (except 0xC4, 0xC8, 0xCC which are DHT, DAC, etc.)
                        if (0xC0..=0xCF).contains(&marker)
                            && marker != 0xC4
                            && marker != 0xC8
                            && marker != 0xCC
                        {
                            // SOF marker found, check component count
                            // JPEG SOF structure: FF marker [2 bytes length] [precision] [height] [width] [components]
                            // Component count is at offset +7 from marker start, or +9 from current position
                            const SOF_COMPONENT_COUNT_OFFSET: usize = 9;
                            if i + SOF_COMPONENT_COUNT_OFFSET < data.len() {
                                let num_components = data[i + SOF_COMPONENT_COUNT_OFFSET];
                                // 4 components typically indicates CMYK (or YCCK)
                                if num_components == 4 {
                                    return Err(Error::InvalidFormat(
                                        "Thumbnail JPEG uses CMYK color space, only RGB is allowed"
                                            .to_string(),
                                    ));
                                }
                            }
                            // Don't break - continue checking for more SOF markers
                            // (file may have embedded thumbnails with different color spaces)
                        }
                        // Skip this marker - length includes the 2-byte length field itself
                        if i + 3 < data.len() {
                            let len = ((data[i + 2] as usize) << 8) | (data[i + 3] as usize);
                            // Verify we won't overflow: check that len is at least 2 and won't cause overflow
                            if len >= 2 {
                                // Use saturating_add to prevent overflow
                                let next_pos = i.saturating_add(len).saturating_add(2);
                                if next_pos <= data.len() {
                                    i = next_pos;
                                } else {
                                    break; // Invalid marker, stop parsing
                                }
                            } else {
                                break; // Invalid length, stop parsing
                            }
                        } else {
                            break;
                        }
                    } else {
                        i += 1;
                    }
                }
            }
        }

        // Note: While thumbnails are typically image/* content types, some valid 3MF files
        // (per the official test suite) may use other content types for thumbnail relationships.
        // For example, model files can be referenced as thumbnails in certain production extension contexts.
        // We accept all content types but prefer image/* types.

        Ok(Some(crate::model::Thumbnail::new(thumb_path, content_type)))
    }

    /// Validate that thumbnails are not defined ONLY in model-level relationship files
    ///
    /// Per 3MF Core Specification and OPC (Open Packaging Conventions),
    /// if a thumbnail relationship is defined at the part/model level
    /// (e.g., 3D/_rels/3dmodel.model.rels), there MUST also be a thumbnail
    /// relationship at the package level (_rels/.rels).
    ///
    /// This validation checks all relationship files in the package and returns
    /// an error if any non-root relationship file contains a thumbnail relationship
    /// but no thumbnail exists at the package level.
    ///
    /// Test cases: N_SPX_0417_01, N_SPX_0419_01
    pub fn validate_no_model_level_thumbnails(&mut self) -> Result<()> {
        // First, check if there's a package-level thumbnail using proper XML parsing
        let has_package_thumbnail = if self.has_file(RELS_PATH) {
            let rels_content = self.get_file(RELS_PATH)?;
            let mut reader = Reader::from_str(&rels_content);
            reader.config_mut().trim_text(true);
            let mut buf = Vec::new();
            let mut found = false;

            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                        let name = e.name();
                        let name_str = std::str::from_utf8(name.as_ref())
                            .map_err(|e| Error::InvalidXml(e.to_string()))?;

                        if name_str.ends_with("Relationship") {
                            for attr in e.attributes() {
                                let attr = attr?;
                                let key = std::str::from_utf8(attr.key.as_ref())
                                    .map_err(|e| Error::InvalidXml(e.to_string()))?;
                                let value = std::str::from_utf8(&attr.value)
                                    .map_err(|e| Error::InvalidXml(e.to_string()))?;

                                if key == "Type" && value == THUMBNAIL_REL_TYPE {
                                    found = true;
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
                if found {
                    break;
                }
            }
            found
        } else {
            false
        };

        // If there's a package-level thumbnail, we're done (model-level thumbnails are OK)
        if has_package_thumbnail {
            return Ok(());
        }

        // No package-level thumbnail - check if any model-level thumbnails exist
        // Only need to check the main model relationships file
        if self.has_file(MODEL_RELS_PATH) {
            let rels_content = self.get_file(MODEL_RELS_PATH)?;
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
                            for attr in e.attributes() {
                                let attr = attr?;
                                let key = std::str::from_utf8(attr.key.as_ref())
                                    .map_err(|e| Error::InvalidXml(e.to_string()))?;
                                let value = std::str::from_utf8(&attr.value)
                                    .map_err(|e| Error::InvalidXml(e.to_string()))?;

                                if key == "Type" && value == THUMBNAIL_REL_TYPE {
                                    return Err(Error::InvalidFormat(format!(
                                        "Thumbnail relationship found in model-level relationship file '{}' \
                                         but no thumbnail relationship exists at the package level. \
                                         Per 3MF Core Specification and OPC standard, if thumbnail relationships \
                                         are defined at the part/model level, a thumbnail relationship \
                                         MUST also be defined at the package level (_rels/.rels).",
                                        MODEL_RELS_PATH
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
        }

        Ok(())
    }

    /// Discover keystore file path from package relationships
    ///
    /// Returns the path to the keystore file if one exists, or None if no keystore is found.
    /// The keystore is identified by relationship type for the Secure Content extension.
    pub fn discover_keystore_path(&mut self) -> Result<Option<String>> {
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

                            match key {
                                "Target" => target = Some(value.to_string()),
                                "Type" => rel_type = Some(value.to_string()),
                                _ => {}
                            }
                        }

                        // Check if this is a keystore relationship (support both 2019/04 and 2019/07)
                        if let (Some(t), Some(rt)) = (target, rel_type) {
                            if rt == KEYSTORE_REL_TYPE_2019_04 || rt == KEYSTORE_REL_TYPE_2019_07 {
                                // Remove leading slash if present
                                let path = if let Some(stripped) = t.strip_prefix('/') {
                                    stripped.to_string()
                                } else {
                                    t
                                };
                                return Ok(Some(path));
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

        Ok(None)
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
        // Normalize the target path for comparison (remove leading slash)
        let target_normalized = target_path.trim_start_matches('/');

        // Build a list of .rels files to check
        let mut rels_files_to_check = Vec::new();

        if let Some(source) = source_file {
            // If a source file is specified, check only its corresponding .rels file
            let source_normalized = source.trim_start_matches('/');
            // Construct the .rels file path for this source file
            // Format: <dir>/_rels/<filename>.rels
            if let Some(slash_pos) = source_normalized.rfind('/') {
                let dir = &source_normalized[..slash_pos];
                let filename = &source_normalized[slash_pos + 1..];
                let rels_path = format!("{}/_rels/{}.rels", dir, filename);
                rels_files_to_check.push(rels_path);
            } else {
                // File is at root level
                let rels_path = format!("_rels/{}.rels", source_normalized);
                rels_files_to_check.push(rels_path);
            }
        } else {
            // No source file specified - check ALL .rels files in the package
            // Note: This is only called during validation for encrypted files,
            // which is infrequent, so performance impact is minimal
            for i in 0..self.archive.len() {
                if let Ok(file) = self.archive.by_index(i) {
                    let name = file.name();
                    if name.ends_with(".rels") {
                        rels_files_to_check.push(name.to_string());
                    }
                }
            }
        }

        // Check each .rels file
        for rels_file in &rels_files_to_check {
            if !self.has_file(rels_file) {
                continue; // This .rels file doesn't exist, skip it
            }

            let rels_content = match self.get_file(rels_file) {
                Ok(content) => content,
                Err(_e) => {
                    // Failed to read .rels file (e.g., permission error, corrupt file)
                    // Skip this file and continue validation with other .rels files
                    continue;
                }
            };

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

                            // Check if this relationship matches what we're looking for
                            if let (Some(t), Some(rt)) = (target, rel_type) {
                                let t_normalized = t.trim_start_matches('/');
                                if rt == relationship_type && t_normalized == target_normalized {
                                    return Ok(true);
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

        Ok(false)
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
        let rels_content = self.get_file(RELS_PATH)?;
        let mut reader = Reader::from_str(&rels_content);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();

        // Normalize the keystore path for comparison
        let keystore_normalized = keystore_path.trim_start_matches('/');

        let mut has_valid_keystore_rel = false;

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

                        // Check if this is a keystore relationship with correct type
                        if let (Some(t), Some(rt)) = (target, rel_type) {
                            let t_normalized = t.trim_start_matches('/');
                            if t_normalized == keystore_normalized
                                && (rt == KEYSTORE_REL_TYPE_2019_04
                                    || rt == KEYSTORE_REL_TYPE_2019_07)
                            {
                                has_valid_keystore_rel = true;
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

        if !has_valid_keystore_rel {
            return Err(Error::InvalidSecureContent(format!(
                "Keystore file '{}' is missing required keystore relationship in root .rels. \
                 Per 3MF SecureContent specification, the keystore must have a relationship of type \
                 '{}' or '{}' (EPX-2606)",
                keystore_path, KEYSTORE_REL_TYPE_2019_04, KEYSTORE_REL_TYPE_2019_07
            )));
        }

        Ok(())
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
        let content = self.get_file(CONTENT_TYPES_PATH)?;
        let mut reader = Reader::from_str(&content);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();

        // Normalize the keystore path for comparison
        let keystore_normalized = Self::normalize_path(keystore_path);

        let mut has_override = false;
        let mut has_xml_default = false;

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

                        if let Some(pn) = part_name {
                            let pn_normalized = Self::normalize_path(&pn);
                            if pn_normalized == keystore_normalized {
                                // Check if it's the correct content type
                                if let Some(ct) = content_type {
                                    if ct
                                        == "application/vnd.ms-package.3dmanufacturing-keystore+xml"
                                    {
                                        has_override = true;
                                    }
                                }
                            }
                        }
                    } else if name_str.ends_with("Default") {
                        // Check for Default extension="xml" with keystore content type
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
                            if e == "xml"
                                && ct == "application/vnd.ms-package.3dmanufacturing-keystore+xml"
                            {
                                has_xml_default = true;
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

        if !has_override && !has_xml_default {
            return Err(Error::InvalidSecureContent(format!(
                "Keystore file '{}' is missing required content type in [Content_Types].xml. \
                 Per 3MF SecureContent specification, the keystore must have either an Override \
                 or a Default for .xml extension with content type \
                 'application/vnd.ms-package.3dmanufacturing-keystore+xml' (EPX-2606)",
                keystore_path
            )));
        }

        Ok(())
    }

    /// Get content type for a file from [Content_Types].xml
    fn get_content_type(&mut self, path: &str) -> Result<String> {
        let content = self.get_file(CONTENT_TYPES_PATH)?;
        let mut reader = Reader::from_str(&content);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();

        let path_normalized = Self::normalize_path(path);
        let extension = path.rsplit('.').next();

        // Parse content types file once, checking both Override and Default elements
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let name_str = std::str::from_utf8(name.as_ref())
                        .map_err(|e| Error::InvalidXml(e.to_string()))?;

                    // Check for Override elements (specific path matches)
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
                            let pn_normalized = Self::normalize_path(&pn);
                            if pn_normalized == path_normalized {
                                return Ok(ct);
                            }
                        }
                    }
                    // Check for Default elements (extension-based matches)
                    else if name_str.ends_with("Default") {
                        if let Some(ext) = extension {
                            let mut ext_attr = None;
                            let mut content_type = None;

                            for attr in e.attributes() {
                                let attr = attr?;
                                let key = std::str::from_utf8(attr.key.as_ref())
                                    .map_err(|e| Error::InvalidXml(e.to_string()))?;
                                let value = std::str::from_utf8(&attr.value)
                                    .map_err(|e| Error::InvalidXml(e.to_string()))?;

                                match key {
                                    "Extension" => ext_attr = Some(value.to_string()),
                                    "ContentType" => content_type = Some(value.to_string()),
                                    _ => {}
                                }
                            }

                            if let (Some(e), Some(ct)) = (ext_attr, content_type) {
                                if e.eq_ignore_ascii_case(ext) {
                                    return Ok(ct);
                                }
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

        Err(Error::InvalidFormat(format!(
            "No content type found for file: {}",
            path
        )))
    }

    /// Normalize OPC path by removing leading slash
    fn normalize_path(path: &str) -> &str {
        path.strip_prefix('/').unwrap_or(path)
    }
}

/// Create a 3MF package (ZIP archive) from model data
///
/// This function creates a complete 3MF file including:
/// - [Content_Types].xml
/// - _rels/.rels
/// - 3D/3dmodel.model
///
/// # Arguments
///
/// * `writer` - The writer to write the 3MF package to
/// * `model_xml` - The XML content of the 3D model
///
/// # Returns
///
/// Returns the writer after finishing the ZIP archive
pub fn create_package<W: std::io::Write + std::io::Seek>(writer: W, model_xml: &str) -> Result<W> {
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let mut zip = ZipWriter::new(writer);
    let options = SimpleFileOptions::default();

    // Write [Content_Types].xml
    let content_types = r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
</Types>"#;

    zip.start_file(CONTENT_TYPES_PATH, options)
        .map_err(|e| Error::xml_write(format!("Failed to create Content_Types file: {}", e)))?;
    zip.write_all(content_types.as_bytes())
        .map_err(|e| Error::xml_write(format!("Failed to write Content_Types: {}", e)))?;

    // Write _rels/.rels
    let rels = r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Target="/3D/3dmodel.model" Id="rel0" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"#;

    zip.start_file(RELS_PATH, options)
        .map_err(|e| Error::xml_write(format!("Failed to create relationships file: {}", e)))?;
    zip.write_all(rels.as_bytes())
        .map_err(|e| Error::xml_write(format!("Failed to write relationships: {}", e)))?;

    // Write 3D/3dmodel.model
    zip.start_file(MODEL_PATH, options)
        .map_err(|e| Error::xml_write(format!("Failed to create model file: {}", e)))?;
    zip.write_all(model_xml.as_bytes())
        .map_err(|e| Error::xml_write(format!("Failed to write model XML: {}", e)))?;

    // Finish and return the writer
    let writer = zip
        .finish()
        .map_err(|e| Error::xml_write(format!("Failed to finalize ZIP archive: {}", e)))?;

    Ok(writer)
}

/// Create a 3MF package with thumbnail support
///
/// This function creates a complete 3MF file including:
/// - [Content_Types].xml (with thumbnail content type)
/// - _rels/.rels (with thumbnail relationship)
/// - 3D/3dmodel.model
/// - Metadata/thumbnail.png (or other format)
///
/// # Arguments
///
/// * `writer` - The writer to write the 3MF package to
/// * `model_xml` - The XML content of the 3D model
/// * `thumbnail_data` - Optional thumbnail image data
/// * `thumbnail_content_type` - Content type of thumbnail (e.g., "image/png")
///
/// # Returns
///
/// Returns the writer after finishing the ZIP archive
pub fn create_package_with_thumbnail<W: std::io::Write + std::io::Seek>(
    writer: W,
    model_xml: &str,
    thumbnail_data: Option<&[u8]>,
    thumbnail_content_type: Option<&str>,
) -> Result<W> {
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let mut zip = ZipWriter::new(writer);
    let options = SimpleFileOptions::default();

    let has_thumbnail = thumbnail_data.is_some();
    let thumbnail_extension = if let Some(content_type) = thumbnail_content_type {
        match content_type {
            "image/png" => "png",
            "image/jpeg" => "jpeg",
            _ => "png", // default
        }
    } else {
        "png"
    };

    // Write [Content_Types].xml with optional thumbnail
    let content_types = if has_thumbnail {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
  <Default Extension="{}" ContentType="{}"/>
</Types>"#,
            thumbnail_extension,
            thumbnail_content_type.unwrap_or("image/png")
        )
    } else {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
</Types>"#
            .to_string()
    };

    zip.start_file(CONTENT_TYPES_PATH, options)
        .map_err(|e| Error::xml_write(format!("Failed to create Content_Types file: {}", e)))?;
    zip.write_all(content_types.as_bytes())
        .map_err(|e| Error::xml_write(format!("Failed to write Content_Types: {}", e)))?;

    // Write _rels/.rels with optional thumbnail relationship
    let rels = if has_thumbnail {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Target="/3D/3dmodel.model" Id="rel0" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
  <Relationship Target="/Metadata/thumbnail.{}" Id="rel1" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail"/>
</Relationships>"#,
            thumbnail_extension
        )
    } else {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Target="/3D/3dmodel.model" Id="rel0" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"#
            .to_string()
    };

    zip.start_file(RELS_PATH, options)
        .map_err(|e| Error::xml_write(format!("Failed to create relationships file: {}", e)))?;
    zip.write_all(rels.as_bytes())
        .map_err(|e| Error::xml_write(format!("Failed to write relationships: {}", e)))?;

    // Write thumbnail if provided
    if let Some(thumb_data) = thumbnail_data {
        let thumbnail_path = format!("Metadata/thumbnail.{}", thumbnail_extension);
        zip.start_file(&thumbnail_path, options)
            .map_err(|e| Error::xml_write(format!("Failed to create thumbnail file: {}", e)))?;
        zip.write_all(thumb_data)
            .map_err(|e| Error::xml_write(format!("Failed to write thumbnail data: {}", e)))?;
    }

    // Write 3D/3dmodel.model
    zip.start_file(MODEL_PATH, options)
        .map_err(|e| Error::xml_write(format!("Failed to create model file: {}", e)))?;
    zip.write_all(model_xml.as_bytes())
        .map_err(|e| Error::xml_write(format!("Failed to write model XML: {}", e)))?;

    // Finish and return the writer
    let writer = zip
        .finish()
        .map_err(|e| Error::xml_write(format!("Failed to finalize ZIP archive: {}", e)))?;

    Ok(writer)
}

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
