//! Data structures representing 3MF models

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

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
/// Allows consumers to specify which extensions they support and register
/// custom extension handlers.
#[derive(Clone)]
pub struct ParserConfig {
    /// Set of extensions supported by the consumer
    /// Core is always implicitly supported
    supported_extensions: HashSet<Extension>,
    /// Registered custom extensions with their handlers
    custom_extensions: HashMap<String, CustomExtensionInfo>,
}

impl ParserConfig {
    /// Create a new parser configuration with only core support
    pub fn new() -> Self {
        let mut supported = HashSet::new();
        supported.insert(Extension::Core);
        Self {
            supported_extensions: supported,
            custom_extensions: HashMap::new(),
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
            custom_extensions: HashMap::new(),
        }
    }

    /// Add support for a specific extension
    pub fn with_extension(mut self, extension: Extension) -> Self {
        self.supported_extensions.insert(extension);
        self
    }

    /// Register a custom extension with optional handlers
    ///
    /// # Arguments
    ///
    /// * `namespace` - The namespace URI of the custom extension
    /// * `name` - A human-readable name for the extension
    ///
    /// # Example
    ///
    /// ```
    /// use lib3mf::ParserConfig;
    /// use std::sync::Arc;
    ///
    /// let config = ParserConfig::new()
    ///     .with_custom_extension(
    ///         "http://example.com/myextension/2024/01",
    ///         "MyExtension"
    ///     );
    /// ```
    pub fn with_custom_extension(
        mut self,
        namespace: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        let namespace = namespace.into();
        let name = name.into();
        self.custom_extensions.insert(
            namespace.clone(),
            CustomExtensionInfo {
                namespace,
                name,
                element_handler: None,
                validation_handler: None,
            },
        );
        self
    }

    /// Register a custom extension with an element handler
    ///
    /// # Arguments
    ///
    /// * `namespace` - The namespace URI of the custom extension
    /// * `name` - A human-readable name for the extension
    /// * `handler` - Callback function to handle elements from this extension
    ///
    /// # Example
    ///
    /// ```
    /// use lib3mf::{ParserConfig, CustomExtensionContext, CustomElementResult};
    /// use std::sync::Arc;
    ///
    /// let config = ParserConfig::new()
    ///     .with_custom_extension_handler(
    ///         "http://example.com/myextension/2024/01",
    ///         "MyExtension",
    ///         Arc::new(|ctx: &CustomExtensionContext| {
    ///             println!("Handling element: {}", ctx.element_name);
    ///             Ok(CustomElementResult::Handled)
    ///         })
    ///     );
    /// ```
    pub fn with_custom_extension_handler(
        mut self,
        namespace: impl Into<String>,
        name: impl Into<String>,
        handler: CustomElementHandler,
    ) -> Self {
        let namespace = namespace.into();
        let name = name.into();
        self.custom_extensions.insert(
            namespace.clone(),
            CustomExtensionInfo {
                namespace,
                name,
                element_handler: Some(handler),
                validation_handler: None,
            },
        );
        self
    }

    /// Register a custom extension with both element and validation handlers
    ///
    /// # Arguments
    ///
    /// * `namespace` - The namespace URI of the custom extension
    /// * `name` - A human-readable name for the extension
    /// * `element_handler` - Callback function to handle elements from this extension
    /// * `validation_handler` - Callback function to validate the model
    ///
    /// # Example
    ///
    /// ```
    /// use lib3mf::{ParserConfig, CustomExtensionContext, CustomElementResult};
    /// use std::sync::Arc;
    ///
    /// let config = ParserConfig::new()
    ///     .with_custom_extension_handlers(
    ///         "http://example.com/myextension/2024/01",
    ///         "MyExtension",
    ///         Arc::new(|ctx: &CustomExtensionContext| {
    ///             Ok(CustomElementResult::Handled)
    ///         }),
    ///         Arc::new(|model| {
    ///             Ok(())
    ///         })
    ///     );
    /// ```
    pub fn with_custom_extension_handlers(
        mut self,
        namespace: impl Into<String>,
        name: impl Into<String>,
        element_handler: CustomElementHandler,
        validation_handler: CustomValidationHandler,
    ) -> Self {
        let namespace = namespace.into();
        let name = name.into();
        self.custom_extensions.insert(
            namespace.clone(),
            CustomExtensionInfo {
                namespace,
                name,
                element_handler: Some(element_handler),
                validation_handler: Some(validation_handler),
            },
        );
        self
    }

