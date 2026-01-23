//! Error types for 3MF parsing
//!
//! This module provides comprehensive error handling for 3MF file operations.
//! All errors include error codes for categorization and detailed context to help
//! with debugging.
//!
//! # Error Codes
//!
//! Error codes follow the pattern: `E<category><number>`
//!
//! Categories:
//! - **E1xxx**: I/O and archive errors
//! - **E2xxx**: XML parsing and structure errors
//! - **E3xxx**: Model validation errors
//! - **E4xxx**: Unsupported features
//!
//! ## Common Error Codes
//!
//! - `E1001`: I/O error reading file
//! - `E1002`: ZIP archive format error
//! - `E1003`: Missing required file in archive
//! - `E2001`: XML parsing error
//! - `E2002`: XML attribute error
//! - `E2003`: Invalid XML structure
//! - `E2004`: Invalid 3MF format
//! - `E3001`: Invalid model structure
//! - `E3002`: Numeric parse error
//! - `E4001`: Unsupported feature
//! - `E4002`: Required extension not supported

use std::io;
use thiserror::Error;

/// Result type for 3MF operations
pub type Result<T> = std::result::Result<T, Error>;

/// Additional context for errors
///
/// Provides optional supplementary information to help with debugging:
/// - File location information (when available from XML parsing)
/// - Line and column numbers (when available from XML parsing)
/// - Helpful hints for resolving common issues
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorContext {
    /// The file where the error occurred (typically a path within the 3MF archive)
    pub file: Option<String>,

    /// Line number where the error occurred (when available from XML parsing)
    pub line: Option<usize>,

    /// Column number where the error occurred (when available from XML parsing)
    pub column: Option<usize>,

    /// A helpful hint for resolving the error
    pub hint: Option<String>,
}

impl ErrorContext {
    /// Create a new empty error context
    pub fn new() -> Self {
        Self {
            file: None,
            line: None,
            column: None,
            hint: None,
        }
    }

    /// Create an error context with just a hint
    pub fn with_hint(hint: impl Into<String>) -> Self {
        Self {
            file: None,
            line: None,
            column: None,
            hint: Some(hint.into()),
        }
    }

    /// Set the file location
    pub fn file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Set the line number
    pub fn line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// Set the column number
    pub fn column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    /// Set the hint
    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();

        if let Some(ref file) = self.file {
            parts.push(format!("File: {}", file));
        }

        if let (Some(line), Some(column)) = (self.line, self.column) {
            parts.push(format!("Location: line {}, column {}", line, column));
        } else if let Some(line) = self.line {
            parts.push(format!("Line: {}", line));
        }

        if let Some(ref hint) = self.hint {
            parts.push(format!("Hint: {}", hint));
        }

        if !parts.is_empty() {
            write!(f, "\n{}", parts.join("\n"))
        } else {
            Ok(())
        }
    }
}

