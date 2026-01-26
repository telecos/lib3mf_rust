//! Beam lattice extension writing for 3MF model files
//!
//! This module provides functionality to write Beam Lattice extension elements.

use crate::error::{Error, Result};
use crate::model::*;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use std::io::Write as IoWrite;

/// Write beamset (beam lattice extension)
pub(super) fn write_beamset<W: IoWrite>(writer: &mut Writer<W>, beamset: &BeamSet) -> Result<()> {
    let mut elem = BytesStart::new("b:beamset");

    elem.push_attribute(("radius", beamset.radius.to_string().as_str()));
    elem.push_attribute(("minlength", beamset.min_length.to_string().as_str()));
    elem.push_attribute(("capmode", beamset.cap_mode.to_string().as_str()));

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write beamset element: {}", e)))?;

    for beam in &beamset.beams {
        let mut beam_elem = BytesStart::new("b:beam");
        beam_elem.push_attribute(("v1", beam.v1.to_string().as_str()));
        beam_elem.push_attribute(("v2", beam.v2.to_string().as_str()));

        if let Some(r1) = beam.r1 {
            beam_elem.push_attribute(("r1", r1.to_string().as_str()));
        }

        if let Some(r2) = beam.r2 {
            beam_elem.push_attribute(("r2", r2.to_string().as_str()));
        }

        if let Some(cap1) = beam.cap1 {
            beam_elem.push_attribute(("cap1", cap1.to_string().as_str()));
        }

        if let Some(cap2) = beam.cap2 {
            beam_elem.push_attribute(("cap2", cap2.to_string().as_str()));
        }

        writer
            .write_event(Event::Empty(beam_elem))
            .map_err(|e| Error::xml_write(format!("Failed to write beam: {}", e)))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("b:beamset")))
        .map_err(|e| Error::xml_write(format!("Failed to close beamset element: {}", e)))?;

    Ok(())
}
