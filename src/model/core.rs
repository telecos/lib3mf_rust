//! Core 3MF types and structures

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::extension::ExtensionRegistry;

use super::beam_lattice::BeamSet;
use super::boolean_ops::BooleanShape;
use super::production::ProductionInfo;

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
            // Also accept the earlier 2019/04 namespace for backward compatibility
            "http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/04" => {
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
    /// Optional key provider for decrypting SecureContent
    /// If provided, this will be used to decrypt encrypted files
    /// If not provided, test keys will be used for Suite 8 conformance
    key_provider: Option<Arc<dyn crate::key_provider::KeyProvider>>,
    /// Extension registry for managing extension handlers
    registry: ExtensionRegistry,
}

impl ParserConfig {
    /// Create a new parser configuration with only core support
    pub fn new() -> Self {
        let mut supported = HashSet::new();
        supported.insert(Extension::Core);
        Self {
            supported_extensions: supported,
            custom_extensions: HashMap::new(),
            key_provider: None,
            registry: ExtensionRegistry::new(),
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
            key_provider: None,
            registry: crate::extensions::create_default_registry(),
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

    /// Set a custom key provider for SecureContent decryption
    ///
    /// # Arguments
    ///
    /// * `provider` - The key provider to use for decryption
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lib3mf::{ParserConfig, KeyProvider};
    /// use std::sync::Arc;
    /// # use lib3mf::{Result, SecureContentInfo, AccessRight, CEKParams, KEKParams};
    ///
    /// # struct MyKeyProvider;
    /// # impl KeyProvider for MyKeyProvider {
    /// #     fn decrypt(&self, _: &[u8], _: &CEKParams, _: &AccessRight, _: &SecureContentInfo) -> Result<Vec<u8>> { Ok(vec![]) }
    /// #     fn encrypt(&self, _: &[u8], _: &str, _: bool) -> Result<(Vec<u8>, CEKParams, KEKParams, String)> { unimplemented!() }
    /// # }
    ///
    /// let provider: Arc<dyn KeyProvider> = Arc::new(MyKeyProvider);
    /// let config = ParserConfig::new()
    ///     .with_key_provider(provider);
    /// ```
    pub fn with_key_provider(
        mut self,
        provider: Arc<dyn crate::key_provider::KeyProvider>,
    ) -> Self {
        self.key_provider = Some(provider);
        self
    }

    /// Get the key provider if one is configured
    pub fn key_provider(&self) -> Option<&Arc<dyn crate::key_provider::KeyProvider>> {
        self.key_provider.as_ref()
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

    /// Register an extension handler
    ///
    /// # Arguments
    ///
    /// * `handler` - The extension handler to register
    ///
    /// # Example
    ///
    /// ```
    /// use lib3mf::{ParserConfig, Extension};
    /// use lib3mf::extensions::MaterialExtensionHandler;
    /// use std::sync::Arc;
    ///
    /// let config = ParserConfig::new()
    ///     .with_extension_handler(Arc::new(MaterialExtensionHandler));
    /// ```
    pub fn with_extension_handler(mut self, handler: Arc<dyn crate::extension::ExtensionHandler>) -> Self {
        self.registry.register(handler);
        self
    }

    /// Get a reference to the extension registry
    ///
    /// # Returns
    ///
    /// A reference to the internal extension registry
    pub fn registry(&self) -> &ExtensionRegistry {
        &self.registry
    }

    /// Get a mutable reference to the extension registry
    ///
    /// # Returns
    ///
    /// A mutable reference to the internal extension registry
    pub fn registry_mut(&mut self) -> &mut ExtensionRegistry {
        &mut self.registry
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

/// A triangle in a displacement mesh
#[derive(Debug, Clone, PartialEq)]
pub struct DisplacementTriangle {
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
    /// Optional disp2d group ID (referenced in triangles element or individual triangle)
    pub did: Option<usize>,
    /// Optional displacement coordinate index for vertex 1
    pub d1: Option<usize>,
    /// Optional displacement coordinate index for vertex 2
    pub d2: Option<usize>,
    /// Optional displacement coordinate index for vertex 3
    pub d3: Option<usize>,
}

impl DisplacementTriangle {
    /// Create a new displacement triangle
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
            did: None,
            d1: None,
            d2: None,
            d3: None,
        }
    }
}

/// A displacement mesh containing vertices and displacement triangles
#[derive(Debug, Clone)]
pub struct DisplacementMesh {
    /// List of vertices
    pub vertices: Vec<Vertex>,
    /// List of displacement triangles
    pub triangles: Vec<DisplacementTriangle>,
}

impl DisplacementMesh {
    /// Create a new empty displacement mesh
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            triangles: Vec::new(),
        }
    }
}

impl Default for DisplacementMesh {
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
    /// Optional path to external model file (Production extension: p:path attribute)
    ///
    /// When set, indicates the component references an object in an external model file
    /// rather than in the current file's resources. Used with Production extension
    /// to reference objects from separate model streams.
    pub path: Option<String>,
    /// Production extension information (UUID, path)
    pub production: Option<ProductionInfo>,
}

impl Component {
    /// Create a new component with the given object reference
    pub fn new(objectid: usize) -> Self {
        Self {
            objectid,
            transform: None,
            path: None,
            production: None,
        }
    }

