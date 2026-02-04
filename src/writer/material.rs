//! Material extension writing for 3MF model files
//!
//! This module provides functionality to write Material extension elements like
//! base materials, textures, color groups, composites, and multi-properties.

use crate::error::{Error, Result};
use crate::model::*;
use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use std::io::Write as IoWrite;

/// Write a base material group
pub(super) fn write_base_material_group<W: IoWrite>(
    writer: &mut Writer<W>,
    group: &BaseMaterialGroup,
) -> Result<()> {
    let mut elem = BytesStart::new("m:basematerials");
    elem.push_attribute(("id", group.id.to_string().as_str()));

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write basematerials element: {}", e)))?;

    for material in &group.materials {
        let mut mat_elem = BytesStart::new("m:base");
        mat_elem.push_attribute(("name", material.name.as_str()));

        let color = format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            material.displaycolor.0,
            material.displaycolor.1,
            material.displaycolor.2,
            material.displaycolor.3
        );
        mat_elem.push_attribute(("displaycolor", color.as_str()));

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
    let mut elem = BytesStart::new("m:texture2d");
    elem.push_attribute(("id", texture.id.to_string().as_str()));
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
    let mut elem = BytesStart::new("m:texture2dgroup");
    elem.push_attribute(("id", group.id.to_string().as_str()));
    elem.push_attribute(("texid", group.texid.to_string().as_str()));

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write texture2dgroup element: {}", e)))?;

    for coord in &group.tex2coords {
        let mut coord_elem = BytesStart::new("m:tex2coord");
        coord_elem.push_attribute(("u", coord.u.to_string().as_str()));
        coord_elem.push_attribute(("v", coord.v.to_string().as_str()));

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
    let mut elem = BytesStart::new("m:colorgroup");
    elem.push_attribute(("id", group.id.to_string().as_str()));

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write colorgroup element: {}", e)))?;

    for color in &group.colors {
        let mut color_elem = BytesStart::new("m:color");
        let color_str = format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            color.0, color.1, color.2, color.3
        );
        color_elem.push_attribute(("color", color_str.as_str()));

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
    let mut elem = BytesStart::new("m:compositematerials");
    elem.push_attribute(("id", composite.id.to_string().as_str()));
    elem.push_attribute(("matid", composite.matid.to_string().as_str()));

    // Convert matindices to space-separated string
    let matindices_str = composite
        .matindices
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    elem.push_attribute(("matindices", matindices_str.as_str()));

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
    let mut elem = BytesStart::new("m:composite");

    // Convert values to space-separated string
    let values_str = composite
        .values
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    elem.push_attribute(("values", values_str.as_str()));

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
    let mut elem = BytesStart::new("m:multiproperties");
    elem.push_attribute(("id", multi.id.to_string().as_str()));

    // Convert pids to space-separated string
    let pids_str = multi
        .pids
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    elem.push_attribute(("pids", pids_str.as_str()));

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
    let mut elem = BytesStart::new("m:multi");

    // Convert pindices to space-separated string
    let pindices_str = multi
        .pindices
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    elem.push_attribute(("pindices", pindices_str.as_str()));

    writer
        .write_event(Event::Empty(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write multi: {}", e)))?;

    Ok(())
}
