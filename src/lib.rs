//! # lib3mf
//!
//! A pure Rust implementation for parsing 3MF (3D Manufacturing Format) files.
//!
//! This library provides functionality to read and parse 3MF files, which are ZIP-based
//! containers following the Open Packaging Conventions (OPC) standard and containing
//! XML-based 3D model data.
//!
//! ## Features
//!
//! - Pure Rust implementation with no unsafe code
//! - Parse 3MF file structure (ZIP/OPC container)
//! - Read 3D model data including meshes, vertices, and triangles
//! - Support for materials and colors
//! - Metadata extraction
//!
//! ## Example
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

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod model;
pub mod opc;
pub mod parser;
pub mod streaming;
mod validator;
mod writer;

pub use error::{Error, Result};
pub use model::{
    BaseMaterial, BaseMaterialGroup, Beam, BeamCapMode, BeamSet, BlendMethod, BooleanOpType,
    BooleanRef, BooleanShape, Build, BuildItem, Channel, ColorGroup, Component, Composite,
    CompositeMaterials, Disp2DCoords, Disp2DGroup, Displacement2D, Extension, FilterMode, Material,
    Mesh, MetadataEntry, Model, Multi, MultiProperties, NormVector, NormVectorGroup, Object,
    ObjectType, ParserConfig, ProductionInfo, Resources, SecureContentInfo, Tex2Coord, Texture2D,
    Texture2DGroup, Thumbnail, TileStyle, Triangle, Vertex,
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

    /// Write a 3MF file to a file path
    ///
    /// This is a convenience method that creates a file and writes the 3MF data to it.
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
        self.to_writer(file)?;
        Ok(())
    }
}
