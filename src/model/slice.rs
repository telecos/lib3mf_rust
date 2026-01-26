//! Slice extension types

/// A 2D vertex with x, y coordinates (used in slice extension)
#[derive(Debug, Clone, PartialEq)]
pub struct Vertex2D {
    /// X coordinate
    pub x: f64,
    /// Y coordinate
    pub y: f64,
}

impl Vertex2D {
    /// Create a new 2D vertex
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// A segment in a slice polygon (slice extension)
#[derive(Debug, Clone, PartialEq)]
pub struct SliceSegment {
    /// Second vertex index (first is implied by startv or previous segment)
    pub v2: usize,
}

impl SliceSegment {
    /// Create a new slice segment
    pub fn new(v2: usize) -> Self {
        Self { v2 }
    }
}

/// A polygon in a slice (slice extension)
#[derive(Debug, Clone)]
pub struct SlicePolygon {
    /// Starting vertex index
    pub startv: usize,
    /// List of segments forming the polygon
    pub segments: Vec<SliceSegment>,
}

impl SlicePolygon {
    /// Create a new slice polygon
    pub fn new(startv: usize) -> Self {
        Self {
            startv,
            segments: Vec::new(),
        }
    }
}

/// A single slice at a specific Z height (slice extension)
#[derive(Debug, Clone)]
pub struct Slice {
    /// Z coordinate of the top of this slice
    pub ztop: f64,
    /// List of 2D vertices for this slice
    pub vertices: Vec<Vertex2D>,
    /// List of polygons for this slice
    pub polygons: Vec<SlicePolygon>,
}

impl Slice {
    /// Create a new slice
    pub fn new(ztop: f64) -> Self {
        Self {
            ztop,
            vertices: Vec::new(),
            polygons: Vec::new(),
        }
    }
}

/// Reference to an external slice file (slice extension)
#[derive(Debug, Clone)]
pub struct SliceRef {
    /// Referenced slice stack ID
    pub slicestackid: usize,
    /// Path to the slice model file
    pub slicepath: String,
}

impl SliceRef {
    /// Create a new slice reference
    pub fn new(slicestackid: usize, slicepath: String) -> Self {
        Self {
            slicestackid,
            slicepath,
        }
    }
}

/// A stack of slices (slice extension)
#[derive(Debug, Clone)]
pub struct SliceStack {
    /// SliceStack ID
    pub id: usize,
    /// Z coordinate of the bottom of the slice stack
    pub zbottom: f64,
    /// List of slices in this stack
    pub slices: Vec<Slice>,
    /// List of references to external slice files
    pub slice_refs: Vec<SliceRef>,
}

impl SliceStack {
    /// Create a new slice stack
    pub fn new(id: usize, zbottom: f64) -> Self {
        Self {
            id,
            zbottom,
            slices: Vec::new(),
            slice_refs: Vec::new(),
        }
    }
}
