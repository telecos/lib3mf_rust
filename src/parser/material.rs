//! Material extension parsing
//!
//! This module handles parsing of 3MF Material extension elements including
//! base materials, color groups, textures, composites, and multi-properties.

use crate::Model;
use crate::error::{Error, Result};
use crate::model::*;
use crate::opc::Package;
use quick_xml::Reader;
use std::io::Read;

use super::{parse_attributes, validate_attributes};

/// Parse color from hex string format (#RRGGBB or #RRGGBBAA)
pub(super) fn parse_color(color_str: &str) -> Option<(u8, u8, u8, u8)> {
    let color_str = color_str.trim_start_matches('#');

    if color_str.len() == 6 {
        // #RRGGBB format (assume full opacity)
        let r = u8::from_str_radix(&color_str[0..2], 16).ok()?;
        let g = u8::from_str_radix(&color_str[2..4], 16).ok()?;
        let b = u8::from_str_radix(&color_str[4..6], 16).ok()?;
        Some((r, g, b, 255))
    } else if color_str.len() == 8 {
        // #RRGGBBAA format
        let r = u8::from_str_radix(&color_str[0..2], 16).ok()?;
        let g = u8::from_str_radix(&color_str[2..4], 16).ok()?;
        let b = u8::from_str_radix(&color_str[4..6], 16).ok()?;
        let a = u8::from_str_radix(&color_str[6..8], 16).ok()?;
        Some((r, g, b, a))
    } else {
        None
    }
}

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
    if let Some(color_str) = attrs.get("displaycolor")
        && let Some(color) = parse_color(color_str)
    {
        material.color = Some(color);
    }

    Ok(material)
}

/// Parse texture2d element
pub(super) fn parse_texture2d<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
    resource_parse_order: usize,
) -> Result<Texture2D> {
    let attrs = parse_attributes(reader, e)?;
    let id = attrs
        .get("id")
        .ok_or_else(|| Error::missing_attribute("texture2d", "id"))?
        .parse::<usize>()?;
    let path = attrs
        .get("path")
        .ok_or_else(|| Error::missing_attribute("texture2d", "path"))?
        .to_string();
    let contenttype = attrs
        .get("contenttype")
        .ok_or_else(|| Error::InvalidXml("texture2d missing contenttype attribute".to_string()))?
        .to_string();

    let mut texture = Texture2D::new(id, path, contenttype);
    texture.parse_order = resource_parse_order;

    // Parse optional attributes with spec defaults
    if let Some(tileu_str) = attrs.get("tilestyleu") {
        texture.tilestyleu = match tileu_str.to_lowercase().as_str() {
            "wrap" => TileStyle::Wrap,
            "mirror" => TileStyle::Mirror,
            "clamp" => TileStyle::Clamp,
            "none" => TileStyle::None,
            _ => TileStyle::Wrap,
        };
    }

    if let Some(tilev_str) = attrs.get("tilestylev") {
        texture.tilestylev = match tilev_str.to_lowercase().as_str() {
            "wrap" => TileStyle::Wrap,
            "mirror" => TileStyle::Mirror,
            "clamp" => TileStyle::Clamp,
            "none" => TileStyle::None,
            _ => TileStyle::Wrap,
        };
    }

    if let Some(filter_str) = attrs.get("filter") {
        texture.filter = match filter_str.to_lowercase().as_str() {
            "auto" => FilterMode::Auto,
            "linear" => FilterMode::Linear,
            "nearest" => FilterMode::Nearest,
            _ => FilterMode::Auto,
        };
    }

    Ok(texture)
}

/// Validate texture file paths exist in the 3MF package
pub(super) fn validate_texture_file_paths<R: Read + std::io::Seek>(
    package: &mut Package<R>,
    model: &Model,
) -> Result<()> {
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

/// Parse basematerials group start and return initialized group
pub(super) fn parse_basematerials_start<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
    resource_parse_order: usize,
) -> Result<BaseMaterialGroup> {
    let attrs = parse_attributes(reader, e)?;
    let id = attrs
        .get("id")
        .ok_or_else(|| Error::missing_attribute("basematerials", "id"))?
        .parse::<usize>()?;
    let mut group = BaseMaterialGroup::new(id);
    group.parse_order = resource_parse_order;
    Ok(group)
}

