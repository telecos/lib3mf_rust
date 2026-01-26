//! Boolean Operations extension types

/// Boolean operation type for volumetric modeling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOpType {
    /// Union - combine two volumes
    Union,
    /// Intersection - keep only overlapping volume
    Intersection,
    /// Difference - subtract second volume from first
    Difference,
}

impl BooleanOpType {
    /// Parse operation type from string
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "union" => Some(BooleanOpType::Union),
            "intersection" => Some(BooleanOpType::Intersection),
            "difference" => Some(BooleanOpType::Difference),
            _ => None,
        }
    }

    /// Convert operation type to string
    pub fn as_str(&self) -> &'static str {
        match self {
            BooleanOpType::Union => "union",
            BooleanOpType::Intersection => "intersection",
            BooleanOpType::Difference => "difference",
        }
    }
}

/// A single boolean operation reference
#[derive(Debug, Clone)]
pub struct BooleanRef {
    /// ID of the object to use in the operation
    pub objectid: usize,
    /// Optional path to external file (Production extension)
    pub path: Option<String>,
}

impl BooleanRef {
    /// Create a new boolean reference
    pub fn new(objectid: usize) -> Self {
        Self {
            objectid,
            path: None,
        }
    }
}

/// Boolean shape definition
#[derive(Debug, Clone)]
pub struct BooleanShape {
    /// The base object ID
    pub objectid: usize,
    /// The boolean operation to perform
    pub operation: BooleanOpType,
    /// Optional path to external file for base object (Production extension)
    pub path: Option<String>,
    /// List of operand objects
    pub operands: Vec<BooleanRef>,
}

impl BooleanShape {
    /// Create a new boolean shape
    pub fn new(objectid: usize, operation: BooleanOpType) -> Self {
        Self {
            objectid,
            operation,
            path: None,
            operands: Vec::new(),
        }
    }
}
