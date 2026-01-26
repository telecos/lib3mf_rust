//! Secure Content extension parsing
//!
//! This module handles parsing of 3MF Secure Content extension elements including
//! keystore, consumers, resource data groups, access rights, and encryption parameters.
//! It also provides decryption support for encrypted files using test keys.

use crate::error::{Error, Result};
use crate::model::*;
use crate::opc::{Package, ENCRYPTEDFILE_REL_TYPE};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashSet;
use std::io::Read;

use super::get_local_name;

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

/// Default buffer capacity for XML parsing (4KB)
const XML_BUFFER_CAPACITY: usize = 4096;

/// Validate KEK params attributes
///
/// Validates wrapping algorithm, MGF algorithm, and digest method according to
/// EPX-2603 specification requirements.
pub(super) fn validate_kekparams_attributes(
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
pub(super) fn load_keystore<R: Read + std::io::Seek>(
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

    // EPX-2606: Validate keystore has proper relationship in root .rels
    // This catches cases where the keystore file exists but only has a mustpreserve
    // relationship instead of the proper keystore relationship type
    package.validate_keystore_relationship(&keystore_path)?;

    // EPX-2606: Validate keystore has proper content type override
    // This catches cases where the keystore file exists but is missing the
    // required content type declaration in [Content_Types].xml
    package.validate_keystore_content_type(&keystore_path)?;

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

                    // Store the KEK params in the current access right
                    if let Some(ref mut access_right) = current_access_right {
                        access_right.kek_params = KEKParams {
                            wrapping_algorithm,
                            mgf_algorithm: if mgf_algorithm.is_empty() {
                                None
                            } else {
                                Some(mgf_algorithm)
                            },
                            digest_method: if digest_method.is_empty() {
                                None
                            } else {
                                Some(digest_method)
                            },
                        };
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

        // EPX-2606: Validate encrypted files have EncryptedFile relationships
        // Per 3MF SecureContent specification, all encrypted files referenced by
        // resourcedata elements MUST have an EncryptedFile relationship in the OPC package
        for group in &sc.resource_data_groups {
            for resource_data in &group.resource_data {
                let encrypted_path = &resource_data.path;

                // Check if this encrypted file has an EncryptedFile relationship
                // We don't specify a source file since the relationship could be in any .rels file
                let has_encrypted_rel = package.has_relationship_to_target(
                    encrypted_path,
                    ENCRYPTEDFILE_REL_TYPE,
                    None,
                )?;

                if !has_encrypted_rel {
                    return Err(Error::InvalidSecureContent(format!(
                        "Encrypted file '{}' is missing required EncryptedFile relationship. \
                         Per 3MF SecureContent specification, all encrypted files referenced in the keystore \
                         must have a corresponding EncryptedFile relationship in the OPC package (EPX-2606)",
                        encrypted_path
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
pub(super) fn load_file_with_decryption<R: Read + std::io::Seek>(
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

/// Validate that an encrypted file can be loaded and decrypted
///
/// This checks that:
/// - The file exists in the package
/// - The file can be decrypted using the test consumer keys
/// - The decrypted content is valid
pub(super) fn validate_encrypted_file_can_be_loaded<R: Read + std::io::Seek>(
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
