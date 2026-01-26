//! Production extension validation

use crate::error::{Error, Result};
use crate::model::{Extension, Model, ParserConfig};
use std::collections::HashSet;

/// Validates production extension path format and usage
pub fn validate_production_extension(model: &Model) -> Result<()> {
    // Helper function to validate p:path format
    let validate_path = |path: &str, context: &str| -> Result<()> {
        // Per 3MF Production Extension spec:
        // - Path MUST start with / (absolute path within the package)
        // - Path MUST NOT contain .. (no parent directory references)
        // - Path MUST NOT end with / (must reference a file, not a directory)
        // - Filename MUST NOT start with . (hidden files not allowed)

        if !path.starts_with('/') {
            return Err(Error::InvalidModel(format!(
                "{}: Production path '{}' must start with / (absolute path required)",
                context, path
            )));
        }

        if path.contains("..") {
            return Err(Error::InvalidModel(format!(
                "{}: Production path '{}' must not contain .. (parent directory traversal not allowed)",
                context, path
            )));
        }

        if path.ends_with('/') {
            return Err(Error::InvalidModel(format!(
                "{}: Production path '{}' must not end with / (must reference a file)",
                context, path
            )));
        }

        // Check for hidden files (filename starting with .)
        if let Some(filename) = path.rsplit('/').next() {
            if filename.starts_with('.') {
                return Err(Error::InvalidModel(format!(
                    "{}: Production path '{}' references a hidden file (filename cannot start with .)",
                    context, path
                )));
            }
        }

        // Path should reference a .model file
        if !path.ends_with(".model") {
            return Err(Error::InvalidModel(format!(
                "{}: Production path '{}' must reference a .model file",
                context, path
            )));
        }

        Ok(())
    };

    // Check all objects to validate production paths
    for object in &model.resources.objects {
        // Note: The thumbnail attribute is deprecated in 3MF v1.4+ when production extension is used,
        // but deprecation doesn't make it invalid. Per the official 3MF test suite, files with
        // thumbnail attributes and production extension should still parse successfully.
        // Therefore, we do not reject files with thumbnail attributes.

        // Validate production extension usage
        if let Some(ref prod_info) = object.production {
            // If object has production path, validate it
            if let Some(ref path) = prod_info.path {
                validate_path(path, &format!("Object {}", object.id))?;
            }
        }

        // Check components
        for (idx, component) in object.components.iter().enumerate() {
            if let Some(ref prod_info) = component.production {
                // Validate production path format if present
                // Note: component.path is set from prod_info.path during parsing
                // Per 3MF Production Extension spec:
                // - p:UUID can be used on components to uniquely identify them
                // - p:path is only required when referencing external objects (not in current file)
                // - A component with p:UUID but no p:path references a local object
                if let Some(ref path) = prod_info.path {
                    validate_path(path, &format!("Object {}, Component {}", object.id, idx))?;
                }
            }
        }
    }

    // Check build items for production path validation
    for (idx, item) in model.build.items.iter().enumerate() {
        if let Some(ref path) = item.production_path {
            validate_path(path, &format!("Build Item {}", idx))?;
        }
    }

    // Note: We don't validate that production attributes require the production extension
    // to be in requiredextensions, because per the 3MF spec, extensions can be declared
    // in namespaces (xmlns:p) without being in requiredextensions - they are then optional
    // extensions. The parser already validates that the production namespace is declared
    // when production attributes are used.

    Ok(())
}

