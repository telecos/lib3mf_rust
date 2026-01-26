//! Displacement extension types

/// Tile style for displacement texture mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileStyle {
    /// Repeat the texture
    Wrap,
    /// Mirror the texture
    Mirror,
    /// Clamp to edge pixels
    Clamp,
    /// No displacement outside \[0,1\]
    None,
}

/// Texture filter mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    /// Auto select best quality
    Auto,
    /// Bilinear interpolation
    Linear,
    /// Nearest neighbor
    Nearest,
}

/// Displacement texture channel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    /// Red channel
    R,
    /// Green channel
    G,
    /// Blue channel
    B,
    /// Alpha channel
    A,
}

/// 2D displacement map resource from displacement extension
#[derive(Debug, Clone)]
pub struct Displacement2D {
    /// Displacement map ID
    pub id: usize,
    /// Path to the PNG file
    pub path: String,
    /// Channel to use (R, G, B, or A)
    pub channel: Channel,
    /// Tile style for u axis
    pub tilestyleu: TileStyle,
    /// Tile style for v axis
    pub tilestylev: TileStyle,
    /// Texture filter mode
    pub filter: FilterMode,
}

impl Displacement2D {
    /// Create a new displacement map
    ///
    /// Default values match the 3MF Displacement Extension specification:
    /// - channel: G (Green channel, as per spec default)
    /// - tilestyleu/tilestylev: Wrap
    /// - filter: Auto
    pub fn new(id: usize, path: String) -> Self {
        Self {
            id,
            path,
            channel: Channel::G, // Spec default is 'G'
            tilestyleu: TileStyle::Wrap,
            tilestylev: TileStyle::Wrap,
            filter: FilterMode::Auto,
        }
    }
}

/// Normalized displacement vector
#[derive(Debug, Clone, Copy)]
pub struct NormVector {
    /// X component
    pub x: f64,
    /// Y component
    pub y: f64,
    /// Z component
    pub z: f64,
}

impl NormVector {
    /// Create a new normalized vector
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

/// Group of normalized displacement vectors
#[derive(Debug, Clone)]
pub struct NormVectorGroup {
    /// Vector group ID
    pub id: usize,
    /// List of normalized vectors
    pub vectors: Vec<NormVector>,
}

impl NormVectorGroup {
    /// Create a new normalized vector group
    pub fn new(id: usize) -> Self {
        Self {
            id,
            vectors: Vec::new(),
        }
    }
}

/// 2D displacement coordinates
#[derive(Debug, Clone, Copy)]
pub struct Disp2DCoords {
    /// U coordinate
    pub u: f64,
    /// V coordinate
    pub v: f64,
    /// Index to normalized vector
    pub n: usize,
    /// Displacement factor (default 1.0)
    pub f: f64,
}

impl Disp2DCoords {
    /// Create new displacement coordinates
    pub fn new(u: f64, v: f64, n: usize) -> Self {
        Self { u, v, n, f: 1.0 }
    }
}

/// Group of 2D displacement coordinates
#[derive(Debug, Clone)]
pub struct Disp2DGroup {
    /// Group ID
    pub id: usize,
    /// Reference to Displacement2D resource
    pub dispid: usize,
    /// Reference to NormVectorGroup resource
    pub nid: usize,
    /// Height (amplitude) of displacement
    pub height: f64,
    /// Offset to displacement map
    pub offset: f64,
    /// List of displacement coordinates
    pub coords: Vec<Disp2DCoords>,
}

impl Disp2DGroup {
    /// Create a new displacement coordinate group
    pub fn new(id: usize, dispid: usize, nid: usize, height: f64) -> Self {
        Self {
            id,
            dispid,
            nid,
            height,
            offset: 0.0,
            coords: Vec::new(),
        }
    }
}
