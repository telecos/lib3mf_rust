//! Content type parsing and validation

use crate::error::{Error, Result};
use crate::opc::CONTENT_TYPES_PATH;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::Read;
use zip::ZipArchive;

use super::validation::normalize_path;

/// Validate [Content_Types].xml structure
pub(crate) fn validate_content_types<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
) -> Result<()> {
    let content = get_file_content(archive, CONTENT_TYPES_PATH)?;
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

/// Get content type for a file from [Content_Types].xml
pub(crate) fn get_content_type<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    path: &str,
) -> Result<String> {
    let content = get_file_content(archive, CONTENT_TYPES_PATH)?;
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

/// Validate that a keystore file has the correct content type override
///
/// EPX-2606 validation: If a keystore file exists, it must have a content type
/// defined in [Content_Types].xml, either as an Override for the specific file
/// or as a Default for the .xml extension.
pub(crate) fn validate_keystore_content_type<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    keystore_path: &str,
) -> Result<()> {
    let content = get_file_content(archive, CONTENT_TYPES_PATH)?;
    let mut reader = Reader::from_str(&content);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    // Normalize the keystore path for comparison
    let keystore_normalized = normalize_path(keystore_path);

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
                        let pn_normalized = normalize_path(&pn);
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