/// Validate production extension requirements with parser configuration
///
/// This is a variant of `validate_production_extension` that accepts a parser config.
/// When the parser config explicitly supports the production extension, we allow
/// production attributes to be used even if the file doesn't declare the production
/// extension in requiredextensions. This is useful for backward compatibility and
/// for files that use production attributes but were created before strict validation.
pub fn validate_production_extension_with_config(
    model: &Model,
    config: &ParserConfig,
) -> Result<()> {
    // Check if production extension is required in the file
    let has_production = model.required_extensions.contains(&Extension::Production);

    // Check if the parser config explicitly supports production extension
    let config_supports_production = config.supports(&Extension::Production);

    // Track whether any production attributes are used (for validation later)
    let mut has_production_attrs = false;

    // Helper function to validate p:path format
    let validate_path = |path: &str, context: &str| -> Result<()> {
        // Per 3MF Production Extension spec:
        // - Path MUST start with / (absolute path within the package)
        // - Path MUST NOT contain .. (no parent directory references)
        // - Path MUST NOT end with / (must reference a file, not a directory)
        // - Filename MUST NOT start with . (hidden files not allowed)

        if !path.starts_with('/') {
            return Err(Error::InvalidModel(format!(
                "{}: Production path '{}' must start with / (absolute path required)",
                context, path
            )));
        }

        if path.contains("..") {
            return Err(Error::InvalidModel(format!(
                "{}: Production path '{}' must not contain .. (parent directory traversal not allowed)",
                context, path
            )));
        }

        if path.ends_with('/') {
            return Err(Error::InvalidModel(format!(
                "{}: Production path '{}' must not end with / (must reference a file)",
                context, path
            )));
        }

        // Check for hidden files (filename starting with .)
        if let Some(filename) = path.rsplit('/').next() {
            if filename.starts_with('.') {
                return Err(Error::InvalidModel(format!(
                    "{}: Production path '{}' references a hidden file (filename cannot start with .)",
                    context, path
                )));
            }
        }

        // Path should reference a .model file
        if !path.ends_with(".model") {
            return Err(Error::InvalidModel(format!(
                "{}: Production path '{}' must reference a .model file",
                context, path
            )));
        }

        Ok(())
    };

    // Check all objects to validate production paths
    for object in &model.resources.objects {
        // Note: The thumbnail attribute is deprecated in 3MF v1.4+ when production extension is used,
        // but deprecation doesn't make it invalid. Per the official 3MF test suite, files with
        // thumbnail attributes and production extension should still parse successfully.
        // Therefore, we do not reject files with thumbnail attributes.

        // Validate production extension usage and track attributes
        if let Some(ref prod_info) = object.production {
            has_production_attrs = true;

            // If object has production path, validate it
            if let Some(ref path) = prod_info.path {
                validate_path(path, &format!("Object {}", object.id))?;
            }
        }

        // Check components
        for (idx, component) in object.components.iter().enumerate() {
            if let Some(ref prod_info) = component.production {
                has_production_attrs = true;

                // Per 3MF Production Extension spec:
                // - p:UUID can be used on components to uniquely identify them
                // - p:path is only required when referencing external objects (not in current file)
                // - A component with p:UUID but no p:path references a local object
                // - When p:path is used (external reference), p:UUID is REQUIRED to identify the object

                // Validate that p:UUID is present when p:path is used
                if prod_info.path.is_some() && prod_info.uuid.is_none() {
                    return Err(Error::InvalidModel(format!(
                        "Object {}, Component {}: Component has p:path but missing required p:UUID.\n\
                         Per 3MF Production Extension spec, components with external references (p:path) \
                         must have p:UUID to identify the referenced object.\n\
                         Add p:UUID attribute to the component element.",
                        object.id, idx
                    )));
                }

                // Validate production path format if present
                // Note: component.path is set from prod_info.path during parsing
                if let Some(ref path) = prod_info.path {
                    validate_path(path, &format!("Object {}, Component {}", object.id, idx))?;
                }
            }
        }
    }

    // Check build items for production path validation
    for (idx, item) in model.build.items.iter().enumerate() {
        if item.production_uuid.is_some() || item.production_path.is_some() {
            has_production_attrs = true;
        }

        if let Some(ref path) = item.production_path {
            validate_path(path, &format!("Build Item {}", idx))?;
        }
    }

    // Check build production UUID
    if model.build.production_uuid.is_some() {
        has_production_attrs = true;
    }

    // Validate that production attributes are only used when production extension is declared
    // UNLESS the parser config explicitly supports production extension (for backward compatibility)
    if has_production_attrs && !has_production && !config_supports_production {
        return Err(Error::InvalidModel(
            "Production extension attributes (p:UUID, p:path) are used but production extension \
             is not declared in requiredextensions.\n\
             Per 3MF Production Extension specification, when using production attributes, \
             you must add 'p' to the requiredextensions attribute in the <model> element.\n\
             Example: requiredextensions=\"p\" or requiredextensions=\"m p\" for materials and production."
                .to_string(),
        ));
    }

    Ok(())
}

