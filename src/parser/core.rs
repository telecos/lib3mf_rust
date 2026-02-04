//! Core 3MF element parsing
//!
//! This module handles parsing of core 3MF elements including objects, meshes,
//! vertices, triangles, components, and build items.

use crate::error::{Error, Result};
use crate::model::*;
use quick_xml::Reader;

use super::{TRANSFORM_MATRIX_SIZE, get_attr_by_local_name, parse_attributes, validate_attributes};

/// Parse object element attributes
pub fn parse_object<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<Object> {
    let attrs = parse_attributes(reader, e)?;

    // Validate only allowed attributes are present
    // Per 3MF Core spec v1.4.0, valid object attributes are: id, name, type, pid, partnumber, thumbnail
    // Per Materials Extension: pindex can be used with pid, basematerialid for base material references
    // Per Slice Extension: s:slicestackid (handled via extension attribute skipping)
    // Note: thumbnail is deprecated in the spec but still commonly used in valid files
    validate_attributes(
        &attrs,
        &[
            "id",
            "name",
            "type",
            "pid",
            "pindex",
            "basematerialid",
            "partnumber",
            "thumbnail",
        ],
        "object",
    )?;

    let id = attrs
        .get("id")
        .ok_or_else(|| Error::InvalidXml("Object missing id attribute".to_string()))?
        .parse::<usize>()?;

    let mut object = Object::new(id);
    object.name = attrs.get("name").cloned();

    // Validate object type if present
    // Per 3MF Core spec 1.4.0, valid types: model, support, solidsupport, surface, other
    if let Some(type_str) = attrs.get("type") {
        object.object_type = match type_str.as_str() {
            "model" => ObjectType::Model,
            "support" => ObjectType::Support,
            "solidsupport" => ObjectType::SolidSupport,
            "surface" => ObjectType::Surface,
            "other" => ObjectType::Other,
            _ => {
                return Err(Error::InvalidXml(format!(
                    "Invalid object type '{}'. Must be one of: model, support, solidsupport, surface, other",
                    type_str
                )));
            }
        };
    }

    if let Some(pid) = attrs.get("pid") {
        object.pid = Some(pid.parse::<usize>()?);
    }

    if let Some(pindex) = attrs.get("pindex") {
        object.pindex = Some(pindex.parse::<usize>()?);
    }

    if let Some(basematerialid) = attrs.get("basematerialid") {
        object.basematerialid = Some(basematerialid.parse::<usize>()?);
    }

    // Check for both namespaced and non-namespaced slicestackid
    // The attribute may appear as "s:slicestackid" in the XML
    if let Some(slicestackid) = attrs
        .get("slicestackid")
        .or_else(|| attrs.get("s:slicestackid"))
    {
        object.slicestackid = Some(slicestackid.parse::<usize>()?);
    }

    // Track if thumbnail attribute is present (for validation)
    if attrs.contains_key("thumbnail") {
        object.has_thumbnail_attribute = true;
    }

    // Extract Production extension attributes (UUID, path - any namespace prefix)
    let p_uuid = get_attr_by_local_name(&attrs, "UUID");
    let p_path = get_attr_by_local_name(&attrs, "path");

    if p_uuid.is_some() || p_path.is_some() {
        let mut prod_info = ProductionInfo::new();
        prod_info.uuid = p_uuid;
        prod_info.path = p_path;
        object.production = Some(prod_info);
    }

    Ok(object)
}

