//! Data structures representing 3MF models

use std::collections::{HashMap, HashSet};

/// 3MF extension specification
///
/// Represents the various official 3MF extensions that can be used in 3MF files.
/// Extensions add additional capabilities beyond the core specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Extension {
    /// Core 3MF specification (always required)
    Core,
    /// Materials & Properties Extension
    Material,
    /// Production Extension
    Production,
    /// Slice Extension
    Slice,
    /// Beam Lattice Extension
    BeamLattice,
    /// Secure Content Extension
    SecureContent,
    /// Boolean Operations Extension
    BooleanOperations,
    /// Displacement Extension
    Displacement,
}

impl Extension {
    /// Get the namespace URI for this extension
    pub fn namespace(&self) -> &'static str {
        match self {
            Extension::Core => "http://schemas.microsoft.com/3dmanufacturing/core/2015/02",
            Extension::Material => "http://schemas.microsoft.com/3dmanufacturing/material/2015/02",
            Extension::Production => {
                "http://schemas.microsoft.com/3dmanufacturing/production/2015/06"
            }
            Extension::Slice => "http://schemas.microsoft.com/3dmanufacturing/slice/2015/07",
            Extension::BeamLattice => {
                "http://schemas.microsoft.com/3dmanufacturing/beamlattice/2017/02"
            }
            Extension::SecureContent => {
                "http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07"
            }
            Extension::BooleanOperations => {
                "http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07"
            }
            Extension::Displacement => {
                "http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07"
            }
        }
    }

    /// Get extension from namespace URI
    pub fn from_namespace(namespace: &str) -> Option<Self> {
        match namespace {
            "http://schemas.microsoft.com/3dmanufacturing/core/2015/02" => Some(Extension::Core),
            "http://schemas.microsoft.com/3dmanufacturing/material/2015/02" => {
                Some(Extension::Material)
            }
            "http://schemas.microsoft.com/3dmanufacturing/production/2015/06" => {
                Some(Extension::Production)
            }
            "http://schemas.microsoft.com/3dmanufacturing/slice/2015/07" => Some(Extension::Slice),
            "http://schemas.microsoft.com/3dmanufacturing/beamlattice/2017/02" => {
                Some(Extension::BeamLattice)
            }
            "http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07" => {
                Some(Extension::SecureContent)
            }
            "http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07" => {
                Some(Extension::BooleanOperations)
            }
            "http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07" => {
                Some(Extension::Displacement)
            }
            _ => None,
        }
    }

    /// Get a human-readable name for this extension
    pub fn name(&self) -> &'static str {
        match self {
            Extension::Core => "Core",
            Extension::Material => "Material",
            Extension::Production => "Production",
            Extension::Slice => "Slice",
            Extension::BeamLattice => "BeamLattice",
            Extension::SecureContent => "SecureContent",
            Extension::BooleanOperations => "BooleanOperations",
            Extension::Displacement => "Displacement",
        }
    }
}

/// Configuration for parsing 3MF files
///
/// Allows consumers to specify which extensions they support.
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Set of extensions supported by the consumer
    /// Core is always implicitly supported
    supported_extensions: HashSet<Extension>,
}

impl ParserConfig {
    /// Create a new parser configuration with only core support
    pub fn new() -> Self {
        let mut supported = HashSet::new();
        supported.insert(Extension::Core);
        Self {
            supported_extensions: supported,
        }
    }

    /// Create a parser configuration that supports all known extensions
    ///
    /// Note: When new extensions are added to the Extension enum, they must
    /// be manually added here as well.
    pub fn with_all_extensions() -> Self {
        let mut supported = HashSet::new();
        supported.insert(Extension::Core);
        supported.insert(Extension::Material);
        supported.insert(Extension::Production);
        supported.insert(Extension::Slice);
        supported.insert(Extension::BeamLattice);
        supported.insert(Extension::SecureContent);
        supported.insert(Extension::BooleanOperations);
        supported.insert(Extension::Displacement);
        Self {
            supported_extensions: supported,
        }
    }

    /// Add support for a specific extension
    pub fn with_extension(mut self, extension: Extension) -> Self {
        self.supported_extensions.insert(extension);
        self
    }

