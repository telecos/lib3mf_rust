//! Material extension parsing
//!
//! This module handles parsing of 3MF Material extension elements including
//! base materials, color groups, textures, composites, and multi-properties.

use crate::error::Result;
use crate::model::Material;
use crate::opc::Package;
use crate::Model;
use quick_xml::Reader;
use std::io::Read;

use super::{parse_attributes, parse_color, validate_attributes};

/// Parse material (base) element attributes
/// Base materials within a basematerials group use sequential indices (0, 1, 2, ...)
pub(super) fn parse_base_material<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
    index: usize,
) -> Result<Material> {
    let attrs = parse_attributes(reader, e)?;

    // Validate only allowed attributes are present
    // Per 3MF Core spec: name, displaycolor
    validate_attributes(&attrs, &["name", "displaycolor"], "base")?;

    // Use the provided index as the material ID
    let mut material = Material::new(index);
    material.name = attrs.get("name").cloned();

    // Parse displaycolor attribute (format: #RRGGBBAA or #RRGGBB)
    if let Some(color_str) = attrs.get("displaycolor") {
        if let Some(color) = parse_color(color_str) {
            material.color = Some(color);
        }
    }

    Ok(material)
}

/// Validate texture file paths exist in the 3MF package
pub(super) fn validate_texture_file_paths<R: Read + std::io::Seek>(
    package: &mut Package<R>,
    model: &Model,
) -> Result<()> {
    use crate::error::Error;
    
    // Get list of encrypted files to skip validation for them
    let encrypted_files: Vec<String> = model
        .secure_content
        .as_ref()
        .map(|sc| sc.encrypted_files.clone())
        .unwrap_or_default();

    for texture in &model.resources.texture2d_resources {
        // Skip validation for encrypted files (they may not follow standard paths)
        if encrypted_files.contains(&texture.path) {
            continue;
        }

        // Normalize path: remove leading slash if present for lookup
        // The path in the model may start with "/" but the ZIP file paths typically don't
        let normalized_path = texture.path.trim_start_matches('/');

        // Check if file exists in the package
        // Try both with and without leading slash as different 3MF implementations vary
        let file_exists = package.has_file(normalized_path) || package.has_file(&texture.path);

        if !file_exists {
            return Err(Error::InvalidModel(format!(
                "Texture2D resource {}: Path '{}' references a file that does not exist in the 3MF package.\n\
                 Per 3MF Material Extension spec, texture paths must reference valid files in the package.\n\
                 Check that:\n\
                 - The texture file is included in the 3MF package\n\
                 - The path is correct (case-sensitive)\n\
                 - The path format follows 3MF conventions\n\
                 Available files can be checked using ZIP archive tools.",
                texture.id, texture.path
            )));
        }
    }

    Ok(())
}
