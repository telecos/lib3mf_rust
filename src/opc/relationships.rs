//! Relationship discovery and validation

use crate::error::{Error, Result};
use super::{Package, RELS_PATH, KEYSTORE_REL_TYPE_2019_04, KEYSTORE_REL_TYPE_2019_07};
use super::reader::{get_file, has_file};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::Read;

/// Discover keystore file path from package relationships
pub(super) fn discover_keystore_path<R: Read + std::io::Seek>(
    package: &mut Package<R>,
) -> Result<Option<String>> {
    let rels_content = get_file(package, RELS_PATH)?;
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
pub(super) fn has_relationship_to_target<R: Read + std::io::Seek>(
    package: &mut Package<R>,
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
        for i in 0..package.archive.len() {
            if let Ok(file) = package.archive.by_index(i) {
                let name = file.name();
                if name.ends_with(".rels") {
                    rels_files_to_check.push(name.to_string());
                }
            }
        }
    }

    // Check each .rels file
    for rels_file in &rels_files_to_check {
        if !has_file(package, rels_file) {
            continue; // This .rels file doesn't exist, skip it
        }

        let rels_content = match get_file(package, rels_file) {
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

/// Validate keystore relationship
pub(super) fn validate_keystore_relationship<R: Read + std::io::Seek>(
    package: &mut Package<R>,
    keystore_path: &str,
) -> Result<()> {
    let rels_content = get_file(package, RELS_PATH)?;
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