/// Parse base material element and add to group
pub(super) fn parse_base_element<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<BaseMaterial> {
    let attrs = parse_attributes(reader, e)?;

    // Validate only allowed attributes are present
    // Per 3MF Materials & Properties Extension spec: name, displaycolor
    validate_attributes(&attrs, &["name", "displaycolor"], "base")?;

    let name = attrs.get("name").cloned().unwrap_or_default();

    // Parse displaycolor attribute (format: #RRGGBBAA or #RRGGBB)
    // If displaycolor is missing or invalid, use white as default
    let displaycolor = if let Some(color_str) = attrs.get("displaycolor") {
        parse_color(color_str).unwrap_or((255, 255, 255, 255))
    } else {
        (255, 255, 255, 255)
    };

    Ok(BaseMaterial::new(name, displaycolor))
}

/// Parse colorgroup start and return initialized group
pub(super) fn parse_colorgroup_start<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
    resource_parse_order: usize,
) -> Result<ColorGroup> {
    let attrs = parse_attributes(reader, e)?;
    let id = attrs
        .get("id")
        .ok_or_else(|| Error::missing_attribute("colorgroup", "id"))?
        .parse::<usize>()?;
    let mut group = ColorGroup::new(id);
    group.parse_order = resource_parse_order;
    Ok(group)
}

/// Parse color element
pub(super) fn parse_color_element<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
    colorgroup_id: usize,
) -> Result<(u8, u8, u8, u8)> {
    let attrs = parse_attributes(reader, e)?;
    let color_str = attrs
        .get("color")
        .ok_or_else(|| Error::missing_attribute("color", "color"))?;

    parse_color(color_str).ok_or_else(|| {
        Error::InvalidXml(format!(
            "Invalid color format '{}' in colorgroup {}.\n\
             Colors must be in format #RRGGBB or #RRGGBBAA where each component is a hexadecimal value (0-9, A-F).\n\
             Examples: #FF0000 (red), #00FF0080 (semi-transparent green)",
            color_str, colorgroup_id
        ))
    })
}

/// Parse texture2dgroup start and return initialized group
pub(super) fn parse_texture2dgroup_start<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
    resource_parse_order: usize,
) -> Result<Texture2DGroup> {
    let attrs = parse_attributes(reader, e)?;
    let id = attrs
        .get("id")
        .ok_or_else(|| Error::missing_attribute("texture2dgroup", "id"))?
        .parse::<usize>()?;
    let texid = attrs
        .get("texid")
        .ok_or_else(|| Error::missing_attribute("texture2dgroup", "texid"))?
        .parse::<usize>()?;
    let mut group = Texture2DGroup::new(id, texid);
    group.parse_order = resource_parse_order;
    Ok(group)
}

/// Parse tex2coord element
pub(super) fn parse_tex2coord<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<Tex2Coord> {
    let attrs = parse_attributes(reader, e)?;
    let u = attrs
        .get("u")
        .ok_or_else(|| Error::missing_attribute("tex2coord", "u"))?
        .parse::<f32>()?;
    let v = attrs
        .get("v")
        .ok_or_else(|| Error::missing_attribute("tex2coord", "v"))?
        .parse::<f32>()?;
    Ok(Tex2Coord::new(u, v))
}

/// Parse compositematerials start and return initialized group
pub(super) fn parse_compositematerials_start<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
    resource_parse_order: usize,
) -> Result<CompositeMaterials> {
    let attrs = parse_attributes(reader, e)?;
    let id = attrs
        .get("id")
        .ok_or_else(|| Error::InvalidXml("compositematerials missing id attribute".to_string()))?
        .parse::<usize>()?;
    let matid = attrs
        .get("matid")
        .ok_or_else(|| Error::InvalidXml("compositematerials missing matid attribute".to_string()))?
        .parse::<usize>()?;
    let matindices_str = attrs.get("matindices").ok_or_else(|| {
        Error::InvalidXml("compositematerials missing matindices attribute".to_string())
    })?;
    let matindices: Vec<usize> = matindices_str
        .split_whitespace()
        .map(|s| {
            s.parse::<usize>().map_err(|_| {
                Error::InvalidXml(format!(
                    "compositematerials matindices contains invalid value '{}'",
                    s
                ))
            })
        })
        .collect::<Result<Vec<usize>>>()?;

    // Validate we parsed at least one index
    if matindices.is_empty() {
        return Err(Error::InvalidXml(
            "compositematerials matindices must contain at least one valid index".to_string(),
        ));
    }

    let mut group = CompositeMaterials::new(id, matid, matindices);
    group.parse_order = resource_parse_order;
    Ok(group)
}

