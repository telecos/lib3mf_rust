//! Slice extension parsing
//!
//! This module handles loading and parsing of 3MF Slice extension elements
//! including slice references and external slice files.

use crate::error::{Error, Result};
use crate::model::{Model, Object, ParserConfig, Slice};
use crate::opc::Package;
use std::io::Read;

use super::{load_file_with_decryption, parse_model_xml_with_config};

/// Load external slice files referenced by SliceRef elements
///
/// This function processes all SliceRef elements in the model, loading the
/// external slice files and merging their content into the main model.
pub(super) fn load_slice_references<R: Read + std::io::Seek>(
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

        // Rule: SliceStack must contain either slices OR slicerefs, not both
        // Per 3MF Slice Extension spec, a slicestack MUST contain either <slice> elements
        // or <sliceref> elements, but MUST NOT contain both element types concurrently.
        if !slice_stack.slices.is_empty() && !slice_stack.slice_refs.is_empty() {
            return Err(Error::InvalidModel(format!(
                "SliceStack {}: Contains both <slice> and <sliceref> elements.\n\
                 Per 3MF Slice Extension spec, a slicestack MUST contain either \
                 <slice> elements or <sliceref> elements, but MUST NOT contain both element types concurrently.",
                slice_stack.id
            )));
        }

        for slice_ref in &slice_stack.slice_refs {
            // Validate slicepath starts with /2D/
            // Per 3MF Slice Extension spec: "For package readability and organization,
            // slice models SHOULD be stored in the 2D folder UNLESS they are part of
            // the root model part."
            // We enforce this as a MUST for SliceRef elements (external slice references)
            // to catch packaging errors. SliceRef elements by definition reference external
            // files and must use the /2D/ folder per spec conventions.
            if !slice_ref.slicepath.starts_with("/2D/") {
                return Err(Error::InvalidModel(format!(
                    "SliceStack {}: SliceRef references invalid path '{}'.\n\
                     Per 3MF Slice Extension spec, external slice models must be stored in the /2D/ folder. \
                     Slicepath must start with '/2D/'.",
                    slice_stack.id, slice_ref.slicepath
                )));
            }

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

    // Collect available IDs first (before mutable borrow)
    let available_ids: Vec<usize> = external_model
        .resources
        .slice_stacks
        .iter()
        .map(|s| s.id)
        .collect();

    // Find the slice stack with the expected ID
    let stack_option = external_model
        .resources
        .slice_stacks
        .iter_mut()
        .find(|stack| stack.id == expected_stack_id);

    let stack = match stack_option {
        Some(s) => s,
        None => {
            return Err(Error::InvalidModel(format!(
                "SliceRef references non-existent slicestackid {}.\n\
                 Per 3MF Slice Extension spec, the slicestackid attribute in a <sliceref> element \
                 must reference a valid <slicestack> defined in the external slice file.\n\
                 Available slicestack IDs in external file: {:?}",
                expected_stack_id, available_ids
            )));
        }
    };

    // N_SPX_1606_01: Validate slices against the external slicestack's zbottom
    // before extracting them
    let zbottom = stack.zbottom;
    for (slice_idx, slice) in stack.slices.iter().enumerate() {
        if slice.ztop < zbottom {
            return Err(Error::InvalidModel(format!(
                "External SliceStack {}: Slice {} has ztop={} which is less than zbottom={}.\n\
                 Per 3MF Slice Extension spec, each slice's ztop must be >= the slicestack's zbottom.",
                expected_stack_id, slice_idx, slice.ztop, zbottom
            )));
        }
    }

    // Extract slices
    let slices = std::mem::take(&mut stack.slices);

    // Extract all objects from the external model
    let objects = std::mem::take(&mut external_model.resources.objects);

    Ok((slices, objects))
}
