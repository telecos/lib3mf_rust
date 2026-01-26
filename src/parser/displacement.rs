//! Displacement extension parsing
//!
//! This module handles parsing of 3MF Displacement extension elements.

use crate::error::{Error, Result};
use crate::model::DisplacementTriangle;
use quick_xml::Reader;

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
