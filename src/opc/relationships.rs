//! OPC relationship parsing and validation

use crate::error::{Error, Result};
use crate::opc::RELS_PATH;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::Read;
use urlencoding::decode;
use zip::ZipArchive;

use super::validation::validate_opc_part_name;

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

/// Discover the model file path from the relationships file
pub(crate) fn discover_model_path<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
) -> Result<String> {
    // Read the _rels/.rels file
    let rels_content = get_file_content(archive, RELS_PATH)?;

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

/// Validate model relationship exists and points to a valid file
pub(crate) fn validate_model_relationship<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
) -> Result<()> {
    let model_path = discover_model_path(archive)?;

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
    let file_exists = has_file(archive, &model_path) || {
        if let Ok(decoded) = decode(&model_path) {
            let decoded_path = decoded.into_owned();
            decoded_path != model_path && has_file(archive, &decoded_path)
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
pub(crate) fn validate_all_relationships<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
) -> Result<()> {
    // Collect all .rels files in the archive
    let mut rels_files = Vec::new();
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
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
                    if !has_file(archive, &expected_part_path) {
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
        let rels_content = get_file_content(archive, rels_file)?;
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

                            // N_XXM_0605_01: For 3dmodel.model.rels, check for incorrect texture relationship type
                            // If relationship target appears to be an image file (png/jpeg), it should use
                            // TEXTURE_REL_TYPE, not MODEL_REL_TYPE
                            if rels_file.contains("3dmodel.model.rels") {
                                if let Some(ref t) = target {
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
                            let file_exists = if has_file(archive, &path_with_slash) {
                                true
                            } else {
                                // Try URL-decoding in case ZIP has UTF-8 but XML has percent-encoding
                                if let Ok(decoded) = decode(&path_with_slash) {
                                    let decoded_path = decoded.into_owned();
                                    if decoded_path != path_with_slash {
                                        has_file(archive, &decoded_path)
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

/// Discover keystore file path from package relationships
///
/// Returns the path to the keystore file if one exists, or None if no keystore is found.
/// The keystore is identified by relationship type for the Secure Content extension.
pub(crate) fn discover_keystore_path<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
) -> Result<Option<String>> {
    let rels_content = get_file_content(archive, RELS_PATH)?;
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
pub(crate) fn has_relationship_to_target<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
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
        for i in 0..archive.len() {
            if let Ok(file) = archive.by_index(i) {
                let name = file.name();
                if name.ends_with(".rels") {
                    rels_files_to_check.push(name.to_string());
                }
            }
        }
    }

    // Check each .rels file
    for rels_file in &rels_files_to_check {
        if !has_file(archive, rels_file) {
            continue; // This .rels file doesn't exist, skip it
        }

        let rels_content = match get_file_content(archive, rels_file) {
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
pub(crate) fn validate_keystore_relationship<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    keystore_path: &str,
) -> Result<()> {
    let rels_content = get_file_content(archive, RELS_PATH)?;
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

/// Helper function to get file content as String from archive
fn get_file_content<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    name: &str,
) -> Result<String> {
    let mut file = archive
        .by_name(name)
        .map_err(|_| Error::MissingFile(name.to_string()))?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

/// Helper function to check if file exists in archive
fn has_file<R: Read + std::io::Seek>(archive: &mut ZipArchive<R>, name: &str) -> bool {
    archive.by_name(name).is_ok()
}
