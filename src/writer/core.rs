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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ProductionInfo;
    use crate::model::{Component, Mesh, Object, ObjectType, Triangle, Vertex};
    use quick_xml::Writer;

    fn xml_from_object(object: &Object) -> String {
        let mut buf = Vec::new();
        let mut writer = Writer::new(&mut buf);
        write_object(&mut writer, object).expect("write_object failed");
        String::from_utf8(buf).unwrap()
    }

    fn xml_from_components(components: &[Component]) -> String {
        let mut buf = Vec::new();
        let mut writer = Writer::new(&mut buf);
        write_components(&mut writer, components).expect("write_components failed");
        String::from_utf8(buf).unwrap()
    }

    fn xml_from_mesh(mesh: &Mesh) -> String {
        let mut buf = Vec::new();
        let mut writer = Writer::new(&mut buf);
        write_mesh(&mut writer, mesh).expect("write_mesh failed");
        String::from_utf8(buf).unwrap()
    }

    fn simple_mesh() -> Mesh {
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));
        mesh
    }

    #[test]
    fn test_write_object_type_model() {
        let mut obj = Object::new(1);
        obj.object_type = ObjectType::Model;
        obj.mesh = Some(simple_mesh());
        let xml = xml_from_object(&obj);
        assert!(xml.contains("type=\"model\""));
    }

    #[test]
    fn test_write_object_type_support() {
        let mut obj = Object::new(2);
        obj.object_type = ObjectType::Support;
        obj.mesh = Some(simple_mesh());
        let xml = xml_from_object(&obj);
        assert!(xml.contains("type=\"support\""));
    }

    #[test]
    fn test_write_object_type_solidsupport() {
        let mut obj = Object::new(3);
        obj.object_type = ObjectType::SolidSupport;
        obj.mesh = Some(simple_mesh());
        let xml = xml_from_object(&obj);
        assert!(xml.contains("type=\"solidsupport\""));
    }

    #[test]
    fn test_write_object_type_surface() {
        let mut obj = Object::new(4);
        obj.object_type = ObjectType::Surface;
        obj.mesh = Some(simple_mesh());
        let xml = xml_from_object(&obj);
        assert!(xml.contains("type=\"surface\""));
    }

    #[test]
    fn test_write_object_type_other() {
        let mut obj = Object::new(5);
        obj.object_type = ObjectType::Other;
        obj.mesh = Some(simple_mesh());
        let xml = xml_from_object(&obj);
        assert!(xml.contains("type=\"other\""));
    }

    #[test]
    fn test_write_object_pid_pindex() {
        let mut obj = Object::new(1);
        obj.pid = Some(10);
        obj.pindex = Some(3);
        obj.mesh = Some(simple_mesh());
        let xml = xml_from_object(&obj);
        assert!(xml.contains("pid=\"10\""));
        assert!(xml.contains("pindex=\"3\""));
    }

    #[test]
    fn test_write_object_production_info() {
        let mut obj = Object::new(1);
        let mut prod = ProductionInfo::new();
        prod.uuid = Some("urn:uuid:11111111-1111-1111-1111-111111111111".to_string());
        prod.path = Some("/3D/other.model".to_string());
        obj.production = Some(prod);
        obj.mesh = Some(simple_mesh());
        let xml = xml_from_object(&obj);
        assert!(xml.contains("p:UUID=\"urn:uuid:11111111-1111-1111-1111-111111111111\""));
        assert!(xml.contains("p:path=\"/3D/other.model\""));
    }

    #[test]
    fn test_write_object_production_uuid_only() {
        let mut obj = Object::new(1);
        obj.production = Some(ProductionInfo::with_uuid(
            "urn:uuid:22222222-2222-2222-2222-222222222222".to_string(),
        ));
        obj.mesh = Some(simple_mesh());
        let xml = xml_from_object(&obj);
        assert!(xml.contains("p:UUID=\"urn:uuid:22222222-2222-2222-2222-222222222222\""));
        assert!(!xml.contains("p:path="));
    }

    #[test]
    fn test_write_object_with_components() {
        let mut obj = Object::new(1);
        obj.components.push(Component::new(2));
        obj.components.push(Component::new(3));
        let xml = xml_from_object(&obj);
        assert!(xml.contains("<components>"));
        assert!(xml.contains("objectid=\"2\""));
        assert!(xml.contains("objectid=\"3\""));
        assert!(xml.contains("</components>"));
    }

    #[test]
    fn test_write_components_with_transform() {
        let transform = [
            1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 10.0, 20.0, 30.0,
        ];
        let component = Component::with_transform(5, transform);
        let xml = xml_from_components(&[component]);
        assert!(xml.contains("objectid=\"5\""));
        assert!(xml.contains("transform="));
        assert!(xml.contains("10"));
        assert!(xml.contains("20"));
        assert!(xml.contains("30"));
    }

    #[test]
    fn test_write_components_with_production_info() {
        let mut component = Component::new(7);
        let mut prod = ProductionInfo::new();
        prod.uuid = Some("urn:uuid:33333333-3333-3333-3333-333333333333".to_string());
        prod.path = Some("/3D/part.model".to_string());
        component.production = Some(prod);
        let xml = xml_from_components(&[component]);
        assert!(xml.contains("p:UUID=\"urn:uuid:33333333-3333-3333-3333-333333333333\""));
        assert!(xml.contains("p:path=\"/3D/part.model\""));
    }

    #[test]
    fn test_write_mesh_triangle_per_vertex_properties() {
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
        let mut triangle = Triangle::new(0, 1, 2);
        triangle.p1 = Some(0);
        triangle.p2 = Some(1);
        triangle.p3 = Some(2);
        mesh.triangles.push(triangle);
        let xml = xml_from_mesh(&mesh);
        assert!(xml.contains("p1=\"0\""));
        assert!(xml.contains("p2=\"1\""));
        assert!(xml.contains("p3=\"2\""));
    }
}
