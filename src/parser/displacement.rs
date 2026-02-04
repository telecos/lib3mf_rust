//! Displacement extension parsing
//!
//! This module handles parsing of 3MF Displacement extension elements.

use crate::error::{Error, Result};
use crate::model::*;
use quick_xml::Reader;
use std::collections::HashSet;

use super::{parse_attributes, validate_attributes};

/// Validate that an element uses the displacement namespace prefix
///
/// Per 3MF Displacement Extension spec 4.1, all elements under displacementmesh
/// MUST use the displacement namespace prefix (e.g., d:vertex, d:triangle).
///
/// # Arguments
/// * `name_str` - The full element name with potential namespace prefix
/// * `element_type` - Human-readable element type for error messages
///
/// # Returns
/// Ok(()) if element has displacement prefix, Err otherwise
pub(super) fn validate_displacement_namespace_prefix(
    name_str: &str,
    element_type: &str,
) -> Result<()> {
    let has_displacement_prefix =
        name_str.starts_with("d:") || name_str.starts_with("displacement:");
    if !has_displacement_prefix {
        return Err(Error::InvalidXml(format!(
            "Element <{}> under displacementmesh must use the displacement namespace prefix (e.g., <d:{}>). \
             Per 3MF Displacement Extension spec 4.1, all elements under <displacementmesh> MUST specify \
             the displacement namespace prefix.",
            name_str, element_type
        )));
    }
    Ok(())
}

/// Parse a displacement triangle element
pub fn parse_displacement_triangle<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<DisplacementTriangle> {
    let attrs = parse_attributes(reader, e)?;

    // Validate only allowed attributes are present
    // Per Displacement Extension spec: v1, v2, v3, pid, pindex, p1, p2, p3, did, d1, d2, d3
    validate_attributes(
        &attrs,
        &[
            "v1", "v2", "v3", "pid", "pindex", "p1", "p2", "p3", "did", "d1", "d2", "d3",
        ],
        "triangle",
    )?;

    let v1 = attrs
        .get("v1")
        .ok_or_else(|| Error::InvalidXml("Displacement triangle missing v1 attribute".to_string()))?
        .parse::<usize>()?;

    let v2 = attrs
        .get("v2")
        .ok_or_else(|| Error::InvalidXml("Displacement triangle missing v2 attribute".to_string()))?
        .parse::<usize>()?;

    let v3 = attrs
        .get("v3")
        .ok_or_else(|| Error::InvalidXml("Displacement triangle missing v3 attribute".to_string()))?
        .parse::<usize>()?;

    let mut triangle = DisplacementTriangle::new(v1, v2, v3);

    if let Some(pid) = attrs.get("pid") {
        triangle.pid = Some(pid.parse::<usize>()?);
    }

    if let Some(pindex) = attrs.get("pindex") {
        triangle.pindex = Some(pindex.parse::<usize>()?);
    }

    if let Some(p1) = attrs.get("p1") {
        triangle.p1 = Some(p1.parse::<usize>()?);
    }

    if let Some(p2) = attrs.get("p2") {
        triangle.p2 = Some(p2.parse::<usize>()?);
    }

    if let Some(p3) = attrs.get("p3") {
        triangle.p3 = Some(p3.parse::<usize>()?);
    }

    // Parse displacement-specific attributes
    if let Some(did) = attrs.get("did") {
        triangle.did = Some(did.parse::<usize>()?);
    }

    if let Some(d1) = attrs.get("d1") {
        triangle.d1 = Some(d1.parse::<usize>()?);
    }

    if let Some(d2) = attrs.get("d2") {
        triangle.d2 = Some(d2.parse::<usize>()?);
    }

    if let Some(d3) = attrs.get("d3") {
        triangle.d3 = Some(d3.parse::<usize>()?);
    }

    Ok(triangle)
}