/// Validate displacement extension usage
///
/// Per Displacement Extension spec:
/// - Displacement2D resources must reference existing texture files in the package
/// - Disp2DGroup must reference existing Displacement2D and NormVectorGroup resources
/// - Disp2DCoord must reference valid normvector indices
/// - NormVectors must be normalized (unit length)
/// - DisplacementTriangle did must reference existing Disp2DGroup resources
/// - DisplacementTriangle d1, d2, d3 must reference valid displacement coordinates
///
/// Validates that production paths don't reference OPC internal files
pub fn validate_production_paths(model: &Model) -> Result<()> {
    // Helper function to validate that a path doesn't reference OPC internal files
    let validate_not_opc_internal = |path: &str, context: &str| -> Result<()> {
        // OPC internal paths that should not be referenced:
        // - /_rels/.rels or any path starting with /_rels/
        // - /[Content_Types].xml

        if path.starts_with("/_rels/") || path == "/_rels" {
            return Err(Error::InvalidModel(format!(
                "{}: Production path '{}' references OPC internal relationships directory.\n\
                 Production paths must not reference package internal files.",
                context, path
            )));
        }

        if path == "/[Content_Types].xml" {
            return Err(Error::InvalidModel(format!(
                "{}: Production path '{}' references OPC content types file.\n\
                 Production paths must not reference package internal files.",
                context, path
            )));
        }

        Ok(())
    };

    // Check all objects
    for object in &model.resources.objects {
        if let Some(ref prod_info) = object.production {
            if let Some(ref path) = prod_info.path {
                validate_not_opc_internal(path, &format!("Object {}", object.id))?;
            }
        }

        // Check components
        for (idx, component) in object.components.iter().enumerate() {
            if let Some(ref prod_info) = component.production {
                if let Some(ref path) = prod_info.path {
                    validate_not_opc_internal(
                        path,
                        &format!("Object {}, Component {}", object.id, idx),
                    )?;
                }
            }
        }
    }

    // Check build items - validate p:path doesn't reference OPC internal files
    for (idx, item) in model.build.items.iter().enumerate() {
        if let Some(ref path) = item.production_path {
            validate_not_opc_internal(path, &format!("Build item {}", idx))?;
        }
    }

    Ok(())
}

/// Validate transform matrices for build items
///
/// Per 3MF spec, transform matrices must have a non-negative determinant.
/// A negative determinant indicates a mirror transformation which would
/// invert the object's orientation (inside-out).
///
/// Exception: For sliced objects (objects with slicestackid), the transform
/// restrictions are different per the 3MF Slice Extension spec. Sliced objects
/// must have planar transforms (validated separately in validate_slice_extension),
/// but can have negative determinants (mirror transformations).
/// Validates that required UUIDs are present when production extension is used
pub fn validate_production_uuids_required(model: &Model) -> Result<()> {
    // Only validate if production extension is explicitly required in the model
    // The config.supports() tells us what the parser accepts, but we need to check
    // what the model file actually requires
    let production_required = model.required_extensions.contains(&Extension::Production);

    if !production_required {
        return Ok(());
    }

    // When production extension is required:
    // 1. Build MUST have UUID (Chapter 4.1) if it has items
    // Per spec, the build UUID is required to identify builds across devices/jobs
    if !model.build.items.is_empty() && model.build.production_uuid.is_none() {
        return Err(Error::InvalidModel(
            "Production extension requires build to have p:UUID attribute when build items are present".to_string(),
        ));
    }

    // 2. Build items MUST have UUID (Chapter 4.1.1)
    for (idx, item) in model.build.items.iter().enumerate() {
        if item.production_uuid.is_none() {
            return Err(Error::InvalidModel(format!(
                "Production extension requires build item {} to have p:UUID attribute",
                idx
            )));
        }
    }

    // 3. Objects MUST have UUID (Chapter 4.2)
    for object in &model.resources.objects {
        // Check if object has production info with UUID
        let has_uuid = object
            .production
            .as_ref()
            .and_then(|p| p.uuid.as_ref())
            .is_some();

        if !has_uuid {
            return Err(Error::InvalidModel(format!(
                "Production extension requires object {} to have p:UUID attribute",
                object.id
            )));
        }
    }

    Ok(())
}

