//! Thumbnail handling functionality

use super::reader::{get_file, get_file_binary, has_file};
use super::{CONTENT_TYPES_PATH, MODEL_RELS_PATH, Package, RELS_PATH, THUMBNAIL_REL_TYPE};
use crate::error::{Error, Result};
use quick_xml::Reader;
use quick_xml::events::Event;
use std::io::Read;

/// Normalize OPC path by removing leading slash
fn normalize_path(path: &str) -> &str {
    path.strip_prefix('/').unwrap_or(path)
}

/// Get content type for a file from [Content_Types].xml
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

/// Get thumbnail metadata from the package
pub(super) fn get_thumbnail_metadata<R: Read + std::io::Seek>(
    package: &mut Package<R>,
) -> Result<Option<crate::model::Thumbnail>> {
    // Check if relationships file exists
    if !has_file(package, RELS_PATH) {
        return Ok(None);
    }

    // Parse relationships to find thumbnail
    let rels_content = get_file(package, RELS_PATH)?;
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
                    if let (Some(t), Some(rt)) = (target, rel_type)
                        && rt == THUMBNAIL_REL_TYPE
                    {
                        let path = normalize_path(&t).to_string();
                        thumbnail_path = Some(path);
                        break;
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
    if !has_file(package, &thumb_path) {
        return Err(Error::InvalidFormat(format!(
            "Thumbnail relationship points to non-existent file: {}",
            thumb_path
        )));
    }

    // Get content type from [Content_Types].xml
    let content_type = get_content_type(package, &thumb_path)?;

    // N_XPX_0419_01: Validate JPEG thumbnails are not CMYK
    if content_type.starts_with("image/jpeg") || content_type.starts_with("image/jpg") {
        let data = get_file_binary(package, &thumb_path)?;
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

/// Validate no model-level thumbnails exist
pub(super) fn validate_no_model_level_thumbnails<R: Read + std::io::Seek>(
    package: &mut Package<R>,
) -> Result<()> {
    // First, check if there's a package-level thumbnail using proper XML parsing
    let has_package_thumbnail = if has_file(package, RELS_PATH) {
        let rels_content = get_file(package, RELS_PATH)?;
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
    if has_file(package, MODEL_RELS_PATH) {
        let rels_content = get_file(package, MODEL_RELS_PATH)?;
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
