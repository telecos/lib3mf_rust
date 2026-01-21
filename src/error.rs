//! Error types for 3MF parsing
//!
//! This module provides comprehensive error handling for 3MF file parsing and validation.
//! Each error includes:
//! - An error code for categorization and programmatic handling
//! - A descriptive message explaining what went wrong
//! - Optional context (file location, line numbers, element paths)
//! - Optional suggestions for fixing common issues
//!
//! # Error Codes
//!
//! Error codes are organized by category:
//! - `E1xxx`: IO and file system errors
//! - `E2xxx`: ZIP/archive errors
//! - `E3xxx`: XML parsing errors
//! - `E4xxx`: Model structure validation errors
//! - `E5xxx`: Extension and feature support errors
//!
//! # Examples
//!
//! ```
//! use lib3mf::Error;
//!
//! // Errors provide detailed context
//! let err = Error::validation_error(
//!     "Object ID must be a positive integer",
//!     Some("Object IDs start from 1. Found: 0"),
//! );
//! println!("{}", err); // Shows code, message, and suggestion
//! ```

use std::io;
use thiserror::Error;

/// Result type for 3MF operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error codes for categorizing different types of errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// IO error (E1xxx)
    Io = 1000,
    /// ZIP archive error (E2xxx)
    Zip = 2000,
    /// XML parsing error (E3xxx)
    XmlParse = 3000,
    /// XML attribute error (E3xxx)
    XmlAttr = 3001,
    /// Missing required file (E3xxx)
    MissingFile = 3002,
    /// Invalid XML structure (E3xxx)
    InvalidXml = 3003,
    /// Invalid 3MF format (E4xxx)
    InvalidFormat = 4000,
    /// Invalid model structure (E4xxx)
    InvalidModel = 4001,
    /// Invalid object ID (E4xxx)
    InvalidObjectId = 4002,
    /// Invalid mesh geometry (E4xxx)
    InvalidMeshGeometry = 4003,
    /// Invalid build reference (E4xxx)
    InvalidBuildReference = 4004,
    /// Invalid material reference (E4xxx)
    InvalidMaterialReference = 4005,
    /// Parse error (E4xxx)
    ParseError = 4006,
    /// Unsupported feature (E5xxx)
    Unsupported = 5000,
    /// Unsupported extension (E5xxx)
    UnsupportedExtension = 5001,
}

