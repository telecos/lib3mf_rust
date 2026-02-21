//! Material extension writing for 3MF model files
//!
//! This module provides functionality to write Material extension elements like
//! base materials, textures, color groups, composites, and multi-properties.

use crate::error::{Error, Result};
use crate::model::*;
use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;

/// Write a base material group
pub(super) fn write_base_material_group<W: IoWrite>(
    writer: &mut Writer<W>,
    group: &BaseMaterialGroup,
) -> Result<()> {
    let mut fmt_buf = String::with_capacity(32);
    let mut elem = BytesStart::new("m:basematerials");

    write!(fmt_buf, "{}", group.id).unwrap();
    elem.push_attribute(("id", fmt_buf.as_str()));

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write basematerials element: {}", e)))?;

    for material in &group.materials {
        let mut mat_elem = BytesStart::new("m:base");
        mat_elem.push_attribute(("name", material.name.as_str()));

        fmt_buf.clear();
        write!(
            fmt_buf,
            "#{:02X}{:02X}{:02X}{:02X}",
            material.displaycolor.0,
            material.displaycolor.1,
            material.displaycolor.2,
            material.displaycolor.3
        )
        .unwrap();
        mat_elem.push_attribute(("displaycolor", fmt_buf.as_str()));

        writer
            .write_event(Event::Empty(mat_elem))
            .map_err(|e| Error::xml_write(format!("Failed to write base material: {}", e)))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("m:basematerials")))
        .map_err(|e| Error::xml_write(format!("Failed to close basematerials element: {}", e)))?;

    Ok(())
}

/// Write a texture2d resource
pub(super) fn write_texture2d<W: IoWrite>(
    writer: &mut Writer<W>,
    texture: &Texture2D,
) -> Result<()> {
    let mut fmt_buf = String::with_capacity(32);
    let mut elem = BytesStart::new("m:texture2d");

    write!(fmt_buf, "{}", texture.id).unwrap();
    elem.push_attribute(("id", fmt_buf.as_str()));
    elem.push_attribute(("path", texture.path.as_str()));
    elem.push_attribute(("contenttype", texture.contenttype.as_str()));

    let tilestyleu = match texture.tilestyleu {
        TileStyle::Wrap => "wrap",
        TileStyle::Mirror => "mirror",
        TileStyle::Clamp => "clamp",
        TileStyle::None => "none",
    };
    elem.push_attribute(("tilestyleu", tilestyleu));

    let tilestylev = match texture.tilestylev {
        TileStyle::Wrap => "wrap",
        TileStyle::Mirror => "mirror",
        TileStyle::Clamp => "clamp",
        TileStyle::None => "none",
    };
    elem.push_attribute(("tilestylev", tilestylev));

    let filter = match texture.filter {
        FilterMode::Auto => "auto",
        FilterMode::Linear => "linear",
        FilterMode::Nearest => "nearest",
    };
    elem.push_attribute(("filter", filter));

    writer
        .write_event(Event::Empty(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write texture2d: {}", e)))?;

    Ok(())
}

/// Write a texture2dgroup resource
pub(super) fn write_texture2d_group<W: IoWrite>(
    writer: &mut Writer<W>,
    group: &Texture2DGroup,
) -> Result<()> {
    let mut fmt_buf = String::with_capacity(32);
    let mut elem = BytesStart::new("m:texture2dgroup");

    write!(fmt_buf, "{}", group.id).unwrap();
    elem.push_attribute(("id", fmt_buf.as_str()));

    fmt_buf.clear();
    write!(fmt_buf, "{}", group.texid).unwrap();
    elem.push_attribute(("texid", fmt_buf.as_str()));

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write texture2dgroup element: {}", e)))?;

    for coord in &group.tex2coords {
        let mut coord_elem = BytesStart::new("m:tex2coord");

        fmt_buf.clear();
        write!(fmt_buf, "{}", coord.u).unwrap();
        coord_elem.push_attribute(("u", fmt_buf.as_str()));

        fmt_buf.clear();
        write!(fmt_buf, "{}", coord.v).unwrap();
        coord_elem.push_attribute(("v", fmt_buf.as_str()));

        writer
            .write_event(Event::Empty(coord_elem))
            .map_err(|e| Error::xml_write(format!("Failed to write tex2coord: {}", e)))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("m:texture2dgroup")))
        .map_err(|e| Error::xml_write(format!("Failed to close texture2dgroup element: {}", e)))?;

    Ok(())
}

/// Write a color group resource
pub(super) fn write_color_group<W: IoWrite>(
    writer: &mut Writer<W>,
    group: &ColorGroup,
) -> Result<()> {
    let mut fmt_buf = String::with_capacity(32);
    let mut elem = BytesStart::new("m:colorgroup");

    write!(fmt_buf, "{}", group.id).unwrap();
    elem.push_attribute(("id", fmt_buf.as_str()));

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write colorgroup element: {}", e)))?;

    for color in &group.colors {
        let mut color_elem = BytesStart::new("m:color");
        fmt_buf.clear();
        write!(
            fmt_buf,
            "#{:02X}{:02X}{:02X}{:02X}",
            color.0, color.1, color.2, color.3
        )
        .unwrap();
        color_elem.push_attribute(("color", fmt_buf.as_str()));

        writer
            .write_event(Event::Empty(color_elem))
            .map_err(|e| Error::xml_write(format!("Failed to write color: {}", e)))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("m:colorgroup")))
        .map_err(|e| Error::xml_write(format!("Failed to close colorgroup element: {}", e)))?;

    Ok(())
}

