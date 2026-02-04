//! Package reading and validation functionality

use super::{
    CONTENT_TYPES_PATH, MODEL_REL_TYPE, Package, RELS_PATH, TEXTURE_REL_TYPE, THUMBNAIL_REL_TYPE,
};
use crate::error::{Error, Result};
use quick_xml::Reader;
use quick_xml::events::Event;
use std::io::Read;
use urlencoding::decode;
use zip::ZipArchive;

/// Open a 3MF package from a reader
pub(super) fn open<R: Read + std::io::Seek>(reader: R) -> Result<Package<R>> {
    let archive = ZipArchive::new(reader)?;
    let mut package = Package { archive };

    // Validate required OPC structure
    validate_opc_structure(&mut package)?;

    Ok(package)
}

/// Validate OPC package structure according to 3MF spec
fn validate_opc_structure<R: Read + std::io::Seek>(package: &mut Package<R>) -> Result<()> {
    // Validate required files exist
    if !has_file(package, CONTENT_TYPES_PATH) {
        return Err(Error::invalid_format_context(
            "OPC package structure",
            &format!(
                "Missing required file '{}'. \
                 This file defines content types for the package and is required by the OPC specification. \
                 The 3MF file may be corrupt or improperly formatted.",
                CONTENT_TYPES_PATH
            ),
        ));
    }

    if !has_file(package, RELS_PATH) {
        return Err(Error::invalid_format_context(
            "OPC package structure",
            &format!(
                "Missing required file '{}'. \
                 This file defines package relationships and is required by the OPC specification. \
                 The 3MF file may be corrupt or improperly formatted.",
                RELS_PATH
            ),
        ));
    }

    // Validate Content Types
    validate_content_types(package)?;

    // Validate that model relationship exists and points to valid file
    validate_model_relationship(package)?;

    // Validate all relationships point to existing files
    validate_all_relationships(package)?;

    Ok(())
}

/// Validate [Content_Types].xml structure
fn validate_content_types<R: Read + std::io::Seek>(package: &mut Package<R>) -> Result<()> {
    let content = get_file(package, CONTENT_TYPES_PATH)?;
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
                    if let Some(ref pn) = part_name
                        && pn.is_empty()
                    {
                        return Err(Error::InvalidFormat(
                            "Content type Override element cannot have empty PartName attribute"
                                .to_string(),
                        ));
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
             Check that the model file has a proper content type declaration.",
        ));
    }

    Ok(())
}