/// Parse displacement2d resource element
pub(super) fn parse_displacement2d<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<Displacement2D> {
    let attrs = parse_attributes(reader, e)?;

    // Validate only allowed attributes are present
    // Per Displacement Extension spec 3.1: id, path, channel, tilestyleu, tilestylev, filter
    validate_attributes(
        &attrs,
        &[
            "id",
            "path",
            "channel",
            "tilestyleu",
            "tilestylev",
            "filter",
        ],
        "displacement2d",
    )?;

    let id = attrs
        .get("id")
        .ok_or_else(|| Error::InvalidXml("displacement2d missing id attribute".to_string()))?
        .parse::<usize>()?;
    let path = attrs
        .get("path")
        .ok_or_else(|| Error::InvalidXml("displacement2d missing path attribute".to_string()))?
        .to_string();

    let mut disp = Displacement2D::new(id, path);

    // Parse optional attributes with spec-defined defaults
    // Strict validation: reject invalid enum values per DPX 3316
    if let Some(channel_str) = attrs.get("channel") {
        disp.channel = match channel_str.to_uppercase().as_str() {
            "R" => Channel::R,
            "G" => Channel::G,
            "B" => Channel::B,
            "A" => Channel::A,
            _ => {
                return Err(Error::InvalidXml(format!(
                    "Invalid channel value '{}'. Valid values are: R, G, B, A",
                    channel_str
                )));
            }
        };
    }

    if let Some(tileu_str) = attrs.get("tilestyleu") {
        disp.tilestyleu = match tileu_str.to_lowercase().as_str() {
            "wrap" => TileStyle::Wrap,
            "mirror" => TileStyle::Mirror,
            "clamp" => TileStyle::Clamp,
            "none" => TileStyle::None,
            _ => {
                return Err(Error::InvalidXml(format!(
                    "Invalid tilestyleu value '{}'. Valid values are: wrap, mirror, clamp, none",
                    tileu_str
                )));
            }
        };
    }

    if let Some(tilev_str) = attrs.get("tilestylev") {
        disp.tilestylev = match tilev_str.to_lowercase().as_str() {
            "wrap" => TileStyle::Wrap,
            "mirror" => TileStyle::Mirror,
            "clamp" => TileStyle::Clamp,
            "none" => TileStyle::None,
            _ => {
                return Err(Error::InvalidXml(format!(
                    "Invalid tilestylev value '{}'. Valid values are: wrap, mirror, clamp, none",
                    tilev_str
                )));
            }
        };
    }

    if let Some(filter_str) = attrs.get("filter") {
        disp.filter = match filter_str.to_lowercase().as_str() {
            "auto" => FilterMode::Auto,
            "linear" => FilterMode::Linear,
            "nearest" => FilterMode::Nearest,
            _ => {
                return Err(Error::InvalidXml(format!(
                    "Invalid filter value '{}'. Valid values are: auto, linear, nearest",
                    filter_str
                )));
            }
        };
    }

    Ok(disp)
}

/// Parse normvectorgroup start and return initialized group
pub(super) fn parse_normvectorgroup_start<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<NormVectorGroup> {
    let attrs = parse_attributes(reader, e)?;

    // Validate only allowed attributes are present
    // Per Displacement Extension spec 3.2: id
    validate_attributes(&attrs, &["id"], "normvectorgroup")?;

    let id = attrs
        .get("id")
        .ok_or_else(|| Error::InvalidXml("normvectorgroup missing id attribute".to_string()))?
        .parse::<usize>()?;
    Ok(NormVectorGroup::new(id))
}

/// Parse normvector element
pub(super) fn parse_normvector<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<NormVector> {
    let attrs = parse_attributes(reader, e)?;

    // Validate only allowed attributes are present
    // Per Displacement Extension spec 3.2.1: x, y, z
    validate_attributes(&attrs, &["x", "y", "z"], "normvector")?;

    let x = attrs
        .get("x")
        .ok_or_else(|| Error::InvalidXml("normvector missing x attribute".to_string()))?
        .parse::<f64>()?;
    let y = attrs
        .get("y")
        .ok_or_else(|| Error::InvalidXml("normvector missing y attribute".to_string()))?
        .parse::<f64>()?;
    let z = attrs
        .get("z")
        .ok_or_else(|| Error::InvalidXml("normvector missing z attribute".to_string()))?
        .parse::<f64>()?;

    // Validate values are finite
    if !x.is_finite() || !y.is_finite() || !z.is_finite() {
        return Err(Error::InvalidXml(format!(
            "NormVector has non-finite values: x={}, y={}, z={}",
            x, y, z
        )));
    }

    Ok(NormVector::new(x, y, z))
}