    /// Check if an extension is supported
    pub fn supports(&self, extension: &Extension) -> bool {
        self.supported_extensions.contains(extension)
    }

    /// Check if a custom extension is registered by namespace
    pub fn has_custom_extension(&self, namespace: &str) -> bool {
        self.custom_extensions.contains_key(namespace)
    }

    /// Get the set of supported extensions
    pub fn supported_extensions(&self) -> &HashSet<Extension> {
        &self.supported_extensions
    }

    /// Get information about a custom extension by namespace
    pub fn get_custom_extension(&self, namespace: &str) -> Option<&CustomExtensionInfo> {
        self.custom_extensions.get(namespace)
    }

    /// Get all registered custom extensions
    pub fn custom_extensions(&self) -> &HashMap<String, CustomExtensionInfo> {
        &self.custom_extensions
    }
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ParserConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParserConfig")
            .field("supported_extensions", &self.supported_extensions)
            .field("custom_extensions_count", &self.custom_extensions.len())
            .finish()
    }
}

impl std::fmt::Debug for CustomExtensionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomExtensionInfo")
            .field("namespace", &self.namespace)
            .field("name", &self.name)
            .field("has_element_handler", &self.element_handler.is_some())
            .field("has_validation_handler", &self.validation_handler.is_some())
            .finish()
    }
}

/// Context information passed to custom extension callbacks
#[derive(Debug, Clone)]
pub struct CustomExtensionContext {
    /// The element name (without namespace prefix)
    pub element_name: String,
    /// The namespace URI of the element
    pub namespace: String,
    /// Attributes of the element as key-value pairs
    pub attributes: HashMap<String, String>,
}

/// Result of a custom element handler
#[derive(Debug, Clone)]
pub enum CustomElementResult {
    /// Element was handled by the callback
    Handled,
    /// Element was not recognized/handled by the callback
    NotHandled,
}

/// Callback function for handling custom extension elements
///
/// This callback is invoked when the parser encounters an element from a namespace
/// that is not a known 3MF extension. The callback can inspect the element and
/// its attributes, and decide whether to handle it.
///
/// # Arguments
///
/// * `context` - Information about the element being parsed
///
/// # Returns
///
/// * `Ok(CustomElementResult::Handled)` - The element was recognized and handled
/// * `Ok(CustomElementResult::NotHandled)` - The element was not recognized
/// * `Err(error_message)` - An error occurred while handling the element
pub type CustomElementHandler =
    Arc<dyn Fn(&CustomExtensionContext) -> Result<CustomElementResult, String> + Send + Sync>;

/// Callback function for custom extension validation
///
/// This callback is invoked during model validation to allow custom validation
/// rules for custom extensions.
///
/// # Arguments
///
/// * `model` - The parsed 3MF model
///
/// # Returns
///
/// * `Ok(())` - Validation passed
/// * `Err(error_message)` - Validation failed with an error message
pub type CustomValidationHandler = Arc<dyn Fn(&Model) -> Result<(), String> + Send + Sync>;

/// Information about a registered custom extension
#[derive(Clone)]
pub struct CustomExtensionInfo {
    /// The namespace URI of the custom extension
    pub namespace: String,
    /// Human-readable name for the extension
    pub name: String,
    /// Optional element handler callback
    pub element_handler: Option<CustomElementHandler>,
    /// Optional validation handler callback
    pub validation_handler: Option<CustomValidationHandler>,
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

    /// Create a new mesh with pre-allocated capacity
    ///
    /// This is useful for performance when the number of vertices and triangles
    /// is known in advance, as it avoids multiple reallocations.
    pub fn with_capacity(vertices: usize, triangles: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertices),
            triangles: Vec::with_capacity(triangles),
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
    /// Hemisphere cap (half sphere at end)
    Hemisphere,
}

impl std::fmt::Display for BeamCapMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BeamCapMode::Sphere => write!(f, "sphere"),
            BeamCapMode::Butt => write!(f, "butt"),
            BeamCapMode::Hemisphere => write!(f, "hemisphere"),
        }
    }
}

