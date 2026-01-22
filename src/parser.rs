//! XML parsing for 3MF model files

use crate::error::{Error, Result};
use crate::model::*;
use crate::opc::Package;
use crate::validator;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::io::Read;

/// Size of 3MF transformation matrix (4x3 affine transform in row-major order)
const TRANSFORM_MATRIX_SIZE: usize = 12;

/// Default buffer capacity for XML parsing (4KB)
const XML_BUFFER_CAPACITY: usize = 4096;

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

    // Extract thumbnail metadata
    let thumbnail = package.get_thumbnail_metadata()?;

    // Clone config before moving it to parse_model_xml_with_config
    let config_clone = config.clone();
    let model_xml = package.get_model()?;
    let mut model = parse_model_xml_with_config(&model_xml, config)?;

    // Add thumbnail metadata to model
    model.thumbnail = thumbnail;

    // Load keystore to identify encrypted files (SecureContent extension)
    // This MUST happen before validation so that component validation can
    // skip components referencing encrypted files
    load_keystore(&mut package, &mut model)?;

    // Load external slice files if any slice stacks have references
    load_slice_references(&mut package, &mut model)?;

    // Validate boolean operation external paths before general validation
    // This requires access to the package to check if referenced files exist
    validate_boolean_external_paths(&mut package, &model)?;

    // Validate the model AFTER loading keystore and slices
    // This ensures validation can check encrypted file references correctly
    validator::validate_model_with_config(&model, &config_clone)?;

    Ok(model)
}

