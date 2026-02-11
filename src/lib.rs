//! # lib3mf
//!
//! A pure Rust implementation for reading and writing 3MF (3D Manufacturing Format) files.
//!
//! This library provides functionality to read, parse, create, and write 3MF files, which are ZIP-based
//! containers following the Open Packaging Conventions (OPC) standard and containing
//! XML-based 3D model data.
//!
//! ## Features
//!
//! - Pure Rust implementation with no unsafe code
//! - Parse 3MF file structure (ZIP/OPC container)
//! - Read 3D model data including meshes, vertices, and triangles
//! - **Write and serialize 3MF files**
//! - Support for materials and colors
//! - Metadata extraction and writing
//! - Round-trip support (read-write-read)
//! - **Mesh-plane slicing with contour extraction**
//!
//! ## Reading Example
//!
//! ```no_run
//! use lib3mf::Model;
//! use std::fs::File;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let file = File::open("model.3mf")?;
//! let model = Model::from_reader(file)?;
//!
//! println!("Model contains {} objects", model.resources.objects.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Writing Example
//!
//! ```no_run
//! use lib3mf::{Model, Object, Mesh, Vertex, Triangle, BuildItem};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a new model
//! let mut model = Model::new();
//!
//! // Create a mesh with a simple triangle
//! let mut mesh = Mesh::new();
//! mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
//! mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
//! mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));
//! mesh.triangles.push(Triangle::new(0, 1, 2));
//!
//! // Create an object with the mesh
//! let mut object = Object::new(1);
//! object.mesh = Some(mesh);
//!
//! // Add to resources and build
//! model.resources.objects.push(object);
//! model.build.items.push(BuildItem::new(1));
//!
//! // Write to file
//! model.write_to_file("output.3mf")?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Mesh Slicing Example
//!
//! Requires the `mesh-ops` feature (enabled by default).
//!
//! ```ignore
//! use lib3mf::{collect_intersection_segments, assemble_contours, Mesh, Vertex, Triangle};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create or load a mesh
//! let mut mesh = Mesh::new();
//! // ... add vertices and triangles ...
//!
//! // Slice the mesh at Z=5.0
//! let segments = collect_intersection_segments(&mesh, 5.0);
//!
//! // Assemble segments into closed contours
//! let contours = assemble_contours(segments, 1e-6);
//!
//! println!("Found {} contours at Z=5.0", contours.len());
//! for (i, contour) in contours.iter().enumerate() {
//!     println!("Contour {} has {} vertices", i, contour.len());
//! }
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod extension;
pub mod extensions;
#[cfg(feature = "mesh-ops")]
pub mod mesh_ops;
pub mod model;
pub mod opc;
pub mod parser;
#[cfg(feature = "polygon-ops")]
pub mod polygon_clipping;
#[cfg(feature = "polygon-ops")]
pub mod polygon_triangulation;
pub mod streaming;
pub mod validator;
mod writer;

// Decryption support for SecureContent (test keys only, for Suite 8 validation)
#[cfg(feature = "crypto")]
pub mod decryption;

// Key provider trait for custom encryption/decryption
pub mod key_provider;

pub use error::{Error, Result};
pub use extension::{ExtensionHandler, ExtensionRegistry};
pub use extensions::{
    BeamLatticeExtensionHandler, BooleanOperationsExtensionHandler, DisplacementExtensionHandler,
    MaterialExtensionHandler, ProductionExtensionHandler, SecureContentExtensionHandler,
    SliceExtensionHandler, create_default_registry, register_all_handlers,
};
pub use key_provider::KeyProvider;
#[cfg(feature = "mesh-ops")]
pub use mesh_ops::{
    Point2D, SubdivisionMethod, SubdivisionOptions, assemble_contours,
    collect_intersection_segments, subdivide, subdivide_simple, triangle_plane_intersection,
};
pub use model::{
    AccessRight, BaseMaterial, BaseMaterialGroup, Beam, BeamCapMode, BeamSet, BlendMethod,
    BooleanOpType, BooleanRef, BooleanShape, Build, BuildItem, CEKParams, Channel, ColorGroup,
    Component, Composite, CompositeMaterials, Consumer, CustomElementHandler, CustomElementResult,
    CustomExtensionContext, CustomExtensionInfo, CustomValidationHandler, Disp2DCoords,
    Disp2DGroup, Displacement2D, DisplacementMesh, DisplacementTriangle, Extension, FilterMode,
    ImplicitVolume, KEKParams, Material, Mesh, MetadataEntry, Model, Multi, MultiProperties,
    NormVector, NormVectorGroup, Object, ObjectType, ParserConfig, ProductionInfo, ResourceData,
    ResourceDataGroup, Resources, SecureContentInfo, Tex2Coord, Texture2D, Texture2DGroup,
    Thumbnail, TileStyle, Triangle, Vertex, VolumetricBoundary, VolumetricData, VolumetricProperty,
    VolumetricPropertyGroup, Voxel, VoxelGrid,
};

use std::io::Read;

impl Model {
    /// Parse a 3MF file from a reader
    ///
    /// This method uses the default parser configuration which supports all known extensions.
    /// For backward compatibility, this will accept files with any required extensions.
    ///
    /// # Arguments
    ///
    /// * `reader` - A reader containing the 3MF file data
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
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_reader<R: Read + std::io::Seek>(reader: R) -> Result<Self> {
        // Use default config which supports all extensions for backward compatibility
        Self::from_reader_with_config(reader, ParserConfig::with_all_extensions())
    }

