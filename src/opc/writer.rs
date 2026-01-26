//! Package writing functionality for creating 3MF files

use crate::error::{Error, Result};
use super::{CONTENT_TYPES_PATH, MODEL_PATH, RELS_PATH};

/// Create a 3MF package (ZIP archive) from model data
///
/// This function creates a complete 3MF file including:
/// - `[Content_Types].xml`
/// - `_rels/.rels`
/// - `3D/3dmodel.model`
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
/// - `[Content_Types].xml` (with thumbnail content type)
/// - `_rels/.rels` (with thumbnail relationship)
/// - `3D/3dmodel.model`
/// - `Metadata/thumbnail.png` (or other format)
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