impl std::str::FromStr for BeamCapMode {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sphere" => Ok(BeamCapMode::Sphere),
            "butt" => Ok(BeamCapMode::Butt),
            "hemisphere" => Ok(BeamCapMode::Hemisphere),
            _ => Err(crate::error::Error::InvalidXml(format!(
                "Invalid cap mode '{}'. Must be 'sphere', 'butt', or 'hemisphere'",
                s
            ))),
        }
    }
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
    /// Cap mode at first vertex (optional, defaults to beamset cap mode)
    pub cap1: Option<BeamCapMode>,
    /// Cap mode at second vertex (optional, defaults to beamset cap mode)
    pub cap2: Option<BeamCapMode>,
}

impl Beam {
    /// Create a new beam between two vertices
    pub fn new(v1: usize, v2: usize) -> Self {
        Self {
            v1,
            v2,
            r1: None,
            r2: None,
            cap1: None,
            cap2: None,
        }
    }

    /// Create a new beam with a specific radius at v1
    pub fn with_radius(v1: usize, v2: usize, r1: f64) -> Self {
        Self {
            v1,
            v2,
            r1: Some(r1),
            r2: None,
            cap1: None,
            cap2: None,
        }
    }

    /// Create a new beam with different radii at both ends
    pub fn with_radii(v1: usize, v2: usize, r1: f64, r2: f64) -> Self {
        Self {
            v1,
            v2,
            r1: Some(r1),
            r2: Some(r2),
            cap1: None,
            cap2: None,
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

/// Tile style for displacement texture mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileStyle {
    /// Repeat the texture
    Wrap,
    /// Mirror the texture
    Mirror,
    /// Clamp to edge pixels
    Clamp,
    /// No displacement outside [0,1]
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
}

impl Texture2DGroup {
    /// Create a new texture2d group
    pub fn new(id: usize, texid: usize) -> Self {
        Self {
            id,
            texid,
            tex2coords: Vec::new(),
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
}

impl CompositeMaterials {
    /// Create a new composite materials group
    pub fn new(id: usize, matid: usize, matindices: Vec<usize>) -> Self {
        Self {
            id,
            matid,
            matindices,
            composites: Vec::new(),
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
}

impl MultiProperties {
    /// Create a new multi-properties group
    pub fn new(id: usize, pids: Vec<usize>) -> Self {
        Self {
            id,
            pids,
            blendmethods: Vec::new(),
            multis: Vec::new(),
        }
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

/// A component that references another object with optional transformation
///
/// Components allow objects to reference other objects to create assemblies.
/// The referenced object can be transformed using a 4x3 affine transformation matrix.
#[derive(Debug, Clone)]
pub struct Component {
    /// ID of the referenced object
    pub objectid: usize,
    /// Optional 4x3 transformation matrix (12 floats in row-major order)
    ///
    /// Format: [m00 m01 m02 m10 m11 m12 m20 m21 m22 tx ty tz]
    ///
    /// The first 9 values form a 3x3 rotation/scale matrix:
    /// ```text
    /// | m00 m01 m02 |
    /// | m10 m11 m12 |
    /// | m20 m21 m22 |
    /// ```
    ///
    /// The last 3 values are translation components:
    /// - tx (index 9): translation along X axis
    /// - ty (index 10): translation along Y axis  
    /// - tz (index 11): translation along Z axis
    pub transform: Option<[f64; 12]>,
}

impl Component {
    /// Create a new component with the given object reference
    pub fn new(objectid: usize) -> Self {
        Self {
            objectid,
            transform: None,
        }
    }

    /// Create a new component with a transformation matrix
    pub fn with_transform(objectid: usize, transform: [f64; 12]) -> Self {
        Self {
            objectid,
            transform: Some(transform),
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
    /// Optional material ID (property ID)
    pub pid: Option<usize>,
    /// Optional material index (property index) - used with pid to select from color groups
    pub pindex: Option<usize>,
    /// Optional base material ID reference (materials extension)
    pub basematerialid: Option<usize>,
    /// Optional slice stack ID (slice extension)
    pub slicestackid: Option<usize>,
    /// Production extension information (UUID, path)
    pub production: Option<ProductionInfo>,
    /// Boolean shape definition (Boolean Operations extension)
    pub boolean_shape: Option<BooleanShape>,
    /// Components that reference other objects (assemblies)
    pub components: Vec<Component>,
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
            basematerialid: None,
            slicestackid: None,
            production: None,
            boolean_shape: None,
            components: Vec::new(),
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
    /// List of displacement maps (displacement extension)
    pub displacement_maps: Vec<Displacement2D>,
    /// List of normalized vector groups (displacement extension)
    pub norm_vector_groups: Vec<NormVectorGroup>,
    /// List of displacement coordinate groups (displacement extension)
    pub disp2d_groups: Vec<Disp2DGroup>,
    /// List of slice stacks (slice extension)
    pub slice_stacks: Vec<SliceStack>,
    /// List of base material groups (materials extension)
    pub base_material_groups: Vec<BaseMaterialGroup>,
    /// List of texture2d resources (materials extension)
    pub texture2d_resources: Vec<Texture2D>,
    /// List of texture2d groups (materials extension)
    pub texture2d_groups: Vec<Texture2DGroup>,
    /// List of composite materials groups (materials extension)
    pub composite_materials: Vec<CompositeMaterials>,
    /// List of multi-properties groups (materials extension)
    pub multi_properties: Vec<MultiProperties>,
}

impl Resources {
    /// Create a new empty resources section
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            materials: Vec::new(),
            color_groups: Vec::new(),
            displacement_maps: Vec::new(),
            norm_vector_groups: Vec::new(),
            disp2d_groups: Vec::new(),
            slice_stacks: Vec::new(),
            base_material_groups: Vec::new(),
            texture2d_resources: Vec::new(),
            texture2d_groups: Vec::new(),
            composite_materials: Vec::new(),
            multi_properties: Vec::new(),
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
    /// Production extension path (p:path attribute) - references a separate model file
    /// Used in multi-part 3MF files to reference objects defined in other model files
    pub production_path: Option<String>,
}

impl BuildItem {
    /// Create a new build item
    pub fn new(objectid: usize) -> Self {
        Self {
            objectid,
            transform: None,
            production_uuid: None,
            production_path: None,
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

/// Metadata entry for 3MF package
///
/// Represents a metadata key-value pair with optional preservation flag.
/// According to the 3MF Core Specification Chapter 4, metadata elements
/// contain a required `name` attribute and text content value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataEntry {
    /// Name of the metadata entry (required by 3MF spec)
    pub name: String,
    /// Value of the metadata entry
    pub value: String,
    /// Preservation flag (optional attribute)
    /// When true, indicates this metadata should be preserved during editing
    pub preserve: Option<bool>,
}

impl MetadataEntry {
    /// Create a new metadata entry
    pub fn new(name: String, value: String) -> Self {
        Self {
            name,
            value,
            preserve: None,
        }
    }

    /// Create a new metadata entry with preservation flag
    pub fn new_with_preserve(name: String, value: String, preserve: bool) -> Self {
        Self {
            name,
            value,
            preserve: Some(preserve),
        }
    }
}

/// Thumbnail metadata for 3MF package
///
/// Represents a thumbnail image referenced in the package relationships.
/// Thumbnails are typically stored in `/Metadata/thumbnail.png` or similar paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Thumbnail {
    /// Path to the thumbnail file within the package
    pub path: String,
    /// Content type (e.g., "image/png", "image/jpeg")
    pub content_type: String,
}

impl Thumbnail {
    /// Create a new thumbnail metadata
    pub fn new(path: String, content_type: String) -> Self {
        Self { path, content_type }
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
    /// Required custom extension namespaces (not part of standard 3MF)
    pub required_custom_extensions: Vec<String>,
    /// Metadata entries with name, value, and optional preservation flag
    pub metadata: Vec<MetadataEntry>,
    /// Thumbnail metadata (if present in the package)
    pub thumbnail: Option<Thumbnail>,
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
            required_custom_extensions: Vec::new(),
            metadata: Vec::new(),
            thumbnail: None,
            resources: Resources::new(),
            build: Build::new(),
            secure_content: None,
        }
    }

    /// Get metadata value by name (helper for backward compatibility)
    pub fn get_metadata(&self, name: &str) -> Option<&str> {
        self.metadata
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| entry.value.as_str())
    }

    /// Check if metadata entry exists with the given name
    pub fn has_metadata(&self, name: &str) -> bool {
        self.metadata.iter().any(|entry| entry.name == name)
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}
