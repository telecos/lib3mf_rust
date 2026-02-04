//! XML parsing for 3MF model files

mod beam_lattice;
mod boolean_ops;
mod core;
mod displacement;
mod material;
mod production;
mod secure_content;
mod slice;

use crate::error::{Error, Result};
use crate::model::*;
use crate::opc::Package;
use crate::validator;
use boolean_ops::validate_boolean_external_paths;
use production::validate_production_external_paths;
use quick_xml::Reader;
use quick_xml::events::Event;
use secure_content::{load_file_with_decryption, load_keystore};
use std::collections::{HashMap, HashSet};
use std::io::Read;

// Import functions from submodules for internal use
use beam_lattice::{parse_ball, parse_beam, parse_beamlattice_start};
use core::parse_component;
use displacement::{
    parse_disp2dcoord, parse_disp2dgroup_start, parse_displacement2d, parse_normvector,
    parse_normvectorgroup_start, validate_displacement_namespace_prefix,
};
use material::{
    parse_base_element, parse_base_material, parse_basematerials_start, parse_color_element,
    parse_colorgroup_start, parse_composite, parse_compositematerials_start, parse_multi,
    parse_multiproperties_start, parse_tex2coord, parse_texture2d, parse_texture2dgroup_start,
    validate_texture_file_paths,
};
use slice::{
    load_slice_references, parse_slice_polygon_start, parse_slice_segment, parse_slice_start,
    parse_slice_vertex, parse_sliceref, parse_slicestack_start,
};

// Re-export public functions to maintain API compatibility
pub use core::{parse_build_item, parse_object, parse_triangle, parse_vertex};
pub use displacement::parse_displacement_triangle;

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

    // N_SPX_0417_01, N_SPX_0419_01: Validate thumbnails are not in model-level relationships
    // Per 3MF spec, thumbnails MUST be at package level, not part/model level
    package.validate_no_model_level_thumbnails()?;

    // Clone config before moving it to parse_model_xml_with_config
    let config_clone = config.clone();
    let model_xml = package.get_model()?;
    let mut model = parse_model_xml_with_config(&model_xml, config)?;

    // Add thumbnail metadata to model
    model.thumbnail = thumbnail;

    // Load keystore to identify encrypted files (SecureContent extension)
    // This MUST happen before validation so that component validation can
    // skip components referencing encrypted files
    load_keystore(&mut package, &mut model, &config_clone)?;

    // Load external slice files if any slice stacks have references
    load_slice_references(&mut package, &mut model, &config_clone)?;

    // Validate boolean operation external paths before general validation
    // This requires access to the package to check if referenced files exist
    validate_boolean_external_paths(&mut package, &model, &config_clone)?;

    // Validate production extension external paths
    // This checks that external files exist and referenced objects/UUIDs are valid
    validate_production_external_paths(&mut package, &model, &config_clone)?;

    // Validate texture paths exist in the package (N_XPM_0610_01)
    // This must be done before general validation since it requires package access
    validate_texture_file_paths(&mut package, &model)?;

    // Validate the model AFTER loading keystore and slices
    // This ensures validation can check encrypted file references correctly
    validator::validate_model_with_config(&model, &config_clone)?;

    // Call post_parse hooks for all registered extensions
    // This allows extensions to perform post-processing after parsing and validation
    config_clone.registry().post_parse_all(&mut model)?;

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

