//! Data structures representing 3MF models

use std::collections::HashMap;

/// A 3D vertex with x, y, z coordinates
#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    /// X coordinate
    pub x: f64,
    /// Y coordinate
    pub y: f64,
    /// Z coordinate
    pub z: f64,
}

impl Vertex {
    /// Create a new vertex
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

/// A triangle defined by three vertex indices
#[derive(Debug, Clone, PartialEq)]
pub struct Triangle {
    /// Index of first vertex
    pub v1: usize,
    /// Index of second vertex
    pub v2: usize,
    /// Index of third vertex
    pub v3: usize,
    /// Optional material ID
    pub pid: Option<usize>,
}

impl Triangle {
    /// Create a new triangle
    pub fn new(v1: usize, v2: usize, v3: usize) -> Self {
        Self {
            v1,
            v2,
            v3,
            pid: None,
        }
    }

    /// Create a new triangle with material ID
    pub fn with_material(v1: usize, v2: usize, v3: usize, pid: usize) -> Self {
        Self {
            v1,
            v2,
            v3,
            pid: Some(pid),
        }
    }
}

/// A 3D mesh containing vertices and triangles
#[derive(Debug, Clone)]
pub struct Mesh {
    /// List of vertices
    pub vertices: Vec<Vertex>,
    /// List of triangles
    pub triangles: Vec<Triangle>,
}

impl Mesh {
    /// Create a new empty mesh
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            triangles: Vec::new(),
        }
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}

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
}

impl ColorGroup {
    /// Create a new color group
    pub fn new(id: usize) -> Self {
        Self {
            id,
            colors: Vec::new(),
        }
    }
}

/// A 3D object that can be a mesh or reference other objects
#[derive(Debug, Clone)]
pub struct Object {
    /// Object ID
    pub id: usize,
    /// Object name (optional)
    pub name: Option<String>,
    /// Type of object
    pub object_type: ObjectType,
    /// Optional mesh data
    pub mesh: Option<Mesh>,
    /// Optional material ID
    pub pid: Option<usize>,
}

/// Type of 3D object
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectType {
    /// A standard model object
    Model,
    /// A support structure
    Support,
    /// Other types
    Other,
}

impl Object {
    /// Create a new object
    pub fn new(id: usize) -> Self {
        Self {
            id,
            name: None,
            object_type: ObjectType::Model,
            mesh: None,
            pid: None,
        }
    }
}

/// Resources section containing objects and materials
#[derive(Debug, Clone)]
pub struct Resources {
    /// List of objects
    pub objects: Vec<Object>,
    /// List of materials
    pub materials: Vec<Material>,
    /// List of color groups (materials extension)
    pub color_groups: Vec<ColorGroup>,
}

impl Resources {
    /// Create a new empty resources section
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            materials: Vec::new(),
            color_groups: Vec::new(),
        }
    }
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}

/// An item to be built, referencing an object
#[derive(Debug, Clone)]
pub struct BuildItem {
    /// Reference to object ID
    pub objectid: usize,
    /// Optional transformation matrix (4x3 affine transformation stored as 12 values)
    /// Represents a 3x4 matrix in row-major order for affine transformations
    pub transform: Option<[f64; 12]>,
}

impl BuildItem {
    /// Create a new build item
    pub fn new(objectid: usize) -> Self {
        Self {
            objectid,
            transform: None,
        }
    }
}

/// Build section specifying which objects to manufacture
#[derive(Debug, Clone)]
pub struct Build {
    /// List of items to build
    pub items: Vec<BuildItem>,
}

impl Build {
    /// Create a new empty build section
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
}

impl Default for Build {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete 3MF model
#[derive(Debug, Clone)]
pub struct Model {
    /// Unit of measurement (e.g., "millimeter", "inch")
    pub unit: String,
    /// XML namespace
    pub xmlns: String,
    /// Metadata key-value pairs
    pub metadata: HashMap<String, String>,
    /// Resources (objects, materials)
    pub resources: Resources,
    /// Build specification
    pub build: Build,
}

impl Model {
    /// Create a new empty model
    pub fn new() -> Self {
        Self {
            unit: "millimeter".to_string(),
            xmlns: "http://schemas.microsoft.com/3dmanufacturing/core/2015/02".to_string(),
            metadata: HashMap::new(),
            resources: Resources::new(),
            build: Build::new(),
        }
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}