    /// Create a new component with a transformation matrix
    pub fn with_transform(objectid: usize, transform: [f64; 12]) -> Self {
        Self {
            objectid,
            transform: Some(transform),
            path: None,
            production: None,
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
    /// Optional displacement mesh data (Displacement extension)
    /// An object can have either a regular mesh OR a displacement mesh, not both
    pub displacement_mesh: Option<DisplacementMesh>,
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
    /// Thumbnail attribute (deprecated, should not be used with production extension)
    /// This is stored only for validation purposes - the attribute is accepted but not functional
    pub(crate) has_thumbnail_attribute: bool,
    /// Tracks if object has extension-specific shape elements (beamlattice, displacement, etc.)
    /// Used for validation - per Boolean Operations spec, operands must be simple meshes only
    pub(crate) has_extension_shapes: bool,
    /// Parse order (for validation of resource ordering)
    #[doc(hidden)]
    pub parse_order: usize,
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
            displacement_mesh: None,
            pid: None,
            pindex: None,
            basematerialid: None,
            slicestackid: None,
            production: None,
            boolean_shape: None,
            components: Vec::new(),
            has_thumbnail_attribute: false,
            has_extension_shapes: false,
            parse_order: 0,
        }
    }
}

/// Resources section containing objects and materials
#[derive(Debug, Clone)]
pub struct Resources {
    /// List of objects
    pub objects: Vec<Object>,
    /// List of materials
    pub materials: Vec<super::Material>,
    /// List of color groups (materials extension)
    pub color_groups: Vec<super::ColorGroup>,
    /// List of displacement maps (displacement extension)
    pub displacement_maps: Vec<super::Displacement2D>,
    /// List of normalized vector groups (displacement extension)
    pub norm_vector_groups: Vec<super::NormVectorGroup>,
    /// List of displacement coordinate groups (displacement extension)
    pub disp2d_groups: Vec<super::Disp2DGroup>,
    /// List of slice stacks (slice extension)
    pub slice_stacks: Vec<super::SliceStack>,
    /// List of base material groups (materials extension)
    pub base_material_groups: Vec<super::BaseMaterialGroup>,
    /// List of texture2d resources (materials extension)
    pub texture2d_resources: Vec<super::Texture2D>,
    /// List of texture2d groups (materials extension)
    pub texture2d_groups: Vec<super::Texture2DGroup>,
    /// List of composite materials groups (materials extension)
    pub composite_materials: Vec<super::CompositeMaterials>,
    /// List of multi-properties groups (materials extension)
    pub multi_properties: Vec<super::MultiProperties>,
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
    /// Production extension path (p:path attribute) - references external file
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
    pub secure_content: Option<super::SecureContentInfo>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::MaterialExtensionHandler;

    #[test]
    fn test_parser_config_new_has_empty_registry() {
        let config = ParserConfig::new();
        assert_eq!(config.registry().handlers().len(), 0);
    }

    #[test]
    fn test_parser_config_with_all_extensions_has_default_registry() {
        let config = ParserConfig::with_all_extensions();
        // Should have all 7 standard extension handlers
        assert_eq!(config.registry().handlers().len(), 7);
        
        // Verify specific handlers are present
        assert!(config.registry().get_handler(Extension::Material).is_some());
        assert!(config.registry().get_handler(Extension::Production).is_some());
        assert!(config.registry().get_handler(Extension::BeamLattice).is_some());
        assert!(config.registry().get_handler(Extension::Slice).is_some());
        assert!(config.registry().get_handler(Extension::BooleanOperations).is_some());
        assert!(config.registry().get_handler(Extension::Displacement).is_some());
        assert!(config.registry().get_handler(Extension::SecureContent).is_some());
    }

    #[test]
    fn test_parser_config_with_extension_handler() {
        let config = ParserConfig::new()
            .with_extension_handler(Arc::new(MaterialExtensionHandler));
        
        assert_eq!(config.registry().handlers().len(), 1);
        assert!(config.registry().get_handler(Extension::Material).is_some());
    }

    #[test]
    fn test_parser_config_registry_mut() {
        let mut config = ParserConfig::new();
        assert_eq!(config.registry().handlers().len(), 0);
        
        config.registry_mut().register(Arc::new(MaterialExtensionHandler));
        
        assert_eq!(config.registry().handlers().len(), 1);
        assert!(config.registry().get_handler(Extension::Material).is_some());
    }

    #[test]
    fn test_parser_config_clone() {
        let config1 = ParserConfig::new()
            .with_extension_handler(Arc::new(MaterialExtensionHandler));
        
        let config2 = config1.clone();
        
        // Both should have the same handlers
        assert_eq!(config1.registry().handlers().len(), config2.registry().handlers().len());
        assert_eq!(config1.registry().handlers().len(), 1);
        assert!(config2.registry().get_handler(Extension::Material).is_some());
    }

    #[test]
    fn test_parser_config_chaining() {
        let config = ParserConfig::new()
            .with_extension(Extension::Material)
            .with_extension_handler(Arc::new(MaterialExtensionHandler));
        
        assert!(config.supports(&Extension::Material));
        assert_eq!(config.registry().handlers().len(), 1);
    }
}