impl ErrorCode {
    /// Get the error code as a string (e.g., "E1000")
    pub fn as_str(&self) -> String {
        format!("E{:04}", *self as u16)
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Errors that can occur when parsing 3MF files
#[derive(Error, Debug)]
pub enum Error {
    /// IO error occurred while reading the file
    #[error("[{code}] IO error: {message}")]
    Io {
        /// Error code
        code: ErrorCode,
        /// Error message
        message: String,
    },

    /// ZIP archive error
    #[error("[{code}] ZIP error: {message}")]
    Zip {
        /// Error code
        code: ErrorCode,
        /// Error message
        message: String,
    },

    /// XML parsing error
    #[error("[{code}] XML error: {message}")]
    Xml {
        /// Error code
        code: ErrorCode,
        /// Error message
        message: String,
    },

    /// XML attribute error
    #[error("[{code}] XML attribute error: {message}{context}{suggestion}")]
    XmlAttr {
        /// Error code
        code: ErrorCode,
        /// Error message
        message: String,
        /// Optional context information
        context: String,
        /// Optional suggestion for fixing the error
        suggestion: String,
    },

    /// Missing required file in the 3MF archive
    #[error("[{code}] Missing required file: {file}{suggestion}")]
    MissingFile {
        /// Error code
        code: ErrorCode,
        /// File path
        file: String,
        /// Optional suggestion
        suggestion: String,
    },

    /// Invalid 3MF format
    #[error("[{code}] Invalid 3MF format: {message}{context}{suggestion}")]
    InvalidFormat {
        /// Error code
        code: ErrorCode,
        /// Error message
        message: String,
        /// Optional context
        context: String,
        /// Optional suggestion
        suggestion: String,
    },

    /// Invalid XML structure
    #[error("[{code}] Invalid XML structure: {message}{context}{suggestion}")]
    InvalidXml {
        /// Error code
        code: ErrorCode,
        /// Error message
        message: String,
        /// Optional context
        context: String,
        /// Optional suggestion
        suggestion: String,
    },

    /// Invalid model structure or validation failure
    #[error("[{code}] Invalid model: {message}{context}{suggestion}")]
    InvalidModel {
        /// Error code
        code: ErrorCode,
        /// Error message
        message: String,
        /// Optional context
        context: String,
        /// Optional suggestion
        suggestion: String,
    },

    /// Parse error for numeric values
    #[error("[{code}] Parse error: {message}{context}{suggestion}")]
    ParseError {
        /// Error code
        code: ErrorCode,
        /// Error message
        message: String,
        /// Optional context
        context: String,
        /// Optional suggestion
        suggestion: String,
    },

    /// Unsupported feature or extension
    #[error("[{code}] Unsupported feature: {message}{suggestion}")]
    Unsupported {
        /// Error code
        code: ErrorCode,
        /// Error message
        message: String,
        /// Optional suggestion
        suggestion: String,
    },

    /// Required extension not supported
    #[error("[{code}] Required extension not supported: {extension}{suggestion}")]
    UnsupportedExtension {
        /// Error code
        code: ErrorCode,
        /// Extension name
        extension: String,
        /// Optional suggestion
        suggestion: String,
    },
}

impl Error {
    /// Create a validation error with optional context and suggestion
    pub fn validation_error(message: impl Into<String>, suggestion: Option<&str>) -> Self {
        Error::InvalidModel {
            code: ErrorCode::InvalidModel,
            message: message.into(),
            context: String::new(),
            suggestion: suggestion.map(|s| format!("\n  Suggestion: {}", s)).unwrap_or_default(),
        }
    }

    /// Create a validation error with context and suggestion
    pub fn validation_error_with_context(
        message: impl Into<String>,
        context: impl Into<String>,
        suggestion: Option<&str>,
    ) -> Self {
        Error::InvalidModel {
            code: ErrorCode::InvalidModel,
            message: message.into(),
            context: format!("\n  Context: {}", context.into()),
            suggestion: suggestion.map(|s| format!("\n  Suggestion: {}", s)).unwrap_or_default(),
        }
    }

    /// Create an XML error with optional context and suggestion
    pub fn xml_error(message: impl Into<String>, suggestion: Option<&str>) -> Self {
        Error::InvalidXml {
            code: ErrorCode::InvalidXml,
            message: message.into(),
            context: String::new(),
            suggestion: suggestion.map(|s| format!("\n  Suggestion: {}", s)).unwrap_or_default(),
        }
    }

    /// Create an XML error with context and suggestion
    pub fn xml_error_with_context(
        message: impl Into<String>,
        context: impl Into<String>,
        suggestion: Option<&str>,
    ) -> Self {
        Error::InvalidXml {
            code: ErrorCode::InvalidXml,
            message: message.into(),
            context: format!("\n  Context: {}", context.into()),
            suggestion: suggestion.map(|s| format!("\n  Suggestion: {}", s)).unwrap_or_default(),
        }
    }

    /// Get the error code for this error
    pub fn code(&self) -> ErrorCode {
        match self {
            Error::Io { code, .. } => *code,
            Error::Zip { code, .. } => *code,
            Error::Xml { code, .. } => *code,
            Error::XmlAttr { code, .. } => *code,
            Error::MissingFile { code, .. } => *code,
            Error::InvalidFormat { code, .. } => *code,
            Error::InvalidXml { code, .. } => *code,
            Error::InvalidModel { code, .. } => *code,
            Error::ParseError { code, .. } => *code,
            Error::Unsupported { code, .. } => *code,
            Error::UnsupportedExtension { code, .. } => *code,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io {
            code: ErrorCode::Io,
            message: err.to_string(),
        }
    }
}

impl From<zip::result::ZipError> for Error {
    fn from(err: zip::result::ZipError) -> Self {
        Error::Zip {
            code: ErrorCode::Zip,
            message: err.to_string(),
        }
    }
}

impl From<quick_xml::Error> for Error {
    fn from(err: quick_xml::Error) -> Self {
        Error::Xml {
            code: ErrorCode::XmlParse,
            message: err.to_string(),
        }
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(err: std::num::ParseFloatError) -> Self {
        Error::ParseError {
            code: ErrorCode::ParseError,
            message: err.to_string(),
            context: String::new(),
            suggestion: "\n  Suggestion: Ensure numeric values are valid floating-point numbers".to_string(),
        }
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Error::ParseError {
            code: ErrorCode::ParseError,
            message: err.to_string(),
            context: String::new(),
            suggestion: "\n  Suggestion: Ensure numeric values are valid integers".to_string(),
        }
    }
}

impl From<quick_xml::events::attributes::AttrError> for Error {
    fn from(err: quick_xml::events::attributes::AttrError) -> Self {
        Error::XmlAttr {
            code: ErrorCode::XmlAttr,
            message: err.to_string(),
            context: String::new(),
            suggestion: "\n  Suggestion: Check that all required XML attributes are present and properly formatted".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = Error::validation_error("Test message", None);
        assert_eq!(err.code(), ErrorCode::InvalidModel);
        assert_eq!(err.code().as_str(), "E4001");
    }

    #[test]
    fn test_validation_error_with_suggestion() {
        let err = Error::validation_error(
            "Object ID must be positive",
            Some("Use id=\"1\" or higher"),
        );
        let err_str = err.to_string();
        assert!(err_str.contains("[E4001]"));
        assert!(err_str.contains("Object ID must be positive"));
        assert!(err_str.contains("Suggestion: Use id=\"1\" or higher"));
    }

    #[test]
    fn test_validation_error_with_context_and_suggestion() {
        let err = Error::validation_error_with_context(
            "Invalid vertex index",
            "Object 5, Triangle 10",
            Some("Vertex indices must be in range 0-99"),
        );
        let err_str = err.to_string();
        assert!(err_str.contains("[E4001]"));
        assert!(err_str.contains("Invalid vertex index"));
        assert!(err_str.contains("Context: Object 5, Triangle 10"));
        assert!(err_str.contains("Suggestion: Vertex indices must be in range 0-99"));
    }

    #[test]
    fn test_xml_error_with_suggestion() {
        let err = Error::xml_error("Missing required attribute", Some("Add the 'id' attribute"));
        let err_str = err.to_string();
        assert!(err_str.contains("[E3003]"));
        assert!(err_str.contains("Missing required attribute"));
        assert!(err_str.contains("Suggestion: Add the 'id' attribute"));
    }

    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode::Io.as_str(), "E1000");
        assert_eq!(ErrorCode::Zip.as_str(), "E2000");
        assert_eq!(ErrorCode::XmlParse.as_str(), "E3000");
        assert_eq!(ErrorCode::InvalidModel.as_str(), "E4001");
        assert_eq!(ErrorCode::Unsupported.as_str(), "E5000");
    }

    #[test]
    fn test_parse_error_from_conversion() {
        let parse_err: Result<f64> = "not_a_number".parse().map_err(Error::from);
        let err = parse_err.unwrap_err();
        assert_eq!(err.code(), ErrorCode::ParseError);
        let err_str = err.to_string();
        assert!(err_str.contains("[E4006]"));
        assert!(err_str.contains("Suggestion"));
    }
}
