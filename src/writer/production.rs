//! Production extension writing for 3MF model files
//!
//! This module provides functionality to write Production extension elements
//! like build sections and build items.

use crate::error::{Error, Result};
use crate::model::*;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use std::io::Write as IoWrite;

/// Write build section
pub(super) fn write_build<W: IoWrite>(writer: &mut Writer<W>, build: &Build) -> Result<()> {
    let mut elem = BytesStart::new("build");

    if let Some(ref uuid) = build.production_uuid {
        elem.push_attribute(("p:UUID", uuid.as_str()));
    }

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write build element: {}", e)))?;

    for item in &build.items {
        write_build_item(writer, item)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("build")))
        .map_err(|e| Error::xml_write(format!("Failed to close build element: {}", e)))?;

    Ok(())
}

/// Write a build item
pub(super) fn write_build_item<W: IoWrite>(writer: &mut Writer<W>, item: &BuildItem) -> Result<()> {
    let mut elem = BytesStart::new("item");
    elem.push_attribute(("objectid", item.objectid.to_string().as_str()));

    if let Some(transform) = item.transform {
        let transform_str = transform
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        elem.push_attribute(("transform", transform_str.as_str()));
    }

    // Production extension attributes
    if let Some(ref uuid) = item.production_uuid {
        elem.push_attribute(("p:UUID", uuid.as_str()));
    }

    if let Some(ref path) = item.production_path {
        elem.push_attribute(("p:path", path.as_str()));
    }

    writer
        .write_event(Event::Empty(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write item: {}", e)))?;

    Ok(())
}
