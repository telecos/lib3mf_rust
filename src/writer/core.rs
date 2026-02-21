//! Core element writing for 3MF model files
//!
//! This module provides functionality to write core 3MF elements like
//! objects, meshes, and components.

use crate::error::{Error, Result};
use crate::model::*;
use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;

use super::beam_lattice::write_beamset;
use super::boolean_ops::write_boolean_shape;

/// Write an object
pub(super) fn write_object<W: IoWrite>(writer: &mut Writer<W>, object: &Object) -> Result<()> {
    let mut fmt_buf = String::with_capacity(32);
    let mut elem = BytesStart::new("object");

    write!(fmt_buf, "{}", object.id).unwrap();
    elem.push_attribute(("id", fmt_buf.as_str()));

    let obj_type = match object.object_type {
        ObjectType::Model => "model",
        ObjectType::Support => "support",
        ObjectType::SolidSupport => "solidsupport",
        ObjectType::Surface => "surface",
        ObjectType::Other => "other",
    };
    elem.push_attribute(("type", obj_type));

    if let Some(ref name) = object.name {
        elem.push_attribute(("name", name.as_str()));
    }

    if let Some(pid) = object.pid {
        fmt_buf.clear();
        write!(fmt_buf, "{}", pid).unwrap();
        elem.push_attribute(("pid", fmt_buf.as_str()));
    }

    if let Some(pindex) = object.pindex {
        fmt_buf.clear();
        write!(fmt_buf, "{}", pindex).unwrap();
        elem.push_attribute(("pindex", fmt_buf.as_str()));
    }

    if let Some(basematerialid) = object.basematerialid {
        fmt_buf.clear();
        write!(fmt_buf, "{}", basematerialid).unwrap();
        elem.push_attribute(("basematerialid", fmt_buf.as_str()));
    }

    // Production extension attributes
    if let Some(ref production) = object.production {
        if let Some(ref uuid) = production.uuid {
            elem.push_attribute(("p:UUID", uuid.as_str()));
        }

        if let Some(ref path) = production.path {
            elem.push_attribute(("p:path", path.as_str()));
        }
    }

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write object element: {}", e)))?;

    // Write mesh if present
    if let Some(ref mesh) = object.mesh {
        write_mesh(writer, mesh)?;
    }

    // Write components if present
    if !object.components.is_empty() {
        write_components(writer, &object.components)?;
    }

    // Write boolean shape if present (boolean operations extension)
    if let Some(ref boolean_shape) = object.boolean_shape {
        write_boolean_shape(writer, boolean_shape)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("object")))
        .map_err(|e| Error::xml_write(format!("Failed to close object element: {}", e)))?;

    Ok(())
}

