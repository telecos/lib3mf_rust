//! Beam lattice extension writing for 3MF model files
//!
//! This module provides functionality to write Beam Lattice extension elements.

use crate::error::{Error, Result};
use crate::model::*;
use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;

/// Write beamset (beam lattice extension)
pub(super) fn write_beamset<W: IoWrite>(writer: &mut Writer<W>, beamset: &BeamSet) -> Result<()> {
    let mut fmt_buf = String::with_capacity(32);
    let mut elem = BytesStart::new("b:beamset");

    write!(fmt_buf, "{}", beamset.radius).unwrap();
    elem.push_attribute(("radius", fmt_buf.as_str()));

    fmt_buf.clear();
    write!(fmt_buf, "{}", beamset.min_length).unwrap();
    elem.push_attribute(("minlength", fmt_buf.as_str()));

    elem.push_attribute(("capmode", beamset.cap_mode.to_string().as_str()));

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write beamset element: {}", e)))?;

    for beam in &beamset.beams {
        let mut beam_elem = BytesStart::new("b:beam");

        fmt_buf.clear();
        write!(fmt_buf, "{}", beam.v1).unwrap();
        beam_elem.push_attribute(("v1", fmt_buf.as_str()));

        fmt_buf.clear();
        write!(fmt_buf, "{}", beam.v2).unwrap();
        beam_elem.push_attribute(("v2", fmt_buf.as_str()));

        if let Some(r1) = beam.r1 {
            fmt_buf.clear();
            write!(fmt_buf, "{}", r1).unwrap();
            beam_elem.push_attribute(("r1", fmt_buf.as_str()));
        }

        if let Some(r2) = beam.r2 {
            fmt_buf.clear();
            write!(fmt_buf, "{}", r2).unwrap();
            beam_elem.push_attribute(("r2", fmt_buf.as_str()));
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
