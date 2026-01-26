//! Boolean Operations extension validation
//!
//! This module handles validation of the Boolean Operations extension in 3MF files.
//! The Boolean Operations extension allows defining boolean operations (union, difference,
//! intersection) on mesh objects, including references to objects in external model files
//! within the same 3MF package.

use crate::error::{Error, Result};
use crate::model::{Model, ParserConfig};
use crate::opc::Package;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::{HashMap, HashSet};
use std::io::Read;

use super::secure_content::load_file_with_decryption;
use super::{get_local_name, parse_model_xml_with_config};

/// Maximum number of object IDs to display in error messages
const MAX_DISPLAYED_OBJECT_IDS: usize = 20;

/// Default buffer capacity for XML parsing (4KB)
const XML_BUFFER_CAPACITY: usize = 4096;

/// Validate boolean operations external paths
///
/// This function validates all external references in boolean shapes and their operands.
/// For each external reference (path + objectid), it ensures:
///
/// 1. The referenced file exists in the package
/// 2. The referenced object ID exists in that file (unless encrypted)
///
/// Special handling for encrypted files (Secure Content extension):
/// - Encrypted files cannot be parsed to validate object IDs
/// - We skip validation for encrypted files
///
/// # Arguments
///
/// * `package` - The 3MF package containing all files
/// * `model` - The parsed model with boolean operations
///
/// # Returns
///
/// * `Ok(())` if all external references are valid
/// * `Err` if any reference is invalid or points to a missing file/object
pub(super) fn validate_boolean_external_paths<R: Read + std::io::Seek>(
    package: &mut Package<R>,
    model: &Model,
    config: &ParserConfig,
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
                    config,
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
                        config,
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
    config: &ParserConfig,
) -> Result<()> {
    // Check cache first and load if needed
    if !cache.contains_key(file_path) {
        // Load and parse the external model file (decrypt if encrypted)
        let external_xml = load_file_with_decryption(package, file_path, file_path, model, config)?;

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

/// Validate that an object ID (and optionally UUID) exists in an external model file
///
/// Uses a cache to avoid re-parsing the same file multiple times
/// Cache stores: (object_id, optional_uuid)
pub(super) fn validate_external_object_reference<R: Read + std::io::Seek>(
    package: &mut Package<R>,
    file_path: &str,
    object_id: usize,
    _expected_uuid: &Option<String>,
    reference_context: &str,
    cache: &mut HashMap<String, Vec<(usize, Option<String>)>>,
    model: &Model,
    config: &ParserConfig,
) -> Result<()> {
    // Check cache first and get object info
    if !cache.contains_key(file_path) {
        // Load and parse the external model file (decrypt if encrypted)
        let external_xml = load_file_with_decryption(package, file_path, file_path, model, config)?;

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
                    } else if local_name == "component" {
                        // N_XPM_0803_01: Validate that non-root model files don't have components with p:path
                        // Per 3MF Production Extension spec Chapter 2:
                        // "Non-root model file components MUST only reference objects in the same model file"
                        // This prevents component reference chains across multiple files
                        for attr in e.attributes() {
                            let attr = attr.map_err(|e| Error::InvalidXml(e.to_string()))?;
                            let attr_name = std::str::from_utf8(attr.key.as_ref())
                                .map_err(|e| Error::InvalidXml(e.to_string()))?;

                            // Check for p:path attribute (standard production extension namespace)
                            // We check for the exact "p:path" attribute name
                            if attr_name == "p:path" {
                                let path_value = std::str::from_utf8(&attr.value)
                                    .map_err(|e| Error::InvalidXml(e.to_string()))?;
                                return Err(Error::InvalidModel(format!(
                                    "External model file '{}' contains a component with p:path=\"{}\". \
                                     Per 3MF Production Extension specification (Chapter 2), only components \
                                     in the root model file may have p:path attributes. Non-root model files \
                                     must only reference objects within the same file. This restriction \
                                     prevents component reference chains across multiple files.",
                                    file_path, path_value
                                )));
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

/// Validate an external model file's triangles for material property consistency
///
/// N_XXM_0601_02: External model files (non-root) must have proper material properties
/// When an object has some triangles with material properties and some without,
/// the object must have a default pid to provide material for unmaterialized triangles
pub(super) fn validate_external_model_triangles<R: Read + std::io::Seek>(
    package: &mut Package<R>,
    file_path: &str,
    model: &Model,
    validated_files: &mut HashSet<String>,
    config: &ParserConfig,
) -> Result<()> {
    // Skip if already validated or is encrypted
    if validated_files.contains(file_path) {
        return Ok(());
    }

    let is_encrypted = model
        .secure_content
        .as_ref()
        .map(|sc| {
            sc.encrypted_files.iter().any(|encrypted_path| {
                let enc_normalized = encrypted_path.trim_start_matches('/');
                enc_normalized == file_path
            })
        })
        .unwrap_or(false);

    if is_encrypted {
        // Skip validation for encrypted files
        validated_files.insert(file_path.to_string());
        return Ok(());
    }

    // Load and fully parse the external model file
    let external_xml = load_file_with_decryption(package, file_path, file_path, model, config)?;

    // Parse the external model file with all extensions enabled plus common custom extensions
    // We use a comprehensive config instead of the main model's config because:
    // 1. External files may declare different required extensions than the main model
    // 2. We're only validating triangle material properties, not enforcing extension requirements
    // 3. This prevents failures when external files use extensions not in the main model's config
    let external_config = ParserConfig::with_all_extensions()
        .with_custom_extension(
            "http://schemas.3mf.io/3dmanufacturing/displacement/2023/10",
            "Displacement 2023/10",
        )
        .with_custom_extension(
            "http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/04",
            "SecureContent 2019/04",
        )
        .with_custom_extension(
            "http://schemas.microsoft.com/3dmanufacturing/beamlattice/balls/2020/07",
            "BeamLattice Balls",
        )
        .with_custom_extension(
            "http://schemas.microsoft.com/3dmanufacturing/trianglesets/2021/07",
            "TriangleSets",
        );

    let external_model = match parse_model_xml_with_config(&external_xml, external_config) {
        Ok(model) => model,
        Err(e) => {
            return Err(Error::InvalidModel(format!(
                "External model file '{}' failed to parse: {}",
                file_path, e
            )));
        }
    };

    // Validate triangle properties in the external model using the shared helper function
    for object in &external_model.resources.objects {
        if let Some(ref mesh) = object.mesh {
            // Use the shared validation function from validator module
            crate::validator::validate_object_triangle_materials(
                object.id,
                object.pid,
                mesh,
                &format!("External model file '{}': Object {}", file_path, object.id),
            )?;
        }
    }

    validated_files.insert(file_path.to_string());
    Ok(())
}