    /// Check if an extension is supported
    pub fn supports(&self, extension: &Extension) -> bool {
        self.supported_extensions.contains(extension)
    }

    /// Get the set of supported extensions
    pub fn supported_extensions(&self) -> &HashSet<Extension> {
        &self.supported_extensions
    }
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self::new()
    }
}

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
    /// Optional material ID (property ID)
    pub pid: Option<usize>,
    /// Optional material index for entire triangle (property index)
    pub pindex: Option<usize>,
    /// Optional material index for vertex 1 (property index)
    pub p1: Option<usize>,
    /// Optional material index for vertex 2 (property index)
    pub p2: Option<usize>,
    /// Optional material index for vertex 3 (property index)
    pub p3: Option<usize>,
}

impl Triangle {
    /// Create a new triangle
    pub fn new(v1: usize, v2: usize, v3: usize) -> Self {
        Self {
            v1,
            v2,
            v3,
            pid: None,
            pindex: None,
            p1: None,
            p2: None,
            p3: None,
        }
    }

    /// Create a new triangle with material ID
    pub fn with_material(v1: usize, v2: usize, v3: usize, pid: usize) -> Self {
        Self {
            v1,
            v2,
            v3,
            pid: Some(pid),
            pindex: None,
            p1: None,
            p2: None,
            p3: None,
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
    /// Optional beam lattice structure (Beam Lattice Extension)
    pub beamset: Option<BeamSet>,
}

impl Mesh {
    /// Create a new empty mesh
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            triangles: Vec::new(),
            beamset: None,
        }
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}

/// Cap mode for beam lattice ends
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BeamCapMode {
    /// Sphere cap (rounded ends)
    #[default]
    Sphere,
    /// Butt cap (flat ends)
    Butt,
}

/// A single beam in a beam lattice structure
///
/// Beams connect two vertices with optional varying radii along the beam.
/// Part of the Beam Lattice Extension specification.
#[derive(Debug, Clone, PartialEq)]
pub struct Beam {
    /// Index of first vertex
    pub v1: usize,
    /// Index of second vertex
    pub v2: usize,
    /// Radius at first vertex (optional, defaults to beamset radius)
    pub r1: Option<f64>,
    /// Radius at second vertex (optional, defaults to r1 or beamset radius)
    pub r2: Option<f64>,
}

impl Beam {
    /// Create a new beam between two vertices
    pub fn new(v1: usize, v2: usize) -> Self {
        Self {
            v1,
            v2,
            r1: None,
            r2: None,
        }
    }

    /// Create a new beam with a specific radius at v1
    pub fn with_radius(v1: usize, v2: usize, r1: f64) -> Self {
        Self {
            v1,
            v2,
            r1: Some(r1),
            r2: None,
        }
    }

    /// Create a new beam with different radii at both ends
    pub fn with_radii(v1: usize, v2: usize, r1: f64, r2: f64) -> Self {
        Self {
            v1,
            v2,
            r1: Some(r1),
            r2: Some(r2),
        }
    }
}

/// A beam lattice structure containing beams and lattice properties
///
/// Part of the Beam Lattice Extension specification.
/// Defines a lattice structure made of beams connecting vertices.
#[derive(Debug, Clone)]
pub struct BeamSet {
    /// Default radius for beams (when not specified per-beam)
    pub radius: f64,
    /// Minimum length for beams
    pub min_length: f64,
    /// Cap mode for beam ends
    pub cap_mode: BeamCapMode,
    /// List of beams in the lattice
    pub beams: Vec<Beam>,
}

impl BeamSet {
    /// Create a new beam set with default values
    pub fn new() -> Self {
        Self {
            radius: 1.0,
            min_length: 0.0001,
            cap_mode: BeamCapMode::Sphere,
            beams: Vec::new(),
        }
    }

