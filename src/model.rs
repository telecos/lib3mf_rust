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
                "http://schemas.microsoft.com/3dmanufacturing/volumetric/2021/08"
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
            "http://schemas.microsoft.com/3dmanufacturing/volumetric/2021/08" => {
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

/// Data structure representing parsed custom extension elements
///
/// This holds arbitrary XML element data from custom extensions that can be
/// accessed after parsing.
#[derive(Debug, Clone, PartialEq)]
pub struct CustomElementData {
    /// The element name (without namespace prefix)
    pub local_name: String,
    /// The namespace URI
    pub namespace: String,
    /// Element attributes as key-value pairs
    pub attributes: HashMap<String, String>,
    /// Optional text content
    pub text_content: Option<String>,
    /// Nested child elements
    pub children: Vec<CustomElementData>,
}

impl CustomElementData {
    /// Create a new custom element data structure
    pub fn new(local_name: String, namespace: String) -> Self {
        Self {
            local_name,
            namespace,
            attributes: HashMap::new(),
            text_content: None,
            children: Vec::new(),
        }
    }
}

/// Callback function type for custom element parsing
///
/// This function is called when a custom extension element is encountered.
/// It receives the element data and should return Ok(()) if the element is valid,
/// or an error if validation fails.
///
/// # Arguments
///
/// * `element` - The parsed custom element data
///
/// # Returns
///
/// Ok(()) if the element is valid, or an error describing the validation failure
pub type CustomElementCallback = Arc<dyn Fn(&CustomElementData) -> crate::error::Result<()> + Send + Sync>;

/// Custom extension specification
///
/// Represents a user-defined 3MF extension with its namespace URI and
/// optional parsing/validation callback.
#[derive(Clone)]
pub struct CustomExtension {
    /// Namespace URI for this custom extension
    namespace: String,
    /// Human-readable name for this extension
    name: String,
    /// Optional callback for parsing and validating custom elements
    /// If provided, this callback will be invoked for each element in this namespace
    callback: Option<CustomElementCallback>,
}

impl CustomExtension {
    /// Create a new custom extension
    ///
    /// # Arguments
    ///
    /// * `namespace` - The namespace URI for this extension
    /// * `name` - A human-readable name for this extension
    ///
    /// # Example
    ///
    /// ```
    /// use lib3mf::CustomExtension;
    ///
    /// let ext = CustomExtension::new(
    ///     "http://example.com/myextension".to_string(),
    ///     "MyExtension".to_string()
    /// );
    /// ```
    pub fn new(namespace: String, name: String) -> Self {
        Self {
            namespace,
            name,
            callback: None,
        }
    }

    /// Create a custom extension with a parsing callback
    ///
    /// # Arguments
    ///
    /// * `namespace` - The namespace URI for this extension
    /// * `name` - A human-readable name for this extension
    /// * `callback` - Callback function to be invoked when parsing elements from this namespace
    ///
    /// # Example
    ///
    /// ```
    /// use lib3mf::{CustomExtension, CustomElementData};
    /// use std::sync::Arc;
    ///
    /// let callback = Arc::new(|element: &CustomElementData| {
    ///     // Validate custom element
    ///     if element.local_name == "myelem" {
    ///         Ok(())
    ///     } else {
    ///         Err(lib3mf::Error::InvalidXml(
    ///             format!("Unknown element: {}", element.local_name)
    ///         ))
    ///     }
    /// });
    ///
    /// let ext = CustomExtension::with_callback(
    ///     "http://example.com/myextension".to_string(),
    ///     "MyExtension".to_string(),
    ///     callback
    /// );
    /// ```
    pub fn with_callback(
        namespace: String,
        name: String,
        callback: CustomElementCallback,
    ) -> Self {
        Self {
            namespace,
            name,
            callback: Some(callback),
        }
    }

    /// Get the namespace URI for this extension
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Get the human-readable name for this extension
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the callback function if one is registered
    pub(crate) fn callback(&self) -> Option<&CustomElementCallback> {
        self.callback.as_ref()
    }
}

impl std::fmt::Debug for CustomExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomExtension")
            .field("namespace", &self.namespace)
            .field("name", &self.name)
            .field("callback", &self.callback.is_some())
            .finish()
    }
}

/// Configuration for parsing 3MF files
///
/// Allows consumers to specify which extensions they support.
#[derive(Clone)]
pub struct ParserConfig {
    /// Set of extensions supported by the consumer
    /// Core is always implicitly supported
    supported_extensions: HashSet<Extension>,
    /// Map of custom extensions by namespace URI
    custom_extensions: HashMap<String, CustomExtension>,
}

impl std::fmt::Debug for ParserConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParserConfig")
            .field("supported_extensions", &self.supported_extensions)
            .field("custom_extensions", &self.custom_extensions.keys())
            .finish()
    }
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

    /// Register a custom extension
    ///
    /// This allows you to register a custom/proprietary 3MF extension that is not
    /// part of the official 3MF specification. When the parser encounters elements
    /// from this namespace, it will invoke the callback (if provided) for validation.
    ///
    /// # Arguments
    ///
    /// * `extension` - The custom extension to register
    ///
    /// # Example
    ///
    /// ```
    /// use lib3mf::{ParserConfig, CustomExtension};
    ///
    /// let custom_ext = CustomExtension::new(
    ///     "http://example.com/myextension".to_string(),
    ///     "MyExtension".to_string()
    /// );
    ///
    /// let config = ParserConfig::new()
    ///     .with_custom_extension(custom_ext);
    /// ```
    pub fn with_custom_extension(mut self, extension: CustomExtension) -> Self {
        self.custom_extensions.insert(extension.namespace().to_string(), extension);
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

    /// Get a custom extension by namespace
    pub(crate) fn get_custom_extension(&self, namespace: &str) -> Option<&CustomExtension> {
        self.custom_extensions.get(namespace)
    }

    /// Get the set of supported extensions
    pub fn supported_extensions(&self) -> &HashSet<Extension> {
        &self.supported_extensions
    }

    /// Get all registered custom extensions
    pub fn custom_extensions(&self) -> impl Iterator<Item = &CustomExtension> {
        self.custom_extensions.values()
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
    /// Optional material ID (property ID)
    pub pid: Option<usize>,
    /// Optional material index (property index) - used with pid to select from color groups
    pub pindex: Option<usize>,
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
    /// Required extensions for this model
    /// Extensions that the consumer must support to properly process this file
    pub required_extensions: Vec<Extension>,
    /// Metadata key-value pairs
    pub metadata: HashMap<String, String>,
    /// Resources (objects, materials)
    pub resources: Resources,
    /// Build specification
    pub build: Build,
    /// Custom extension elements parsed from the file
    /// Organized by namespace URI
    pub custom_extension_elements: HashMap<String, Vec<CustomElementData>>,
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
            custom_extension_elements: HashMap::new(),
        }
    }

    /// Get custom extension elements for a specific namespace
    ///
    /// Returns the custom extension elements parsed from the file for the given namespace.
    ///
    /// # Arguments
    ///
    /// * `namespace` - The namespace URI to look up
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lib3mf::Model;
    /// use std::fs::File;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = File::open("model.3mf")?;
    /// let model = Model::from_reader(file)?;
    ///
    /// if let Some(elements) = model.get_custom_elements("http://example.com/myextension") {
    ///     println!("Found {} custom elements", elements.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_custom_elements(&self, namespace: &str) -> Option<&Vec<CustomElementData>> {
        self.custom_extension_elements.get(namespace)
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}
