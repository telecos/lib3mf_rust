//! XML parsing for 3MF model files

use crate::error::{Error, Result};
use crate::model::*;
use crate::opc::Package;
use crate::validator;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::io::Read;

/// Parse a 3MF file from a reader
pub fn parse_3mf<R: Read + std::io::Seek>(reader: R) -> Result<Model> {
    // Use default config that supports all extensions for backward compatibility
    parse_3mf_with_config(reader, ParserConfig::with_all_extensions())
}

/// Parse a 3MF file from a reader with custom configuration
pub fn parse_3mf_with_config<R: Read + std::io::Seek>(
    reader: R,
    config: ParserConfig,
) -> Result<Model> {
    let mut package = Package::open(reader)?;
    let model_xml = package.get_model()?;
    parse_model_xml_with_config(&model_xml, config)
}

/// Extract local name from potentially namespaced XML element name
///
/// 3MF files use XML namespaces for extensions. This function extracts
/// the local element name without the namespace prefix.
///
/// # Examples
///
/// - `"m:colorgroup"` returns `"colorgroup"`
/// - `"p:UUID"` returns `"UUID"`
/// - `"object"` returns `"object"`
fn get_local_name(name_str: &str) -> &str {
    if let Some(pos) = name_str.rfind(':') {
        &name_str[pos + 1..]
    } else {
        name_str
    }
}

/// Parse the 3D model XML content
///
/// This is primarily used for testing. For production use, use `Model::from_reader()`.
///
/// Note: This function is public to enable integration testing, but marked #[doc(hidden)]
/// to discourage use in production code. We can't use #[cfg(test)] because integration
/// tests in the tests/ directory are compiled separately and wouldn't have access.
#[doc(hidden)]
pub fn parse_model_xml(xml: &str) -> Result<Model> {
    parse_model_xml_with_config(xml, ParserConfig::with_all_extensions())
}

