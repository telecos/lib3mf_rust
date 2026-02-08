//! Volumetric extension parsing for 3MF model files
//!
//! This module provides functions to parse Volumetric extension elements
//! such as `<v:volumetricdata>`, `<v:boundary>`, `<v:voxels>`, `<v:voxel>`,
//! `<v:volumetricpropertygroup>`, and `<v:property>`.

use crate::error::{Error, Result};
use crate::model::*;
use quick_xml::Reader;
use quick_xml::events::BytesStart;

use super::parse_attributes;

/// Parse the start of a `<v:volumetricdata>` element
pub(super) fn parse_volumetricdata_start<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &BytesStart,
) -> Result<VolumetricData> {
    let attrs = parse_attributes(reader, e)?;

    let id = attrs
        .get("id")
        .ok_or_else(|| Error::InvalidXml("volumetricdata missing required 'id' attribute".to_string()))?
        .parse::<usize>()
        .map_err(|_| Error::InvalidXml("volumetricdata 'id' must be a valid non-negative integer".to_string()))?;

    Ok(VolumetricData::new(id))
}

/// Parse a `<v:boundary>` element and return a `VolumetricBoundary`
pub(super) fn parse_boundary<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &BytesStart,
) -> Result<VolumetricBoundary> {
    let attrs = parse_attributes(reader, e)?;

    let min = parse_xyz_triple(
        attrs.get("min"),
        "boundary missing required 'min' attribute",
        "boundary 'min'",
    )?;

    let max = parse_xyz_triple(
        attrs.get("max"),
        "boundary missing required 'max' attribute",
        "boundary 'max'",
    )?;

    Ok(VolumetricBoundary::new(min, max))
}

/// Parse the start of a `<v:voxels>` element and return a `VoxelGrid`
pub(super) fn parse_voxels_start<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &BytesStart,
) -> Result<VoxelGrid> {
    let attrs = parse_attributes(reader, e)?;

    let dimensions = parse_usize_triple(
        attrs.get("dimensions"),
        "voxels missing required 'dimensions' attribute",
        "voxels 'dimensions'",
    )?;

    let mut grid = VoxelGrid::new(dimensions);

    if let Some(spacing_str) = attrs.get("spacing") {
        grid.spacing = Some(parse_f64_triple(spacing_str, "voxels 'spacing'")?);
    }

    if let Some(origin_str) = attrs.get("origin") {
        grid.origin = Some(parse_f64_triple(origin_str, "voxels 'origin'")?);
    }

    Ok(grid)
}

/// Parse a `<v:voxel>` element and return a `Voxel`
pub(super) fn parse_voxel<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &BytesStart,
) -> Result<Voxel> {
    let attrs = parse_attributes(reader, e)?;

    let x = attrs
        .get("x")
        .ok_or_else(|| Error::InvalidXml("voxel missing required 'x' attribute".to_string()))?
        .parse::<usize>()
        .map_err(|_| Error::InvalidXml("voxel 'x' must be a valid non-negative integer".to_string()))?;

    let y = attrs
        .get("y")
        .ok_or_else(|| Error::InvalidXml("voxel missing required 'y' attribute".to_string()))?
        .parse::<usize>()
        .map_err(|_| Error::InvalidXml("voxel 'y' must be a valid non-negative integer".to_string()))?;

    let z = attrs
        .get("z")
        .ok_or_else(|| Error::InvalidXml("voxel missing required 'z' attribute".to_string()))?
        .parse::<usize>()
        .map_err(|_| Error::InvalidXml("voxel 'z' must be a valid non-negative integer".to_string()))?;

    let mut voxel = Voxel::new((x, y, z));

    if let Some(prop_str) = attrs.get("property") {
        voxel.property_id = Some(prop_str.parse::<usize>().map_err(|_| {
            Error::InvalidXml("voxel 'property' must be a valid non-negative integer".to_string())
        })?);
    }

    if let Some(color_str) = attrs.get("color") {
        voxel.color_id = Some(color_str.parse::<usize>().map_err(|_| {
            Error::InvalidXml("voxel 'color' must be a valid non-negative integer".to_string())
        })?);
    }

    Ok(voxel)
}

/// Parse the start of a `<v:volumetricpropertygroup>` element
pub(super) fn parse_volumetricpropertygroup_start<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &BytesStart,
) -> Result<VolumetricPropertyGroup> {
    let attrs = parse_attributes(reader, e)?;

    let id = attrs
        .get("id")
        .ok_or_else(|| {
            Error::InvalidXml(
                "volumetricpropertygroup missing required 'id' attribute".to_string(),
            )
        })?
        .parse::<usize>()
        .map_err(|_| {
            Error::InvalidXml(
                "volumetricpropertygroup 'id' must be a valid non-negative integer".to_string(),
            )
        })?;

    Ok(VolumetricPropertyGroup::new(id))
}