/// Write a mesh
pub(super) fn write_mesh<W: IoWrite>(writer: &mut Writer<W>, mesh: &Mesh) -> Result<()> {
    writer
        .write_event(Event::Start(BytesStart::new("mesh")))
        .map_err(|e| Error::xml_write(format!("Failed to write mesh element: {}", e)))?;

    // Write vertices
    writer
        .write_event(Event::Start(BytesStart::new("vertices")))
        .map_err(|e| Error::xml_write(format!("Failed to write vertices element: {}", e)))?;

    // Reuse a single format buffer to avoid per-attribute String allocations
    let mut fmt_buf = String::with_capacity(32);

    for vertex in &mesh.vertices {
        let mut v_elem = BytesStart::new("vertex");

        fmt_buf.clear();
        write!(fmt_buf, "{}", vertex.x).unwrap();
        v_elem.push_attribute(("x", fmt_buf.as_str()));

        fmt_buf.clear();
        write!(fmt_buf, "{}", vertex.y).unwrap();
        v_elem.push_attribute(("y", fmt_buf.as_str()));

        fmt_buf.clear();
        write!(fmt_buf, "{}", vertex.z).unwrap();
        v_elem.push_attribute(("z", fmt_buf.as_str()));

        writer
            .write_event(Event::Empty(v_elem))
            .map_err(|e| Error::xml_write(format!("Failed to write vertex: {}", e)))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("vertices")))
        .map_err(|e| Error::xml_write(format!("Failed to close vertices element: {}", e)))?;

    // Write triangles
    writer
        .write_event(Event::Start(BytesStart::new("triangles")))
        .map_err(|e| Error::xml_write(format!("Failed to write triangles element: {}", e)))?;

    for triangle in &mesh.triangles {
        let mut t_elem = BytesStart::new("triangle");

        fmt_buf.clear();
        write!(fmt_buf, "{}", triangle.v1).unwrap();
        t_elem.push_attribute(("v1", fmt_buf.as_str()));

        fmt_buf.clear();
        write!(fmt_buf, "{}", triangle.v2).unwrap();
        t_elem.push_attribute(("v2", fmt_buf.as_str()));

        fmt_buf.clear();
        write!(fmt_buf, "{}", triangle.v3).unwrap();
        t_elem.push_attribute(("v3", fmt_buf.as_str()));

        if let Some(pid) = triangle.pid {
            fmt_buf.clear();
            write!(fmt_buf, "{}", pid).unwrap();
            t_elem.push_attribute(("pid", fmt_buf.as_str()));
        }

        if let Some(pindex) = triangle.pindex {
            fmt_buf.clear();
            write!(fmt_buf, "{}", pindex).unwrap();
            t_elem.push_attribute(("pindex", fmt_buf.as_str()));
        }

        if let Some(p1) = triangle.p1 {
            fmt_buf.clear();
            write!(fmt_buf, "{}", p1).unwrap();
            t_elem.push_attribute(("p1", fmt_buf.as_str()));
        }

        if let Some(p2) = triangle.p2 {
            fmt_buf.clear();
            write!(fmt_buf, "{}", p2).unwrap();
            t_elem.push_attribute(("p2", fmt_buf.as_str()));
        }

        if let Some(p3) = triangle.p3 {
            fmt_buf.clear();
            write!(fmt_buf, "{}", p3).unwrap();
            t_elem.push_attribute(("p3", fmt_buf.as_str()));
        }

        writer
            .write_event(Event::Empty(t_elem))
            .map_err(|e| Error::xml_write(format!("Failed to write triangle: {}", e)))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("triangles")))
        .map_err(|e| Error::xml_write(format!("Failed to close triangles element: {}", e)))?;

    // Write beamset if present (beam lattice extension)
    if let Some(ref beamset) = mesh.beamset {
        write_beamset(writer, beamset)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("mesh")))
        .map_err(|e| Error::xml_write(format!("Failed to close mesh element: {}", e)))?;

    Ok(())
}

/// Write components
pub(super) fn write_components<W: IoWrite>(
    writer: &mut Writer<W>,
    components: &[Component],
) -> Result<()> {
    writer
        .write_event(Event::Start(BytesStart::new("components")))
        .map_err(|e| Error::xml_write(format!("Failed to write components element: {}", e)))?;

    // Reuse a single format buffer to avoid per-attribute String allocations
    let mut fmt_buf = String::with_capacity(32);

    for component in components {
        let mut elem = BytesStart::new("component");

        fmt_buf.clear();
        write!(fmt_buf, "{}", component.objectid).unwrap();
        elem.push_attribute(("objectid", fmt_buf.as_str()));

        if let Some(transform) = component.transform {
            fmt_buf.clear();
            for (i, v) in transform.iter().enumerate() {
                if i > 0 {
                    fmt_buf.push(' ');
                }
                write!(fmt_buf, "{}", v).unwrap();
            }
            elem.push_attribute(("transform", fmt_buf.as_str()));
        }

        // Production extension attributes
        if let Some(ref prod_info) = component.production {
            if let Some(ref uuid) = prod_info.uuid {
                elem.push_attribute(("p:UUID", uuid.as_str()));
            }
            if let Some(ref path) = prod_info.path {
                elem.push_attribute(("p:path", path.as_str()));
            }
        }

        writer
            .write_event(Event::Empty(elem))
            .map_err(|e| Error::xml_write(format!("Failed to write component: {}", e)))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("components")))
        .map_err(|e| Error::xml_write(format!("Failed to close components element: {}", e)))?;

    Ok(())
}
