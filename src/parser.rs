//! XML parsing for 3MF model files

use crate::error::{Error, Result};
use crate::model::*;
use crate::opc::Package;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::io::Read;

/// Parse a 3MF file from a reader
pub fn parse_3mf<R: Read + std::io::Seek>(reader: R) -> Result<Model> {
    let mut package = Package::open(reader)?;
    let model_xml = package.get_model()?;
    parse_model_xml(&model_xml)
}

/// Parse the 3D model XML content
fn parse_model_xml(xml: &str) -> Result<Model> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut model = Model::new();
    let mut buf = Vec::new();
    let mut in_resources = false;
    let mut in_build = false;
    let mut current_object: Option<Object> = None;
    let mut current_mesh: Option<Mesh> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name = e.name();
                let name_str = std::str::from_utf8(name.as_ref())
                    .map_err(|e| Error::InvalidXml(e.to_string()))?;

                match name_str {
                    "model" => {
                        // Parse model attributes
                        for attr in e.attributes() {
                            let attr = attr?;
                            let key = std::str::from_utf8(attr.key.as_ref())
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;
                            let value = std::str::from_utf8(&attr.value)
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;

                            match key {
                                "unit" => model.unit = value.to_string(),
                                "xmlns" => model.xmlns = value.to_string(),
                                _ => {}
                            }
                        }
                    }
                    "metadata" => {
                        let attrs = parse_attributes(&reader, e)?;
                        if let Some(name) = attrs.get("name") {
                            // Read the text content
                            if let Ok(Event::Text(t)) = reader.read_event_into(&mut buf) {
                                let value = t
                                    .unescape()
                                    .map_err(|e| Error::InvalidXml(e.to_string()))?;
                                model.metadata.insert(name.clone(), value.to_string());
                            }
                        }
                    }
                    "resources" => {
                        in_resources = true;
                    }
                    "build" => {
                        in_build = true;
                    }
                    "object" if in_resources => {
                        current_object = Some(parse_object(&reader, e)?);
                    }
                    "mesh" if in_resources && current_object.is_some() => {
                        current_mesh = Some(Mesh::new());
                    }
                    "vertices" if current_mesh.is_some() => {
                        // Vertices will be parsed as individual vertex elements
                    }
                    "vertex" if current_mesh.is_some() => {
                        if let Some(ref mut mesh) = current_mesh {
                            let vertex = parse_vertex(&reader, e)?;
                            mesh.vertices.push(vertex);
                        }
                    }
                    "triangles" if current_mesh.is_some() => {
                        // Triangles will be parsed as individual triangle elements
                    }
                    "triangle" if current_mesh.is_some() => {
                        if let Some(ref mut mesh) = current_mesh {
                            let triangle = parse_triangle(&reader, e)?;
                            mesh.triangles.push(triangle);
                        }
                    }
                    "item" if in_build => {
                        let item = parse_build_item(&reader, e)?;
                        model.build.items.push(item);
                    }
                    "basematerials" if in_resources => {
                        // Base materials group
                    }
                    "base" if in_resources => {
                        let material = parse_material(&reader, e)?;
                        model.resources.materials.push(material);
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                let name_str = std::str::from_utf8(name.as_ref())
                    .map_err(|e| Error::InvalidXml(e.to_string()))?;

                match name_str {
                    "resources" => {
                        in_resources = false;
                    }
                    "build" => {
                        in_build = false;
                    }
                    "object" => {
                        if let Some(mut obj) = current_object.take() {
                            if let Some(mesh) = current_mesh.take() {
                                obj.mesh = Some(mesh);
                            }
                            model.resources.objects.push(obj);
                        }
                    }
                    "mesh" => {
                        // Mesh parsing complete
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Error::Xml(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(model)
}

/// Parse object element attributes
fn parse_object<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<Object> {
    let attrs = parse_attributes(reader, e)?;

    let id = attrs
        .get("id")
        .ok_or_else(|| Error::InvalidXml("Object missing id attribute".to_string()))?
        .parse::<usize>()?;

    let mut object = Object::new(id);
    object.name = attrs.get("name").cloned();

    if let Some(type_str) = attrs.get("type") {
        object.object_type = match type_str.as_str() {
            "model" => ObjectType::Model,
            "support" => ObjectType::Support,
            _ => ObjectType::Other,
        };
    }

    if let Some(pid) = attrs.get("pid") {
        object.pid = Some(pid.parse::<usize>()?);
    }

    Ok(object)
}

/// Parse vertex element attributes
fn parse_vertex<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<Vertex> {
    let attrs = parse_attributes(reader, e)?;

    let x = attrs
        .get("x")
        .ok_or_else(|| Error::InvalidXml("Vertex missing x attribute".to_string()))?
        .parse::<f64>()?;

    let y = attrs
        .get("y")
        .ok_or_else(|| Error::InvalidXml("Vertex missing y attribute".to_string()))?
        .parse::<f64>()?;

    let z = attrs
        .get("z")
        .ok_or_else(|| Error::InvalidXml("Vertex missing z attribute".to_string()))?
        .parse::<f64>()?;

    Ok(Vertex::new(x, y, z))
}

/// Parse triangle element attributes
fn parse_triangle<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<Triangle> {
    let attrs = parse_attributes(reader, e)?;

    let v1 = attrs
        .get("v1")
        .ok_or_else(|| Error::InvalidXml("Triangle missing v1 attribute".to_string()))?
        .parse::<usize>()?;

    let v2 = attrs
        .get("v2")
        .ok_or_else(|| Error::InvalidXml("Triangle missing v2 attribute".to_string()))?
        .parse::<usize>()?;

    let v3 = attrs
        .get("v3")
        .ok_or_else(|| Error::InvalidXml("Triangle missing v3 attribute".to_string()))?
        .parse::<usize>()?;

    let mut triangle = Triangle::new(v1, v2, v3);

    if let Some(pid) = attrs.get("pid") {
        triangle.pid = Some(pid.parse::<usize>()?);
    }

    Ok(triangle)
}

/// Parse build item element attributes
fn parse_build_item<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<BuildItem> {
    let attrs = parse_attributes(reader, e)?;

    let objectid = attrs
        .get("objectid")
        .ok_or_else(|| Error::InvalidXml("Build item missing objectid attribute".to_string()))?
        .parse::<usize>()?;

    let mut item = BuildItem::new(objectid);

    if let Some(transform_str) = attrs.get("transform") {
        // Parse transformation matrix (12 values)
        let values: Result<Vec<f64>> = transform_str
            .split_whitespace()
            .map(|s| s.parse::<f64>().map_err(Error::from))
            .collect();

        let values = values?;
        if values.len() == 12 {
            let mut transform = [0.0; 12];
            transform.copy_from_slice(&values);
            item.transform = Some(transform);
        }
    }

    Ok(item)
}

/// Parse material (base) element attributes
fn parse_material<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<Material> {
    let attrs = parse_attributes(reader, e)?;

    // Generate a sequential ID - using a thread-safe counter
    use std::sync::atomic::{AtomicUsize, Ordering};
    static MATERIAL_COUNTER: AtomicUsize = AtomicUsize::new(0);
    
    let id = if let Some(id_str) = attrs.get("id") {
        id_str.parse::<usize>()?
    } else {
        MATERIAL_COUNTER.fetch_add(1, Ordering::SeqCst)
    };

    let mut material = Material::new(id);
    material.name = attrs.get("name").cloned();

    // Parse displaycolor attribute (format: #RRGGBBAA or #RRGGBB)
    if let Some(color_str) = attrs.get("displaycolor") {
        if let Some(color) = parse_color(color_str) {
            material.color = Some(color);
        }
    }

    Ok(material)
}

/// Parse color string in format #RRGGBBAA or #RRGGBB
fn parse_color(color_str: &str) -> Option<(u8, u8, u8, u8)> {
    let color_str = color_str.trim_start_matches('#');

    if color_str.len() == 6 {
        // #RRGGBB format (assume full opacity)
        let r = u8::from_str_radix(&color_str[0..2], 16).ok()?;
        let g = u8::from_str_radix(&color_str[2..4], 16).ok()?;
        let b = u8::from_str_radix(&color_str[4..6], 16).ok()?;
        Some((r, g, b, 255))
    } else if color_str.len() == 8 {
        // #RRGGBBAA format
        let r = u8::from_str_radix(&color_str[0..2], 16).ok()?;
        let g = u8::from_str_radix(&color_str[2..4], 16).ok()?;
        let b = u8::from_str_radix(&color_str[4..6], 16).ok()?;
        let a = u8::from_str_radix(&color_str[6..8], 16).ok()?;
        Some((r, g, b, a))
    } else {
        None
    }
}

/// Parse attributes from an XML element
fn parse_attributes<R: std::io::BufRead>(
    _reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<HashMap<String, String>> {
    let mut attrs = HashMap::new();

    for attr in e.attributes() {
        let attr = attr?;
        let key = std::str::from_utf8(attr.key.as_ref())
            .map_err(|e| Error::InvalidXml(e.to_string()))?;
        let value = std::str::from_utf8(&attr.value)
            .map_err(|e| Error::InvalidXml(e.to_string()))?;

        attrs.insert(key.to_string(), value.to_string());
    }

    Ok(attrs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color() {
        // Test #RRGGBB format
        assert_eq!(parse_color("#FF0000"), Some((255, 0, 0, 255)));
        assert_eq!(parse_color("#00FF00"), Some((0, 255, 0, 255)));
        assert_eq!(parse_color("#0000FF"), Some((0, 0, 255, 255)));

        // Test #RRGGBBAA format
        assert_eq!(parse_color("#FF000080"), Some((255, 0, 0, 128)));
        assert_eq!(parse_color("#00FF00FF"), Some((0, 255, 0, 255)));

        // Test invalid formats
        assert_eq!(parse_color("#FF"), None);
        assert_eq!(parse_color("FF0000"), Some((255, 0, 0, 255)));
    }

    #[test]
    fn test_parse_minimal_model() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <resources>
  </resources>
  <build>
  </build>
</model>"#;

        let model = parse_model_xml(xml).unwrap();
        assert_eq!(model.unit, "millimeter");
        assert_eq!(
            model.xmlns,
            "http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
        );
    }
}
