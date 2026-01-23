//! XML parsing for 3MF model files

use crate::error::{Error, Result};
use crate::model::*;
use crate::opc::Package;
use crate::validator;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::{HashMap, HashSet};
use std::io::Read;

/// Size of 3MF transformation matrix (4x3 affine transform in row-major order)
const TRANSFORM_MATRIX_SIZE: usize = 12;

/// Default buffer capacity for XML parsing (4KB)
const XML_BUFFER_CAPACITY: usize = 4096;

/// Valid wrapping algorithm for SecureContent (2001 version)
const VALID_WRAPPING_ALGORITHM_2001: &str = "http://www.w3.org/2001/04/xmlenc#rsa-oaep-mgf1p";

/// Valid wrapping algorithm for SecureContent (2009 version)
const VALID_WRAPPING_ALGORITHM_2009: &str = "http://www.w3.org/2009/xmlenc11#rsa-oaep";

/// Default compression value for SecureContent CEK params
const DEFAULT_COMPRESSION: &str = "none";

/// Valid MGF algorithms for SecureContent kekparams
const VALID_MGF_ALGORITHMS: &[&str] = &[
    "http://www.w3.org/2009/xmlenc11#mgf1sha1",
    "http://www.w3.org/2009/xmlenc11#mgf1sha256",
    "http://www.w3.org/2009/xmlenc11#mgf1sha384",
    "http://www.w3.org/2009/xmlenc11#mgf1sha512",
];

/// Valid digest methods for SecureContent kekparams
const VALID_DIGEST_METHODS: &[&str] = &[
    "http://www.w3.org/2000/09/xmldsig#sha1",
    "http://www.w3.org/2001/04/xmlenc#sha256",
    "http://www.w3.org/2001/04/xmlenc#sha384",
    "http://www.w3.org/2001/04/xmlenc#sha512",
];

/// Maximum number of object IDs to display in error messages
const MAX_DISPLAYED_OBJECT_IDS: usize = 20;

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

    // Validate production extension external paths
    // This checks that external files exist and referenced objects/UUIDs are valid
    validate_production_external_paths(&mut package, &model)?;

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