/// Parse a `<v:property>` element and return a `VolumetricProperty`
pub(super) fn parse_volumetric_property<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &BytesStart,
) -> Result<VolumetricProperty> {
    let attrs = parse_attributes(reader, e)?;

    let index = attrs
        .get("index")
        .ok_or_else(|| Error::InvalidXml("property missing required 'index' attribute".to_string()))?
        .parse::<usize>()
        .map_err(|_| {
            Error::InvalidXml("property 'index' must be a valid non-negative integer".to_string())
        })?;

    let value = attrs
        .get("value")
        .ok_or_else(|| Error::InvalidXml("property missing required 'value' attribute".to_string()))?
        .clone();

    Ok(VolumetricProperty::new(index, value))
}

/// Parse the start of a `<v:implicit>` element and return an `ImplicitVolume`
pub(super) fn parse_implicit_start<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &BytesStart,
) -> Result<ImplicitVolume> {
    let attrs = parse_attributes(reader, e)?;

    let implicit_type = attrs
        .get("type")
        .ok_or_else(|| Error::InvalidXml("implicit missing required 'type' attribute".to_string()))?
        .clone();

    let mut implicit = ImplicitVolume::new(implicit_type);

    // Collect any additional attributes as parameters
    for (key, value) in &attrs {
        if key != "type" {
            implicit.parameters.push((key.clone(), value.clone()));
        }
    }

    Ok(implicit)
}

// ---------------------------------------------------------------------------
// Helper functions for parsing space-separated coordinate triples
// ---------------------------------------------------------------------------

/// Parse a space-separated triple of `f64` values (e.g., `"0 0 0"` → `(0.0, 0.0, 0.0)`)
fn parse_f64_triple(s: &str, context: &str) -> Result<(f64, f64, f64)> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 3 {
        return Err(Error::InvalidXml(format!(
            "{} must have exactly 3 space-separated values, got {}",
            context,
            parts.len()
        )));
    }

    let a = parts[0]
        .parse::<f64>()
        .map_err(|_| Error::InvalidXml(format!("{}: invalid float value '{}'", context, parts[0])))?;
    let b = parts[1]
        .parse::<f64>()
        .map_err(|_| Error::InvalidXml(format!("{}: invalid float value '{}'", context, parts[1])))?;
    let c = parts[2]
        .parse::<f64>()
        .map_err(|_| Error::InvalidXml(format!("{}: invalid float value '{}'", context, parts[2])))?;

    Ok((a, b, c))
}

/// Parse an optional attribute as a space-separated triple of `f64` values
fn parse_xyz_triple(
    attr: Option<&String>,
    missing_msg: &str,
    context: &str,
) -> Result<(f64, f64, f64)> {
    let s = attr.ok_or_else(|| Error::InvalidXml(missing_msg.to_string()))?;
    parse_f64_triple(s, context)
}

/// Parse a space-separated triple of `usize` values (e.g., `"10 10 10"`)
fn parse_usize_triple(
    attr: Option<&String>,
    missing_msg: &str,
    context: &str,
) -> Result<(usize, usize, usize)> {
    let s = attr.ok_or_else(|| Error::InvalidXml(missing_msg.to_string()))?;
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 3 {
        return Err(Error::InvalidXml(format!(
            "{} must have exactly 3 space-separated values, got {}",
            context,
            parts.len()
        )));
    }

    let a = parts[0].parse::<usize>().map_err(|_| {
        Error::InvalidXml(format!("{}: invalid integer value '{}'", context, parts[0]))
    })?;
    let b = parts[1].parse::<usize>().map_err(|_| {
        Error::InvalidXml(format!("{}: invalid integer value '{}'", context, parts[1]))
    })?;
    let c = parts[2].parse::<usize>().map_err(|_| {
        Error::InvalidXml(format!("{}: invalid integer value '{}'", context, parts[2]))
    })?;

    Ok((a, b, c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_f64_triple() {
        assert_eq!(parse_f64_triple("0 0 0", "test").unwrap(), (0.0, 0.0, 0.0));
        assert_eq!(
            parse_f64_triple("1.5 2.5 3.5", "test").unwrap(),
            (1.5, 2.5, 3.5)
        );
        assert!(parse_f64_triple("1 2", "test").is_err());
        assert!(parse_f64_triple("abc", "test").is_err());
    }

    #[test]
    fn test_parse_usize_triple() {
        assert_eq!(
            parse_usize_triple(Some(&"10 20 30".to_string()), "missing", "test").unwrap(),
            (10, 20, 30)
        );
        assert!(parse_usize_triple(None, "missing", "test").is_err());
        assert!(parse_usize_triple(Some(&"1 2".to_string()), "missing", "test").is_err());
    }
}
