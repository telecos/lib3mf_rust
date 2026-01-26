//! Package reading and validation functionality

use crate::error::{Error, Result};
use super::{Package, CONTENT_TYPES_PATH, RELS_PATH, MODEL_PATH};
use quick_xml::events::Event;
use quick_xml::Reader as XmlReader;
use std::io::Read;
use urlencoding::decode;
use zip::ZipArchive;

/// Open a 3MF package from a reader
pub(super) fn open<R: Read + std::io::Seek>(reader: R) -> Result<Package<R>> {
    let archive = ZipArchive::new(reader)?;
    let mut package = Package { archive };

    // Validate required OPC structure
    validate_opc_structure(&mut package)?;

    Ok(package)
}

/// Validate OPC package structure according to 3MF spec
fn validate_opc_structure<R: Read + std::io::Seek>(package: &mut Package<R>) -> Result<()> {
    // Validate required files exist
    if !has_file(package, CONTENT_TYPES_PATH) {
        return Err(Error::invalid_format_context(
            "OPC package structure",
            &format!(
                "Missing required file '{}'. \
                 This file defines content types for the package and is required by the OPC specification. \
                 The 3MF file may be corrupt or improperly formatted.",
                CONTENT_TYPES_PATH
            )
        ));
    }

    if !has_file(package, RELS_PATH) {
        return Err(Error::invalid_format_context(
            "OPC package structure",
            &format!(
                "Missing required file '{}'. \
                 This file defines package relationships and is required by the OPC specification. \
                 The 3MF file may be corrupt or improperly formatted.",
                RELS_PATH
            )
        ));
    }

    // Validate Content Types
    validate_content_types(package)?;

    // Validate that model relationship exists and points to valid file
    validate_model_relationship(package)?;

    // Validate all relationships point to existing files
    validate_all_relationships(package)?;

    Ok(())
}

/// Validate [Content_Types].xml structure
fn validate_content_types<R: Read + std::io::Seek>(package: &mut Package<R>) -> Result<()> {
    // Placeholder - will implement full validation
    let _ = package;
    Ok(())
}

/// Validate model relationship exists and points to a valid file
fn validate_model_relationship<R: Read + std::io::Seek>(package: &mut Package<R>) -> Result<()> {
    // Placeholder - will implement full validation
    let _ = package;
    Ok(())
}

/// Validate all relationships point to existing files
fn validate_all_relationships<R: Read + std::io::Seek>(package: &mut Package<R>) -> Result<()> {
    // Placeholder - will implement full validation
    let _ = package;
    Ok(())
}

/// Get the main 3D model file content
pub(super) fn get_model<R: Read + std::io::Seek>(package: &mut Package<R>) -> Result<String> {
    // Placeholder - will implement
    let _ = package;
    Ok(String::new())
}

/// Get a file from the package by name
pub(super) fn get_file<R: Read + std::io::Seek>(package: &mut Package<R>, name: &str) -> Result<String> {
    let mut file = package
        .archive
        .by_name(name)
        .map_err(|_| Error::MissingFile(name.to_string()))?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

/// Check if a file exists in the package
pub(super) fn has_file<R: Read + std::io::Seek>(package: &mut Package<R>, name: &str) -> bool {
    package.archive.by_name(name).is_ok()
}

/// Get the number of files in the package
pub(super) fn len<R: Read + std::io::Seek>(package: &Package<R>) -> usize {
    package.archive.len()
}

/// Check if the package is empty
pub(super) fn is_empty<R: Read + std::io::Seek>(package: &Package<R>) -> bool {
    package.archive.is_empty()
}

/// Get a list of all file names in the package
pub(super) fn file_names<R: Read + std::io::Seek>(package: &mut Package<R>) -> Vec<String> {
    (0..package.archive.len())
        .filter_map(|i| package.archive.by_index(i).ok().map(|f| f.name().to_string()))
        .collect()
}

/// Get a file as binary data
pub(super) fn get_file_binary<R: Read + std::io::Seek>(package: &mut Package<R>, name: &str) -> Result<Vec<u8>> {
    let mut file = package
        .archive
        .by_name(name)
        .map_err(|_| Error::MissingFile(name.to_string()))?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    Ok(content)
}
