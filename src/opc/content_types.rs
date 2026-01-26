//! Content types parsing and validation

use crate::error::{Error, Result};
use super::{Package, CONTENT_TYPES_PATH};
use super::reader::get_file;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::Read;

/// Normalize OPC path by removing leading slash
fn normalize_path(path: &str) -> &str {
    path.strip_prefix('/').unwrap_or(path)
}

/// Validate keystore content type
pub(super) fn validate_keystore_content_type<R: Read + std::io::Seek>(
    package: &mut Package<R>,
    keystore_path: &str,
) -> Result<()> {
    let content = get_file(package, CONTENT_TYPES_PATH)?;
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