/// Parse vertex element attributes
pub fn parse_vertex<R: std::io::BufRead>(
    _reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<Vertex> {
    // Optimized: parse attributes directly without building HashMap
    let mut x_opt: Option<f64> = None;
    let mut y_opt: Option<f64> = None;
    let mut z_opt: Option<f64> = None;
    let mut invalid_attr_name: Option<String> = None;

    // Helper closure to parse f64 from byte slice
    let parse_f64 = |value: &[u8]| -> Result<f64> {
        let value_str = std::str::from_utf8(value).map_err(|e| Error::InvalidXml(e.to_string()))?;
        Ok(value_str.parse::<f64>()?)
    };

    for attr_result in e.attributes() {
        let attr = attr_result?;
        let key = attr.key.as_ref();

        match key {
            b"x" => x_opt = Some(parse_f64(&attr.value)?),
            b"y" => y_opt = Some(parse_f64(&attr.value)?),
            b"z" => z_opt = Some(parse_f64(&attr.value)?),
            _ => {
                // Store first invalid attribute for error reporting
                if invalid_attr_name.is_none() {
                    invalid_attr_name = Some(
                        std::str::from_utf8(key)
                            .unwrap_or("<invalid UTF-8>")
                            .to_string(),
                    );
                }
            }
        }
    }

    // Validate only allowed attributes are present
    if let Some(attr_name) = invalid_attr_name {
        return Err(Error::InvalidXml(format!(
            "Unexpected attribute '{}' in vertex element. Only x, y, z are allowed.",
            attr_name
        )));
    }

    let x = x_opt.ok_or_else(|| Error::InvalidXml("Vertex missing x attribute".to_string()))?;
    let y = y_opt.ok_or_else(|| Error::InvalidXml("Vertex missing y attribute".to_string()))?;
    let z = z_opt.ok_or_else(|| Error::InvalidXml("Vertex missing z attribute".to_string()))?;

    // Validate numeric values - reject NaN and Infinity
    // Check efficiently: if any is not finite, identify which one
    if !x.is_finite() {
        return Err(Error::InvalidXml(format!(
            "Vertex x coordinate must be finite (got {})",
            x
        )));
    }
    if !y.is_finite() {
        return Err(Error::InvalidXml(format!(
            "Vertex y coordinate must be finite (got {})",
            y
        )));
    }
    if !z.is_finite() {
        return Err(Error::InvalidXml(format!(
            "Vertex z coordinate must be finite (got {})",
            z
        )));
    }

    Ok(Vertex::new(x, y, z))
}

/// Parse triangle element attributes
pub fn parse_triangle<R: std::io::BufRead>(
    _reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<Triangle> {
    // Optimized: parse attributes directly without building HashMap
    let mut v1_opt: Option<usize> = None;
    let mut v2_opt: Option<usize> = None;
    let mut v3_opt: Option<usize> = None;
    let mut pid_opt: Option<usize> = None;
    let mut pindex_opt: Option<usize> = None;
    let mut p1_opt: Option<usize> = None;
    let mut p2_opt: Option<usize> = None;
    let mut p3_opt: Option<usize> = None;
    let mut invalid_attr_name: Option<String> = None;

    for attr_result in e.attributes() {
        let attr = attr_result?;
        let key = attr.key.as_ref();

        match key {
            b"v1" | b"v2" | b"v3" | b"pid" | b"pindex" | b"p1" | b"p2" | b"p3" => {
                // Only parse UTF-8 for known attributes
                let value_str = std::str::from_utf8(&attr.value)
                    .map_err(|e| Error::InvalidXml(e.to_string()))?;
                let value = value_str.parse::<usize>()?;

                match key {
                    b"v1" => v1_opt = Some(value),
                    b"v2" => v2_opt = Some(value),
                    b"v3" => v3_opt = Some(value),
                    b"pid" => pid_opt = Some(value),
                    b"pindex" => pindex_opt = Some(value),
                    b"p1" => p1_opt = Some(value),
                    b"p2" => p2_opt = Some(value),
                    b"p3" => p3_opt = Some(value),
                    _ => unreachable!(),
                }
            }
            _ => {
                // Store first invalid attribute for error reporting
                if invalid_attr_name.is_none() {
                    invalid_attr_name = Some(
                        std::str::from_utf8(key)
                            .unwrap_or("<invalid UTF-8>")
                            .to_string(),
                    );
                }
            }
        }
    }

    // Validate only allowed attributes are present
    if let Some(attr_name) = invalid_attr_name {
        return Err(Error::InvalidXml(format!(
            "Unexpected attribute '{}' in triangle element. Only v1, v2, v3, pid, pindex, p1, p2, p3 are allowed.",
            attr_name
        )));
    }

    let v1 =
        v1_opt.ok_or_else(|| Error::InvalidXml("Triangle missing v1 attribute".to_string()))?;
    let v2 =
        v2_opt.ok_or_else(|| Error::InvalidXml("Triangle missing v2 attribute".to_string()))?;
    let v3 =
        v3_opt.ok_or_else(|| Error::InvalidXml("Triangle missing v3 attribute".to_string()))?;

    let mut triangle = Triangle::new(v1, v2, v3);
    triangle.pid = pid_opt;
    triangle.pindex = pindex_opt;
    triangle.p1 = p1_opt;
    triangle.p2 = p2_opt;
    triangle.p3 = p3_opt;

    Ok(triangle)
}

/// Parse build item element attributes
pub fn parse_build_item<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<BuildItem> {
    let attrs = parse_attributes(reader, e)?;

    // Validate only allowed attributes are present
    // Per 3MF Core spec: objectid, transform, partnumber, thumbnail
    // Production extension adds: p:UUID, p:path
    // Note: thumbnail is deprecated in the spec but still commonly used in valid files
    validate_attributes(
        &attrs,
        &[
            "objectid",
            "transform",
            "partnumber",
            "thumbnail",
            "p:UUID",
            "p:path",
        ],
        "item",
    )?;

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

        // Transform must have exactly 12 values (TRANSFORM_MATRIX_SIZE)
        if values.len() != TRANSFORM_MATRIX_SIZE {
            return Err(Error::InvalidXml(format!(
                "Transform matrix must have exactly {} values (got {})",
                TRANSFORM_MATRIX_SIZE,
                values.len()
            )));
        }

        // Validate all values are finite (no NaN or Infinity)
        for (idx, &val) in values.iter().enumerate() {
            if !val.is_finite() {
                return Err(Error::InvalidXml(format!(
                    "Transform matrix value at index {} must be finite (got {})",
                    idx, val
                )));
            }
        }

        let mut transform = [0.0; 12];
        transform.copy_from_slice(&values);
        item.transform = Some(transform);
    }

    // Extract Production extension UUID (any namespace prefix)
    if let Some(p_uuid) = get_attr_by_local_name(&attrs, "UUID") {
        item.production_uuid = Some(p_uuid);
    }

    // Extract Production extension path (any namespace prefix)
    if let Some(p_path) = get_attr_by_local_name(&attrs, "path") {
        item.production_path = Some(p_path);
    }

    Ok(item)
}

/// Parse component element attributes
pub(super) fn parse_component<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<Component> {
    let attrs = parse_attributes(reader, e)?;

    // Validate only allowed attributes are present
    // Per 3MF Core spec: objectid, transform
    // Production extension adds: p:UUID, p:path
    validate_attributes(
        &attrs,
        &["objectid", "transform", "p:UUID", "p:path"],
        "component",
    )?;

    let objectid = attrs
        .get("objectid")
        .ok_or_else(|| Error::InvalidXml("Component missing objectid attribute".to_string()))?
        .parse::<usize>()?;

    let mut component = Component::new(objectid);

    if let Some(transform_str) = attrs.get("transform") {
        // Parse transformation matrix (12 values)
        let values: Result<Vec<f64>> = transform_str
            .split_whitespace()
            .map(|s| s.parse::<f64>().map_err(Error::from))
            .collect();

        let values = values?;

        // Transform must have exactly 12 values (TRANSFORM_MATRIX_SIZE)
        if values.len() != TRANSFORM_MATRIX_SIZE {
            return Err(Error::InvalidXml(format!(
                "Component transform matrix must have exactly {} values (got {})",
                TRANSFORM_MATRIX_SIZE,
                values.len()
            )));
        }

        // Validate all values are finite (no NaN or Infinity)
        for (idx, &val) in values.iter().enumerate() {
            if !val.is_finite() {
                return Err(Error::InvalidXml(format!(
                    "Component transform matrix value at index {} must be finite (got {})",
                    idx, val
                )));
            }
        }

        let mut transform = [0.0; 12];
        transform.copy_from_slice(&values);
        component.transform = Some(transform);
    }

    // Extract Production extension attributes (UUID, path - any namespace prefix)
    let p_uuid = get_attr_by_local_name(&attrs, "UUID");
    let p_path = get_attr_by_local_name(&attrs, "path");

    // For backward compatibility, also set component.path directly
    if p_path.is_some() {
        component.path = p_path.clone();
    }

    if p_uuid.is_some() || p_path.is_some() {
        let mut prod_info = ProductionInfo::new();
        prod_info.uuid = p_uuid;
        prod_info.path = p_path;
        component.production = Some(prod_info);
    }

    Ok(component)
}