/// Errors that can occur when parsing 3MF files
#[derive(Error, Debug)]
pub enum Error {
    /// IO error occurred while reading the file
    ///
    /// **Error Code**: E1001
    ///
    /// **Common Causes**:
    /// - File not found
    /// - Insufficient permissions
    /// - Disk read error
    #[error("[E1001] I/O error: {0}")]
    Io(#[from] io::Error),

    /// ZIP archive error
    ///
    /// **Error Code**: E1002
    ///
    /// **Common Causes**:
    /// - Corrupted ZIP file
    /// - Unsupported compression method
    /// - Truncated archive
    ///
    /// **Suggestions**:
    /// - Verify the file is a valid 3MF (ZIP) archive
    /// - Try re-downloading or re-exporting the file
    #[error("[E1002] ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// XML parsing error
    ///
    /// **Error Code**: E2001
    ///
    /// **Common Causes**:
    /// - Malformed XML syntax
    /// - Invalid character encoding
    /// - Unclosed tags
    #[error("[E2001] XML parsing error: {0}")]
    Xml(#[from] quick_xml::Error),

    /// XML attribute error
    ///
    /// **Error Code**: E2002
    ///
    /// **Common Causes**:
    /// - Missing required attribute
    /// - Invalid attribute value
    /// - Duplicate attribute
    ///
    /// **Suggestions**:
    /// - Check the 3MF specification for required attributes
    /// - Verify attribute values are properly formatted
    #[error("[E2002] XML attribute error: {0}")]
    XmlAttr(String),

    /// Missing required file in the 3MF archive
    ///
    /// **Error Code**: E1003
    ///
    /// **Common Causes**:
    /// - Incomplete 3MF package
    /// - Missing 3D model file
    /// - Missing content types file
    ///
    /// **Suggestions**:
    /// - Ensure the 3MF archive contains all required files
    /// - Check for [Content_Types].xml and 3D/3dmodel.model files
    #[error("[E1003] Missing required file: {0}")]
    MissingFile(String),

    /// Invalid 3MF format
    ///
    /// **Error Code**: E2004
    ///
    /// **Common Causes**:
    /// - Non-compliant OPC structure
    /// - Invalid content types
    /// - Missing required OPC relationships
    ///
    /// **Suggestions**:
    /// - Verify the file was exported correctly
    /// - Check the 3MF specification for OPC requirements
    #[error("[E2004] Invalid 3MF format: {0}")]
    InvalidFormat(String),

    /// Invalid XML structure
    ///
    /// **Error Code**: E2003
    ///
    /// **Common Causes**:
    /// - Missing required XML elements
    /// - Elements in wrong order
    /// - Invalid element nesting
    ///
    /// **Suggestions**:
    /// - Check element hierarchy matches 3MF specification
    /// - Verify required child elements are present
    #[error("[E2003] Invalid XML structure: {0}")]
    InvalidXml(String),

    /// Invalid model structure or validation failure
    ///
    /// **Error Code**: E3001
    ///
    /// **Common Causes**:
    /// - Mesh topology errors (non-manifold, degenerate triangles)
    /// - Invalid object references
    /// - Out-of-bounds vertex indices
    /// - Duplicate object IDs
    ///
    /// **Suggestions**:
    /// - Check mesh for manifold edges (each edge shared by at most 2 triangles)
    /// - Verify all vertex indices are within bounds
    /// - Ensure all object IDs are unique and positive
    /// - Validate all references point to existing objects
    #[error("[E3001] Invalid model: {0}")]
    InvalidModel(String),

    /// Parse error for numeric values
    ///
    /// **Error Code**: E3002
    ///
    /// **Common Causes**:
    /// - Invalid number format
    /// - Out-of-range values
    /// - Non-numeric characters in numeric fields
    ///
    /// **Suggestions**:
    /// - Verify numeric values use proper format (e.g., "1.5" not "1,5")
    /// - Check for special characters or extra whitespace
    #[error("[E3002] Parse error: {0}")]
    ParseError(String),

    /// Unsupported feature or extension
    ///
    /// **Error Code**: E4001
    ///
    /// **Common Causes**:
    /// - Using 3MF extensions not implemented by this parser
    /// - Future 3MF specification features
    ///
    /// **Suggestions**:
    /// - Check if the feature is part of a 3MF extension
    /// - Verify this parser supports the required extensions
    #[error("[E4001] Unsupported feature: {0}")]
    Unsupported(String),

    /// Required extension not supported
    ///
    /// **Error Code**: E4002
    ///
    /// **Common Causes**:
    /// - 3MF file requires an extension not implemented by this parser
    /// - Extension marked as required in the XML namespace
    ///
    /// **Suggestions**:
    /// - Check the file's required extensions in the model XML
    /// - Use a different parser that supports the required extension
    /// - If possible, re-export without the extension
    #[error("[E4002] Required extension not supported: {0}")]
    UnsupportedExtension(String),

    /// Invalid SecureContent keystore
    ///
    /// **Error Code**: E4003
    ///
    /// **Common Causes**:
    /// - Invalid consumer index reference (EPX-2601)
    /// - Missing consumer element when accessright is defined (EPX-2602)
    /// - Invalid encryption algorithm (EPX-2603)
    /// - Duplicate consumer IDs (EPX-2604)
    /// - Invalid encrypted file path (EPX-2605)
    /// - Missing required keystore elements (EPX-2606)
    /// - Referenced file doesn't exist in package (EPX-2607)
    ///
    /// **Suggestions**:
    /// - Verify the keystore.xml follows the 3MF SecureContent specification
    /// - Check consumer definitions and accessright references
    /// - Ensure all referenced files exist in the package
    #[error("[E4003] Invalid SecureContent: {0}")]
    InvalidSecureContent(String),

    /// XML writing error
    ///
    /// **Error Code**: E2005
    ///
    /// **Common Causes**:
    /// - Failed to serialize XML
    /// - Invalid data for XML writing
    /// - I/O error during writing
    ///
    /// **Suggestions**:
    /// - Check that data structures are valid
    /// - Ensure output stream is writable
    #[error("[E2005] XML writing error: {0}")]
    XmlWrite(String),
}

impl From<std::num::ParseFloatError> for Error {
    fn from(err: std::num::ParseFloatError) -> Self {
        Error::ParseError(format!("Failed to parse floating-point number: {}", err))
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Error::ParseError(format!("Failed to parse integer: {}", err))
    }
}

impl From<quick_xml::events::attributes::AttrError> for Error {
    fn from(err: quick_xml::events::attributes::AttrError) -> Self {
        Error::XmlAttr(format!("Attribute parsing failed: {}", err))
    }
}

impl Error {
    /// Create an InvalidXml error with element context
    ///
    /// # Arguments
    /// * `element` - The XML element name where the error occurred
    /// * `message` - Description of the error
    ///
    /// # Example
    /// ```ignore
    /// Error::invalid_xml_element("vertex", "Missing required 'x' attribute")
    /// ```
    pub fn invalid_xml_element(element: &str, message: &str) -> Self {
        Error::InvalidXml(format!("Element '<{}>': {}", element, message))
    }

    /// Create an InvalidXml error for a missing required attribute
    ///
    /// # Arguments
    /// * `element` - The XML element name
    /// * `attribute` - The missing attribute name
    ///
    /// # Example
    /// ```ignore
    /// Error::missing_attribute("object", "id")
    /// ```
    pub fn missing_attribute(element: &str, attribute: &str) -> Self {
        Error::InvalidXml(format!(
            "Element '<{}>' is missing required attribute '{}'. \
             Check the 3MF specification for required attributes.",
            element, attribute
        ))
    }

    /// Create an InvalidFormat error with context about what file/structure is invalid
    ///
    /// # Arguments
    /// * `context` - What part of the format is invalid (e.g., "OPC structure", "Content types")
    /// * `message` - Description of the error
    pub fn invalid_format_context(context: &str, message: &str) -> Self {
        Error::InvalidFormat(format!("{}: {}", context, message))
    }

    /// Create a ParseError with context about what was being parsed
    ///
    /// # Arguments
    /// * `field_name` - The name of the field being parsed (e.g., "vertex x coordinate")
    /// * `value` - The value that failed to parse
    /// * `expected_type` - The expected type (e.g., "floating-point number")
    pub fn parse_error_with_context(field_name: &str, value: &str, expected_type: &str) -> Self {
        Error::ParseError(format!(
            "Failed to parse '{}': expected {}, got '{}'. \
             Verify the value is properly formatted.",
            field_name, expected_type, value
        ))
    }

    /// Create an XmlWrite error
    ///
    /// # Arguments
    /// * `message` - Description of the writing error
    pub fn xml_write(message: String) -> Self {
        Error::XmlWrite(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes_in_messages() {
        // Verify error codes are present in error messages
        let io_err = Error::Io(io::Error::new(io::ErrorKind::NotFound, "test"));
        assert!(io_err.to_string().contains("[E1001]"));

        let missing_file = Error::MissingFile("test.model".to_string());
        assert!(missing_file.to_string().contains("[E1003]"));

        let invalid_model = Error::InvalidModel("test error".to_string());
        assert!(invalid_model.to_string().contains("[E3001]"));

        let parse_err = Error::ParseError("test".to_string());
        assert!(parse_err.to_string().contains("[E3002]"));

        let unsupported = Error::Unsupported("test feature".to_string());
        assert!(unsupported.to_string().contains("[E4001]"));
    }

    #[test]
    fn test_invalid_xml_element_helper() {
        let err = Error::invalid_xml_element("vertex", "Missing required 'x' attribute");
        assert!(err.to_string().contains("Element '<vertex>'"));
        assert!(err.to_string().contains("Missing required 'x' attribute"));
        assert!(err.to_string().contains("[E2003]"));
    }

    #[test]
    fn test_missing_attribute_helper() {
        let err = Error::missing_attribute("object", "id");
        assert!(err.to_string().contains("Element '<object>'"));
        assert!(err.to_string().contains("missing required attribute 'id'"));
        assert!(err.to_string().contains("3MF specification"));
        assert!(err.to_string().contains("[E2003]"));
    }

    #[test]
    fn test_invalid_format_context_helper() {
        let err = Error::invalid_format_context("OPC structure", "Missing relationship");
        assert!(err.to_string().contains("OPC structure"));
        assert!(err.to_string().contains("Missing relationship"));
        assert!(err.to_string().contains("[E2004]"));
    }

    #[test]
    fn test_parse_error_with_context_helper() {
        let err =
            Error::parse_error_with_context("vertex x coordinate", "abc", "floating-point number");
        assert!(err.to_string().contains("vertex x coordinate"));
        assert!(err.to_string().contains("floating-point number"));
        assert!(err.to_string().contains("'abc'"));
        assert!(err.to_string().contains("properly formatted"));
        assert!(err.to_string().contains("[E3002]"));
    }

    #[test]
    fn test_parse_float_error_conversion() {
        let parse_err: std::num::ParseFloatError = "not_a_number".parse::<f64>().unwrap_err();
        let err = Error::from(parse_err);
        assert!(err
            .to_string()
            .contains("Failed to parse floating-point number"));
        assert!(err.to_string().contains("[E3002]"));
    }

    #[test]
    fn test_parse_int_error_conversion() {
        let parse_err: std::num::ParseIntError = "not_a_number".parse::<i32>().unwrap_err();
        let err = Error::from(parse_err);
        assert!(err.to_string().contains("Failed to parse integer"));
        assert!(err.to_string().contains("[E3002]"));
    }

    #[test]
    fn test_error_context_with_hint() {
        let ctx = ErrorContext::with_hint("Check the 3MF specification");
        assert_eq!(ctx.hint, Some("Check the 3MF specification".to_string()));
        assert_eq!(ctx.file, None);
        assert_eq!(ctx.line, None);
        assert_eq!(ctx.column, None);
    }

    #[test]
    fn test_error_context_builder() {
        let ctx = ErrorContext::new()
            .file("3D/3dmodel.model")
            .line(42)
            .column(15)
            .hint("Check attribute syntax");

        assert_eq!(ctx.file, Some("3D/3dmodel.model".to_string()));
        assert_eq!(ctx.line, Some(42));
        assert_eq!(ctx.column, Some(15));
        assert_eq!(ctx.hint, Some("Check attribute syntax".to_string()));
    }

    #[test]
    fn test_error_context_display() {
        let ctx = ErrorContext::new()
            .file("3D/3dmodel.model")
            .line(42)
            .column(15)
            .hint("Check attribute syntax");

        let display = ctx.to_string();
        assert!(display.contains("File: 3D/3dmodel.model"));
        assert!(display.contains("Location: line 42, column 15"));
        assert!(display.contains("Hint: Check attribute syntax"));
    }

    #[test]
    fn test_error_context_display_partial() {
        let ctx = ErrorContext::new()
            .file("3D/3dmodel.model")
            .hint("Check the specification");

        let display = ctx.to_string();
        assert!(display.contains("File: 3D/3dmodel.model"));
        assert!(display.contains("Hint: Check the specification"));
        assert!(!display.contains("Location:"));
    }

    #[test]
    fn test_error_context_display_empty() {
        let ctx = ErrorContext::new();
        let display = ctx.to_string();
        assert_eq!(display, "");
    }
}