/// Validate model relationship exists and points to a valid file
fn validate_model_relationship<R: Read + std::io::Seek>(package: &mut Package<R>) -> Result<()> {
    let model_path = discover_model_path(package)?;

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
    let file_exists = has_file(package, &model_path) || {
        if let Ok(decoded) = decode(&model_path) {
            let decoded_path = decoded.into_owned();
            decoded_path != model_path && has_file(package, &decoded_path)
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
fn validate_all_relationships<R: Read + std::io::Seek>(package: &mut Package<R>) -> Result<()> {
    // Collect all .rels files in the archive
    let mut rels_files = Vec::new();
    for i in 0..package.archive.len() {
        if let Ok(file) = package.archive.by_index(i) {
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
                    if !has_file(package, &expected_part_path) {
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
        let rels_content = get_file(package, rels_file)?;
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
                            if rels_file == RELS_PATH
                                && let Some(first_char) = id.chars().next()
                                && first_char.is_ascii_digit()
                            {
                                return Err(Error::InvalidFormat(format!(
                                    "Relationship ID '{}' in root .rels cannot start with a digit",
                                    id
                                )));
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

                            // N_XXM_0605_01: For 3dmodel.model.rels, check for incorrect texture relationship type
                            // If relationship target appears to be an image file (png/jpeg), it should use
                            // TEXTURE_REL_TYPE, not MODEL_REL_TYPE
                            if rels_file.contains("3dmodel.model.rels")
                                && let Some(ref t) = target
                            {
                                let target_lower = t.to_lowercase();
                                if (target_lower.ends_with(".png")
                                    || target_lower.ends_with(".jpeg")
                                    || target_lower.ends_with(".jpg"))
                                    && rt == MODEL_REL_TYPE
                                {
                                    return Err(Error::InvalidFormat(format!(
                                        "Incorrect relationship Type '{}' for texture file '{}' in 3dmodel.model.rels.\n\
                                             Per 3MF Material Extension spec, texture files must use relationship type '{}'.",
                                        rt, t, TEXTURE_REL_TYPE
                                    )));
                                }
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
                            validate_opc_part_name(&t)?;

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
                            let file_exists = if has_file(package, &path_with_slash) {
                                true
                            } else {
                                // Try URL-decoding in case ZIP has UTF-8 but XML has percent-encoding
                                if let Ok(decoded) = decode(&path_with_slash) {
                                    let decoded_path = decoded.into_owned();
                                    if decoded_path != path_with_slash {
                                        has_file(package, &decoded_path)
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

/// Get the main 3D model file content
pub(super) fn get_model<R: Read + std::io::Seek>(package: &mut Package<R>) -> Result<String> {
    // Discover model path from relationships (validation already done in open())
    let model_path = discover_model_path(package)?;

    // Determine which path to use: try the original first, then decoded
    let path_to_use = if has_file(package, &model_path) {
        model_path.clone()
    } else {
        // If the direct path fails, try URL-decoding
        if let Ok(decoded) = decode(&model_path) {
            let decoded_path = decoded.into_owned();
            if decoded_path != model_path && has_file(package, &decoded_path) {
                decoded_path
            } else {
                return Err(Error::MissingFile(model_path));
            }
        } else {
            return Err(Error::MissingFile(model_path));
        }
    };

    // Now read the file
    let mut file = package
        .archive
        .by_name(&path_to_use)
        .map_err(|_| Error::MissingFile(path_to_use.clone()))?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    Ok(content)
}

/// Get a file from the package by name
pub(super) fn get_file<R: Read + std::io::Seek>(
    package: &mut Package<R>,
    name: &str,
) -> Result<String> {
    let mut file = package
        .archive
        .by_name(name)
        .map_err(|_| Error::MissingFile(name.to_string()))?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

/// Check if a file exists in the package
pub(super) fn has_file<R: Read + std::io::Seek>(package: &mut Package<R>, name: &str) -> bool {
    package.archive.by_name(name).is_ok()
}

/// Get the number of files in the package
pub(super) fn len<R: Read + std::io::Seek>(package: &Package<R>) -> usize {
    package.archive.len()
}

/// Check if the package is empty
pub(super) fn is_empty<R: Read + std::io::Seek>(package: &Package<R>) -> bool {
    package.archive.is_empty()
}

/// Get a list of all file names in the package
pub(super) fn file_names<R: Read + std::io::Seek>(package: &mut Package<R>) -> Vec<String> {
    (0..package.archive.len())
        .filter_map(|i| {
            package
                .archive
                .by_index(i)
                .ok()
                .map(|f| f.name().to_string())
        })
        .collect()
}

/// Get a file as binary data
pub(super) fn get_file_binary<R: Read + std::io::Seek>(
    package: &mut Package<R>,
    name: &str,
) -> Result<Vec<u8>> {
    let mut file = package
        .archive
        .by_name(name)
        .map_err(|_| Error::MissingFile(name.to_string()))?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    Ok(content)
}

/// Discover the model file path from the relationships file
fn discover_model_path<R: Read + std::io::Seek>(package: &mut Package<R>) -> Result<String> {
    // Read the _rels/.rels file
    let rels_content = get_file(package, RELS_PATH)?;

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
                    if let (Some(t), Some(rt)) = (target, rel_type)
                        && rt == MODEL_REL_TYPE
                    {
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

/// Get content type for a file path
#[allow(dead_code)]
fn get_content_type<R: Read + std::io::Seek>(
    package: &mut Package<R>,
    path: &str,
) -> Result<String> {
    let content = get_file(package, CONTENT_TYPES_PATH)?;
    let mut reader = Reader::from_str(&content);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    let path_normalized = normalize_path(path);
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
                        let pn_normalized = normalize_path(&pn);
                        if pn_normalized == path_normalized {
                            return Ok(ct);
                        }
                    }
                }
                // Check for Default elements (extension-based matches)
                else if name_str.ends_with("Default")
                    && let Some(ext) = extension
                {
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

                    if let (Some(e), Some(ct)) = (ext_attr, content_type)
                        && e.eq_ignore_ascii_case(ext)
                    {
                        return Ok(ct);
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