/// Validate UUID format per RFC 4122
///
/// UUIDs must follow the format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
/// where x is a hexadecimal digit (0-9, a-f, A-F).
pub fn validate_uuid_formats(model: &Model) -> Result<()> {
    // Helper function to validate a single UUID
    let validate_uuid = |uuid: &str, context: &str| -> Result<()> {
        // UUID format: 8-4-4-4-12 hexadecimal digits separated by hyphens
        // Example: 550e8400-e29b-41d4-a716-446655440000

        // Check length (36 characters including hyphens)
        if uuid.len() != 36 {
            return Err(Error::InvalidModel(format!(
                "{}: Invalid UUID '{}' - must be 36 characters in format xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
                context, uuid
            )));
        }

        // Check hyphen positions (at indices 8, 13, 18, 23)
        if uuid.chars().nth(8) != Some('-')
            || uuid.chars().nth(13) != Some('-')
            || uuid.chars().nth(18) != Some('-')
            || uuid.chars().nth(23) != Some('-')
        {
            return Err(Error::InvalidModel(format!(
                "{}: Invalid UUID '{}' - hyphens must be at positions 8, 13, 18, and 23",
                context, uuid
            )));
        }

        // Check that all other characters are hexadecimal digits
        for (idx, ch) in uuid.chars().enumerate() {
            if idx == 8 || idx == 13 || idx == 18 || idx == 23 {
                continue; // Skip hyphens
            }
            if !ch.is_ascii_hexdigit() {
                return Err(Error::InvalidModel(format!(
                    "{}: Invalid UUID '{}' - character '{}' at position {} is not a hexadecimal digit",
                    context, uuid, ch, idx
                )));
            }
        }

        Ok(())
    };

    // Validate build UUID
    if let Some(ref uuid) = model.build.production_uuid {
        validate_uuid(uuid, "Build")?;
    }

    // Validate build item UUIDs
    for (idx, item) in model.build.items.iter().enumerate() {
        if let Some(ref uuid) = item.production_uuid {
            validate_uuid(uuid, &format!("Build item {}", idx))?;
        }
    }

    // Validate object UUIDs
    for object in &model.resources.objects {
        if let Some(ref prod_info) = object.production {
            if let Some(ref uuid) = prod_info.uuid {
                validate_uuid(uuid, &format!("Object {}", object.id))?;
            }
        }

        // Validate component UUIDs
        for (idx, component) in object.components.iter().enumerate() {
            if let Some(ref prod_info) = component.production {
                if let Some(ref uuid) = prod_info.uuid {
                    validate_uuid(uuid, &format!("Object {}, Component {}", object.id, idx))?;
                }
            }
        }
    }

    Ok(())
}

/// Validates that all UUIDs in the model are unique
///
/// Per 3MF Production Extension spec, UUIDs must be unique across:
/// - Build section
/// - Build items
/// - Objects
/// - Components
pub fn validate_duplicate_uuids(model: &Model) -> Result<()> {
    let mut uuids = HashSet::new();

    // Check build UUID
    if let Some(ref uuid) = model.build.production_uuid {
        if !uuids.insert(uuid.clone()) {
            return Err(Error::InvalidModel(format!(
                "Duplicate UUID '{}' found in build",
                uuid
            )));
        }
    }

    // Check build item UUIDs
    for (idx, item) in model.build.items.iter().enumerate() {
        if let Some(ref uuid) = item.production_uuid {
            if !uuids.insert(uuid.clone()) {
                return Err(Error::InvalidModel(format!(
                    "Duplicate UUID '{}' found in build item {}",
                    uuid, idx
                )));
            }
        }
    }

    // Check object UUIDs
    for object in &model.resources.objects {
        if let Some(ref production) = object.production {
            if let Some(ref uuid) = production.uuid {
                if !uuids.insert(uuid.clone()) {
                    return Err(Error::InvalidModel(format!(
                        "Duplicate UUID '{}' found on object {}",
                        uuid, object.id
                    )));
                }
            }
        }

        // Check component UUIDs within each object
        for (comp_idx, component) in object.components.iter().enumerate() {
            if let Some(ref production) = component.production {
                if let Some(ref uuid) = production.uuid {
                    if !uuids.insert(uuid.clone()) {
                        return Err(Error::InvalidModel(format!(
                            "Duplicate UUID '{}' found in object {} component {}",
                            uuid, object.id, comp_idx
                        )));
                    }
                }
            }
        }
    }
    Ok(())
}

/// N_XPX_0803_01: Validate no component reference chains across multiple model parts
///
/// **Note: This validation is intentionally disabled.**
///
/// Detecting component reference chains requires parsing and analyzing external
/// model files referenced via `p:path`. Since the parser only loads the root model
/// file, we cannot reliably detect multi-level chains.
///
/// A full implementation would require:
/// 1. Loading all referenced external model files
/// 2. Building a dependency graph across files
/// 3. Detecting cycles or chains longer than allowed depth
///
/// This is beyond the scope of single-file validation and would require
/// significant architectural changes to support multi-file analysis.
pub fn validate_component_chain(_model: &Model) -> Result<()> {
    // N_XPM_0803_01: Component reference chain validation
    //
    // The validation for components with p:path referencing local objects
    // is complex and requires more investigation of the 3MF Production Extension spec.
    // The current understanding is insufficient to implement this correctly.
    Ok(())
}
