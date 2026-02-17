//! Displacement extension writing for 3MF model files
//!
//! This module provides functionality to write Displacement extension elements
//! like displacement2d, normvector groups and disp2d groups.

use crate::error::{Error, Result};
use crate::model::*;
use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use std::io::Write as IoWrite;

/// Write displacement2d resource (displacement extension)
pub(super) fn write_displacement2d<W: IoWrite>(
    writer: &mut Writer<W>,
    disp_map: &Displacement2D,
) -> Result<()> {
    let mut elem = BytesStart::new("d:displacement2d");
    elem.push_attribute(("id", disp_map.id.to_string().as_str()));
    elem.push_attribute(("path", disp_map.path.as_str()));

    // Write optional attributes
    let channel_str = match disp_map.channel {
        Channel::R => "r",
        Channel::G => "g",
        Channel::B => "b",
        Channel::A => "a",
    };
    elem.push_attribute(("channel", channel_str));

    let tilestyleu_str = match disp_map.tilestyleu {
        TileStyle::Wrap => "wrap",
        TileStyle::Mirror => "mirror",
        TileStyle::Clamp => "clamp",
        TileStyle::None => "none",
    };
    elem.push_attribute(("tilestyleu", tilestyleu_str));

    let tilestylev_str = match disp_map.tilestylev {
        TileStyle::Wrap => "wrap",
        TileStyle::Mirror => "mirror",
        TileStyle::Clamp => "clamp",
        TileStyle::None => "none",
    };
    elem.push_attribute(("tilestylev", tilestylev_str));

    let filter_str = match disp_map.filter {
        FilterMode::Auto => "auto",
        FilterMode::Linear => "linear",
        FilterMode::Nearest => "nearest",
    };
    elem.push_attribute(("filter", filter_str));

    writer
        .write_event(Event::Empty(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write displacement2d element: {}", e)))?;

    Ok(())
}

/// Write normvector group (displacement extension)
pub(super) fn write_normvector_group<W: IoWrite>(
    writer: &mut Writer<W>,
    group: &NormVectorGroup,
) -> Result<()> {
    let mut elem = BytesStart::new("d:normvectorgroup");
    elem.push_attribute(("id", group.id.to_string().as_str()));

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write normvectorgroup element: {}", e)))?;

    for vector in &group.vectors {
        let mut vec_elem = BytesStart::new("d:normvector");
        vec_elem.push_attribute(("x", vector.x.to_string().as_str()));
        vec_elem.push_attribute(("y", vector.y.to_string().as_str()));
        vec_elem.push_attribute(("z", vector.z.to_string().as_str()));

        writer
            .write_event(Event::Empty(vec_elem))
            .map_err(|e| Error::xml_write(format!("Failed to write normvector: {}", e)))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("d:normvectorgroup")))
        .map_err(|e| Error::xml_write(format!("Failed to close normvectorgroup element: {}", e)))?;

    Ok(())
}

/// Write disp2dgroup (displacement extension)
pub(super) fn write_disp2d_group<W: IoWrite>(
    writer: &mut Writer<W>,
    group: &Disp2DGroup,
) -> Result<()> {
    let mut elem = BytesStart::new("d:disp2dgroup");
    elem.push_attribute(("id", group.id.to_string().as_str()));
    elem.push_attribute(("dispid", group.dispid.to_string().as_str()));
    elem.push_attribute(("nid", group.nid.to_string().as_str()));
    elem.push_attribute(("height", group.height.to_string().as_str()));
    elem.push_attribute(("offset", group.offset.to_string().as_str()));

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write disp2dgroup element: {}", e)))?;

    for coord in &group.coords {
        let mut coord_elem = BytesStart::new("d:disp2d");
        coord_elem.push_attribute(("u", coord.u.to_string().as_str()));
        coord_elem.push_attribute(("v", coord.v.to_string().as_str()));

        writer
            .write_event(Event::Empty(coord_elem))
            .map_err(|e| Error::xml_write(format!("Failed to write disp2d: {}", e)))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("d:disp2dgroup")))
        .map_err(|e| Error::xml_write(format!("Failed to close disp2dgroup element: {}", e)))?;

    Ok(())
}