/// Parse composite element
pub(super) fn parse_composite<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<Composite> {
    let attrs = parse_attributes(reader, e)?;
    let values_str = attrs
        .get("values")
        .ok_or_else(|| Error::InvalidXml("composite missing values attribute".to_string()))?;
    let values: Vec<f32> = values_str
        .split_whitespace()
        .map(|s| {
            s.parse::<f32>().map_err(|_| {
                Error::InvalidXml(format!("composite values contains invalid number '{}'", s))
            })
        })
        .collect::<Result<Vec<f32>>>()?;

    // Validate we parsed at least one value
    if values.is_empty() {
        return Err(Error::InvalidXml(
            "composite values must contain at least one valid number".to_string(),
        ));
    }

    Ok(Composite::new(values))
}

/// Parse multiproperties start and return initialized group
pub(super) fn parse_multiproperties_start<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
    resource_parse_order: usize,
) -> Result<MultiProperties> {
    let attrs = parse_attributes(reader, e)?;
    let id = attrs
        .get("id")
        .ok_or_else(|| Error::InvalidXml("multiproperties missing id attribute".to_string()))?
        .parse::<usize>()?;
    let pids_str = attrs
        .get("pids")
        .ok_or_else(|| Error::InvalidXml("multiproperties missing pids attribute".to_string()))?;
    let pids: Vec<usize> = pids_str
        .split_whitespace()
        .map(|s| {
            s.parse::<usize>().map_err(|_| {
                Error::InvalidXml(format!(
                    "multiproperties pids contains invalid value '{}'",
                    s
                ))
            })
        })
        .collect::<Result<Vec<usize>>>()?;

    // Validate we parsed at least one property ID
    if pids.is_empty() {
        return Err(Error::InvalidXml(
            "multiproperties pids must contain at least one valid ID".to_string(),
        ));
    }

    let mut multi = MultiProperties::new(id, pids);
    multi.parse_order = resource_parse_order;

    // Parse optional blendmethods
    if let Some(blend_str) = attrs.get("blendmethods") {
        multi.blendmethods = blend_str
            .split_whitespace()
            .filter_map(|s| match s.to_lowercase().as_str() {
                "mix" => Some(BlendMethod::Mix),
                "multiply" => Some(BlendMethod::Multiply),
                _ => None,
            })
            .collect();
    }

    Ok(multi)
}

/// Parse multi element
pub(super) fn parse_multi<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<Multi> {
    let attrs = parse_attributes(reader, e)?;
    let pindices_str = attrs
        .get("pindices")
        .ok_or_else(|| Error::InvalidXml("multi missing pindices attribute".to_string()))?;
    let pindices: Vec<usize> = if pindices_str.trim().is_empty() {
        Vec::new()
    } else {
        pindices_str
            .split_whitespace()
            .map(|s| {
                s.parse::<usize>().map_err(|_| {
                    Error::InvalidXml(format!("multi pindices contains invalid value '{}'", s))
                })
            })
            .collect::<Result<Vec<usize>>>()?
    };

    Ok(Multi::new(pindices))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color() {
        // Test #RRGGBB format
        assert_eq!(parse_color("#FF0000"), Some((255, 0, 0, 255)));
        assert_eq!(parse_color("#00FF00"), Some((0, 255, 0, 255)));
        assert_eq!(parse_color("#0000FF"), Some((0, 0, 255, 255)));

        // Test #RRGGBBAA format
        assert_eq!(parse_color("#FF000080"), Some((255, 0, 0, 128)));
        assert_eq!(parse_color("#00FF00FF"), Some((0, 255, 0, 255)));

        // Test invalid formats
        assert_eq!(parse_color("#FF"), None);
        assert_eq!(parse_color("FF0000"), Some((255, 0, 0, 255)));
    }
}