/// Write composite materials
pub(super) fn write_composite_materials<W: IoWrite>(
    writer: &mut Writer<W>,
    composite: &CompositeMaterials,
) -> Result<()> {
    let mut fmt_buf = String::with_capacity(64);
    let mut elem = BytesStart::new("m:compositematerials");

    write!(fmt_buf, "{}", composite.id).unwrap();
    elem.push_attribute(("id", fmt_buf.as_str()));

    fmt_buf.clear();
    write!(fmt_buf, "{}", composite.matid).unwrap();
    elem.push_attribute(("matid", fmt_buf.as_str()));

    // Convert matindices to space-separated string
    fmt_buf.clear();
    for (i, idx) in composite.matindices.iter().enumerate() {
        if i > 0 {
            fmt_buf.push(' ');
        }
        write!(fmt_buf, "{}", idx).unwrap();
    }
    elem.push_attribute(("matindices", fmt_buf.as_str()));

    writer.write_event(Event::Start(elem)).map_err(|e| {
        Error::xml_write(format!("Failed to write compositematerials element: {}", e))
    })?;

    for comp in &composite.composites {
        write_composite(writer, comp)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("m:compositematerials")))
        .map_err(|e| {
            Error::xml_write(format!("Failed to close compositematerials element: {}", e))
        })?;

    Ok(())
}

/// Write a composite entry
pub(super) fn write_composite<W: IoWrite>(
    writer: &mut Writer<W>,
    composite: &Composite,
) -> Result<()> {
    let mut fmt_buf = String::with_capacity(64);
    let mut elem = BytesStart::new("m:composite");

    // Convert values to space-separated string
    for (i, v) in composite.values.iter().enumerate() {
        if i > 0 {
            fmt_buf.push(' ');
        }
        write!(fmt_buf, "{}", v).unwrap();
    }
    elem.push_attribute(("values", fmt_buf.as_str()));

    writer
        .write_event(Event::Empty(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write composite: {}", e)))?;

    Ok(())
}

/// Write multi properties resource
pub(super) fn write_multi_properties<W: IoWrite>(
    writer: &mut Writer<W>,
    multi: &MultiProperties,
) -> Result<()> {
    let mut fmt_buf = String::with_capacity(64);
    let mut elem = BytesStart::new("m:multiproperties");

    write!(fmt_buf, "{}", multi.id).unwrap();
    elem.push_attribute(("id", fmt_buf.as_str()));

    // Convert pids to space-separated string
    fmt_buf.clear();
    for (i, pid) in multi.pids.iter().enumerate() {
        if i > 0 {
            fmt_buf.push(' ');
        }
        write!(fmt_buf, "{}", pid).unwrap();
    }
    elem.push_attribute(("pids", fmt_buf.as_str()));

    // Convert blendmethods to space-separated string
    if !multi.blendmethods.is_empty() {
        let blendmethods_str = multi
            .blendmethods
            .iter()
            .map(|b| match b {
                BlendMethod::Mix => "mix",
                BlendMethod::Multiply => "multiply",
            })
            .collect::<Vec<_>>()
            .join(" ");
        elem.push_attribute(("blendmethods", blendmethods_str.as_str()));
    }

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write multiproperties element: {}", e)))?;

    for multi_elem in &multi.multis {
        write_multi(writer, multi_elem)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("m:multiproperties")))
        .map_err(|e| Error::xml_write(format!("Failed to close multiproperties element: {}", e)))?;

    Ok(())
}

/// Write a multi entry
pub(super) fn write_multi<W: IoWrite>(writer: &mut Writer<W>, multi: &Multi) -> Result<()> {
    let mut fmt_buf = String::with_capacity(64);
    let mut elem = BytesStart::new("m:multi");

    // Convert pindices to space-separated string
    for (i, idx) in multi.pindices.iter().enumerate() {
        if i > 0 {
            fmt_buf.push(' ');
        }
        write!(fmt_buf, "{}", idx).unwrap();
    }
    elem.push_attribute(("pindices", fmt_buf.as_str()));

    writer
        .write_event(Event::Empty(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write multi: {}", e)))?;

    Ok(())
}