/// Parse the 3D model XML content with custom configuration
///
/// This is primarily used for testing. For production use, use `Model::from_reader_with_config()`.
///
/// Note: This function is public to enable integration testing, but marked #[doc(hidden)]
/// to discourage use in production code. We can't use #[cfg(test)] because integration
/// tests in the tests/ directory are compiled separately and wouldn't have access.
#[doc(hidden)]
pub fn parse_model_xml_with_config(xml: &str, config: ParserConfig) -> Result<Model> {
    // Check for DTD declarations before parsing for security
    // DTD declarations can lead to XXE (XML External Entity) attacks
    // Check first ~2000 characters where DOCTYPE declarations typically appear
    let check_len = xml.len().min(2000);
    let xml_start = &xml[..check_len];
    let xml_start_lower = xml_start.to_lowercase();

    if xml_start_lower.contains("<!doctype") {
        return Err(Error::InvalidXml(
            "DTD declarations are not allowed in 3MF files for security reasons".to_string(),
        ));
    }

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

    // Track parse order for resource ordering validation
    let mut resource_parse_order: usize = 0;

    loop {
        let event_result = reader.read_event_into(&mut buf);
        let is_empty_element = matches!(&event_result, Ok(Event::Empty(_)));

        match event_result {
            Ok(Event::Decl(_)) => {
                // XML declaration is allowed
            }
            Ok(Event::DocType(_)) => {
                // DTD declarations are not allowed (security risk)
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
                                        "micron" | "millimeter" | "centimeter" | "inch"
                                        | "foot" | "meter" => model.unit = value.to_string(),
                                        _ => {
                                            return Err(Error::InvalidXml(format!(
                                                "Invalid unit '{}'. Must be one of: micron, millimeter, centimeter, inch, foot, meter",
                                                value
                                            )));
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
                        // Pre-allocate with reasonable capacity to reduce reallocations
                        // Most meshes have roughly 2x triangles as vertices
                        current_mesh = Some(Mesh::with_capacity(1024, 2048));
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
                        validate_displacement_namespace_prefix(name_str, "vertex")?;
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
                        validate_displacement_namespace_prefix(name_str, "triangles")?;
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
                        validate_displacement_namespace_prefix(name_str, "triangle")?;
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
                            if let Some(did) = triangle.did
                                && !declared_disp2dgroup_ids.contains(&did)
                            {
                                return Err(Error::InvalidXml(format!(
                                    "Triangle element references Disp2DGroup with ID {} which has not been declared yet. \
                                         Resources must be declared before they are referenced.",
                                    did
                                )));
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
                        let group = parse_basematerials_start(&reader, e, resource_parse_order)?;
                        resource_parse_order += 1;
                        current_basematerialgroup = Some(group);
                    }
                    "base" if in_basematerials => {
                        // Materials within basematerials group
                        if let Some(ref mut group) = current_basematerialgroup {
                            let base = parse_base_element(&reader, e)?;
                            group.materials.push(base);
                        }

                        // Still parse to materials list for backward compatibility
                        let material = parse_base_material(&reader, e, material_index)?;
                        model.resources.materials.push(material);
                        material_index += 1;
                    }
                    "colorgroup" if in_resources => {
                        in_colorgroup = true;
                        let group = parse_colorgroup_start(&reader, e, resource_parse_order)?;
                        resource_parse_order += 1;
                        current_colorgroup = Some(group);
                    }
                    "color" if in_colorgroup => {
                        if let Some(ref mut colorgroup) = current_colorgroup {
                            let color = parse_color_element(&reader, e, colorgroup.id)?;
                            colorgroup.colors.push(color);
                        }
                    }
                    "texture2d" if in_resources => {
                        let texture = parse_texture2d(&reader, e, resource_parse_order)?;
                        resource_parse_order += 1;
                        model.resources.texture2d_resources.push(texture);
                    }
                    "texture2dgroup" if in_resources => {
                        in_texture2dgroup = true;
                        let group = parse_texture2dgroup_start(&reader, e, resource_parse_order)?;
                        resource_parse_order += 1;
                        current_texture2dgroup = Some(group);
                    }
                    "tex2coord" if in_texture2dgroup => {
                        if let Some(ref mut group) = current_texture2dgroup {
                            let coord = parse_tex2coord(&reader, e)?;
                            group.tex2coords.push(coord);
                        }
                    }
                    "compositematerials" if in_resources => {
                        in_compositematerials = true;
                        let group =
                            parse_compositematerials_start(&reader, e, resource_parse_order)?;
                        resource_parse_order += 1;
                        current_compositematerials = Some(group);
                    }
                    "composite" if in_compositematerials => {
                        if let Some(ref mut group) = current_compositematerials {
                            let composite = parse_composite(&reader, e)?;
                            group.composites.push(composite);
                        }
                    }
                    "multiproperties" if in_resources => {
                        in_multiproperties = true;
                        let multi = parse_multiproperties_start(&reader, e, resource_parse_order)?;
                        resource_parse_order += 1;
                        current_multiproperties = Some(multi);
                    }
                    "multi" if in_multiproperties => {
                        if let Some(ref mut group) = current_multiproperties {
                            let multi = parse_multi(&reader, e)?;
                            group.multis.push(multi);
                        }
                    }
                    "displacement2d" if in_resources => {
                        let disp = parse_displacement2d(&reader, e)?;
                        let id = disp.id;
                        model.resources.displacement_maps.push(disp);
                        declared_displacement2d_ids.insert(id); // Track for forward-reference validation
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

                        let beamset = parse_beamlattice_start(&reader, e)?;
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
                        if let Some(ref mut beamset) = current_beamset {
                            let ball = parse_ball(&reader, e)?;
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
                        let group = parse_normvectorgroup_start(&reader, e)?;
                        current_normvectorgroup = Some(group);
                    }
                    "normvector" if in_normvectorgroup => {
                        if let Some(ref mut nvgroup) = current_normvectorgroup {
                            let vector = parse_normvector(&reader, e)?;
                            nvgroup.vectors.push(vector);
                        }
                    }
                    "disp2dgroup" if in_resources => {
                        in_disp2dgroup = true;
                        let group = parse_disp2dgroup_start(
                            &reader,
                            e,
                            &declared_displacement2d_ids,
                            &declared_normvectorgroup_ids,
                        )?;
                        current_disp2dgroup = Some(group);
                    }
                    "disp2dcoord" if in_disp2dgroup => {
                        if let Some(ref mut d2dgroup) = current_disp2dgroup {
                            let coord = parse_disp2dcoord(&reader, e)?;
                            d2dgroup.coords.push(coord);
                        }
                    }
                    "slicestack" if in_resources => {
                        in_slicestack = true;
                        let stack = parse_slicestack_start(&reader, e)?;
                        current_slicestack = Some(stack);
                    }
                    "slice" if in_slicestack => {
                        in_slice = true;
                        let slice = parse_slice_start(&reader, e)?;

                        // For self-closing empty slice tags like <s:slice ztop="100.060"/>,
                        // immediately push the slice since there won't be an End event
                        if is_empty_element {
                            if let Some(ref mut slicestack) = current_slicestack {
                                slicestack.slices.push(slice);
                            }
                            in_slice = false;
                        } else {
                            current_slice = Some(slice);
                        }
                    }
                    "sliceref" if in_slicestack => {
                        let slice_ref = parse_sliceref(&reader, e)?;
                        if let Some(ref mut slicestack) = current_slicestack {
                            slicestack.slice_refs.push(slice_ref);
                        }
                    }
                    "vertices" if in_slice => {
                        in_slice_vertices = true;
                    }
                    "vertex" if in_slice_vertices => {
                        let vertex = parse_slice_vertex(&reader, e)?;
                        if let Some(ref mut slice) = current_slice {
                            slice.vertices.push(vertex);
                        }
                    }
                    "polygon" if in_slice => {
                        in_slice_polygon = true;
                        let polygon = parse_slice_polygon_start(&reader, e)?;
                        current_slice_polygon = Some(polygon);
                    }
                    "segment" if in_slice_polygon => {
                        let segment = parse_slice_segment(&reader, e)?;
                        if let Some(ref mut polygon) = current_slice_polygon {
                            polygon.segments.push(segment);
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
                        if let Some(name) = attrs.get("name")
                            && name.trim().is_empty()
                        {
                            return Err(Error::InvalidXml(
                                "triangleset name attribute cannot be empty".to_string(),
                            ));
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
                            // Set parse order for resource ordering validation
                            obj.parse_order = resource_parse_order;
                            resource_parse_order += 1;
                            model.resources.objects.push(obj);
                        }
                    }
                    "mesh" => {
                        // Mesh parsing complete
                    }
                    "displacementmesh" => {
                        in_displacement_mesh = false;
                        // Per DPX spec 4.0: Object containing displacementmesh MUST be type="model"
                        if let Some(ref obj) = current_object
                            && obj.object_type != ObjectType::Model
                        {
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
                        if let Some(beamset) = current_beamset.take()
                            && let Some(ref mut mesh) = current_mesh
                        {
                            mesh.beamset = Some(beamset);
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
                        if let Some(slice) = current_slice.take()
                            && let Some(ref mut slicestack) = current_slicestack
                        {
                            slicestack.slices.push(slice);
                        }
                        in_slice = false;
                    }
                    "vertices" => {
                        in_slice_vertices = false;
                    }
                    "polygon" => {
                        if let Some(polygon) = current_slice_polygon.take()
                            && let Some(ref mut slice) = current_slice
                        {
                            slice.polygons.push(polygon);
                        }
                        in_slice_polygon = false;
                    }
                    "booleanshape" => {
                        if let Some(shape) = current_boolean_shape.take()
                            && let Some(ref mut obj) = current_object
                        {
                            obj.boolean_shape = Some(shape);
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

/// Validate that all texture2d resources reference files that exist in the package
///
/// Per 3MF Materials Extension spec, texture paths must point to valid files in the package.
/// This function checks that the file referenced by each texture2d resource actually exists.
///
/// N_XPM_0610_01: Texture path must reference an existing file in the package

#[cfg(test)]
mod tests {
    use super::*;

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
            [
                1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 10.0, 20.0, 30.0
            ]
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
