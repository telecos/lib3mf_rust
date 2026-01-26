//! Production extension validation
//!
//! This module handles validation of the Production extension (p: namespace) in 3MF files.
//! The Production extension allows referencing objects and build items from external model files
//! within the same 3MF package.

use crate::error::{Error, Result};
use crate::model::{Model, ParserConfig};
use crate::opc::Package;
use std::collections::{HashMap, HashSet};
use std::io::Read;

use super::boolean_ops::{validate_external_model_triangles, validate_external_object_reference};
use super::secure_content::validate_encrypted_file_can_be_loaded;

/// Validate production extension external paths
///
/// This function validates all external references in build items and components that use the
/// Production extension. For each external reference (p:path + p:UUID), it ensures:
///
/// 1. The referenced file exists in the package
/// 2. The referenced object ID exists in that file (unless encrypted)
/// 3. The referenced file has valid triangle material properties
///
/// Special handling for encrypted files (Secure Content extension):
/// - Encrypted files cannot be parsed to validate object IDs
/// - Instead, we verify the keystore has valid consumers/keys for decryption
///
/// # Arguments
///
/// * `package` - The 3MF package containing all files
/// * `model` - The parsed model with build items and objects
///
/// # Returns
///
/// * `Ok(())` if all external references are valid
/// * `Err` if any reference is invalid or points to a missing file/object
pub(super) fn validate_production_external_paths<R: Read + std::io::Seek>(
    package: &mut Package<R>,
    model: &Model,
    config: &ParserConfig,
) -> Result<()> {
    // Cache to avoid re-parsing the same external file multiple times
    let mut external_file_cache: HashMap<String, Vec<(usize, Option<String>)>> = HashMap::new();
    // Track which external files we've validated for triangles
    let mut validated_files: HashSet<String> = HashSet::new();

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
                    config,
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
                config,
            )?;

            // N_XXM_0601_02: Validate triangle material properties in external model file
            validate_external_model_triangles(
                package,
                normalized_path,
                model,
                &mut validated_files,
                config,
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
                            config,
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
                        config,
                    )?;

                    // N_XXM_0601_02: Validate triangle material properties in external model file
                    validate_external_model_triangles(
                        package,
                        normalized_path,
                        model,
                        &mut validated_files,
                        config,
                    )?;
                }
            }
        }
    }

    Ok(())
}
