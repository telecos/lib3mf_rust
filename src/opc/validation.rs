//! OPC validation utilities

use crate::error::{Error, Result};

/// Validate OPC part name according to OPC specification constraints
pub(crate) fn validate_opc_part_name(part_name: &str) -> Result<()> {
    // Note: We don't strictly enforce ASCII-only here for compatibility.
    // Per OPC spec, non-ASCII should be percent-encoded, but many real-world
    // files include UTF-8 characters directly. We accept both and handle
    // URL-decoding when looking up files.

    // Check for control characters (newlines, tabs, etc.)
    // Per OPC spec, these are not allowed in part names
    if part_name.chars().any(|c| c.is_control()) {
        return Err(Error::InvalidFormat(format!(
            "Part name cannot contain control characters (newlines, tabs, etc.): {}",
            part_name.escape_debug()
        )));
    }

    // Check for fragment identifiers
    if part_name.contains('#') {
        return Err(Error::InvalidFormat(format!(
            "Part name cannot contain fragment identifier: {}",
            part_name
        )));
    }

    // Check for query strings
    if part_name.contains('?') {
        return Err(Error::InvalidFormat(format!(
            "Part name cannot contain query string: {}",
            part_name
        )));
    }

    // Split into path segments and validate each
    let segments: Vec<&str> = part_name.split('/').collect();

    for (idx, segment) in segments.iter().enumerate() {
        // Check for empty segments (consecutive slashes)
        if segment.is_empty() {
            // Allow leading slash (which creates empty first segment)
            if idx == 0 && part_name.starts_with('/') {
                continue;
            }
            return Err(Error::InvalidFormat(format!(
                "Part name cannot contain empty path segments (consecutive slashes): {}",
                part_name
            )));
        }

        // Check for "." or ".." segments
        if *segment == "." || *segment == ".." {
            return Err(Error::InvalidFormat(format!(
                "Part name cannot contain '.' or '..' segments: {}",
                part_name
            )));
        }

        // Check for segments ending with "."
        if segment.ends_with('.') {
            return Err(Error::InvalidFormat(format!(
                "Part name segments cannot end with '.': {}",
                part_name
            )));
        }
    }

    Ok(())
}

/// Normalize OPC path by removing leading slash
pub(crate) fn normalize_path(path: &str) -> &str {
    path.strip_prefix('/').unwrap_or(path)
}