/// Parse the 3D model XML content with configuration
///
/// This is primarily used for testing. For production use, use `Model::from_reader_with_config()`.
///
/// Note: This function is public to enable integration testing, but marked #[doc(hidden)]
/// to discourage use in production code. We can't use #[cfg(test)] because integration
/// tests in the tests/ directory are compiled separately and wouldn't have access.
#[doc(hidden)]
pub fn parse_model_xml_with_config(xml: &str, config: ParserConfig) -> Result<Model> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut model = Model::new();
    let mut buf = Vec::new();
    let mut in_resources = false;
    let mut in_build = false;
    let mut current_object: Option<Object> = None;
    let mut current_mesh: Option<Mesh> = None;
    let mut in_basematerials = false;
    let mut material_index: usize = 0;
    let mut current_colorgroup: Option<ColorGroup> = None;
    let mut in_colorgroup = false;

    // Track required elements for validation
    let mut resources_count = 0;
    let mut build_count = 0;

    // Track namespace declarations from model element
    let mut declared_namespaces: HashMap<String, String> = HashMap::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name = e.name();
                let name_str = std::str::from_utf8(name.as_ref())
                    .map_err(|e| Error::InvalidXml(e.to_string()))?;

                let local_name = get_local_name(name_str);

                match local_name {
                    "model" => {
                        // Parse model element using two-pass approach:
                        // 1. First collect all namespace declarations (xmlns:prefix attributes)
                        // 2. Then resolve requiredextensions which may use those prefixes
                        let mut namespaces = HashMap::new();
                        let mut required_ext_value = None;
                        let mut all_attrs = HashMap::new();

                        // Parse model attributes
                        for attr in e.attributes() {
                            let attr = attr?;
                            let key = std::str::from_utf8(attr.key.as_ref())
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;
                            let value = std::str::from_utf8(&attr.value)
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;

                            all_attrs.insert(key.to_string(), value.to_string());

                            match key {
                                "unit" => {
                                    // Validate unit value - must be one of the allowed units
                                    match value {
                                        "micron" | "millimeter" | "centimeter" | "inch" | "foot" | "meter" => {
                                            model.unit = value.to_string()
                                        }
                                        _ => {
                                            return Err(Error::InvalidXml(format!(
                                                "Invalid unit '{}'. Must be one of: micron, millimeter, centimeter, inch, foot, meter",
                                                value
                                            )))
                                        }
                                    }
                                }
                                "xmlns" => model.xmlns = value.to_string(),
                                "requiredextensions" => {
                                    required_ext_value = Some(value.to_string());
                                }
                                _ => {
                                    // Check if it's a namespace declaration (xmlns:prefix)
                                    if let Some(prefix) = key.strip_prefix("xmlns:") {
                                        namespaces.insert(prefix.to_string(), value.to_string());
                                    }
                                }
                            }
                        }

                        // Store namespace declarations for later use (e.g., for metadata validation)
                        declared_namespaces = namespaces.clone();

                        // Validate model attributes - only allow specific attributes
                        // Per 3MF Core spec: unit, xml:lang, requiredextensions, and xmlns declarations
                        validate_attributes(
                            &all_attrs,
                            &["unit", "xml:lang", "requiredextensions", "xmlns"],
                            "model",
                        )?;

                        // Now parse required extensions with namespace context
                        if let Some(ext_value) = required_ext_value {
                            model.required_extensions =
                                parse_required_extensions_with_namespaces(&ext_value, &namespaces)?;
                            // Validate that all required extensions are supported
                            validate_extensions(&model.required_extensions, &config)?;
                        }
                    }
                    "metadata" => {
                        let attrs = parse_attributes(&reader, e)?;
                        if let Some(name) = attrs.get("name") {
                            // Validate metadata name - allow namespaced metadata if namespace is declared
                            // Per 3MF spec, metadata names with ':' indicate namespaced metadata
                            if name.contains(':') {
                                // Extract the namespace prefix (part before the colon)
                                if let Some(namespace_prefix) = name.split(':').next() {
                                    // Check if this is a known XML prefix (xml, xmlns) or a declared namespace
                                    if namespace_prefix != "xml"
                                        && namespace_prefix != "xmlns"
                                        && !declared_namespaces.contains_key(namespace_prefix)
                                    {
                                        // Undeclared custom namespace prefix - reject
                                        return Err(Error::InvalidXml(format!(
                                            "Metadata name '{}' uses undeclared namespace prefix '{}'",
                                            name, namespace_prefix
                                        )));
                                    }
                                }
                            }

                            // Read the text content
                            if let Ok(Event::Text(t)) = reader.read_event_into(&mut buf) {
                                let value =
                                    t.unescape().map_err(|e| Error::InvalidXml(e.to_string()))?;
                                model.metadata.insert(name.clone(), value.to_string());
                            }
                        }
                    }
                    "resources" => {
                        resources_count += 1;
                        if resources_count > 1 {
                            return Err(Error::InvalidXml(
                                "Model must contain exactly one <resources> element".to_string(),
                            ));
                        }
                        in_resources = true;
                    }
                    "build" => {
                        build_count += 1;
                        if build_count > 1 {
                            return Err(Error::InvalidXml(
                                "Model must contain exactly one <build> element".to_string(),
                            ));
                        }
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
                        in_basematerials = true;
                        material_index = 0;
                        // basematerials can have an ID attribute, but we use sequential indices
                        // for individual materials within the group
                    }
                    "base" if in_basematerials => {
                        // Materials within basematerials use sequential indices
                        let material = parse_base_material(&reader, e, material_index)?;
                        model.resources.materials.push(material);
                        material_index += 1;
                    }
                    "colorgroup" if in_resources => {
                        in_colorgroup = true;
                        let attrs = parse_attributes(&reader, e)?;
                        let id = attrs
                            .get("id")
                            .ok_or_else(|| {
                                Error::InvalidXml("ColorGroup missing id attribute".to_string())
                            })?
                            .parse::<usize>()?;
                        current_colorgroup = Some(ColorGroup::new(id));
                    }
                    "color" if in_colorgroup => {
                        if let Some(ref mut colorgroup) = current_colorgroup {
                            let attrs = parse_attributes(&reader, e)?;
                            if let Some(color_str) = attrs.get("color") {
                                if let Some(color) = parse_color(color_str) {
                                    colorgroup.colors.push(color);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                let name_str = std::str::from_utf8(name.as_ref())
                    .map_err(|e| Error::InvalidXml(e.to_string()))?;

                let local_name = get_local_name(name_str);

                match local_name {
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
                    "basematerials" => {
                        in_basematerials = false;
                    }
                    "colorgroup" => {
                        if let Some(colorgroup) = current_colorgroup.take() {
                            model.resources.color_groups.push(colorgroup);
                        }
                        in_colorgroup = false;
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

    // Validate required elements exist
    if resources_count == 0 {
        return Err(Error::InvalidXml(
            "Model must contain a <resources> element".to_string(),
        ));
    }
    if build_count == 0 {
        return Err(Error::InvalidXml(
            "Model must contain a <build> element".to_string(),
        ));
    }

    // Validate the model before returning
    validator::validate_model(&model)?;

    Ok(model)
}

/// Parse object element attributes
fn parse_object<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<Object> {
    let attrs = parse_attributes(reader, e)?;

    // Validate only allowed attributes are present
    // Per 3MF Core spec v1.4.0, valid object attributes are: id, name, type, pid, partnumber, thumbnail
    // Per Materials Extension: pindex can be used with pid
    // Note: thumbnail is deprecated in the spec but still commonly used in valid files
    validate_attributes(
        &attrs,
        &[
            "id",
            "name",
            "type",
            "pid",
            "pindex",
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
                )))
            }
        };
    }

    if let Some(pid) = attrs.get("pid") {
        object.pid = Some(pid.parse::<usize>()?);
    }

    if let Some(pindex) = attrs.get("pindex") {
        object.pindex = Some(pindex.parse::<usize>()?);
    }

    Ok(object)
}

/// Parse vertex element attributes
fn parse_vertex<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<Vertex> {
    let attrs = parse_attributes(reader, e)?;

    // Validate only allowed attributes are present
    // Per 3MF Core spec: x, y, z
    validate_attributes(&attrs, &["x", "y", "z"], "vertex")?;

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

    // Validate numeric values - reject NaN and Infinity
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
fn parse_triangle<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<Triangle> {
    let attrs = parse_attributes(reader, e)?;

    // Validate only allowed attributes are present
    // Per 3MF Core spec: v1, v2, v3, pid
    // Per Materials Extension: pindex (for entire triangle), p1, p2, p3 (for per-vertex properties)
    validate_attributes(
        &attrs,
        &["v1", "v2", "v3", "pid", "pindex", "p1", "p2", "p3"],
        "triangle",
    )?;

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

    if let Some(pindex) = attrs.get("pindex") {
        triangle.pindex = Some(pindex.parse::<usize>()?);
    }

    if let Some(p1) = attrs.get("p1") {
        triangle.p1 = Some(p1.parse::<usize>()?);
    }

    if let Some(p2) = attrs.get("p2") {
        triangle.p2 = Some(p2.parse::<usize>()?);
    }

    if let Some(p3) = attrs.get("p3") {
        triangle.p3 = Some(p3.parse::<usize>()?);
    }

    Ok(triangle)
}

/// Parse build item element attributes
fn parse_build_item<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<BuildItem> {
    let attrs = parse_attributes(reader, e)?;

    // Validate only allowed attributes are present
    // Per 3MF Core spec: objectid, transform, partnumber
    validate_attributes(&attrs, &["objectid", "transform", "partnumber"], "item")?;

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

        // Transform must have exactly 12 values
        if values.len() != 12 {
            return Err(Error::InvalidXml(format!(
                "Transform matrix must have exactly 12 values (got {})",
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

    Ok(item)
}

/// Parse material (base) element attributes
/// Base materials within a basematerials group use sequential indices (0, 1, 2, ...)
fn parse_base_material<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
    index: usize,
) -> Result<Material> {
    let attrs = parse_attributes(reader, e)?;

    // Validate only allowed attributes are present
    // Per 3MF Core spec: name, displaycolor
    validate_attributes(&attrs, &["name", "displaycolor"], "base")?;

    // Use the provided index as the material ID
    let mut material = Material::new(index);
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

/// Parse required extensions from a space-separated list of namespace URIs
#[allow(dead_code)] // Kept for backward compatibility
fn parse_required_extensions(extensions_str: &str) -> Result<Vec<Extension>> {
    parse_required_extensions_with_namespaces(extensions_str, &HashMap::new())
}

/// Parse required extensions from a space-separated list that may contain prefixes or URIs
fn parse_required_extensions_with_namespaces(
    extensions_str: &str,
    namespaces: &HashMap<String, String>,
) -> Result<Vec<Extension>> {
    let mut extensions = Vec::new();

    for item in extensions_str.split_whitespace() {
        // Try to resolve it as a full URI first
        if let Some(ext) = Extension::from_namespace(item) {
            extensions.push(ext);
        } else if let Some(namespace_uri) = namespaces.get(item) {
            // It's a namespace prefix - resolve it to a URI
            if let Some(ext) = Extension::from_namespace(namespace_uri) {
                extensions.push(ext);
            }
            // Unknown URIs are silently ignored - we only track known extensions
        }
        // Unknown items (not a known URI or resolvable prefix) are silently ignored
    }

    Ok(extensions)
}

/// Validate that all required extensions are supported by the parser configuration
fn validate_extensions(required: &[Extension], config: &ParserConfig) -> Result<()> {
    for ext in required {
        if !config.supports(ext) {
            return Err(Error::UnsupportedExtension(format!(
                "Extension '{}' (namespace: {}) is required but not supported",
                ext.name(),
                ext.namespace()
            )));
        }
    }
    Ok(())
}

/// Parse attributes from an XML element
fn parse_attributes<R: std::io::BufRead>(
    _reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<HashMap<String, String>> {
    let mut attrs = HashMap::new();

    for attr in e.attributes() {
        let attr = attr?;
        let key =
            std::str::from_utf8(attr.key.as_ref()).map_err(|e| Error::InvalidXml(e.to_string()))?;
        let value =
            std::str::from_utf8(&attr.value).map_err(|e| Error::InvalidXml(e.to_string()))?;

        attrs.insert(key.to_string(), value.to_string());
    }

    Ok(attrs)
}

/// Check if an attribute key should be skipped during validation
///
/// Returns true for:
/// - XML namespace declarations (xmlns, xmlns:prefix)
/// - XML standard attribute xml:lang (xml:space is NOT allowed on 3MF elements)
/// - Extension-namespaced attributes (p:UUID, m:colorid, s:slicestackid, etc.)
fn should_skip_attribute(key: &str) -> bool {
    // Allow xmlns and extension attributes (with colons)
    // Allow xml:lang but NOT xml:space (xml:space is not allowed per 3MF spec)
    if key.starts_with("xmlns") {
        return true;
    }
    if key == "xml:lang" {
        return true;
    }
    // Allow extension attributes (contain colon but not xml:)
    if key.contains(':') && !key.starts_with("xml:") {
        return true;
    }
    false
}

/// Validate that all attributes in the map are in the allowed list
///
/// This function validates that only known/allowed attributes are present on an element,
/// while allowing extension-specific attributes to pass through.
///
/// # Skipped Attributes
/// - XML namespace attributes: `xmlns`, `xmlns:p`, `xmlns:m`, etc.
/// - XML standard attribute: `xml:lang` (note: `xml:space` is NOT allowed per 3MF spec)
/// - Extension attributes: `p:UUID`, `m:colorid`, `s:slicestackid`, etc.
///
/// # Examples
/// ```ignore
/// // These would be rejected:
/// validate_attributes(&attrs, &["id", "name"], "object")?;
/// // - attrs contains "thumbnail" -> Error
/// // - attrs contains "foo" -> Error
///
/// // These would pass:
/// // - attrs contains "id", "name", "p:UUID" -> OK (p:UUID skipped as extension attr)
/// // - attrs contains "id", "xmlns:p" -> OK (xmlns:p skipped as namespace)
/// ```
fn validate_attributes(
    attrs: &HashMap<String, String>,
    allowed: &[&str],
    element_name: &str,
) -> Result<()> {
    use std::collections::HashSet;
    let allowed_set: HashSet<&str> = allowed.iter().copied().collect();

    for key in attrs.keys() {
        // Skip namespace and extension attributes
        if should_skip_attribute(key) {
            continue;
        }

        if !allowed_set.contains(key.as_str()) {
            return Err(Error::InvalidXml(format!(
                "Unknown attribute '{}' on <{}>",
                key, element_name
            )));
        }
    }
    Ok(())
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
    <object id="1">
      <mesh>
        <vertices>
          <vertex x="0" y="0" z="0"/>
          <vertex x="1" y="0" z="0"/>
          <vertex x="0" y="1" z="0"/>
        </vertices>
        <triangles>
          <triangle v1="0" v2="1" v3="2"/>
        </triangles>
      </mesh>
    </object>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"#;

        let model = parse_model_xml(xml).unwrap();
        assert_eq!(model.unit, "millimeter");
        assert_eq!(
            model.xmlns,
            "http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
        );
        assert_eq!(model.resources.objects.len(), 1);
        assert_eq!(model.build.items.len(), 1);
    }
}
