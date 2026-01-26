//! Material extension types

use super::displacement::{FilterMode, TileStyle};

/// Material definition with color information
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    /// Material ID
    pub id: usize,
    /// Material name (optional)
    pub name: Option<String>,
    /// Color in RGBA format (red, green, blue, alpha)
    pub color: Option<(u8, u8, u8, u8)>,
}

impl Material {
    /// Create a new material with ID
    pub fn new(id: usize) -> Self {
        Self {
            id,
            name: None,
            color: None,
        }
    }

    /// Create a new material with color
    pub fn with_color(id: usize, r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            id,
            name: None,
            color: Some((r, g, b, a)),
        }
    }
}

/// Color group from materials extension
#[derive(Debug, Clone)]
pub struct ColorGroup {
    /// Color group ID
    pub id: usize,
    /// List of colors in this group
    pub colors: Vec<(u8, u8, u8, u8)>,
    /// Parse order (for validation of forward references)
    #[doc(hidden)]
    pub parse_order: usize,
}

impl ColorGroup {
    /// Create a new color group
    pub fn new(id: usize) -> Self {
        Self {
            id,
            colors: Vec::new(),
            parse_order: 0,
        }
    }
}

/// Base material group from materials extension
#[derive(Debug, Clone)]
pub struct BaseMaterialGroup {
    /// Base material group ID
    pub id: usize,
    /// List of base materials in this group
    pub materials: Vec<BaseMaterial>,
    /// Parse order (for validation of forward references)
    #[doc(hidden)]
    pub parse_order: usize,
}

impl BaseMaterialGroup {
    /// Create a new base material group
    pub fn new(id: usize) -> Self {
        Self {
            id,
            materials: Vec::new(),
            parse_order: 0,
        }
    }
}

/// Individual base material within a base material group
#[derive(Debug, Clone)]
pub struct BaseMaterial {
    /// Material name
    pub name: String,
    /// Display color in RGBA format (red, green, blue, alpha)
    pub displaycolor: (u8, u8, u8, u8),
}

impl BaseMaterial {
    /// Create a new base material
    pub fn new(name: String, displaycolor: (u8, u8, u8, u8)) -> Self {
        Self { name, displaycolor }
    }
}

/// Texture2D resource from materials extension
#[derive(Debug, Clone)]
pub struct Texture2D {
    /// Texture ID
    pub id: usize,
    /// Path to the texture file within the 3MF package
    pub path: String,
    /// Content type (image/jpeg or image/png)
    pub contenttype: String,
    /// Tile style for u axis
    pub tilestyleu: TileStyle,
    /// Tile style for v axis
    pub tilestylev: TileStyle,
    /// Texture filter mode
    pub filter: FilterMode,
    /// Parse order (for validation of forward references)
    #[doc(hidden)]
    pub parse_order: usize,
}

impl Texture2D {
    /// Create a new Texture2D resource
    pub fn new(id: usize, path: String, contenttype: String) -> Self {
        Self {
            id,
            path,
            contenttype,
            tilestyleu: TileStyle::Wrap,
            tilestylev: TileStyle::Wrap,
            filter: FilterMode::Auto,
            parse_order: 0,
        }
    }
}

/// Texture 2D coordinate
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tex2Coord {
    /// U coordinate (horizontal, from left)
    pub u: f32,
    /// V coordinate (vertical, from bottom)
    pub v: f32,
}

impl Tex2Coord {
    /// Create a new texture coordinate
    pub fn new(u: f32, v: f32) -> Self {
        Self { u, v }
    }
}

/// Texture2D group from materials extension
#[derive(Debug, Clone)]
pub struct Texture2DGroup {
    /// Texture2D group ID
    pub id: usize,
    /// Reference to texture2d resource
    pub texid: usize,
    /// List of texture coordinates
    pub tex2coords: Vec<Tex2Coord>,
    /// Parse order (for validation of forward references)
    #[doc(hidden)]
    pub parse_order: usize,
}

impl Texture2DGroup {
    /// Create a new texture2d group
    pub fn new(id: usize, texid: usize) -> Self {
        Self {
            id,
            texid,
            tex2coords: Vec::new(),
            parse_order: 0,
        }
    }
}

/// Composite material mixing multiple base materials
#[derive(Debug, Clone)]
pub struct Composite {
    /// Proportions of each base material (values between 0 and 1)
    pub values: Vec<f32>,
}

impl Composite {
    /// Create a new composite material
    pub fn new(values: Vec<f32>) -> Self {
        Self { values }
    }
}

/// Composite materials group from materials extension
#[derive(Debug, Clone)]
pub struct CompositeMaterials {
    /// Composite materials group ID
    pub id: usize,
    /// Reference to base material group
    pub matid: usize,
    /// Indices of materials used in composites
    pub matindices: Vec<usize>,
    /// List of composite materials
    pub composites: Vec<Composite>,
    /// Parse order (for validation of forward references)
    #[doc(hidden)]
    pub parse_order: usize,
}

impl CompositeMaterials {
    /// Create a new composite materials group
    pub fn new(id: usize, matid: usize, matindices: Vec<usize>) -> Self {
        Self {
            id,
            matid,
            matindices,
            composites: Vec::new(),
            parse_order: 0,
        }
    }
}

/// Blend method for multi-properties
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMethod {
    /// Linear mix interpolation
    Mix,
    /// Multiplicative blending
    Multiply,
}

/// Multi element combining multiple property indices
#[derive(Debug, Clone)]
pub struct Multi {
    /// Property indices corresponding to pids in parent group
    pub pindices: Vec<usize>,
}

impl Multi {
    /// Create a new multi element
    pub fn new(pindices: Vec<usize>) -> Self {
        Self { pindices }
    }
}

/// Multi-properties group from materials extension
#[derive(Debug, Clone)]
pub struct MultiProperties {
    /// Multi-properties group ID
    pub id: usize,
    /// Property group IDs to layer and blend
    pub pids: Vec<usize>,
    /// Blend methods for each layer (length = pids.len() - 1)
    pub blendmethods: Vec<BlendMethod>,
    /// List of multi elements
    pub multis: Vec<Multi>,
    /// Parse order (for validation of forward references)
    #[doc(hidden)]
    pub parse_order: usize,
}

impl MultiProperties {
    /// Create a new multi-properties group
    pub fn new(id: usize, pids: Vec<usize>) -> Self {
        Self {
            id,
            pids,
            blendmethods: Vec::new(),
            multis: Vec::new(),
            parse_order: 0,
        }
    }
}