/// Read thumbnail binary data from a 3MF file
///
/// Returns the thumbnail image data as a byte vector if a thumbnail is present.
/// Returns None if no thumbnail is found.
///
/// # Arguments
///
/// * `reader` - A reader containing the 3MF file data
///
/// # Example
///
/// ```no_run
/// use lib3mf::parser::read_thumbnail;
/// use std::fs::File;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let file = File::open("model.3mf")?;
/// if let Some(thumbnail_data) = read_thumbnail(file)? {
///     println!("Thumbnail size: {} bytes", thumbnail_data.len());
///     // Save to file or process the image
///     std::fs::write("thumbnail.png", thumbnail_data)?;
/// }
/// # Ok(())
/// # }
/// ```
pub fn read_thumbnail<R: Read + std::io::Seek>(reader: R) -> Result<Option<Vec<u8>>> {
    let mut package = Package::open(reader)?;

    // Get thumbnail metadata
    let thumbnail = package.get_thumbnail_metadata()?;

    match thumbnail {
        Some(thumb) => {
            // Read the binary data
            let data = package.get_file_binary(&thumb.path)?;
            Ok(Some(data))
        }
        None => Ok(None),
    }
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
pub(crate) fn get_local_name(name_str: &str) -> &str {
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
    // Pre-allocate buffer with reasonable capacity to reduce allocations
    let mut buf = Vec::with_capacity(XML_BUFFER_CAPACITY);
    let mut in_resources = false;
    let mut in_build = false;
    let mut current_object: Option<Object> = None;
    let mut current_mesh: Option<Mesh> = None;
    let mut in_basematerials = false;
    let mut material_index: usize = 0;
    let mut current_basematerialgroup: Option<BaseMaterialGroup> = None;
    let mut current_colorgroup: Option<ColorGroup> = None;
    let mut in_colorgroup = false;
    let mut current_beamset: Option<BeamSet> = None;
    let mut in_beamset = false;
    let mut current_slicestack: Option<SliceStack> = None;
    let mut in_slicestack = false;
    let mut current_slice: Option<Slice> = None;
    let mut in_slice = false;
    let mut current_slice_polygon: Option<SlicePolygon> = None;
    let mut in_slice_polygon = false;
    let mut in_slice_vertices = false;
    let mut current_boolean_shape: Option<BooleanShape> = None;
    let mut in_boolean_shape = false;

    // Displacement extension state
    let mut current_normvectorgroup: Option<NormVectorGroup> = None;
    let mut in_normvectorgroup = false;
    let mut current_disp2dgroup: Option<Disp2DGroup> = None;
    let mut in_disp2dgroup = false;

    // Materials extension state for advanced features
    let mut current_texture2dgroup: Option<Texture2DGroup> = None;
    let mut in_texture2dgroup = false;
    let mut current_compositematerials: Option<CompositeMaterials> = None;
    let mut in_compositematerials = false;
    let mut current_multiproperties: Option<MultiProperties> = None;
    let mut in_multiproperties = false;

    // Component state
    let mut in_components = false;

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
                                "recommendedextensions" => {
                                    // Per 3MF spec, recommendedextensions are optional and may be ignored
                                    // They suggest extensions that enhance user experience but are not required
                                    // We allow the attribute for spec compliance but don't store it
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
                        // Per 3MF Core spec: unit, xml:lang, requiredextensions, recommendedextensions, and xmlns declarations
                        // Note: thumbnail is deprecated in v1.4+ but still allowed for backward compatibility
                        validate_attributes(
                            &all_attrs,
                            &[
                                "unit",
                                "xml:lang",
                                "requiredextensions",
                                "recommendedextensions",
                                "xmlns",
                                "thumbnail",
                            ],
                            "model",
                        )?;

                        // Now parse required extensions with namespace context
                        if let Some(ext_value) = required_ext_value {
                            let (extensions, custom_extensions) =
                                parse_required_extensions_with_namespaces(&ext_value, &namespaces)?;
                            model.required_extensions = extensions;
                            model.required_custom_extensions = custom_extensions;
                            // Validate that all required extensions are supported
                            validate_extensions(
                                &model.required_extensions,
                                &model.required_custom_extensions,
                                &config,
                            )?;
                        }
                    }
                    "metadata" => {
                        let attrs = parse_attributes(&reader, e)?;

                        // Validate that name attribute exists (required by 3MF Core spec)
                        let name = attrs.get("name").ok_or_else(|| {
                            Error::InvalidXml(
                                "Metadata element must have a 'name' attribute".to_string(),
                            )
                        })?;

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

                        // Parse optional preserve attribute
                        let preserve = if let Some(preserve_str) = attrs.get("preserve") {
                            // Validate preserve attribute value
                            match preserve_str.as_str() {
                                "0" | "false" => Some(false),
                                "1" | "true" => Some(true),
                                _ => {
                                    return Err(Error::InvalidXml(format!(
                                        "Invalid preserve attribute value '{}'. Must be '0', '1', 'false', or 'true'",
                                        preserve_str
                                    )));
                                }
                            }
                        } else {
                            None
                        };

                        // Read the text content
                        if let Ok(Event::Text(t)) = reader.read_event_into(&mut buf) {
                            let value =
                                t.unescape().map_err(|e| Error::InvalidXml(e.to_string()))?;

                            // Check for duplicate metadata names
                            // Per 3MF Core spec: metadata element names must be unique
                            if model.has_metadata(name) {
                                return Err(Error::InvalidXml(format!(
                                    "Duplicate metadata name '{}'. Each metadata element must have a unique name attribute",
                                    name
                                )));
                            }

                            // Create metadata entry with preserve flag if present
                            let entry = if let Some(preserve_val) = preserve {
                                crate::model::MetadataEntry::new_with_preserve(
                                    name.clone(),
                                    value.to_string(),
                                    preserve_val,
                                )
                            } else {
                                crate::model::MetadataEntry::new(name.clone(), value.to_string())
                            };

                            model.metadata.push(entry);
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

                        // Extract and validate build element attributes
                        let attrs = parse_attributes(&reader, e)?;
                        // Per 3MF Core spec: build element has no standard attributes
                        // Only extension attributes (like p:UUID) are allowed
                        validate_attributes(&attrs, &[], "build")?;

                        // Extract Production extension UUID (p:UUID) from build element
                        if let Some(p_uuid) = attrs.get("p:UUID") {
                            model.build.production_uuid = Some(p_uuid.clone());
                        }
                    }
                    "object" if in_resources => {
                        current_object = Some(parse_object(&reader, e)?);
                    }
                    "mesh" if in_resources && current_object.is_some() => {
                        current_mesh = Some(Mesh::new());
                    }
                    "components" if in_resources && current_object.is_some() => {
                        in_components = true;
                    }
                    "component" if in_components => {
                        if let Some(ref mut obj) = current_object {
                            let component = parse_component(&reader, e)?;
                            obj.components.push(component);
                        }
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
                        let attrs = parse_attributes(&reader, e)?;
                        let id = attrs
                            .get("id")
                            .ok_or_else(|| Error::missing_attribute("basematerials", "id"))?
                            .parse::<usize>()?;
                        current_basematerialgroup = Some(BaseMaterialGroup::new(id));
                    }
                    "base" if in_basematerials => {
                        // Materials within basematerials group
                        if let Some(ref mut group) = current_basematerialgroup {
                            let attrs = parse_attributes(&reader, e)?;

                            // Validate only allowed attributes are present
                            // Per 3MF Materials & Properties Extension spec: name, displaycolor
                            validate_attributes(&attrs, &["name", "displaycolor"], "base")?;

                            let name = attrs.get("name").cloned().unwrap_or_default();

                            // Parse displaycolor attribute (format: #RRGGBBAA or #RRGGBB)
                            // If displaycolor is missing or invalid, use white as default
                            let displaycolor = if let Some(color_str) = attrs.get("displaycolor") {
                                parse_color(color_str).unwrap_or((255, 255, 255, 255))
                            } else {
                                (255, 255, 255, 255)
                            };

                            group.materials.push(BaseMaterial::new(name, displaycolor));
                        }

                        // Still parse to materials list for backward compatibility
                        let material = parse_base_material(&reader, e, material_index)?;
                        model.resources.materials.push(material);
                        material_index += 1;
                    }
                    "colorgroup" if in_resources => {
                        in_colorgroup = true;
                        let attrs = parse_attributes(&reader, e)?;
                        let id = attrs
                            .get("id")
                            .ok_or_else(|| Error::missing_attribute("colorgroup", "id"))?
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
                    "texture2d" if in_resources => {
                        let attrs = parse_attributes(&reader, e)?;
                        let id = attrs
                            .get("id")
                            .ok_or_else(|| Error::missing_attribute("texture2d", "id"))?
                            .parse::<usize>()?;
                        let path = attrs
                            .get("path")
                            .ok_or_else(|| Error::missing_attribute("texture2d", "path"))?
                            .to_string();
                        let contenttype = attrs
                            .get("contenttype")
                            .ok_or_else(|| {
                                Error::InvalidXml(
                                    "texture2d missing contenttype attribute".to_string(),
                                )
                            })?
                            .to_string();

                        let mut texture = Texture2D::new(id, path, contenttype);

                        // Parse optional attributes with spec defaults
                        if let Some(tileu_str) = attrs.get("tilestyleu") {
                            texture.tilestyleu = match tileu_str.to_lowercase().as_str() {
                                "wrap" => TileStyle::Wrap,
                                "mirror" => TileStyle::Mirror,
                                "clamp" => TileStyle::Clamp,
                                "none" => TileStyle::None,
                                _ => TileStyle::Wrap,
                            };
                        }

                        if let Some(tilev_str) = attrs.get("tilestylev") {
                            texture.tilestylev = match tilev_str.to_lowercase().as_str() {
                                "wrap" => TileStyle::Wrap,
                                "mirror" => TileStyle::Mirror,
                                "clamp" => TileStyle::Clamp,
                                "none" => TileStyle::None,
                                _ => TileStyle::Wrap,
                            };
                        }

                        if let Some(filter_str) = attrs.get("filter") {
                            texture.filter = match filter_str.to_lowercase().as_str() {
                                "auto" => FilterMode::Auto,
                                "linear" => FilterMode::Linear,
                                "nearest" => FilterMode::Nearest,
                                _ => FilterMode::Auto,
                            };
                        }

                        model.resources.texture2d_resources.push(texture);
                    }
                    "texture2dgroup" if in_resources => {
                        in_texture2dgroup = true;
                        let attrs = parse_attributes(&reader, e)?;
                        let id = attrs
                            .get("id")
                            .ok_or_else(|| Error::missing_attribute("texture2dgroup", "id"))?
                            .parse::<usize>()?;
                        let texid = attrs
                            .get("texid")
                            .ok_or_else(|| Error::missing_attribute("texture2dgroup", "texid"))?
                            .parse::<usize>()?;
                        current_texture2dgroup = Some(Texture2DGroup::new(id, texid));
                    }
                    "tex2coord" if in_texture2dgroup => {
                        if let Some(ref mut group) = current_texture2dgroup {
                            let attrs = parse_attributes(&reader, e)?;
                            let u = attrs
                                .get("u")
                                .ok_or_else(|| Error::missing_attribute("tex2coord", "u"))?
                                .parse::<f32>()?;
                            let v = attrs
                                .get("v")
                                .ok_or_else(|| Error::missing_attribute("tex2coord", "v"))?
                                .parse::<f32>()?;
                            group.tex2coords.push(Tex2Coord::new(u, v));
                        }
                    }
                    "compositematerials" if in_resources => {
                        in_compositematerials = true;
                        let attrs = parse_attributes(&reader, e)?;
                        let id = attrs
                            .get("id")
                            .ok_or_else(|| {
                                Error::InvalidXml(
                                    "compositematerials missing id attribute".to_string(),
                                )
                            })?
                            .parse::<usize>()?;
                        let matid = attrs
                            .get("matid")
                            .ok_or_else(|| {
                                Error::InvalidXml(
                                    "compositematerials missing matid attribute".to_string(),
                                )
                            })?
                            .parse::<usize>()?;
                        let matindices_str = attrs.get("matindices").ok_or_else(|| {
                            Error::InvalidXml(
                                "compositematerials missing matindices attribute".to_string(),
                            )
                        })?;
                        let matindices: Vec<usize> = matindices_str
                            .split_whitespace()
                            .filter_map(|s| s.parse::<usize>().ok())
                            .collect();

                        // Validate we parsed at least one index
                        if matindices.is_empty() {
                            return Err(Error::InvalidXml(
                                "compositematerials matindices must contain at least one valid index"
                                    .to_string(),
                            ));
                        }

                        current_compositematerials =
                            Some(CompositeMaterials::new(id, matid, matindices));
                    }
                    "composite" if in_compositematerials => {
                        if let Some(ref mut group) = current_compositematerials {
                            let attrs = parse_attributes(&reader, e)?;
                            let values_str = attrs.get("values").ok_or_else(|| {
                                Error::InvalidXml("composite missing values attribute".to_string())
                            })?;
                            let values: Vec<f32> = values_str
                                .split_whitespace()
                                .filter_map(|s| s.parse::<f32>().ok())
                                .collect();

                            // Validate we parsed at least one value
                            if values.is_empty() {
                                return Err(Error::InvalidXml(
                                    "composite values must contain at least one valid number"
                                        .to_string(),
                                ));
                            }

                            group.composites.push(Composite::new(values));
                        }
                    }
                    "multiproperties" if in_resources => {
                        in_multiproperties = true;
                        let attrs = parse_attributes(&reader, e)?;
                        let id = attrs
                            .get("id")
                            .ok_or_else(|| {
                                Error::InvalidXml(
                                    "multiproperties missing id attribute".to_string(),
                                )
                            })?
                            .parse::<usize>()?;
                        let pids_str = attrs.get("pids").ok_or_else(|| {
                            Error::InvalidXml("multiproperties missing pids attribute".to_string())
                        })?;
                        let pids: Vec<usize> = pids_str
                            .split_whitespace()
                            .filter_map(|s| s.parse::<usize>().ok())
                            .collect();

                        // Validate we parsed at least one property ID
                        if pids.is_empty() {
                            return Err(Error::InvalidXml(
                                "multiproperties pids must contain at least one valid ID"
                                    .to_string(),
                            ));
                        }

                        let mut multi = MultiProperties::new(id, pids);

                        // Parse optional blendmethods
                        if let Some(blend_str) = attrs.get("blendmethods") {
                            multi.blendmethods = blend_str
                                .split_whitespace()
                                .filter_map(|s| match s.to_lowercase().as_str() {
                                    "mix" => Some(BlendMethod::Mix),
                                    "multiply" => Some(BlendMethod::Multiply),
                                    _ => None,
                                })
                                .collect();
                        }

                        current_multiproperties = Some(multi);
                    }
                    "multi" if in_multiproperties => {
                        if let Some(ref mut group) = current_multiproperties {
                            let attrs = parse_attributes(&reader, e)?;
                            let pindices_str = attrs.get("pindices").ok_or_else(|| {
                                Error::InvalidXml("multi missing pindices attribute".to_string())
                            })?;
                            let pindices: Vec<usize> = pindices_str
                                .split_whitespace()
                                .filter_map(|s| s.parse::<usize>().ok())
                                .collect();

                            // Note: Empty pindices is allowed per spec - defaults to 0
                            // But if there's text that all failed to parse, that's an error
                            if !pindices_str.trim().is_empty() && pindices.is_empty() {
                                return Err(Error::InvalidXml(
                                    "multi pindices contains invalid values that could not be parsed"
                                        .to_string(),
                                ));
                            }

                            group.multis.push(Multi::new(pindices));
                        }
                    }
                    "displacement2d" if in_resources => {
                        let attrs = parse_attributes(&reader, e)?;
                        let id = attrs
                            .get("id")
                            .ok_or_else(|| {
                                Error::InvalidXml("displacement2d missing id attribute".to_string())
                            })?
                            .parse::<usize>()?;
                        let path = attrs
                            .get("path")
                            .ok_or_else(|| {
                                Error::InvalidXml(
                                    "displacement2d missing path attribute".to_string(),
                                )
                            })?
                            .to_string();

                        let mut disp = Displacement2D::new(id, path);

                        // Parse optional attributes with spec-defined defaults
                        // If attribute value is invalid, fall back to spec default (lenient parsing)
                        if let Some(channel_str) = attrs.get("channel") {
                            disp.channel = match channel_str.to_uppercase().as_str() {
                                "R" => Channel::R,
                                "G" => Channel::G,
                                "B" => Channel::B,
                                "A" => Channel::A,
                                _ => Channel::G, // Spec default is 'G', fall back on invalid value
                            };
                        }

                        if let Some(tileu_str) = attrs.get("tilestyleu") {
                            disp.tilestyleu = match tileu_str.to_lowercase().as_str() {
                                "wrap" => TileStyle::Wrap,
                                "mirror" => TileStyle::Mirror,
                                "clamp" => TileStyle::Clamp,
                                "none" => TileStyle::None,
                                _ => TileStyle::Wrap, // Spec default is 'wrap', fall back on invalid value
                            };
                        }

                        if let Some(tilev_str) = attrs.get("tilestylev") {
                            disp.tilestylev = match tilev_str.to_lowercase().as_str() {
                                "wrap" => TileStyle::Wrap,
                                "mirror" => TileStyle::Mirror,
                                "clamp" => TileStyle::Clamp,
                                "none" => TileStyle::None,
                                _ => TileStyle::Wrap, // Spec default is 'wrap', fall back on invalid value
                            };
                        }

                        if let Some(filter_str) = attrs.get("filter") {
                            disp.filter = match filter_str.to_lowercase().as_str() {
                                "auto" => FilterMode::Auto,
                                "linear" => FilterMode::Linear,
                                "nearest" => FilterMode::Nearest,
                                _ => FilterMode::Auto, // Spec default is 'auto', fall back on invalid value
                            };
                        }

                        model.resources.displacement_maps.push(disp);
                    }
                    "normvectorgroup" if in_resources => {
                        in_normvectorgroup = true;
                        let attrs = parse_attributes(&reader, e)?;
                        let id = attrs
                            .get("id")
                            .ok_or_else(|| {
                                Error::InvalidXml(
                                    "normvectorgroup missing id attribute".to_string(),
                                )
                            })?
                            .parse::<usize>()?;
                        current_normvectorgroup = Some(NormVectorGroup::new(id));
                    }
                    "normvector" if in_normvectorgroup => {
                        if let Some(ref mut nvgroup) = current_normvectorgroup {
                            let attrs = parse_attributes(&reader, e)?;
                            let x = attrs
                                .get("x")
                                .ok_or_else(|| {
                                    Error::InvalidXml("normvector missing x attribute".to_string())
                                })?
                                .parse::<f64>()?;
                            let y = attrs
                                .get("y")
                                .ok_or_else(|| {
                                    Error::InvalidXml("normvector missing y attribute".to_string())
                                })?
                                .parse::<f64>()?;
                            let z = attrs
                                .get("z")
                                .ok_or_else(|| {
                                    Error::InvalidXml("normvector missing z attribute".to_string())
                                })?
                                .parse::<f64>()?;
                            nvgroup.vectors.push(NormVector::new(x, y, z));
                        }
                    }
                    "beamlattice" if current_mesh.is_some() => {
                        in_beamset = true;
                        let attrs = parse_attributes(&reader, e)?;
                        let mut beamset = BeamSet::new();

                        // Parse radius attribute (default 1.0)
                        if let Some(radius_str) = attrs.get("radius") {
                            let radius = radius_str.parse::<f64>()?;
                            // Validate radius is finite and positive
                            if !radius.is_finite() || radius <= 0.0 {
                                return Err(Error::InvalidXml(format!(
                                    "BeamLattice radius must be positive and finite (got {})",
                                    radius
                                )));
                            }
                            beamset.radius = radius;
                        }

                        // Parse minlength attribute (default 0.0001)
                        if let Some(minlength_str) = attrs.get("minlength") {
                            let minlength = minlength_str.parse::<f64>()?;
                            // Validate minlength is finite and non-negative
                            if !minlength.is_finite() || minlength < 0.0 {
                                return Err(Error::InvalidXml(format!(
                                    "BeamLattice minlength must be non-negative and finite (got {})",
                                    minlength
                                )));
                            }
                            beamset.min_length = minlength;
                        }

                        // Parse cap mode attribute (default sphere)
                        if let Some(cap_str) = attrs.get("cap") {
                            beamset.cap_mode = cap_str.parse()?;
                        }

                        current_beamset = Some(beamset);
                    }
                    "beams" if in_beamset => {
                        // Beams container element - beams will be parsed as individual beam elements
                    }
                    "beam" if in_beamset => {
                        if let Some(ref mut beamset) = current_beamset {
                            let beam = parse_beam(&reader, e)?;
                            beamset.beams.push(beam);
                        }
                    }
                    "normvectorgroup" if in_resources => {
                        in_normvectorgroup = true;
                        let attrs = parse_attributes(&reader, e)?;
                        let id = attrs
                            .get("id")
                            .ok_or_else(|| {
                                Error::InvalidXml(
                                    "normvectorgroup missing id attribute".to_string(),
                                )
                            })?
                            .parse::<usize>()?;
                        current_normvectorgroup = Some(NormVectorGroup::new(id));
                    }
                    "normvector" if in_normvectorgroup => {
                        if let Some(ref mut nvgroup) = current_normvectorgroup {
                            let attrs = parse_attributes(&reader, e)?;
                            let x = attrs
                                .get("x")
                                .ok_or_else(|| {
                                    Error::InvalidXml("normvector missing x attribute".to_string())
                                })?
                                .parse::<f64>()?;
                            let y = attrs
                                .get("y")
                                .ok_or_else(|| {
                                    Error::InvalidXml("normvector missing y attribute".to_string())
                                })?
                                .parse::<f64>()?;
                            let z = attrs
                                .get("z")
                                .ok_or_else(|| {
                                    Error::InvalidXml("normvector missing z attribute".to_string())
                                })?
                                .parse::<f64>()?;
                            nvgroup.vectors.push(NormVector::new(x, y, z));
                        }
                    }
                    "disp2dgroup" if in_resources => {
                        in_disp2dgroup = true;
                        let attrs = parse_attributes(&reader, e)?;
                        let id = attrs
                            .get("id")
                            .ok_or_else(|| {
                                Error::InvalidXml("disp2dgroup missing id attribute".to_string())
                            })?
                            .parse::<usize>()?;
                        let dispid = attrs
                            .get("dispid")
                            .ok_or_else(|| {
                                Error::InvalidXml(
                                    "disp2dgroup missing dispid attribute".to_string(),
                                )
                            })?
                            .parse::<usize>()?;
                        let nid = attrs
                            .get("nid")
                            .ok_or_else(|| {
                                Error::InvalidXml("disp2dgroup missing nid attribute".to_string())
                            })?
                            .parse::<usize>()?;
                        let height = attrs
                            .get("height")
                            .ok_or_else(|| {
                                Error::InvalidXml(
                                    "disp2dgroup missing height attribute".to_string(),
                                )
                            })?
                            .parse::<f64>()?;

                        let mut disp2dgroup = Disp2DGroup::new(id, dispid, nid, height);

                        // Parse optional offset
                        if let Some(offset_str) = attrs.get("offset") {
                            disp2dgroup.offset = offset_str.parse::<f64>()?;
                        }

                        current_disp2dgroup = Some(disp2dgroup);
                    }
                    "disp2dcoords" if in_disp2dgroup => {
                        if let Some(ref mut d2dgroup) = current_disp2dgroup {
                            let attrs = parse_attributes(&reader, e)?;
                            let u = attrs
                                .get("u")
                                .ok_or_else(|| {
                                    Error::InvalidXml(
                                        "disp2dcoords missing u attribute".to_string(),
                                    )
                                })?
                                .parse::<f64>()?;
                            let v = attrs
                                .get("v")
                                .ok_or_else(|| {
                                    Error::InvalidXml(
                                        "disp2dcoords missing v attribute".to_string(),
                                    )
                                })?
                                .parse::<f64>()?;
                            let n = attrs
                                .get("n")
                                .ok_or_else(|| {
                                    Error::InvalidXml(
                                        "disp2dcoords missing n attribute".to_string(),
                                    )
                                })?
                                .parse::<usize>()?;

                            let mut coords = Disp2DCoords::new(u, v, n);

                            // Parse optional f attribute
                            if let Some(f_str) = attrs.get("f") {
                                coords.f = f_str.parse::<f64>()?;
                            }

                            d2dgroup.coords.push(coords);
                        }
                    }
                    "slicestack" if in_resources => {
                        in_slicestack = true;
                        let attrs = parse_attributes(&reader, e)?;
                        let id = attrs
                            .get("id")
                            .ok_or_else(|| {
                                Error::InvalidXml("SliceStack missing id attribute".to_string())
                            })?
                            .parse::<usize>()?;
                        let zbottom = attrs
                            .get("zbottom")
                            .ok_or_else(|| {
                                Error::InvalidXml(
                                    "SliceStack missing zbottom attribute".to_string(),
                                )
                            })?
                            .parse::<f64>()?;
                        current_slicestack = Some(SliceStack::new(id, zbottom));
                    }
                    "slice" if in_slicestack => {
                        in_slice = true;
                        let attrs = parse_attributes(&reader, e)?;
                        let ztop = attrs
                            .get("ztop")
                            .ok_or_else(|| {
                                Error::InvalidXml("Slice missing ztop attribute".to_string())
                            })?
                            .parse::<f64>()?;
                        current_slice = Some(Slice::new(ztop));
                    }
                    "sliceref" if in_slicestack => {
                        let attrs = parse_attributes(&reader, e)?;
                        let slicestackid = attrs
                            .get("slicestackid")
                            .ok_or_else(|| {
                                Error::InvalidXml(
                                    "SliceRef missing slicestackid attribute".to_string(),
                                )
                            })?
                            .parse::<usize>()?;
                        let slicepath = attrs
                            .get("slicepath")
                            .ok_or_else(|| {
                                Error::InvalidXml(
                                    "SliceRef missing slicepath attribute".to_string(),
                                )
                            })?
                            .to_string();
                        if let Some(ref mut slicestack) = current_slicestack {
                            slicestack
                                .slice_refs
                                .push(SliceRef::new(slicestackid, slicepath));
                        }
                    }
                    "vertices" if in_slice => {
                        in_slice_vertices = true;
                    }
                    "vertex" if in_slice_vertices => {
                        let attrs = parse_attributes(&reader, e)?;
                        let x = attrs
                            .get("x")
                            .ok_or_else(|| {
                                Error::InvalidXml("Slice vertex missing x attribute".to_string())
                            })?
                            .parse::<f64>()?;
                        let y = attrs
                            .get("y")
                            .ok_or_else(|| {
                                Error::InvalidXml("Slice vertex missing y attribute".to_string())
                            })?
                            .parse::<f64>()?;
                        if let Some(ref mut slice) = current_slice {
                            slice.vertices.push(Vertex2D::new(x, y));
                        }
                    }
                    "polygon" if in_slice => {
                        in_slice_polygon = true;
                        let attrs = parse_attributes(&reader, e)?;
                        let startv = attrs
                            .get("startv")
                            .ok_or_else(|| {
                                Error::InvalidXml(
                                    "Slice polygon missing startv attribute".to_string(),
                                )
                            })?
                            .parse::<usize>()?;
                        current_slice_polygon = Some(SlicePolygon::new(startv));
                    }
                    "segment" if in_slice_polygon => {
                        let attrs = parse_attributes(&reader, e)?;
                        let v2 = attrs
                            .get("v2")
                            .ok_or_else(|| {
                                Error::InvalidXml("Slice segment missing v2 attribute".to_string())
                            })?
                            .parse::<usize>()?;
                        if let Some(ref mut polygon) = current_slice_polygon {
                            polygon.segments.push(SliceSegment::new(v2));
                        }
                    }
                    "booleanshape" if in_resources && current_object.is_some() => {
                        // Check if object already has a booleanshape
                        // Per 3MF Boolean Operations spec, an object can only have one booleanshape
                        // We use the in_boolean_shape flag since the object is still being built
                        if in_boolean_shape {
                            return Err(Error::InvalidXml(
                                "Object can only have one booleanshape element".to_string(),
                            ));
                        }
                        let attrs = parse_attributes(&reader, e)?;
                        let objectid = attrs
                            .get("objectid")
                            .ok_or_else(|| {
                                Error::InvalidXml(
                                    "Boolean shape missing objectid attribute".to_string(),
                                )
                            })?
                            .parse::<usize>()?;
                        // Operation defaults to "union" if not specified
                        let operation = attrs
                            .get("operation")
                            .and_then(|s| BooleanOpType::parse(s))
                            .unwrap_or(BooleanOpType::Union);
                        let mut shape = BooleanShape::new(objectid, operation);
                        // Extract optional path attribute for external object reference
                        shape.path = attrs.get("path").cloned();
                        current_boolean_shape = Some(shape);
                        in_boolean_shape = true;
                    }
                    "boolean" if in_boolean_shape => {
                        let attrs = parse_attributes(&reader, e)?;
                        let objectid = attrs
                            .get("objectid")
                            .ok_or_else(|| {
                                Error::InvalidXml(
                                    "Boolean operand missing objectid attribute".to_string(),
                                )
                            })?
                            .parse::<usize>()?;
                        if let Some(ref mut shape) = current_boolean_shape {
                            let mut operand = BooleanRef::new(objectid);
                            // Extract optional path attribute for external object reference
                            operand.path = attrs.get("path").cloned();
                            shape.operands.push(operand);
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
                    "components" => {
                        in_components = false;
                    }
                    "basematerials" => {
                        if let Some(group) = current_basematerialgroup.take() {
                            model.resources.base_material_groups.push(group);
                        }
                        in_basematerials = false;
                    }
                    "colorgroup" => {
                        if let Some(colorgroup) = current_colorgroup.take() {
                            model.resources.color_groups.push(colorgroup);
                        }
                        in_colorgroup = false;
                    }
                    "texture2dgroup" => {
                        if let Some(group) = current_texture2dgroup.take() {
                            model.resources.texture2d_groups.push(group);
                        }
                        in_texture2dgroup = false;
                    }
                    "compositematerials" => {
                        if let Some(group) = current_compositematerials.take() {
                            model.resources.composite_materials.push(group);
                        }
                        in_compositematerials = false;
                    }
                    "multiproperties" => {
                        if let Some(group) = current_multiproperties.take() {
                            model.resources.multi_properties.push(group);
                        }
                        in_multiproperties = false;
                    }
                    "normvectorgroup" => {
                        if let Some(nvgroup) = current_normvectorgroup.take() {
                            model.resources.norm_vector_groups.push(nvgroup);
                        }
                        in_normvectorgroup = false;
                    }
                    "disp2dgroup" => {
                        if let Some(d2dgroup) = current_disp2dgroup.take() {
                            model.resources.disp2d_groups.push(d2dgroup);
                        }
                        in_disp2dgroup = false;
                    }
                    "beamlattice" => {
                        if let Some(beamset) = current_beamset.take() {
                            if let Some(ref mut mesh) = current_mesh {
                                mesh.beamset = Some(beamset);
                            }
                        }
                        in_beamset = false;
                    }
                    "slicestack" => {
                        if let Some(slicestack) = current_slicestack.take() {
                            model.resources.slice_stacks.push(slicestack);
                        }
                        in_slicestack = false;
                    }
                    "slice" => {
                        if let Some(slice) = current_slice.take() {
                            if let Some(ref mut slicestack) = current_slicestack {
                                slicestack.slices.push(slice);
                            }
                        }
                        in_slice = false;
                    }
                    "vertices" => {
                        in_slice_vertices = false;
                    }
                    "polygon" => {
                        if let Some(polygon) = current_slice_polygon.take() {
                            if let Some(ref mut slice) = current_slice {
                                slice.polygons.push(polygon);
                            }
                        }
                        in_slice_polygon = false;
                    }
                    "booleanshape" => {
                        if let Some(shape) = current_boolean_shape.take() {
                            if let Some(ref mut obj) = current_object {
                                obj.boolean_shape = Some(shape);
                            }
                        }
                        in_boolean_shape = false;
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

    // Note: Model validation is performed in parse_3mf_with_config after
    // keystore and slice files are loaded
    Ok(model)
}

/// Load and parse Secure/keystore.xml to identify encrypted files
///
/// Extracts the keystore UUID and list of encrypted file paths from the
/// SecureContent keystore.xml file. If the file doesn't exist, this is not
/// an error (not all packages have encrypted content).
fn load_keystore<R: Read + std::io::Seek>(
    package: &mut Package<R>,
    model: &mut Model,
) -> Result<()> {
    // Try to load the keystore file - it's OK if it doesn't exist
    let keystore_xml = match package.get_file("Secure/keystore.xml") {
        Ok(xml) => xml,
        Err(_) => return Ok(()), // No keystore file, not an error
    };

    // Initialize secure_content if not already present
    if model.secure_content.is_none() {
        model.secure_content = Some(SecureContentInfo::default());
    }

    let mut reader = Reader::from_str(&keystore_xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::with_capacity(XML_BUFFER_CAPACITY);

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name = e.name();
                let name_str = std::str::from_utf8(name.as_ref())
                    .map_err(|e| Error::InvalidXml(e.to_string()))?;

                let local_name = get_local_name(name_str);

                match local_name {
                    "keystore" => {
                        // Extract UUID attribute from keystore element
                        for attr in e.attributes() {
                            let attr = attr.map_err(|e| {
                                Error::InvalidXml(format!("Invalid attribute in keystore: {}", e))
                            })?;
                            let attr_name = std::str::from_utf8(attr.key.as_ref())
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;
                            if attr_name == "UUID" {
                                let uuid = std::str::from_utf8(&attr.value)
                                    .map_err(|e| Error::InvalidXml(e.to_string()))?
                                    .to_string();
                                if let Some(ref mut sc) = model.secure_content {
                                    sc.keystore_uuid = Some(uuid);
                                }
                            }
                        }
                    }
                    "resourcedata" => {
                        // Extract path attribute from resourcedata element
                        for attr in e.attributes() {
                            let attr = attr.map_err(|e| {
                                Error::InvalidXml(format!(
                                    "Invalid attribute in resourcedata: {}",
                                    e
                                ))
                            })?;
                            let attr_name = std::str::from_utf8(attr.key.as_ref())
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;
                            if attr_name == "path" {
                                let path = std::str::from_utf8(&attr.value)
                                    .map_err(|e| Error::InvalidXml(e.to_string()))?
                                    .to_string();
                                if let Some(ref mut sc) = model.secure_content {
                                    sc.encrypted_files.push(path);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(Error::InvalidXml(format!(
                    "Error parsing keystore.xml: {}",
                    e
                )))
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(())
}

/// Load and parse external slice files referenced in slice stacks
///
/// Iterates through all slice stacks in the model and loads any external slice files
/// referenced via sliceref elements. The slices from the external files are merged
/// into the appropriate slice stacks. Additionally, any objects defined in the external
/// slice files are also merged into the main model's resources.
fn load_slice_references<R: Read + std::io::Seek>(
    package: &mut Package<R>,
    model: &mut Model,
) -> Result<()> {
    // Process each slice stack independently
    for slice_stack in &mut model.resources.slice_stacks {
        // Load slices from each referenced file
        for slice_ref in &slice_stack.slice_refs {
            // Normalize the path (remove leading slash if present)
            let normalized_path = if slice_ref.slicepath.starts_with('/') {
                &slice_ref.slicepath[1..]
            } else {
                &slice_ref.slicepath
            };

            // Skip loading encrypted slice files (Secure Content extension)
            // Encrypted files follow the pattern: *_encrypted.model (or other extension)
            // They cannot be decrypted by this library per the Secure Content spec.
            // Check if the filename (without directory path) contains "_encrypted"
            // followed by a file extension (e.g., "_encrypted.model")
            if let Some(filename) = normalized_path.rsplit('/').next() {
                if filename.contains("_encrypted.") {
                    // For encrypted slice files, we acknowledge they exist but can't load them
                    // The file structure is valid even if we can't decrypt the content
                    continue;
                }
            }

            // Load the slice file from the package
            let slice_xml = package.get_file(normalized_path).map_err(|e| {
                Error::InvalidXml(format!(
                    "Failed to load slice reference file '{}': {}",
                    slice_ref.slicepath, e
                ))
            })?;

            // Parse the slice file to extract slices and objects
            // Use the slicestackid from the sliceref, which identifies the stack ID in the external file
            let (slices, objects) =
                parse_slice_file_with_objects(&slice_xml, slice_ref.slicestackid)?;

            // Add the slices to this slice stack
            slice_stack.slices.extend(slices);

            // Merge objects from the external file into the main model
            model.resources.objects.extend(objects);
        }
    }

    Ok(())
}

/// Parse a slice model file and extract both slices and objects
///
/// This parses a referenced slice file (typically in the 2D/ directory) and
/// extracts all slice data including vertices, polygons, and segments, as well as
/// any object definitions that may be present in the file.
///
/// Note: External slice files may have empty or incomplete structures (e.g., empty
/// build sections), so we parse them and skip validation.
fn parse_slice_file_with_objects(
    xml: &str,
    expected_stack_id: usize,
) -> Result<(Vec<Slice>, Vec<Object>)> {
    // Parse the entire model XML
    // Note: We use all extensions here because external slice files are part of the same
    // 3MF package and should be parsed with the same extension support as the main model.
    // The 3MF spec requires that all files in a package share the same extension context.
    let mut external_model = parse_model_xml_with_config(xml, ParserConfig::with_all_extensions())?;

    // Find the slice stack with the expected ID and extract its slices
    let slices = external_model
        .resources
        .slice_stacks
        .iter_mut()
        .find(|stack| stack.id == expected_stack_id)
        .map(|stack| std::mem::take(&mut stack.slices))
        .unwrap_or_else(Vec::new);

    // Extract all objects from the external model
    let objects = std::mem::take(&mut external_model.resources.objects);

    Ok((slices, objects))
}

/// Parse object element attributes
pub(crate) fn parse_object<R: std::io::BufRead>(
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

    // Extract Production extension attributes (p:UUID, p:path)
    let p_uuid = attrs.get("p:UUID");
    let p_path = attrs.get("p:path");

    if p_uuid.is_some() || p_path.is_some() {
        let mut prod_info = ProductionInfo::new();
        prod_info.uuid = p_uuid.cloned();
        prod_info.path = p_path.cloned();
        object.production = Some(prod_info);
    }

    Ok(object)
}

/// Parse vertex element attributes
pub(crate) fn parse_vertex<R: std::io::BufRead>(
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
pub(crate) fn parse_triangle<R: std::io::BufRead>(
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
pub(crate) fn parse_build_item<R: std::io::BufRead>(
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

    // Extract Production extension UUID (p:UUID)
    if let Some(p_uuid) = attrs.get("p:UUID") {
        item.production_uuid = Some(p_uuid.clone());
    }

    // Extract Production extension path (p:path)
    if let Some(p_path) = attrs.get("p:path") {
        item.production_path = Some(p_path.clone());
    }

    Ok(item)
}

/// Parse component element attributes
fn parse_component<R: std::io::BufRead>(
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

    // Extract p:path attribute (Production extension)
    // This indicates the component references an object in an external model file
    if let Some(path_str) = attrs.get("p:path") {
        component.path = Some(path_str.clone());
    }
    // Extract Production extension attributes (p:UUID, p:path)
    let p_uuid = attrs.get("p:UUID");
    let p_path = attrs.get("p:path");

    if p_uuid.is_some() || p_path.is_some() {
        let mut prod_info = ProductionInfo::new();
        prod_info.uuid = p_uuid.cloned();
        prod_info.path = p_path.cloned();
        component.production = Some(prod_info);
    }

    Ok(component)
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

/// Parse beam element attributes
fn parse_beam<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<Beam> {
    let attrs = parse_attributes(reader, e)?;

    // Validate only allowed attributes are present
    // Per Beam Lattice Extension spec v1.2.0: v1, v2, r1, r2, cap1, cap2, p1, p2, pid
    // Currently implemented: v1, v2, r1, r2, cap1, cap2
    // TODO: Implement p1, p2, pid attributes for per-beam property overrides
    validate_attributes(
        &attrs,
        &["v1", "v2", "r1", "r2", "cap1", "cap2", "p1", "p2", "pid"],
        "beam",
    )?;

    let v1 = attrs
        .get("v1")
        .ok_or_else(|| Error::InvalidXml("Beam missing v1 attribute".to_string()))?
        .parse::<usize>()?;

    let v2 = attrs
        .get("v2")
        .ok_or_else(|| Error::InvalidXml("Beam missing v2 attribute".to_string()))?
        .parse::<usize>()?;

    let mut beam = Beam::new(v1, v2);

    if let Some(r1) = attrs.get("r1") {
        let r1_val = r1.parse::<f64>()?;
        // Validate radius is finite and positive
        if !r1_val.is_finite() || r1_val <= 0.0 {
            return Err(Error::InvalidXml(format!(
                "Beam r1 must be positive and finite (got {})",
                r1_val
            )));
        }
        beam.r1 = Some(r1_val);
    }

    if let Some(r2) = attrs.get("r2") {
        let r2_val = r2.parse::<f64>()?;
        // Validate radius is finite and positive
        if !r2_val.is_finite() || r2_val <= 0.0 {
            return Err(Error::InvalidXml(format!(
                "Beam r2 must be positive and finite (got {})",
                r2_val
            )));
        }
        beam.r2 = Some(r2_val);
    }

    // Parse cap1 attribute (optional, defaults to beamset cap mode)
    if let Some(cap1_str) = attrs.get("cap1") {
        beam.cap1 = Some(cap1_str.parse()?);
    }

    // Parse cap2 attribute (optional, defaults to beamset cap mode)
    if let Some(cap2_str) = attrs.get("cap2") {
        beam.cap2 = Some(cap2_str.parse()?);
    }

    Ok(beam)
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
fn parse_required_extensions(extensions_str: &str) -> Result<(Vec<Extension>, Vec<String>)> {
    parse_required_extensions_with_namespaces(extensions_str, &HashMap::new())
}

/// Parse required extensions from a space-separated list that may contain prefixes or URIs
/// Returns both known extensions and unknown namespace URIs (for custom extensions)
fn parse_required_extensions_with_namespaces(
    extensions_str: &str,
    namespaces: &HashMap<String, String>,
) -> Result<(Vec<Extension>, Vec<String>)> {
    let mut extensions = Vec::new();
    let mut custom_namespaces = Vec::new();

    for item in extensions_str.split_whitespace() {
        // Try to resolve it as a full URI first
        if let Some(ext) = Extension::from_namespace(item) {
            extensions.push(ext);
        } else if let Some(namespace_uri) = namespaces.get(item) {
            // It's a namespace prefix - resolve it to a URI
            if let Some(ext) = Extension::from_namespace(namespace_uri) {
                extensions.push(ext);
            } else {
                // Unknown URI - track as custom extension
                custom_namespaces.push(namespace_uri.clone());
            }
        } else {
            // Could be a direct URI that's not a known extension
            // If it looks like a URI (contains ://), track it as custom
            if item.contains("://") {
                custom_namespaces.push(item.to_string());
            }
            // Otherwise, silently ignore invalid items
        }
    }

    Ok((extensions, custom_namespaces))
}

/// Validate that all required extensions are supported by the parser configuration
fn validate_extensions(
    required: &[Extension],
    required_custom: &[String],
    config: &ParserConfig,
) -> Result<()> {
    // Validate known extensions
    for ext in required {
        if !config.supports(ext) {
            return Err(Error::UnsupportedExtension(format!(
                "Extension '{}' (namespace: {}) is required but not supported",
                ext.name(),
                ext.namespace()
            )));
        }
    }

    // Validate custom extensions
    for namespace in required_custom {
        if !config.has_custom_extension(namespace) {
            return Err(Error::UnsupportedExtension(format!(
                "Custom extension with namespace '{}' is required but not registered",
                namespace
            )));
        }
    }

    Ok(())
}

/// Try to handle an element with custom extension handlers
/// Returns true if a handler was invoked, false otherwise
#[allow(dead_code)] // Will be used in future enhancement
fn try_handle_custom_element<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
    config: &ParserConfig,
) -> Result<bool> {
    use crate::model::{CustomElementResult, CustomExtensionContext};

    // Extract namespace from element name
    let name = e.name();
    let name_str =
        std::str::from_utf8(name.as_ref()).map_err(|e| Error::InvalidXml(e.to_string()))?;

    // Try to get namespace prefix
    if let Some((_prefix, local_name)) = name_str.split_once(':') {
        // Look up the namespace URI for this prefix in reader's namespace map
        // For now, we'll check if any registered custom extension matches
        let attrs = parse_attributes(reader, e)?;

        // Check if we have a handler for any custom extension
        for (namespace, ext_info) in config.custom_extensions() {
            if let Some(handler) = &ext_info.element_handler {
                let context = CustomExtensionContext {
                    element_name: local_name.to_string(),
                    namespace: namespace.clone(),
                    attributes: attrs.clone(),
                };

                match handler(&context) {
                    Ok(CustomElementResult::Handled) => {
                        return Ok(true);
                    }
                    Ok(CustomElementResult::NotHandled) => {
                        continue;
                    }
                    Err(err) => {
                        return Err(Error::InvalidXml(format!(
                            "Custom extension handler error: {}",
                            err
                        )));
                    }
                }
            }
        }
    }

    Ok(false)
}

/// Parse attributes from an XML element
pub(crate) fn parse_attributes<R: std::io::BufRead>(
    _reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<HashMap<String, String>> {
    // Pre-allocate reasonable capacity to reduce allocations
    let mut attrs = HashMap::with_capacity(8);

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
pub(crate) fn validate_attributes(
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

    #[test]
    fn test_parse_component_simple() {
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
    <object id="2">
      <components>
        <component objectid="1"/>
      </components>
    </object>
  </resources>
  <build>
    <item objectid="2"/>
  </build>
</model>"#;

        let model = parse_model_xml(xml).unwrap();
        assert_eq!(model.resources.objects.len(), 2);

        // Check object 2 has a component
        let obj2 = &model.resources.objects[1];
        assert_eq!(obj2.id, 2);
        assert_eq!(obj2.components.len(), 1);
        assert_eq!(obj2.components[0].objectid, 1);
        assert!(obj2.components[0].transform.is_none());
    }

    #[test]
    fn test_parse_component_with_transform() {
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
    <object id="2">
      <components>
        <component objectid="1" transform="1 0 0 0 1 0 0 0 1 10 20 30"/>
      </components>
    </object>
  </resources>
  <build>
    <item objectid="2"/>
  </build>
</model>"#;

        let model = parse_model_xml(xml).unwrap();

        // Check object 2 has a component with transform
        let obj2 = &model.resources.objects[1];
        assert_eq!(obj2.components.len(), 1);
        assert_eq!(obj2.components[0].objectid, 1);

        let transform = obj2.components[0]
            .transform
            .expect("Transform should be present");
        // Identity rotation/scale with translation (10, 20, 30)
        assert_eq!(
            transform,
            [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 10.0, 20.0, 30.0]
        );
    }

    #[test]
    fn test_parse_multiple_components() {
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
    <object id="2">
      <mesh>
        <vertices>
          <vertex x="0" y="0" z="0"/>
          <vertex x="2" y="0" z="0"/>
          <vertex x="0" y="2" z="0"/>
        </vertices>
        <triangles>
          <triangle v1="0" v2="1" v3="2"/>
        </triangles>
      </mesh>
    </object>
    <object id="3">
      <components>
        <component objectid="1"/>
        <component objectid="2" transform="1 0 0 0 1 0 0 0 1 5 5 5"/>
      </components>
    </object>
  </resources>
  <build>
    <item objectid="3"/>
  </build>
</model>"#;

        let model = parse_model_xml(xml).unwrap();
        assert_eq!(model.resources.objects.len(), 3);

        // Check object 3 has two components
        let obj3 = &model.resources.objects[2];
        assert_eq!(obj3.id, 3);
        assert_eq!(obj3.components.len(), 2);

        assert_eq!(obj3.components[0].objectid, 1);
        assert!(obj3.components[0].transform.is_none());

        assert_eq!(obj3.components[1].objectid, 2);
        assert!(obj3.components[1].transform.is_some());
    }
}

/// Validate that external file paths referenced in boolean operations exist in the package
///
/// Per 3MF Boolean Operations Extension spec:
/// - The path attribute references objects in non-root model files
/// - Path is an absolute path from the root of the 3MF container
/// - This validation ensures referenced files exist
fn validate_boolean_external_paths<R: Read + std::io::Seek>(
    package: &mut Package<R>,
    model: &Model,
) -> Result<()> {
    for object in &model.resources.objects {
        if let Some(ref boolean_shape) = object.boolean_shape {
            // Check if booleanshape references an external file
            if let Some(ref path) = boolean_shape.path {
                // Normalize path: remove leading slash if present
                let normalized_path = path.trim_start_matches('/');
                
                if !package.has_file(normalized_path) {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Boolean shape references non-existent external file: {}\n\
                         The path attribute in <booleanshape> must reference a valid model file in the 3MF package.\n\
                         Check that:\n\
                         - The file exists in the package\n\
                         - The path is correct (case-sensitive)\n\
                         - The path format follows 3MF conventions (e.g., /3D/filename.model)",
                        object.id, path
                    )));
                }
            }

            // Check if boolean operands reference external files
            for operand in &boolean_shape.operands {
                if let Some(ref path) = operand.path {
                    // Normalize path: remove leading slash if present
                    let normalized_path = path.trim_start_matches('/');
                    
                    if !package.has_file(normalized_path) {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Boolean operand references non-existent external file: {}\n\
                             The path attribute in <boolean> must reference a valid model file in the 3MF package.\n\
                             Check that:\n\
                             - The file exists in the package\n\
                             - The path is correct (case-sensitive)\n\
                             - The path format follows 3MF conventions (e.g., /3D/filename.model)",
                            object.id, path
                        )));
                    }
                }
            }
        }
    }

    Ok(())
}