    /// Create a new beam set with a specific radius
    pub fn with_radius(radius: f64) -> Self {
        Self {
            radius,
            min_length: 0.0001,
            cap_mode: BeamCapMode::Sphere,
            beams: Vec::new(),
        }
    }
}

impl Default for BeamSet {
    fn default() -> Self {
        Self::new()
    }
}

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
    pub fn from_str(s: &str) -> Option<Self> {
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
        Self { objectid, path: None }
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

/// Production extension information
#[derive(Debug, Clone, PartialEq)]
pub struct ProductionInfo {
    /// UUID identifier (p:UUID attribute)
    pub uuid: Option<String>,
    /// Production path (p:path attribute)
    pub path: Option<String>,
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

/// Secure content metadata (read-only awareness)
///
/// This structure provides minimal awareness of secure content elements
/// without implementing actual cryptographic operations. It tracks which
/// files are encrypted and keystore metadata.
///
/// **Note**: These fields are currently unused placeholders reserved for
/// future implementation. Parsing logic to populate these fields has not
/// been implemented yet. The extension is recognized for validation purposes
/// only.
///
/// **Security Warning**: This does NOT decrypt content or verify signatures.
/// See SECURE_CONTENT_SUPPORT.md for security considerations.
#[derive(Debug, Clone, Default)]
pub struct SecureContentInfo {
    /// UUID of the keystore (if present)
    pub keystore_uuid: Option<String>,
    /// Paths to encrypted files in the package
    pub encrypted_files: Vec<String>,
}

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

/// Base material group from materials extension
#[derive(Debug, Clone)]
pub struct BaseMaterialGroup {
    /// Base material group ID
    pub id: usize,
    /// List of base materials in this group
    pub materials: Vec<BaseMaterial>,
}

impl BaseMaterialGroup {
    /// Create a new base material group
    pub fn new(id: usize) -> Self {
        Self {
            id,
            materials: Vec::new(),
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
    /// Optional material ID (property ID)
    pub pid: Option<usize>,
    /// Optional material index (property index) - used with pid to select from color groups
    pub pindex: Option<usize>,
    /// Optional slice stack ID (slice extension)
    pub slicestackid: Option<usize>,
    /// Production extension information (UUID, path)
    pub production: Option<ProductionInfo>,
    /// Boolean shape definition (Boolean Operations extension)
    pub boolean_shape: Option<BooleanShape>,
}

/// Type of 3D object
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectType {
    /// A standard model object
    Model,
    /// A support structure
    Support,
    /// A solid support structure
    SolidSupport,
    /// A surface object  
    Surface,
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
            pindex: None,
            slicestackid: None,
            production: None,
            boolean_shape: None,
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
    /// List of slice stacks (slice extension)
    pub slice_stacks: Vec<SliceStack>,
    /// List of base material groups (materials extension)
    pub base_material_groups: Vec<BaseMaterialGroup>,
}

impl Resources {
    /// Create a new empty resources section
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            materials: Vec::new(),
            color_groups: Vec::new(),
            slice_stacks: Vec::new(),
            base_material_groups: Vec::new(),
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
    /// Production extension UUID (p:UUID attribute)
    pub production_uuid: Option<String>,
}

impl BuildItem {
    /// Create a new build item
    pub fn new(objectid: usize) -> Self {
        Self {
            objectid,
            transform: None,
            production_uuid: None,
        }
    }
}

/// Build section specifying which objects to manufacture
#[derive(Debug, Clone)]
pub struct Build {
    /// List of items to build
    pub items: Vec<BuildItem>,
    /// Production extension UUID (p:UUID attribute)
    pub production_uuid: Option<String>,
}

impl Build {
    /// Create a new empty build section
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            production_uuid: None,
        }
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
    /// Required extensions for this model
    /// Extensions that the consumer must support to properly process this file
    pub required_extensions: Vec<Extension>,
    /// Metadata key-value pairs
    pub metadata: HashMap<String, String>,
    /// Resources (objects, materials)
    pub resources: Resources,
    /// Build specification
    pub build: Build,
    /// Secure content information (if secure content extension is used)
    pub secure_content: Option<SecureContentInfo>,
}

impl Model {
    /// Create a new empty model
    pub fn new() -> Self {
        Self {
            unit: "millimeter".to_string(),
            xmlns: "http://schemas.microsoft.com/3dmanufacturing/core/2015/02".to_string(),
            required_extensions: Vec::new(),
            metadata: HashMap::new(),
            resources: Resources::new(),
            build: Build::new(),
            secure_content: None,
        }
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}
