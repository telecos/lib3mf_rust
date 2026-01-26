//! Beam lattice extension parsing
//!
//! This module handles parsing of 3MF Beam Lattice extension elements.

use crate::error::{Error, Result};
use crate::model::Beam;
use quick_xml::Reader;

use super::{parse_attributes, validate_attributes};

/// Parse a beam element
pub(super) fn parse_beam<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<Beam> {
    let attrs = parse_attributes(reader, e)?;

    // Validate only allowed attributes are present
    // Per Beam Lattice Extension spec v1.2.0: v1, v2, r1, r2, cap1, cap2, p1, p2, pid
    validate_attributes(
        &attrs,
        &["v1", "v2", "r1", "r2", "cap1", "cap2", "p1", "p2", "pid"],
        "beam",
    )?;

    let v1 = attrs
        .get("v1")
        .ok_or_else(|| Error::InvalidXml("Beam missing v1 attribute".to_string()))?
        .parse::<usize>()?;

    let v2 = attrs
        .get("v2")
        .ok_or_else(|| Error::InvalidXml("Beam missing v2 attribute".to_string()))?
        .parse::<usize>()?;

    let mut beam = Beam::new(v1, v2);

    if let Some(r1) = attrs.get("r1") {
        let r1_val = r1.parse::<f64>()?;
        // Validate radius is finite and positive
        if !r1_val.is_finite() || r1_val <= 0.0 {
            return Err(Error::InvalidXml(format!(
                "Beam r1 must be positive and finite (got {})",
                r1_val
            )));
        }
        beam.r1 = Some(r1_val);
    }

    if let Some(r2) = attrs.get("r2") {
        let r2_val = r2.parse::<f64>()?;
        // Validate radius is finite and positive
        if !r2_val.is_finite() || r2_val <= 0.0 {
            return Err(Error::InvalidXml(format!(
                "Beam r2 must be positive and finite (got {})",
                r2_val
            )));
        }
        beam.r2 = Some(r2_val);

        // If r2 is specified, r1 must also be specified (per 3MF Beam Lattice spec)
        if beam.r1.is_none() {
            return Err(Error::InvalidXml(
                "Beam attribute r2 is specified but r1 is not. When specifying r2, r1 must also be provided.".to_string()
            ));
        }
    }

    // Parse cap1 attribute (optional, defaults to beamset cap mode)
    if let Some(cap1_str) = attrs.get("cap1") {
        beam.cap1 = Some(cap1_str.parse()?);
    }

    // Parse cap2 attribute (optional, defaults to beamset cap mode)
    if let Some(cap2_str) = attrs.get("cap2") {
        beam.cap2 = Some(cap2_str.parse()?);
    }

    // Parse pid attribute (optional) - material/property group ID
    if let Some(pid_str) = attrs.get("pid") {
        beam.property_id = Some(pid_str.parse::<u32>()?);
    }

    // Parse p1 attribute (optional) - property index at v1
    if let Some(p1_str) = attrs.get("p1") {
        beam.p1 = Some(p1_str.parse::<u32>()?);
    }

    // Parse p2 attribute (optional) - property index at v2
    if let Some(p2_str) = attrs.get("p2") {
        beam.p2 = Some(p2_str.parse::<u32>()?);

        // If p2 is specified, p1 must also be specified (per 3MF spec convention)
        if beam.p1.is_none() {
            return Err(Error::InvalidXml(
                "Beam attribute p2 is specified but p1 is not. When specifying p2, p1 must also be provided.".to_string()
            ));
        }
    }

    Ok(beam)
}
