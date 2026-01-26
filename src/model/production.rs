//! Production extension types

/// Production extension information
#[derive(Debug, Clone, PartialEq)]
pub struct ProductionInfo {
    /// UUID identifier (p:UUID attribute)
    pub uuid: Option<String>,
    /// Production path (p:path attribute)
    pub path: Option<String>,
}

impl ProductionInfo {
    /// Create a new empty ProductionInfo
    pub fn new() -> Self {
        Self {
            uuid: None,
            path: None,
        }
    }

    /// Create a ProductionInfo with just a UUID
    pub fn with_uuid(uuid: String) -> Self {
        Self {
            uuid: Some(uuid),
            path: None,
        }
    }
}

impl Default for ProductionInfo {
    fn default() -> Self {
        Self::new()
    }
}