/// Get an attribute value by its local name, regardless of namespace prefix
///
/// This is useful for extension attributes that can use different prefixes.
/// For example, `p:UUID` and `y:UUID` both have local name `"UUID"`.
///
/// # Arguments
/// * `attrs` - The attributes HashMap
/// * `local_name` - The local name to search for (e.g., "UUID", "path")
///
/// # Returns
/// The first attribute value found with the matching local name, or None
fn get_attr_by_local_name(attrs: &HashMap<String, String>, local_name: &str) -> Option<String> {
    attrs.iter().find_map(|(key, value)| {
        if get_local_name(key) == local_name {
            Some(value.clone())
        } else {
            None
        }
    })
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
    let mut in_ballsets = false;
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
    let mut current_displacement_mesh: Option<DisplacementMesh> = None;
    let mut in_displacement_mesh = false;
    let mut current_displacement_triangles_did: Option<usize> = None; // did attribute on <d:triangles>
    let mut in_displacement_triangles = false;
    let mut has_displacement_triangles = false; // Track if we've seen triangles element (DPX 3314)

    // Track declared displacement resources for forward-reference validation (DPX 3312)
    let mut declared_displacement2d_ids = std::collections::HashSet::<usize>::new();
    let mut declared_normvectorgroup_ids = std::collections::HashSet::<usize>::new();
    let mut declared_disp2dgroup_ids = std::collections::HashSet::<usize>::new();

    // Materials extension state for advanced features
    let mut current_texture2dgroup: Option<Texture2DGroup> = None;
    let mut in_texture2dgroup = false;
    let mut current_compositematerials: Option<CompositeMaterials> = None;
    let mut in_compositematerials = false;
    let mut current_multiproperties: Option<MultiProperties> = None;
    let mut in_multiproperties = false;

    // Component state
    let mut in_components = false;

    // Triangleset extension state (for validation)
    let mut in_trianglesets = false;

    // Track required elements for validation
    let mut resources_count = 0;
    let mut build_count = 0;

    // Track namespace declarations from model element
    let mut declared_namespaces: HashMap<String, String> = HashMap::new();

    loop {
        let event_result = reader.read_event_into(&mut buf);
        let is_empty_element = matches!(&event_result, Ok(Event::Empty(_)));

        match event_result {
            Ok(Event::Decl(_)) => {
                // XML declaration is allowed
            }
            Ok(Event::DocType(_)) => {
                // N_XPX_0420_01: DTD declarations are not allowed (security risk)
                return Err(Error::InvalidXml(
                    "DTD declarations are not allowed in 3MF files for security reasons"
                        .to_string(),
                ));
            }
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
                        let mut recommended_ext_value = None;
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
                                    // Per 3MF spec, requiredextensions can be empty string (means no extensions)
                                    // or contain space-separated extension prefixes/namespaces
                                    required_ext_value = Some(value.to_string());
                                }
                                "recommendedextensions" => {
                                    // Per 3MF spec, recommendedextensions are optional and may be ignored
                                    // They suggest extensions that enhance user experience but are not required
                                    // Validate that the value is not empty if present
                                    if value.trim().is_empty() {
                                        return Err(Error::InvalidXml(
                                            "recommendedextensions attribute cannot be empty"
                                                .to_string(),
                                        ));
                                    }
                                    recommended_ext_value = Some(value.to_string());
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
                        if let Some(ref ext_value) = required_ext_value {
                            let (extensions, custom_extensions) =
                                parse_required_extensions_with_namespaces(ext_value, &namespaces)?;
                            model.required_extensions = extensions;
                            model.required_custom_extensions = custom_extensions;
                            // Validate that all required extensions are supported
                            validate_extensions(
                                &model.required_extensions,
                                &model.required_custom_extensions,
                                &config,
                            )?;
                        }

                        // Validate that no extension appears in both required and recommended
                        if let (Some(req_value), Some(rec_value)) =
                            (&required_ext_value, &recommended_ext_value)
                        {
                            validate_no_duplicate_extensions(req_value, rec_value, &namespaces)?;
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

                        // Extract Production extension UUID (p:UUID, or any prefix) from build element
                        if let Some(p_uuid) = get_attr_by_local_name(&attrs, "UUID") {
                            model.build.production_uuid = Some(p_uuid);
                        }
                    }
                    "object" if in_resources => {
                        current_object = Some(parse_object(&reader, e)?);
                    }
                    "mesh" if in_resources && current_object.is_some() => {
                        current_mesh = Some(Mesh::new());
                    }
                    "displacementmesh" if in_resources && current_object.is_some() => {
                        current_displacement_mesh = Some(DisplacementMesh::new());
                        in_displacement_mesh = true;
                        has_displacement_triangles = false; // Reset for new displacementmesh
                                                            // Mark object as having extension shapes
                        if let Some(ref mut obj) = current_object {
                            obj.has_extension_shapes = true;
                        }
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
                    "vertices" if in_displacement_mesh => {
                        // Displacement vertices will be parsed as individual vertex elements
                    }
                    "vertex" if in_displacement_mesh => {
                        // Per DPX spec 4.1: All elements under displacementmesh MUST use displacement namespace
                        let has_displacement_prefix = name_str.starts_with("d:") || name_str.starts_with("displacement:");
                        if !has_displacement_prefix {
                            return Err(Error::InvalidXml(format!(
                                "Element <{}> under displacementmesh must use the displacement namespace prefix (e.g., <d:vertex>). \
                                 Per 3MF Displacement Extension spec 4.1, all elements under <displacementmesh> MUST specify \
                                 the displacement namespace prefix.",
                                name_str
                            )));
                        }
                        if let Some(ref mut disp_mesh) = current_displacement_mesh {
                            let vertex = parse_vertex(&reader, e)?;
                            disp_mesh.vertices.push(vertex);
                        }
                    }
                    "vertex" if current_mesh.is_some() => {
                        if let Some(ref mut mesh) = current_mesh {
                            let vertex = parse_vertex(&reader, e)?;
                            mesh.vertices.push(vertex);
                        }
                    }
                    // Displacement mesh triangles must be checked BEFORE regular mesh triangles
                    // because an object can have both <mesh> and <d:displacementmesh>, and
                    // current_mesh stays Some() even when processing displacement elements.
                    // If we check current_mesh.is_some() first, displacement triangles would be
                    // parsed as regular triangles and d1/d2/d3 attributes would be rejected.
                    "triangles" if in_displacement_mesh => {
                        // Per DPX spec 4.1: All elements under displacementmesh MUST use displacement namespace
                        let has_displacement_prefix = name_str.starts_with("d:") || name_str.starts_with("displacement:");
                        if !has_displacement_prefix {
                            return Err(Error::InvalidXml(format!(
                                "Element <{}> under displacementmesh must use the displacement namespace prefix (e.g., <d:triangles>). \
                                 Per 3MF Displacement Extension spec 4.1, all elements under <displacementmesh> MUST specify \
                                 the displacement namespace prefix.",
                                name_str
                            )));
                        }
                        // Per DPX spec 4.1: Only one triangles element allowed per displacementmesh
                        if has_displacement_triangles {
                            return Err(Error::InvalidXml(
                                "Displacement mesh can only have one <triangles> element"
                                    .to_string(),
                            ));
                        }
                        has_displacement_triangles = true;
                        in_displacement_triangles = true;
                        // Parse did attribute from triangles element
                        let attrs = parse_attributes(&reader, e)?;
                        if let Some(did_str) = attrs.get("did") {
                            let did = did_str.parse::<usize>()?;
                            // Validate forward reference (DPX 3312)
                            if !declared_disp2dgroup_ids.contains(&did) {
                                return Err(Error::InvalidXml(format!(
                                    "Triangles element references Disp2DGroup with ID {} which has not been declared yet. \
                                     Resources must be declared before they are referenced.",
                                    did
                                )));
                            }
                            current_displacement_triangles_did = Some(did);
                        } else {
                            current_displacement_triangles_did = None;
                        }
                    }
                    "triangles" if current_mesh.is_some() => {
                        // Regular mesh triangles - parsed as individual triangle elements
                    }
                    // Displacement triangle must be checked BEFORE regular triangle for same reason
                    "triangle" if in_displacement_triangles => {
                        // Per DPX spec 4.1: All elements under displacementmesh MUST use displacement namespace
                        let has_displacement_prefix = name_str.starts_with("d:") || name_str.starts_with("displacement:");
                        if !has_displacement_prefix {
                            return Err(Error::InvalidXml(format!(
                                "Element <{}> under displacementmesh must use the displacement namespace prefix (e.g., <d:triangle>). \
                                 Per 3MF Displacement Extension spec 4.1, all elements under <displacementmesh> MUST specify \
                                 the displacement namespace prefix.",
                                name_str
                            )));
                        }
                        if let Some(ref mut disp_mesh) = current_displacement_mesh {
                            let mut triangle = parse_displacement_triangle(&reader, e)?;

                            // Per DPX spec 4.1.2.1: Validate displacement attribute consistency
                            // If d2 or d3 specified without d1, that's invalid
                            if (triangle.d2.is_some() || triangle.d3.is_some())
                                && triangle.d1.is_none()
                            {
                                return Err(Error::InvalidXml(
                                    "Displacement triangle: d2 or d3 displacement coordinate index specified without d1. \
                                     If d1 is unspecified, no displacement coordinate indices can be used."
                                        .to_string()
                                ));
                            }

                            // Validate forward reference for did on triangle element (DPX 3312)
                            if let Some(did) = triangle.did {
                                if !declared_disp2dgroup_ids.contains(&did) {
                                    return Err(Error::InvalidXml(format!(
                                        "Triangle element references Disp2DGroup with ID {} which has not been declared yet. \
                                         Resources must be declared before they are referenced.",
                                        did
                                    )));
                                }
                            }

                            // If did not specified on triangle, use the one from triangles element
                            if triangle.did.is_none() {
                                triangle.did = current_displacement_triangles_did;
                            }

                            // If d1 is specified, did MUST be specified (either on triangle or triangles)
                            if triangle.d1.is_some() && triangle.did.is_none() {
                                return Err(Error::InvalidXml(
                                    "Displacement triangle: d1 displacement coordinate index is specified but 'did' attribute is not. \
                                     The 'did' must be specified either on the <triangle> or <triangles> element."
                                        .to_string()
                                ));
                            }

                            disp_mesh.triangles.push(triangle);
                        }
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
                                let color = parse_color(color_str).ok_or_else(|| {
                                    Error::InvalidXml(format!(
                                        "Invalid color format '{}' in colorgroup {}.\n\
                                         Colors must be in format #RRGGBB or #RRGGBBAA where each component is a hexadecimal value (0-9, A-F).\n\
                                         Examples: #FF0000 (red), #00FF0080 (semi-transparent green)",
                                        color_str, colorgroup.id
                                    ))
                                })?;
                                colorgroup.colors.push(color);
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

                        // Validate only allowed attributes are present
                        // Per Displacement Extension spec 3.1: id, path, channel, tilestyleu, tilestylev, filter
                        validate_attributes(
                            &attrs,
                            &[
                                "id",
                                "path",
                                "channel",
                                "tilestyleu",
                                "tilestylev",
                                "filter",
                            ],
                            "displacement2d",
                        )?;

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
                        // Strict validation: reject invalid enum values per DPX 3316
                        if let Some(channel_str) = attrs.get("channel") {
                            disp.channel = match channel_str.to_uppercase().as_str() {
                                "R" => Channel::R,
                                "G" => Channel::G,
                                "B" => Channel::B,
                                "A" => Channel::A,
                                _ => {
                                    return Err(Error::InvalidXml(format!(
                                        "Invalid channel value '{}'. Valid values are: R, G, B, A",
                                        channel_str
                                    )))
                                }
                            };
                        }

                        if let Some(tileu_str) = attrs.get("tilestyleu") {
                            disp.tilestyleu = match tileu_str.to_lowercase().as_str() {
                                "wrap" => TileStyle::Wrap,
                                "mirror" => TileStyle::Mirror,
                                "clamp" => TileStyle::Clamp,
                                "none" => TileStyle::None,
                                _ => return Err(Error::InvalidXml(format!(
                                    "Invalid tilestyleu value '{}'. Valid values are: wrap, mirror, clamp, none",
                                    tileu_str
                                ))),
                            };
                        }

                        if let Some(tilev_str) = attrs.get("tilestylev") {
                            disp.tilestylev = match tilev_str.to_lowercase().as_str() {
                                "wrap" => TileStyle::Wrap,
                                "mirror" => TileStyle::Mirror,
                                "clamp" => TileStyle::Clamp,
                                "none" => TileStyle::None,
                                _ => return Err(Error::InvalidXml(format!(
                                    "Invalid tilestylev value '{}'. Valid values are: wrap, mirror, clamp, none",
                                    tilev_str
                                ))),
                            };
                        }

                        if let Some(filter_str) = attrs.get("filter") {
                            disp.filter = match filter_str.to_lowercase().as_str() {
                                "auto" => FilterMode::Auto,
                                "linear" => FilterMode::Linear,
                                "nearest" => FilterMode::Nearest,
                                _ => return Err(Error::InvalidXml(format!(
                                    "Invalid filter value '{}'. Valid values are: auto, linear, nearest",
                                    filter_str
                                ))),
                            };
                        }

                        model.resources.displacement_maps.push(disp);
                        declared_displacement2d_ids.insert(id); // Track for forward-reference validation
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
                        // Check if we already have a beamset - nested or multiple beamlattice is invalid
                        if in_beamset {
                            return Err(Error::InvalidXml(
                                "Multiple or nested beamlattice elements are not allowed. \
                                 Each mesh can have only one beamlattice element."
                                    .to_string(),
                            ));
                        }

                        in_beamset = true;

                        // Mark that this object has extension shape elements
                        // This is used for validation - per Boolean Ops spec, operands must be simple meshes only
                        if let Some(ref mut obj) = current_object {
                            obj.has_extension_shapes = true;
                        }

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

                        // Parse clippingmesh ID attribute (optional)
                        if let Some(clip_id_str) = attrs.get("clippingmesh") {
                            beamset.clipping_mesh_id = Some(clip_id_str.parse::<u32>()?);
                        }

                        // Parse representationmesh ID attribute (optional)
                        if let Some(rep_id_str) = attrs.get("representationmesh") {
                            beamset.representation_mesh_id = Some(rep_id_str.parse::<u32>()?);
                        }

                        // Parse clippingmode attribute (optional)
                        if let Some(clip_mode) = attrs.get("clippingmode") {
                            beamset.clipping_mode = Some(clip_mode.clone());
                        }

                        // Parse ballmode attribute (optional) - from balls extension
                        // This can be in default namespace or balls namespace (b2:ballmode)
                        if let Some(ball_mode) =
                            attrs.get("ballmode").or_else(|| attrs.get("b2:ballmode"))
                        {
                            beamset.ball_mode = Some(ball_mode.clone());
                        }

                        // Parse ballradius attribute (optional) - from balls extension
                        // This can be in default namespace or balls namespace (b2:ballradius)
                        if let Some(ball_radius_str) = attrs
                            .get("ballradius")
                            .or_else(|| attrs.get("b2:ballradius"))
                        {
                            let ball_radius = ball_radius_str.parse::<f64>()?;
                            // Validate ball radius is finite and positive
                            if !ball_radius.is_finite() || ball_radius <= 0.0 {
                                return Err(Error::InvalidXml(format!(
                                    "BeamLattice ballradius must be positive and finite (got {})",
                                    ball_radius
                                )));
                            }
                            beamset.ball_radius = Some(ball_radius);
                        }

                        // Parse pid attribute (optional) - material/property group ID
                        if let Some(pid_str) = attrs.get("pid") {
                            beamset.property_id = Some(pid_str.parse::<u32>()?);
                        }

                        // Parse pindex attribute (optional) - property index
                        if let Some(pindex_str) = attrs.get("pindex") {
                            beamset.property_index = Some(pindex_str.parse::<u32>()?);
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
                    "beamsets" if in_beamset => {
                        // beamsets container element for grouping beams into sets
                        // No attributes to parse, just contains beamset children
                    }
                    "beamset" if in_beamset => {
                        // beamset element within beamsets - contains ref elements
                        // No attributes to parse, just contains ref children
                    }
                    "ref" if in_beamset => {
                        // ref element can be in beamsets or ballsets
                        let attrs = parse_attributes(&reader, e)?;
                        if let Some(index_str) = attrs.get("index") {
                            let index = index_str.parse::<usize>()?;
                            // Store the ref index for validation
                            if let Some(ref mut beamset) = current_beamset {
                                if in_ballsets {
                                    // This is a ballref (reference to ball)
                                    beamset.ball_set_refs.push(index);
                                } else {
                                    // This is a beamref (reference to beam)
                                    beamset.beam_set_refs.push(index);
                                }
                            }
                        }
                    }
                    "balls" if in_beamset => {
                        // balls container element from balls sub-extension
                        // Contains ball elements
                    }
                    "ball" if in_beamset => {
                        // ball element from balls sub-extension
                        // References a vertex index and may have material properties
                        let attrs = parse_attributes(&reader, e)?;

                        // Parse required vindex attribute
                        let vindex = attrs
                            .get("vindex")
                            .ok_or_else(|| {
                                Error::InvalidXml(
                                    "Ball element missing required vindex attribute".to_string(),
                                )
                            })?
                            .parse::<usize>()?;

                        let mut ball = Ball::new(vindex);

                        // Parse optional radius
                        if let Some(r_str) = attrs.get("r") {
                            ball.radius = Some(r_str.parse::<f64>()?);
                        }

                        // Parse optional pid (property group ID)
                        if let Some(pid_str) = attrs.get("pid") {
                            ball.property_id = Some(pid_str.parse::<u32>()?);
                        }

                        // Parse optional p (property index)
                        if let Some(p_str) = attrs.get("p") {
                            ball.property_index = Some(p_str.parse::<u32>()?);
                        }

                        // Add ball to beamset
                        if let Some(ref mut beamset) = current_beamset {
                            beamset.balls.push(ball);
                        }
                    }
                    "ballsets" if in_beamset => {
                        // ballsets container element for grouping balls into sets
                        // Contains ballset children with ref/ballref elements
                        in_ballsets = true;
                    }
                    "ballset" if in_beamset => {
                        // ballset element within ballsets - contains ref/ballref elements
                        // No attributes to parse
                    }
                    "ballref" if in_beamset => {
                        // ballref element - explicit ball reference (alternative to generic ref in ballsets)
                        let attrs = parse_attributes(&reader, e)?;
                        if let Some(index_str) = attrs.get("index") {
                            let index = index_str.parse::<usize>()?;
                            // Store the ballref index for validation
                            if let Some(ref mut beamset) = current_beamset {
                                beamset.ball_set_refs.push(index);
                            }
                        }
                    }
                    "normvectorgroup" if in_resources => {
                        in_normvectorgroup = true;
                        let attrs = parse_attributes(&reader, e)?;

                        // Validate only allowed attributes are present
                        // Per Displacement Extension spec 3.2: id
                        validate_attributes(&attrs, &["id"], "normvectorgroup")?;

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

                            // Validate only allowed attributes are present
                            // Per Displacement Extension spec 3.2.1: x, y, z
                            validate_attributes(&attrs, &["x", "y", "z"], "normvector")?;

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

                            // Validate values are finite
                            if !x.is_finite() || !y.is_finite() || !z.is_finite() {
                                return Err(Error::InvalidXml(format!(
                                    "NormVector has non-finite values: x={}, y={}, z={}",
                                    x, y, z
                                )));
                            }

                            nvgroup.vectors.push(NormVector::new(x, y, z));
                        }
                    }
                    "disp2dgroup" if in_resources => {
                        in_disp2dgroup = true;
                        let attrs = parse_attributes(&reader, e)?;

                        // Validate only allowed attributes are present
                        // Per Displacement Extension spec 3.3: id, dispid, nid, height, offset
                        validate_attributes(
                            &attrs,
                            &["id", "dispid", "nid", "height", "offset"],
                            "disp2dgroup",
                        )?;

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

                        // Per DPX spec 3.3: Validate dispid references declared Displacement2D resource
                        if !declared_displacement2d_ids.contains(&dispid) {
                            return Err(Error::InvalidXml(format!(
                                "Disp2DGroup references Displacement2D with ID {} which has not been declared yet. \
                                 Resources must be declared before they are referenced.",
                                dispid
                            )));
                        }

                        let nid = attrs
                            .get("nid")
                            .ok_or_else(|| {
                                Error::InvalidXml("disp2dgroup missing nid attribute".to_string())
                            })?
                            .parse::<usize>()?;

                        // Per DPX spec 3.3: Validate nid references declared NormVectorGroup resource
                        if !declared_normvectorgroup_ids.contains(&nid) {
                            return Err(Error::InvalidXml(format!(
                                "Disp2DGroup references NormVectorGroup with ID {} which has not been declared yet. \
                                 Resources must be declared before they are referenced.",
                                nid
                            )));
                        }
                        let height = attrs
                            .get("height")
                            .ok_or_else(|| {
                                Error::InvalidXml(
                                    "disp2dgroup missing height attribute".to_string(),
                                )
                            })?
                            .parse::<f64>()?;

                        // Validate height is finite
                        if !height.is_finite() {
                            return Err(Error::InvalidXml(format!(
                                "Disp2DGroup height must be finite, got: {}",
                                height
                            )));
                        }

                        let mut disp2dgroup = Disp2DGroup::new(id, dispid, nid, height);

                        // Parse optional offset
                        if let Some(offset_str) = attrs.get("offset") {
                            let offset = offset_str.parse::<f64>()?;
                            if !offset.is_finite() {
                                return Err(Error::InvalidXml(format!(
                                    "Disp2DGroup offset must be finite, got: {}",
                                    offset
                                )));
                            }
                            disp2dgroup.offset = offset;
                        }

                        current_disp2dgroup = Some(disp2dgroup);
                    }
                    "disp2dcoord" if in_disp2dgroup => {
                        if let Some(ref mut d2dgroup) = current_disp2dgroup {
                            let attrs = parse_attributes(&reader, e)?;

                            // Validate only allowed attributes are present
                            // Per Displacement Extension spec 3.3.1: u, v, n, f
                            validate_attributes(&attrs, &["u", "v", "n", "f"], "disp2dcoord")?;

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

                            // Validate u,v are finite
                            if !u.is_finite() || !v.is_finite() {
                                return Err(Error::InvalidXml(format!(
                                    "Disp2DCoords u and v must be finite, got: u={}, v={}",
                                    u, v
                                )));
                            }

                            let mut coords = Disp2DCoords::new(u, v, n);

                            // Parse optional f attribute
                            if let Some(f_str) = attrs.get("f") {
                                let f = f_str.parse::<f64>()?;
                                if !f.is_finite() {
                                    return Err(Error::InvalidXml(format!(
                                        "Disp2DCoords f must be finite, got: {}",
                                        f
                                    )));
                                }
                                coords.f = f;
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

                        // For self-closing empty slice tags like <s:slice ztop="100.060"/>,
                        // immediately push the slice since there won't be an End event
                        if is_empty_element {
                            if let Some(slice) = current_slice.take() {
                                if let Some(ref mut slicestack) = current_slicestack {
                                    slicestack.slices.push(slice);
                                }
                            }
                            in_slice = false;
                        }
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
                        // We check both the object's boolean_shape field (for closed booleanshape elements)
                        // and the in_boolean_shape flag (for currently open booleanshape elements)
                        if in_boolean_shape
                            || current_object
                                .as_ref()
                                .map(|obj| obj.boolean_shape.is_some())
                                .unwrap_or(false)
                        {
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
                    "trianglesets" if current_mesh.is_some() => {
                        in_trianglesets = true;
                    }
                    "triangleset" if in_trianglesets => {
                        // Triangleset element - validate name attribute is not empty
                        let attrs = parse_attributes(&reader, e)?;
                        if let Some(name) = attrs.get("name") {
                            if name.trim().is_empty() {
                                return Err(Error::InvalidXml(
                                    "triangleset name attribute cannot be empty".to_string(),
                                ));
                            }
                        }
                    }
                    "ref" if in_trianglesets => {
                        // Validate triangle index in ref element
                        let attrs = parse_attributes(&reader, e)?;
                        if let Some(index_str) = attrs.get("index") {
                            let index = index_str.parse::<usize>().map_err(|_| {
                                Error::InvalidXml(format!("Invalid triangle index: {}", index_str))
                            })?;
                            // Validate index is within bounds
                            if let Some(ref mesh) = current_mesh {
                                validate_triangle_index(mesh, index, "ref")?;
                            }
                        }
                    }
                    "refrange" if in_trianglesets => {
                        // Validate triangle index range in refrange element
                        let attrs = parse_attributes(&reader, e)?;
                        if let (Some(start_str), Some(end_str)) =
                            (attrs.get("startindex"), attrs.get("endindex"))
                        {
                            let start_index = start_str.parse::<usize>().map_err(|_| {
                                Error::InvalidXml(format!(
                                    "Invalid triangle start index: {}",
                                    start_str
                                ))
                            })?;
                            let end_index = end_str.parse::<usize>().map_err(|_| {
                                Error::InvalidXml(format!(
                                    "Invalid triangle end index: {}",
                                    end_str
                                ))
                            })?;

                            // Validate that start_index <= end_index
                            if start_index > end_index {
                                return Err(Error::InvalidXml(format!(
                                    "refrange start index {} is greater than end index {}",
                                    start_index, end_index
                                )));
                            }

                            // Validate indices are within bounds
                            if let Some(ref mesh) = current_mesh {
                                validate_triangle_index(mesh, start_index, "refrange start")?;
                                validate_triangle_index(mesh, end_index, "refrange end")?;
                            }
                        }
                    }
                    _ => {
                        // Unknown/custom extension element - validate that attribute values don't use namespace prefixes
                        let attrs = parse_attributes(&reader, e)?;
                        validate_attribute_values(&attrs, &declared_namespaces)?;
                    }
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
                            if let Some(disp_mesh) = current_displacement_mesh.take() {
                                obj.displacement_mesh = Some(disp_mesh);
                            }
                            model.resources.objects.push(obj);
                        }
                    }
                    "mesh" => {
                        // Mesh parsing complete
                    }
                    "displacementmesh" => {
                        in_displacement_mesh = false;
                        // Per DPX spec 4.0: Object containing displacementmesh MUST be type="model"
                        if let Some(ref obj) = current_object {
                            if obj.object_type != ObjectType::Model {
                                return Err(Error::InvalidXml(format!(
                                    "Object {} with displacementmesh must have type=\"model\", found type=\"{}\"",
                                    obj.id,
                                    match obj.object_type {
                                        ObjectType::Model => "model",
                                        ObjectType::Support => "support",
                                        ObjectType::SolidSupport => "solidsupport",
                                        ObjectType::Surface => "surface",
                                        ObjectType::Other => "other",
                                    }
                                )));
                            }
                        }
                    }
                    "triangles" if in_displacement_triangles => {
                        in_displacement_triangles = false;
                        current_displacement_triangles_did = None;
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
                            declared_normvectorgroup_ids.insert(nvgroup.id); // Track for forward-reference validation
                            model.resources.norm_vector_groups.push(nvgroup);
                        }
                        in_normvectorgroup = false;
                    }
                    "disp2dgroup" => {
                        if let Some(d2dgroup) = current_disp2dgroup.take() {
                            declared_disp2dgroup_ids.insert(d2dgroup.id); // Track for forward-reference validation
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
                        in_ballsets = false;
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
                    "trianglesets" => {
                        in_trianglesets = false;
                    }
                    "ballsets" => {
                        in_ballsets = false;
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
    // keystore and slice files are loaded, not here
    Ok(model)
}

/// Validate kekparams attributes (wrapping algorithm, mgf algorithm, digest method)
///
/// This helper validates the cryptographic algorithm attributes in kekparams elements
/// per EPX-2603 SecureContent specification.
fn validate_kekparams_attributes(
    wrapping_algorithm: &str,
    mgf_algorithm: &str,
    digest_method: &str,
    sc: &mut SecureContentInfo,
) -> Result<()> {
    // EPX-2603: Validate wrapping algorithm
    if !wrapping_algorithm.is_empty() {
        let is_valid = wrapping_algorithm == VALID_WRAPPING_ALGORITHM_2001
            || wrapping_algorithm == VALID_WRAPPING_ALGORITHM_2009;

        if !is_valid {
            return Err(Error::InvalidSecureContent(format!(
                "Invalid wrapping algorithm '{}'. Must be either '{}' or '{}' (EPX-2603)",
                wrapping_algorithm, VALID_WRAPPING_ALGORITHM_2001, VALID_WRAPPING_ALGORITHM_2009
            )));
        }

        sc.wrapping_algorithms.push(wrapping_algorithm.to_string());
    }

    // EPX-2603: Validate mgfalgorithm if present
    if !mgf_algorithm.is_empty() && !VALID_MGF_ALGORITHMS.contains(&mgf_algorithm) {
        return Err(Error::InvalidSecureContent(format!(
                "Invalid mgfalgorithm '{}'. Must be one of mgf1sha1, mgf1sha256, mgf1sha384, or mgf1sha512 (EPX-2603)",
                mgf_algorithm
            )));
    }

    // EPX-2603: Validate digestmethod if present
    if !digest_method.is_empty() && !VALID_DIGEST_METHODS.contains(&digest_method) {
        return Err(Error::InvalidSecureContent(format!(
            "Invalid digestmethod '{}'. Must be one of sha1, sha256, sha384, or sha512 (EPX-2603)",
            digest_method
        )));
    }

    Ok(())
}

/// Load and parse Secure/keystore.xml to identify encrypted files
///
/// This provides the complete structural information needed for applications to
/// implement their own decryption logic using external cryptographic libraries.
///
/// This function also performs validation as per 3MF SecureContent specification:
/// - EPX-2601: Validates consumer index references exist
/// - EPX-2602: Validates consumers exist when access rights are defined
/// - EPX-2603: Validates encryption algorithms are valid
/// - EPX-2604: Validates consumer IDs are unique
/// - EPX-2605: Validates encrypted file paths are valid (not OPC .rels files)
/// - EPX-2607: Validates referenced files exist in the package
fn load_keystore<R: Read + std::io::Seek>(
    package: &mut Package<R>,
    model: &mut Model,
) -> Result<()> {
    // Discover keystore file path from relationships
    // Per 3MF SecureContent spec, the keystore is identified by a relationship of type
    // http://schemas.microsoft.com/3dmanufacturing/{version}/keystore
    let keystore_path = match package.discover_keystore_path()? {
        Some(path) => path,
        None => {
            // Try fallback to default paths for backward compatibility
            // Check both Secure/keystore.xml and Secure/info.store
            if package.has_file("Secure/keystore.xml") {
                "Secure/keystore.xml".to_string()
            } else if package.has_file("Secure/info.store") {
                "Secure/info.store".to_string()
            } else {
                return Ok(()); // No keystore file, not an error
            }
        }
    };

    // Load the keystore file
    // Use get_file_binary() to handle files that may contain encrypted/binary data
    let keystore_bytes = package.get_file_binary(&keystore_path)?;

    // Initialize secure_content if not already present
    if model.secure_content.is_none() {
        model.secure_content = Some(SecureContentInfo::default());
    }

    // Convert bytes to string, using lossy conversion to handle any non-UTF-8 sequences
    // This allows parsing keystore files that may contain encrypted content
    let keystore_xml = String::from_utf8_lossy(&keystore_bytes);

    let mut reader = Reader::from_str(&keystore_xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::with_capacity(XML_BUFFER_CAPACITY);

    // State tracking for nested parsing
    let mut current_consumer: Option<Consumer> = None;
    let mut current_resource_group: Option<ResourceDataGroup> = None;
    let mut current_access_right: Option<AccessRight> = None;
    let mut current_resource_data: Option<ResourceData> = None;
    let mut current_kek_params: Option<KEKParams> = None;
    let mut current_cek_params: Option<CEKParams> = None;
    let mut text_buffer = String::with_capacity(512); // Typical size for base64-encoded values
    let mut encrypted_paths = HashSet::new(); // Track resourcedata paths for duplicate detection

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) => {
                // Handle self-closing tags
                let name = e.name();
                let name_str = std::str::from_utf8(name.as_ref())
                    .map_err(|e| Error::InvalidXml(e.to_string()))?;
                let local_name = get_local_name(name_str);

                // Handle self-closing elements that need validation or tracking
                if local_name == "kekparams" {
                    // EPX-2603: Extract and validate kekparams attributes
                    let mut wrapping_algorithm = String::new();
                    let mut mgf_algorithm = String::new();
                    let mut digest_method = String::new();

                    for attr in e.attributes() {
                        let attr = attr.map_err(|e| {
                            Error::InvalidXml(format!("Invalid attribute in kekparams: {}", e))
                        })?;
                        let attr_name = std::str::from_utf8(attr.key.as_ref())
                            .map_err(|e| Error::InvalidXml(e.to_string()))?;
                        let attr_value = std::str::from_utf8(&attr.value)
                            .map_err(|e| Error::InvalidXml(e.to_string()))?
                            .to_string();

                        match attr_name {
                            "wrappingalgorithm" => wrapping_algorithm = attr_value,
                            "mgfalgorithm" => mgf_algorithm = attr_value,
                            "digestmethod" => digest_method = attr_value,
                            _ => {}
                        }
                    }

                    if let Some(ref mut sc) = model.secure_content {
                        validate_kekparams_attributes(
                            &wrapping_algorithm,
                            &mgf_algorithm,
                            &digest_method,
                            sc,
                        )?;
                    }
                }
            }
            Ok(Event::Start(ref e)) => {
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
                    "consumer" => {
                        let mut consumer_id = String::new();
                        let mut key_id = None;

                        for attr in e.attributes() {
                            let attr = attr.map_err(|e| {
                                Error::InvalidXml(format!("Invalid attribute in consumer: {}", e))
                            })?;
                            let attr_name = std::str::from_utf8(attr.key.as_ref())
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;
                            let attr_value = std::str::from_utf8(&attr.value)
                                .map_err(|e| Error::InvalidXml(e.to_string()))?
                                .to_string();

                            match attr_name {
                                "consumerid" => consumer_id = attr_value,
                                "keyid" => key_id = Some(attr_value),
                                _ => {}
                            }
                        }

                        // EPX-2604: Check for duplicate consumer IDs
                        if let Some(ref mut sc) = model.secure_content {
                            if sc.consumer_ids.contains(&consumer_id) {
                                return Err(Error::InvalidSecureContent(format!(
                                    "Duplicate consumer ID '{}' in keystore (EPX-2604)",
                                    consumer_id
                                )));
                            }
                            sc.consumer_ids.push(consumer_id.clone());
                            sc.consumer_count += 1;
                        }

                        current_consumer = Some(Consumer {
                            consumer_id,
                            key_id,
                            key_value: None,
                        });
                    }
                    "keyvalue" => {
                        text_buffer.clear();
                    }
                    "resourcedatagroup" => {
                        let mut key_uuid = String::new();

                        for attr in e.attributes() {
                            let attr = attr.map_err(|e| {
                                Error::InvalidXml(format!(
                                    "Invalid attribute in resourcedatagroup: {}",
                                    e
                                ))
                            })?;
                            let attr_name = std::str::from_utf8(attr.key.as_ref())
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;
                            if attr_name == "keyuuid" {
                                key_uuid = std::str::from_utf8(&attr.value)
                                    .map_err(|e| Error::InvalidXml(e.to_string()))?
                                    .to_string();
                            }
                        }

                        current_resource_group = Some(ResourceDataGroup {
                            key_uuid,
                            access_rights: Vec::new(),
                            resource_data: Vec::new(),
                        });
                    }
                    "accessright" => {
                        let mut consumer_index = 0;

                        // EPX-2601: Track and validate consumer index
                        // EPX-2606: Track accessright elements that have kekparams
                        // We'll check if they have cipherdata in a subsequent Text event
                        for attr in e.attributes() {
                            let attr = attr.map_err(|e| {
                                Error::InvalidXml(format!(
                                    "Invalid attribute in accessright: {}",
                                    e
                                ))
                            })?;
                            let attr_name = std::str::from_utf8(attr.key.as_ref())
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;
                            if attr_name == "consumerindex" {
                                let index_str = std::str::from_utf8(&attr.value)
                                    .map_err(|e| Error::InvalidXml(e.to_string()))?;
                                consumer_index = index_str.parse::<usize>().map_err(|_| {
                                    Error::InvalidSecureContent(format!(
                                        "Invalid consumer index '{}' (must be a valid number)",
                                        index_str
                                    ))
                                })?;
                            }
                        }

                        current_access_right = Some(AccessRight {
                            consumer_index,
                            kek_params: KEKParams {
                                wrapping_algorithm: String::new(),
                                mgf_algorithm: None,
                                digest_method: None,
                            },
                            cipher_value: String::new(),
                        });
                    }
                    "kekparams" => {
                        // EPX-2603: Extract and validate kekparams attributes
                        let mut wrapping_algorithm = String::new();
                        let mut mgf_algorithm = None;
                        let mut digest_method = None;

                        for attr in e.attributes() {
                            let attr = attr.map_err(|e| {
                                Error::InvalidXml(format!("Invalid attribute in kekparams: {}", e))
                            })?;
                            let attr_name = std::str::from_utf8(attr.key.as_ref())
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;
                            let attr_value = std::str::from_utf8(&attr.value)
                                .map_err(|e| Error::InvalidXml(e.to_string()))?
                                .to_string();

                            match attr_name {
                                "wrappingalgorithm" => wrapping_algorithm = attr_value,
                                "mgfalgorithm" => mgf_algorithm = Some(attr_value),
                                "digestmethod" => digest_method = Some(attr_value),
                                _ => {}
                            }
                        }

                        current_kek_params = Some(KEKParams {
                            wrapping_algorithm,
                            mgf_algorithm,
                            digest_method,
                        });

                        // Assign immediately to current_access_right if this is an Empty element
                        // (self-closing tag like <kekparams ... />)
                        if let Some(kek_params) = current_kek_params.take() {
                            if let Some(ref mut access_right) = current_access_right {
                                access_right.kek_params = kek_params;
                            }
                        }
                    }
                    "cipherdata" => {
                        // cipherdata contains xenc:CipherValue
                    }
                    "CipherValue" => {
                        text_buffer.clear();
                    }
                    "resourcedata" => {
                        let mut path = String::new();

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
                                path = std::str::from_utf8(&attr.value)
                                    .map_err(|e| Error::InvalidXml(e.to_string()))?
                                    .to_string();
                            }
                        }

                        // EPX-2605: Validate path
                        if path.trim().is_empty() {
                            return Err(Error::InvalidSecureContent(
                                "Resource data path attribute cannot be empty (EPX-2605)"
                                    .to_string(),
                            ));
                        }

                        let path_lower = path.to_lowercase();
                        if path_lower.contains("/_rels/") || path_lower.ends_with(".rels") {
                            return Err(Error::InvalidSecureContent(format!(
                                "Invalid encrypted file path '{}'. OPC relationship files cannot be encrypted (EPX-2605)",
                                path
                            )));
                        }

                        // EPX-2607: Validate file exists
                        let lookup_path = path.trim_start_matches('/');
                        if !package.has_file(lookup_path) {
                            return Err(Error::InvalidSecureContent(format!(
                                "Referenced encrypted file '{}' does not exist in package (EPX-2607)",
                                path
                            )));
                        }

                        // EPX-2607: Validate resourcedata paths are unique (no duplicates)
                        if !encrypted_paths.insert(path.clone()) {
                            return Err(Error::InvalidSecureContent(format!(
                                "Duplicate resourcedata path '{}' in keystore (EPX-2607)",
                                path
                            )));
                        }

                        // EPX-2607: Validate referenced file exists in package
                        // Remove leading slash for package lookup
                        let lookup_path = path.trim_start_matches('/');
                        if !package.has_file(lookup_path) {
                            return Err(Error::InvalidSecureContent(format!(
                                        "Referenced encrypted file '{}' does not exist in package (EPX-2607)",
                                        path
                                    )));
                        }

                        // Add to encrypted_files list (for backward compatibility)
                        if let Some(ref mut sc) = model.secure_content {
                            sc.encrypted_files.push(path.clone());
                        }

                        current_resource_data = Some(ResourceData {
                            path,
                            cek_params: CEKParams {
                                encryption_algorithm: String::new(),
                                compression: DEFAULT_COMPRESSION.to_string(),
                                iv: None,
                                tag: None,
                                aad: None,
                            },
                        });
                    }
                    "cekparams" => {
                        let mut encryption_algorithm = String::new();
                        let mut compression = DEFAULT_COMPRESSION.to_string();

                        for attr in e.attributes() {
                            let attr = attr.map_err(|e| {
                                Error::InvalidXml(format!("Invalid attribute in cekparams: {}", e))
                            })?;
                            let attr_name = std::str::from_utf8(attr.key.as_ref())
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;
                            let attr_value = std::str::from_utf8(&attr.value)
                                .map_err(|e| Error::InvalidXml(e.to_string()))?
                                .to_string();

                            match attr_name {
                                "encryptionalgorithm" => encryption_algorithm = attr_value,
                                "compression" => compression = attr_value,
                                _ => {}
                            }
                        }

                        current_cek_params = Some(CEKParams {
                            encryption_algorithm,
                            compression,
                            iv: None,
                            tag: None,
                            aad: None,
                        });
                    }
                    "iv" => {
                        text_buffer.clear();
                    }
                    "tag" => {
                        text_buffer.clear();
                    }
                    "aad" => {
                        text_buffer.clear();
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().map_err(|e| Error::InvalidXml(e.to_string()))?;
                text_buffer.push_str(&text);
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                let name_str = std::str::from_utf8(name.as_ref())
                    .map_err(|e| Error::InvalidXml(e.to_string()))?;
                let local_name = get_local_name(name_str);

                match local_name {
                    "consumer" => {
                        if let Some(consumer) = current_consumer.take() {
                            if let Some(ref mut sc) = model.secure_content {
                                sc.consumers.push(consumer);
                            }
                        }
                    }
                    "keyvalue" => {
                        if let Some(ref mut consumer) = current_consumer {
                            consumer.key_value = Some(text_buffer.trim().to_string());
                        }
                    }
                    "resourcedatagroup" => {
                        if let Some(group) = current_resource_group.take() {
                            if let Some(ref mut sc) = model.secure_content {
                                sc.resource_data_groups.push(group);
                            }
                        }
                    }
                    "accessright" => {
                        if let Some(access_right) = current_access_right.take() {
                            if let Some(ref mut group) = current_resource_group {
                                group.access_rights.push(access_right);
                            }
                        }
                    }
                    "kekparams" => {
                        if let Some(kek_params) = current_kek_params.take() {
                            if let Some(ref mut access_right) = current_access_right {
                                access_right.kek_params = kek_params;
                            }
                        }
                    }
                    "CipherValue" => {
                        if let Some(ref mut access_right) = current_access_right {
                            access_right.cipher_value = text_buffer.trim().to_string();
                        }
                    }
                    "resourcedata" => {
                        if let Some(resource_data) = current_resource_data.take() {
                            if let Some(ref mut group) = current_resource_group {
                                group.resource_data.push(resource_data);
                            }
                        }
                    }
                    "cekparams" => {
                        if let Some(cek_params) = current_cek_params.take() {
                            if let Some(ref mut resource_data) = current_resource_data {
                                resource_data.cek_params = cek_params;
                            }
                        }
                    }
                    "iv" => {
                        if let Some(ref mut cek_params) = current_cek_params {
                            cek_params.iv = Some(text_buffer.trim().to_string());
                        }
                    }
                    "tag" => {
                        if let Some(ref mut cek_params) = current_cek_params {
                            cek_params.tag = Some(text_buffer.trim().to_string());
                        }
                    }
                    "aad" => {
                        if let Some(ref mut cek_params) = current_cek_params {
                            cek_params.aad = Some(text_buffer.trim().to_string());
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

    // Final validation
    if let Some(ref sc) = model.secure_content {
        // EPX-2602: If we have resourcedatagroups, at least one consumer must be defined
        if !sc.resource_data_groups.is_empty() && sc.consumer_count == 0 {
            return Err(Error::InvalidSecureContent(
                "Keystore has resourcedatagroup elements but no consumer elements (EPX-2602)"
                    .to_string(),
            ));
        }

        // EPX-2602: Check if we have access rights but no consumers
        let has_access_rights = sc
            .resource_data_groups
            .iter()
            .any(|g| !g.access_rights.is_empty());
        if has_access_rights && sc.consumer_count == 0 {
            return Err(Error::InvalidSecureContent(
                "Keystore has accessright elements but no consumer elements (EPX-2602)".to_string(),
            ));
        }

        // EPX-2601: Validate all consumer indices
        for group in &sc.resource_data_groups {
            for access_right in &group.access_rights {
                if access_right.consumer_index >= sc.consumer_count {
                    return Err(Error::InvalidSecureContent(format!(
                        "Invalid consumer index {}. Only {} consumer(s) defined (EPX-2601)",
                        access_right.consumer_index, sc.consumer_count
                    )));
                }
            }
        }
    }

    Ok(())
}

/// Load a file from the package, decrypting if it's an encrypted file
///
/// This function checks if the file is in the encrypted files list, and if so,
/// attempts to decrypt it using the test keys. Otherwise, it loads the file normally.
fn load_file_with_decryption<R: Read + std::io::Seek>(
    package: &mut Package<R>,
    normalized_path: &str,
    display_path: &str,
    model: &Model,
) -> Result<String> {
    // Check if this file is encrypted
    let is_encrypted = model
        .secure_content
        .as_ref()
        .map(|sc| {
            let path_with_slash = format!("/{}", normalized_path);
            sc.encrypted_files.contains(&path_with_slash)
                || sc.encrypted_files.contains(&normalized_path.to_string())
        })
        .unwrap_or(false);

    if !is_encrypted {
        // Load normally
        return package.get_file(normalized_path).map_err(|e| {
            Error::InvalidXml(format!("Failed to load file '{}': {}", display_path, e))
        });
    }

    // File is encrypted - decrypt it
    let secure_content = model
        .secure_content
        .as_ref()
        .ok_or_else(|| Error::InvalidSecureContent("No secure content info".to_string()))?;

    // Load the encrypted file
    let encrypted_data = package.get_file_binary(normalized_path).map_err(|e| {
        Error::InvalidXml(format!(
            "Failed to load encrypted file '{}': {}",
            display_path, e
        ))
    })?;

    // Find the resource data for this file
    let path_with_slash = format!("/{}", normalized_path);
    let resource_data = secure_content
        .resource_data_groups
        .iter()
        .flat_map(|group| &group.resource_data)
        .find(|rd| rd.path == path_with_slash || rd.path == normalized_path)
        .ok_or_else(|| {
            Error::InvalidSecureContent(format!(
                "No resource data found for encrypted file '{}'",
                display_path
            ))
        })?;

    // Find an access right we can use (look for test consumer)
    let (access_right, _consumer_index) = secure_content
        .resource_data_groups
        .iter()
        .find_map(|group| {
            // Check if this group contains our resource
            if group
                .resource_data
                .iter()
                .any(|rd| rd.path == path_with_slash || rd.path == normalized_path)
            {
                // Find an access right for the test consumer
                group
                    .access_rights
                    .iter()
                    .enumerate()
                    .find(|(idx, _)| {
                        if *idx < secure_content.consumers.len() {
                            secure_content.consumers[*idx].consumer_id
                                == crate::decryption::TEST_CONSUMER_ID
                        } else {
                            false
                        }
                    })
                    .map(|(idx, ar)| (ar.clone(), idx))
            } else {
                None
            }
        })
        .ok_or_else(|| {
            Error::InvalidSecureContent(format!(
                "No access right found for test consumer for file '{}'",
                display_path
            ))
        })?;

    // Decrypt the file
    let decrypted = crate::decryption::decrypt_with_test_key(
        &encrypted_data,
        &resource_data.cek_params,
        &access_right,
        secure_content,
    )
    .map_err(|e| {
        Error::InvalidSecureContent(format!("Failed to decrypt file '{}': {}", display_path, e))
    })?;

    // Convert to string
    String::from_utf8(decrypted).map_err(|e| {
        Error::InvalidXml(format!(
            "Decrypted file '{}' is not valid UTF-8: {}",
            display_path, e
        ))
    })
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
    // Type alias for complex nested vector to improve readability
    type SliceRefInfo = (String, String, usize);
    type StackSliceRefs = (usize, Vec<SliceRefInfo>);

    // Collect information needed for loading before we start mutating
    let mut slices_to_load: Vec<StackSliceRefs> = Vec::new();

    for (stack_idx, slice_stack) in model.resources.slice_stacks.iter().enumerate() {
        let mut refs_for_stack = Vec::new();
        for slice_ref in &slice_stack.slice_refs {
            let normalized_path = if slice_ref.slicepath.starts_with('/') {
                slice_ref.slicepath[1..].to_string()
            } else {
                slice_ref.slicepath.clone()
            };
            refs_for_stack.push((
                normalized_path,
                slice_ref.slicepath.clone(),
                slice_ref.slicestackid,
            ));
        }
        if !refs_for_stack.is_empty() {
            slices_to_load.push((stack_idx, refs_for_stack));
        }
    }

    // Now load and process each slice reference
    for (stack_idx, refs) in slices_to_load {
        for (normalized_path, display_path, expected_stack_id) in refs {
            // Load the slice file from the package (decrypt if encrypted)
            let slice_xml =
                load_file_with_decryption(package, &normalized_path, &display_path, model)?;

            // Parse the slice file to extract slices and objects
            let (slices, objects) = parse_slice_file_with_objects(&slice_xml, expected_stack_id)?;

            // Add the slices to this slice stack
            model.resources.slice_stacks[stack_idx]
                .slices
                .extend(slices);

            // Merge objects from the external file into the main model
            model.resources.objects.extend(objects);
        }

        // Clear the slice_refs for this stack
        model.resources.slice_stacks[stack_idx].slice_refs.clear();
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

/// Parse displacement triangle element attributes
pub(crate) fn parse_displacement_triangle<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<DisplacementTriangle> {
    let attrs = parse_attributes(reader, e)?;

    // Validate only allowed attributes are present
    // Per Displacement Extension spec: v1, v2, v3, pid, pindex, p1, p2, p3, did, d1, d2, d3
    validate_attributes(
        &attrs,
        &[
            "v1", "v2", "v3", "pid", "pindex", "p1", "p2", "p3", "did", "d1", "d2", "d3",
        ],
        "triangle",
    )?;

    let v1 = attrs
        .get("v1")
        .ok_or_else(|| Error::InvalidXml("Displacement triangle missing v1 attribute".to_string()))?
        .parse::<usize>()?;

    let v2 = attrs
        .get("v2")
        .ok_or_else(|| Error::InvalidXml("Displacement triangle missing v2 attribute".to_string()))?
        .parse::<usize>()?;

    let v3 = attrs
        .get("v3")
        .ok_or_else(|| Error::InvalidXml("Displacement triangle missing v3 attribute".to_string()))?
        .parse::<usize>()?;

    let mut triangle = DisplacementTriangle::new(v1, v2, v3);

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

    // Parse displacement-specific attributes
    if let Some(did) = attrs.get("did") {
        triangle.did = Some(did.parse::<usize>()?);
    }

    if let Some(d1) = attrs.get("d1") {
        triangle.d1 = Some(d1.parse::<usize>()?);
    }

    if let Some(d2) = attrs.get("d2") {
        triangle.d2 = Some(d2.parse::<usize>()?);
    }

    if let Some(d3) = attrs.get("d3") {
        triangle.d3 = Some(d3.parse::<usize>()?);
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

        // If r2 is specified, r1 must also be specified (per 3MF Beam Lattice spec)
        if beam.r1.is_none() {
            return Err(Error::InvalidXml(
                "Beam attribute r2 is specified but r1 is not. When specifying r2, r1 must also be provided.".to_string()
            ));
        }
    }

    // Parse cap1 attribute (optional, defaults to beamset cap mode)
    if let Some(cap1_str) = attrs.get("cap1") {
        beam.cap1 = Some(cap1_str.parse()?);
    }

    // Parse cap2 attribute (optional, defaults to beamset cap mode)
    if let Some(cap2_str) = attrs.get("cap2") {
        beam.cap2 = Some(cap2_str.parse()?);
    }

    // Parse pid attribute (optional) - material/property group ID
    if let Some(pid_str) = attrs.get("pid") {
        beam.property_id = Some(pid_str.parse::<u32>()?);
    }

    // Parse p1 attribute (optional) - property index at v1
    if let Some(p1_str) = attrs.get("p1") {
        beam.p1 = Some(p1_str.parse::<u32>()?);
    }

    // Parse p2 attribute (optional) - property index at v2
    if let Some(p2_str) = attrs.get("p2") {
        beam.p2 = Some(p2_str.parse::<u32>()?);

        // If p2 is specified, p1 must also be specified (per 3MF spec convention)
        if beam.p1.is_none() {
            return Err(Error::InvalidXml(
                "Beam attribute p2 is specified but p1 is not. When specifying p2, p1 must also be provided.".to_string()
            ));
        }
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

/// Validate that no extension appears in both requiredextensions and recommendedextensions
fn validate_no_duplicate_extensions(
    required_str: &str,
    recommended_str: &str,
    namespaces: &HashMap<String, String>,
) -> Result<()> {
    // Parse both extension lists to get their resolved namespaces
    let required_items: Vec<&str> = required_str.split_whitespace().collect();
    let recommended_items: Vec<&str> = recommended_str.split_whitespace().collect();

    // Resolve each item to its namespace URI
    let mut required_namespaces = HashSet::new();
    for item in required_items {
        // If it's a namespace prefix, resolve it
        if let Some(uri) = namespaces.get(item) {
            required_namespaces.insert(uri.as_str());
        } else {
            // It's already a URI
            required_namespaces.insert(item);
        }
    }

    // Check recommended items against required
    for item in recommended_items {
        let resolved = if let Some(uri) = namespaces.get(item) {
            uri.as_str()
        } else {
            item
        };

        if required_namespaces.contains(resolved) {
            return Err(Error::InvalidXml(format!(
                "Extension '{}' cannot appear in both requiredextensions and recommendedextensions",
                item
            )));
        }
    }

    Ok(())
}

/// Validate that a triangle index is within bounds of the mesh
fn validate_triangle_index(mesh: &Mesh, index: usize, context: &str) -> Result<()> {
    if index >= mesh.triangles.len() {
        return Err(Error::InvalidXml(format!(
            "{} triangle index {} is out of bounds (mesh has {} triangles)",
            context,
            index,
            mesh.triangles.len()
        )));
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

/// Validate that attribute values don't contain namespace prefixes
/// Per 3MF spec, namespace prefixes can only be used in element/attribute names, not in values
/// Exception: Some attributes have QName type (like 'identifier') which DO allow prefixes
pub(crate) fn validate_attribute_values(
    attrs: &HashMap<String, String>,
    declared_namespaces: &HashMap<String, String>,
) -> Result<()> {
    for (key, value) in attrs {
        // Skip namespace declarations and XML attributes
        if key.starts_with("xmlns") || key.starts_with("xml:") {
            continue;
        }

        // Skip attributes that are defined as QName type in the spec (they can have namespace prefixes)
        // The trianglesets extension defines 'identifier' as QName
        if key == "identifier" || key.ends_with(":identifier") {
            continue;
        }

        // Skip attributes that legitimately contain colons (URIs, etc.)
        // Only check non-URI-like values
        if value.contains("://") {
            // This looks like a URI, skip it
            continue;
        }

        // Check if the value starts with a declared namespace prefix followed by colon
        if let Some(colon_pos) = value.find(':') {
            let potential_prefix = &value[..colon_pos];
            if declared_namespaces.contains_key(potential_prefix) {
                return Err(Error::InvalidXml(format!(
                    "Attribute '{}' has value '{}' which uses namespace prefix '{}:'. \
                    Namespace prefixes are not allowed in attribute values per 3MF specification",
                    key, value, potential_prefix
                )));
            }
        }
    }
    Ok(())
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

/// Validate that external file paths referenced in boolean operations exist in the package
///
/// Per 3MF Boolean Operations Extension spec:
/// - The path attribute references objects in non-root model files
/// - Path is an absolute path from the root of the 3MF container
/// - This validation ensures referenced files exist and contain the referenced objects
fn validate_boolean_external_paths<R: Read + std::io::Seek>(
    package: &mut Package<R>,
    model: &Model,
) -> Result<()> {
    // Cache to avoid re-parsing the same external file multiple times
    let mut external_file_cache: HashMap<String, Vec<usize>> = HashMap::new();

    for object in &model.resources.objects {
        if let Some(ref boolean_shape) = object.boolean_shape {
            // Check if booleanshape references an external file
            if let Some(ref path) = boolean_shape.path {
                // Normalize path: remove leading slash if present
                let normalized_path = path.trim_start_matches('/');

                // Skip validation for encrypted files (Secure Content extension)
                // Encrypted files cannot be parsed, so we can't validate object IDs
                let is_encrypted = model
                    .secure_content
                    .as_ref()
                    .map(|sc| {
                        sc.encrypted_files.iter().any(|encrypted_path| {
                            // Compare normalized paths (both without leading slash)
                            let enc_normalized = encrypted_path.trim_start_matches('/');
                            enc_normalized == normalized_path
                        })
                    })
                    .unwrap_or(false);

                if is_encrypted {
                    // Skip validation for encrypted files - they can't be parsed
                    continue;
                }

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

                // Validate that the referenced object ID exists in the external file
                validate_external_object_id(
                    package,
                    normalized_path,
                    boolean_shape.objectid,
                    object.id,
                    "booleanshape base",
                    &mut external_file_cache,
                    model,
                )?;
            }

            // Check if boolean operands reference external files
            for operand in &boolean_shape.operands {
                if let Some(ref path) = operand.path {
                    // Normalize path: remove leading slash if present
                    let normalized_path = path.trim_start_matches('/');

                    // Skip validation for encrypted files (Secure Content extension)
                    // Encrypted files cannot be parsed, so we can't validate object IDs
                    let is_encrypted = model
                        .secure_content
                        .as_ref()
                        .map(|sc| {
                            sc.encrypted_files.iter().any(|encrypted_path| {
                                // Compare normalized paths (both without leading slash)
                                let enc_normalized = encrypted_path.trim_start_matches('/');
                                enc_normalized == normalized_path
                            })
                        })
                        .unwrap_or(false);

                    if is_encrypted {
                        // Skip validation for encrypted files - they can't be parsed
                        continue;
                    }

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

                    // Validate that the referenced object ID exists in the external file
                    validate_external_object_id(
                        package,
                        normalized_path,
                        operand.objectid,
                        object.id,
                        "boolean operand",
                        &mut external_file_cache,
                        model,
                    )?;
                }
            }
        }
    }

    Ok(())
}

/// Validate that an object ID exists in an external model file
///
/// Uses a cache to avoid re-parsing the same file multiple times
fn validate_external_object_id<R: Read + std::io::Seek>(
    package: &mut Package<R>,
    file_path: &str,
    object_id: usize,
    referring_object_id: usize,
    reference_type: &str,
    cache: &mut HashMap<String, Vec<usize>>,
    model: &Model,
) -> Result<()> {
    // Check cache first and load if needed
    if !cache.contains_key(file_path) {
        // Load and parse the external model file (decrypt if encrypted)
        let external_xml = load_file_with_decryption(package, file_path, file_path, model)?;

        // Parse just enough to extract object IDs
        let mut reader = Reader::from_str(&external_xml);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::with_capacity(XML_BUFFER_CAPACITY);
        let mut ids = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    let name_str = std::str::from_utf8(name.as_ref())
                        .map_err(|e| Error::InvalidXml(e.to_string()))?;
                    let local_name = get_local_name(name_str);

                    if local_name == "object" {
                        // Extract the id attribute
                        for attr in e.attributes() {
                            let attr = attr.map_err(|e| Error::InvalidXml(e.to_string()))?;
                            let attr_name = std::str::from_utf8(attr.key.as_ref())
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;

                            if attr_name == "id" {
                                let id_str = std::str::from_utf8(&attr.value)
                                    .map_err(|e| Error::InvalidXml(e.to_string()))?;
                                if let Ok(id) = id_str.parse::<usize>() {
                                    ids.push(id);
                                }
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }

        // Cache the results for future use
        cache.insert(file_path.to_string(), ids);
    }

    // Get the cached object IDs
    let object_ids = cache.get(file_path).unwrap();

    // Check if the referenced object ID exists
    if !object_ids.contains(&object_id) {
        // Limit displayed IDs to avoid overwhelming error messages
        let display_ids: Vec<usize> = object_ids
            .iter()
            .take(MAX_DISPLAYED_OBJECT_IDS)
            .copied()
            .collect();
        let id_display = if object_ids.len() > MAX_DISPLAYED_OBJECT_IDS {
            format!("{:?} ... ({} total)", display_ids, object_ids.len())
        } else {
            format!("{:?}", display_ids)
        };

        return Err(Error::InvalidModel(format!(
            "Object {}: {} references object ID {} in external file '{}', but that object does not exist.\n\
             Available object IDs in external file: {}\n\
             Check that the referenced object ID is correct.",
            referring_object_id, reference_type, object_id, file_path, id_display
        )));
    }

    Ok(())
}

/// Validate that an encrypted file can be loaded and decrypted
///
/// This function attempts to load and decrypt an encrypted external file to ensure:
/// 1. The file exists in the package
/// 2. The keystore has a valid consumer that can decrypt it
/// 3. The decryption succeeds with the test keys
///
/// This is crucial for negative test validation - files that can't be decrypted
/// should fail during parsing, not be silently skipped.
fn validate_encrypted_file_can_be_loaded<R: Read + std::io::Seek>(
    package: &mut Package<R>,
    normalized_path: &str,
    display_path: &str,
    model: &Model,
    context: &str,
) -> Result<()> {
    // Check if file exists
    if !package.has_file(normalized_path) {
        return Err(Error::InvalidModel(format!(
            "{}: References non-existent encrypted file: {}\n\
             The p:path attribute must reference a valid encrypted file in the 3MF package.",
            context, display_path
        )));
    }

    // Attempt to load and decrypt the file
    // This will fail if:
    // - The consumer doesn't match test keys (consumerid != "test3mf01")
    // - The keyid doesn't match (keyid != "test3mfkek01")
    // - The consumer has no keyid when one is required
    // - Any other decryption-related issue
    let _decrypted_content =
        load_file_with_decryption(package, normalized_path, display_path, model)?;

    // If we got here, decryption succeeded - the file is valid
    Ok(())
}

/// Validate production extension external file references
///
/// Per 3MF Production Extension spec:
/// - Components and build items with p:path must reference existing files
/// - Referenced object IDs must exist in those files
/// - UUIDs should match when specified
fn validate_production_external_paths<R: Read + std::io::Seek>(
    package: &mut Package<R>,
    model: &Model,
) -> Result<()> {
    // Cache to avoid re-parsing the same external file multiple times
    let mut external_file_cache: HashMap<String, Vec<(usize, Option<String>)>> = HashMap::new();

    // Validate build item external references
    for (idx, item) in model.build.items.iter().enumerate() {
        if let Some(ref path) = item.production_path {
            // Normalize path: remove leading slash if present
            let normalized_path = path.trim_start_matches('/');

            // Skip validation for encrypted files (Secure Content extension)
            // Encrypted files cannot be parsed, so we can't validate object IDs
            let is_encrypted = model
                .secure_content
                .as_ref()
                .map(|sc| {
                    sc.encrypted_files.iter().any(|encrypted_path| {
                        // Compare normalized paths (both without leading slash)
                        let enc_normalized = encrypted_path.trim_start_matches('/');
                        enc_normalized == normalized_path
                    })
                })
                .unwrap_or(false);

            if is_encrypted {
                // For encrypted files, attempt to validate that we can decrypt them
                // This ensures the keystore has valid consumers/keys
                validate_encrypted_file_can_be_loaded(
                    package,
                    normalized_path,
                    path,
                    model,
                    &format!("Build item {}", idx),
                )?;
                continue;
            }

            // Check if file exists
            if !package.has_file(normalized_path) {
                return Err(Error::InvalidModel(format!(
                    "Build item {}: References non-existent external file: {}\n\
                     The p:path attribute must reference a valid model file in the 3MF package.\n\
                     Check that:\n\
                     - The file exists in the package\n\
                     - The path is correct (case-sensitive)\n\
                     - The path format follows 3MF conventions (e.g., /3D/filename.model)",
                    idx, path
                )));
            }

            // Validate that the referenced object ID exists in the external file
            validate_external_object_reference(
                package,
                normalized_path,
                item.objectid,
                &item.production_uuid,
                &format!("Build item {}", idx),
                &mut external_file_cache,
                model,
            )?;
        }
    }

    // Validate component external references
    for object in &model.resources.objects {
        for (comp_idx, component) in object.components.iter().enumerate() {
            if let Some(ref prod_info) = component.production {
                if let Some(ref path) = prod_info.path {
                    // Normalize path: remove leading slash if present
                    let normalized_path = path.trim_start_matches('/');

                    // Skip validation for encrypted files (Secure Content extension)
                    // Encrypted files cannot be parsed, so we can't validate object IDs
                    let is_encrypted = model
                        .secure_content
                        .as_ref()
                        .map(|sc| {
                            sc.encrypted_files.iter().any(|encrypted_path| {
                                // Compare normalized paths (both without leading slash)
                                let enc_normalized = encrypted_path.trim_start_matches('/');
                                enc_normalized == normalized_path
                            })
                        })
                        .unwrap_or(false);

                    if is_encrypted {
                        // For encrypted files, attempt to validate that we can decrypt them
                        // This ensures the keystore has valid consumers/keys
                        validate_encrypted_file_can_be_loaded(
                            package,
                            normalized_path,
                            path,
                            model,
                            &format!("Object {}, Component {}", object.id, comp_idx),
                        )?;
                        continue;
                    }

                    // Check if file exists
                    if !package.has_file(normalized_path) {
                        return Err(Error::InvalidModel(format!(
                            "Object {}, Component {}: References non-existent external file: {}\n\
                             The p:path attribute must reference a valid model file in the 3MF package.\n\
                             Check that:\n\
                             - The file exists in the package\n\
                             - The path is correct (case-sensitive)\n\
                             - The path format follows 3MF conventions (e.g., /3D/filename.model)",
                            object.id, comp_idx, path
                        )));
                    }

                    // Validate that the referenced object ID exists in the external file
                    validate_external_object_reference(
                        package,
                        normalized_path,
                        component.objectid,
                        &prod_info.uuid,
                        &format!("Object {}, Component {}", object.id, comp_idx),
                        &mut external_file_cache,
                        model,
                    )?;
                }
            }
        }
    }

    Ok(())
}

/// Validate that an object ID (and optionally UUID) exists in an external model file
///
/// Uses a cache to avoid re-parsing the same file multiple times
/// Cache stores: (object_id, optional_uuid)
fn validate_external_object_reference<R: Read + std::io::Seek>(
    package: &mut Package<R>,
    file_path: &str,
    object_id: usize,
    _expected_uuid: &Option<String>,
    reference_context: &str,
    cache: &mut HashMap<String, Vec<(usize, Option<String>)>>,
    model: &Model,
) -> Result<()> {
    // Check cache first and get object info
    if !cache.contains_key(file_path) {
        // Load and parse the external model file (decrypt if encrypted)
        let external_xml = load_file_with_decryption(package, file_path, file_path, model)?;

        // Parse to extract object IDs and UUIDs
        let mut reader = Reader::from_str(&external_xml);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::with_capacity(XML_BUFFER_CAPACITY);
        let mut info = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    let name_str = std::str::from_utf8(name.as_ref())
                        .map_err(|e| Error::InvalidXml(e.to_string()))?;
                    let local_name = get_local_name(name_str);

                    if local_name == "object" {
                        let mut obj_id = None;
                        let mut obj_uuid = None;

                        // Extract id and p:UUID attributes
                        for attr in e.attributes() {
                            let attr = attr.map_err(|e| Error::InvalidXml(e.to_string()))?;
                            let attr_name = std::str::from_utf8(attr.key.as_ref())
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;

                            match attr_name {
                                "id" => {
                                    let id_str = std::str::from_utf8(&attr.value)
                                        .map_err(|e| Error::InvalidXml(e.to_string()))?;
                                    obj_id = id_str.parse::<usize>().ok();
                                }
                                "p:UUID" => {
                                    let uuid_str = std::str::from_utf8(&attr.value)
                                        .map_err(|e| Error::InvalidXml(e.to_string()))?;
                                    obj_uuid = Some(uuid_str.to_string());
                                }
                                _ => {}
                            }
                        }

                        if let Some(id) = obj_id {
                            info.push((id, obj_uuid));
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(Error::Xml(e)),
                _ => {}
            }
            buf.clear();
        }

        // Cache the results for future use
        cache.insert(file_path.to_string(), info);
    }

    // Get the cached info
    let object_info = cache.get(file_path).unwrap();

    // Check if the referenced object ID exists
    let found_obj = object_info.iter().find(|(id, _)| *id == object_id);

    if found_obj.is_none() {
        // Object ID not found
        let available_ids: Vec<usize> = object_info
            .iter()
            .map(|(id, _)| *id)
            .take(MAX_DISPLAYED_OBJECT_IDS)
            .collect();
        let id_display = if object_info.len() > MAX_DISPLAYED_OBJECT_IDS {
            format!("{:?} ... ({} total)", available_ids, object_info.len())
        } else {
            format!("{:?}", available_ids)
        };

        return Err(Error::InvalidModel(format!(
            "{}: References object ID {} in external file '{}', but that object does not exist.\n\
             Available object IDs in external file: {}\n\
             Check that the referenced object ID is correct.",
            reference_context, object_id, file_path, id_display
        )));
    }

    // If we have an expected UUID, validate it matches
    // NOTE: Per official 3MF test suite (P_XXX_2203_04_Prod_Ext.3mf, P_OPX_3002_03_production.3mf),
    // UUID mismatches between component p:UUID and referenced object p:UUID are allowed.
    // The component's p:UUID is for identifying the component instance, not for matching
    // the referenced object's UUID. UUID validation is therefore commented out.
    /*
    if let Some(ref expected) = expected_uuid {
        if let Some((_, Some(ref actual_uuid))) = found_obj {
            if expected != actual_uuid {
                return Err(Error::InvalidModel(format!(
                    "{}: UUID mismatch for object {} in external file '{}'.\n\
                     Expected p:UUID='{}' but found p:UUID='{}'.\n\
                     UUIDs must match when referencing external objects.",
                    reference_context, object_id, file_path, expected, actual_uuid
                )));
            }
        }
    }
    */

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
