//! Boolean operations extension writing for 3MF model files
//!
//! This module provides functionality to write Boolean Operations extension elements.

use crate::error::{Error, Result};
use crate::model::*;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use std::io::Write as IoWrite;

/// Write boolean shape (boolean operations extension)
pub(super) fn write_boolean_shape<W: IoWrite>(
    writer: &mut Writer<W>,
    shape: &BooleanShape,
) -> Result<()> {
    let mut elem = BytesStart::new("bool:booleanshape");
    elem.push_attribute(("objectid", shape.objectid.to_string().as_str()));

    let op_type = match shape.operation {
        BooleanOpType::Union => "union",
        BooleanOpType::Intersection => "intersection",
        BooleanOpType::Difference => "difference",
    };
    elem.push_attribute(("op", op_type));

    if let Some(ref path) = shape.path {
        elem.push_attribute(("path", path.as_str()));
    }

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write booleanshape element: {}", e)))?;

    // Write boolean references (operands)
    for boolean_ref in &shape.operands {
        let mut ref_elem = BytesStart::new("bool:booleanref");
        ref_elem.push_attribute(("objectid", boolean_ref.objectid.to_string().as_str()));

        if let Some(ref path) = boolean_ref.path {
            ref_elem.push_attribute(("path", path.as_str()));
        }

        writer
            .write_event(Event::Empty(ref_elem))
            .map_err(|e| Error::xml_write(format!("Failed to write booleanref: {}", e)))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("bool:booleanshape")))
        .map_err(|e| Error::xml_write(format!("Failed to close booleanshape element: {}", e)))?;

    Ok(())
}
