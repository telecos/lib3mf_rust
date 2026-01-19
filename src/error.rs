//! Error types for 3MF parsing

use std::io;
use thiserror::Error;

/// Result type for 3MF operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when parsing 3MF files
#[derive(Error, Debug)]
pub enum Error {
    /// IO error occurred while reading the file
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// ZIP archive error
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// XML parsing error
    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),

    /// XML attribute error
    #[error("XML attribute error: {0}")]
    XmlAttr(String),

    /// Missing required file in the 3MF archive
    #[error("Missing required file: {0}")]
    MissingFile(String),

    /// Invalid 3MF format
    #[error("Invalid 3MF format: {0}")]
    InvalidFormat(String),

    /// Invalid XML structure
    #[error("Invalid XML structure: {0}")]
    InvalidXml(String),

    /// Parse error for numeric values
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Unsupported feature or extension
    #[error("Unsupported feature: {0}")]
    Unsupported(String),
}

impl From<std::num::ParseFloatError> for Error {
    fn from(err: std::num::ParseFloatError) -> Self {
        Error::ParseError(err.to_string())
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Error::ParseError(err.to_string())
    }
}

impl From<quick_xml::events::attributes::AttrError> for Error {
    fn from(err: quick_xml::events::attributes::AttrError) -> Self {
        Error::XmlAttr(err.to_string())
    }
}