/// Parse disp2dgroup start and return initialized group
/// Validates forward references to displacement2d and normvectorgroup resources
pub(super) fn parse_disp2dgroup_start<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
    declared_displacement2d_ids: &HashSet<usize>,
    declared_normvectorgroup_ids: &HashSet<usize>,
) -> Result<Disp2DGroup> {
    let attrs = parse_attributes(reader, e)?;

    // Validate only allowed attributes are present
    // Per Displacement Extension spec 3.3: id, dispid, nid, height, offset
    validate_attributes(
        &attrs,
        &["id", "dispid", "nid", "height", "offset"],
        "disp2dgroup",
    )?;

    let id = attrs
        .get("id")
        .ok_or_else(|| Error::InvalidXml("disp2dgroup missing id attribute".to_string()))?
        .parse::<usize>()?;
    let dispid = attrs
        .get("dispid")
        .ok_or_else(|| Error::InvalidXml("disp2dgroup missing dispid attribute".to_string()))?
        .parse::<usize>()?;

    // Per DPX spec 3.3: Validate dispid references declared Displacement2D resource
    if !declared_displacement2d_ids.contains(&dispid) {
        return Err(Error::InvalidXml(format!(
            "Disp2DGroup references Displacement2D with ID {} which has not been declared yet. \
             Resources must be declared before they are referenced.",
            dispid
        )));
    }

    let nid = attrs
        .get("nid")
        .ok_or_else(|| Error::InvalidXml("disp2dgroup missing nid attribute".to_string()))?
        .parse::<usize>()?;

    // Per DPX spec 3.3: Validate nid references declared NormVectorGroup resource
    if !declared_normvectorgroup_ids.contains(&nid) {
        return Err(Error::InvalidXml(format!(
            "Disp2DGroup references NormVectorGroup with ID {} which has not been declared yet. \
             Resources must be declared before they are referenced.",
            nid
        )));
    }
    let height = attrs
        .get("height")
        .ok_or_else(|| Error::InvalidXml("disp2dgroup missing height attribute".to_string()))?
        .parse::<f64>()?;

    // Validate height is finite
    if !height.is_finite() {
        return Err(Error::InvalidXml(format!(
            "Disp2DGroup height must be finite, got: {}",
            height
        )));
    }

    let mut disp2dgroup = Disp2DGroup::new(id, dispid, nid, height);

    // Parse optional offset
    if let Some(offset_str) = attrs.get("offset") {
        let offset = offset_str.parse::<f64>()?;
        if !offset.is_finite() {
            return Err(Error::InvalidXml(format!(
                "Disp2DGroup offset must be finite, got: {}",
                offset
            )));
        }
        disp2dgroup.offset = offset;
    }

    Ok(disp2dgroup)
}

/// Parse disp2dcoord element
pub(super) fn parse_disp2dcoord<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<Disp2DCoords> {
    let attrs = parse_attributes(reader, e)?;

    // Validate only allowed attributes are present
    // Per Displacement Extension spec 3.3.1: u, v, n, f
    validate_attributes(&attrs, &["u", "v", "n", "f"], "disp2dcoord")?;

    let u = attrs
        .get("u")
        .ok_or_else(|| Error::InvalidXml("disp2dcoord missing u attribute".to_string()))?
        .parse::<f64>()?;
    let v = attrs
        .get("v")
        .ok_or_else(|| Error::InvalidXml("disp2dcoord missing v attribute".to_string()))?
        .parse::<f64>()?;

    // Validate u and v are finite
    if !u.is_finite() || !v.is_finite() {
        return Err(Error::InvalidXml(format!(
            "Disp2DCoord u and v must be finite, got: u={}, v={}",
            u, v
        )));
    }

    // Note: n is required per spec 3.3.1
    let n = attrs
        .get("n")
        .ok_or_else(|| Error::InvalidXml("disp2dcoord missing n attribute".to_string()))?
        .parse::<usize>()?;

    let mut coords = Disp2DCoords::new(u, v, n);

    // Parse optional f (displacement factor)
    if let Some(f_str) = attrs.get("f") {
        let f = f_str.parse::<f64>()?;
        if !f.is_finite() {
            return Err(Error::InvalidXml(format!(
                "Disp2DCoord displacement factor must be finite, got: {}",
                f
            )));
        }
        coords.f = f;
    }

    Ok(coords)
}