    /// Parse a 3MF file from a reader with custom configuration
    ///
    /// This method allows you to specify which extensions you support.
    /// If the file requires an extension you don't support, an error will be returned.
    ///
    /// # Arguments
    ///
    /// * `reader` - A reader containing the 3MF file data
    /// * `config` - Parser configuration specifying supported extensions
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lib3mf::{Model, ParserConfig, Extension};
    /// use std::fs::File;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = File::open("model.3mf")?;
    ///
    /// // Only support core and material extensions
    /// let config = ParserConfig::new()
    ///     .with_extension(Extension::Material);
    ///
    /// let model = Model::from_reader_with_config(file, config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_reader_with_config<R: Read + std::io::Seek>(
        reader: R,
        config: ParserConfig,
    ) -> Result<Self> {
        parser::parse_3mf_with_config(reader, config)
    }

    /// Read thumbnail binary data from a 3MF file
    ///
    /// Returns the thumbnail image data as a byte vector if a thumbnail is present.
    /// Returns None if no thumbnail is found.
    ///
    /// This is a convenience method that reads the thumbnail from a separate reader.
    /// The model metadata must have already been parsed to know if a thumbnail exists.
    ///
    /// # Arguments
    ///
    /// * `reader` - A reader containing the 3MF file data
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lib3mf::Model;
    /// use std::fs::File;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = File::open("model.3mf")?;
    /// if let Some(thumbnail_data) = Model::read_thumbnail(file)? {
    ///     println!("Thumbnail size: {} bytes", thumbnail_data.len());
    ///     std::fs::write("thumbnail.png", thumbnail_data)?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_thumbnail<R: Read + std::io::Seek>(reader: R) -> Result<Option<Vec<u8>>> {
        parser::read_thumbnail(reader)
    }

    /// Write a 3MF file to a writer
    ///
    /// This method serializes the Model to a complete 3MF file (ZIP archive)
    /// and writes it to the provided writer.
    ///
    /// # Arguments
    ///
    /// * `writer` - A writer to write the 3MF file data to
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lib3mf::Model;
    /// use std::fs::File;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new();
    /// // ... populate model with data ...
    ///
    /// let file = File::create("output.3mf")?;
    /// model.to_writer(file)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_writer<W: std::io::Write + std::io::Seek>(self, writer: W) -> Result<W> {
        // Serialize model to XML
        let mut xml_buffer = Vec::new();
        writer::write_model_xml(&self, &mut xml_buffer)?;
        let model_xml = String::from_utf8(xml_buffer)
            .map_err(|e| Error::xml_write(format!("Failed to convert XML to UTF-8: {}", e)))?;

        // Create OPC package
        opc::create_package(writer, &model_xml)
    }

    /// Write a 3MF file to a writer with extension registry
    ///
    /// This method serializes the Model to a complete 3MF file (ZIP archive)
    /// and writes it to the provided writer. Before serialization, it calls
    /// `pre_write()` on all registered extension handlers, allowing extensions
    /// to prepare or transform data before writing.
    ///
    /// # Arguments
    ///
    /// * `writer` - A writer to write the 3MF file data to
    /// * `registry` - Extension registry containing handlers that will be called before writing
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lib3mf::{Model, create_default_registry};
    /// use std::fs::File;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new();
    /// // ... populate model with data ...
    ///
    /// // Create registry with all standard extension handlers
    /// let registry = create_default_registry();
    ///
    /// let file = File::create("output.3mf")?;
    /// model.to_writer_with_registry(file, &registry)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_writer_with_registry<W: std::io::Write + std::io::Seek>(
        mut self,
        writer: W,
        registry: &ExtensionRegistry,
    ) -> Result<W> {
        // Call pre_write hooks on all registered extensions
        registry.pre_write_all(&mut self)?;

        // Serialize model to XML
        let mut xml_buffer = Vec::new();
        writer::write_model_xml(&self, &mut xml_buffer)?;
        let model_xml = String::from_utf8(xml_buffer)
            .map_err(|e| Error::xml_write(format!("Failed to convert XML to UTF-8: {}", e)))?;

        // Create OPC package
        opc::create_package(writer, &model_xml)
    }

    /// Write a 3MF file to a file path
    ///
    /// This is a convenience method that creates a file and writes the 3MF data to it.
    /// The file is automatically flushed and closed when the method completes.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the output file
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lib3mf::Model;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new();
    /// // ... populate model with data ...
    ///
    /// model.write_to_file("output.3mf")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_to_file<P: AsRef<std::path::Path>>(self, path: P) -> Result<()> {
        let file = std::fs::File::create(path)?;
        // File is automatically flushed and closed when dropped
        self.to_writer(file)?;
        Ok(())
    }

    /// Write a 3MF file to a file path with extension registry
    ///
    /// This is a convenience method that creates a file and writes the 3MF data to it.
    /// Before writing, it calls `pre_write()` on all registered extension handlers.
    /// The file is automatically flushed and closed when the method completes.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the output file
    /// * `registry` - Extension registry containing handlers that will be called before writing
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lib3mf::{Model, create_default_registry};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut model = Model::new();
    /// // ... populate model with data ...
    ///
    /// // Create registry with all standard extension handlers
    /// let registry = create_default_registry();
    ///
    /// model.write_to_file_with_registry("output.3mf", &registry)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_to_file_with_registry<P: AsRef<std::path::Path>>(
        self,
        path: P,
        registry: &ExtensionRegistry,
    ) -> Result<()> {
        let file = std::fs::File::create(path)?;
        // File is automatically flushed and closed when dropped
        self.to_writer_with_registry(file, registry)?;
        Ok(())
    }
}
