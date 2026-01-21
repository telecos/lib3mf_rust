//! # lib3mf
//!
//! A pure Rust implementation for parsing and writing 3MF (3D Manufacturing Format) files.
//!
//! This library provides functionality to read and write 3MF files, which are ZIP-based
//! containers following the Open Packaging Conventions (OPC) standard and containing
//! XML-based 3D model data.
//!
//! ## Features
//!
//! - Pure Rust implementation with no unsafe code
//! - Parse 3MF file structure (ZIP/OPC container)
//! - Read and write 3D model data including meshes, vertices, and triangles
//! - Support for materials and colors
//! - Metadata extraction and creation
//! - Extension-aware reading and writing
//!
//! ## Example: Reading a 3MF file
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
//! ## Example: Creating and writing a 3MF file
//!
//! ```no_run
//! use lib3mf::{Model, Object, Mesh, Vertex, Triangle, BuildItem};
//! use std::fs::File;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a new model
//! let mut model = Model::new();
//! model.metadata.insert("Title".to_string(), "My Model".to_string());
//!
//! // Create a simple triangle mesh
//! let mut mesh = Mesh::new();
//! mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
//! mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
//! mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));
//! mesh.triangles.push(Triangle::new(0, 1, 2));
//!
//! // Add object with mesh
//! let mut obj = Object::new(1);
//! obj.mesh = Some(mesh);
//! model.resources.objects.push(obj);
//!
//! // Add to build
//! model.build.items.push(BuildItem::new(1));
//!
//! // Write to file
//! let file = File::create("output.3mf")?;
//! model.to_writer(file)?;
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod model;
pub mod opc;
pub mod parser;
pub mod writer;
mod validator;

pub use error::{Error, Result};
pub use model::{
    Build, BuildItem, ColorGroup, Extension, Material, Mesh, Model, Object, ParserConfig,
    Resources, Triangle, Vertex,
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

    /// Write a 3MF file to a writer
    ///
    /// This method serializes the model to a 3MF file format (ZIP archive containing XML).
    ///
    /// # Arguments
    ///
    /// * `writer` - A writer to write the 3MF file data to
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lib3mf::{Model, Object, Mesh, Vertex, Triangle, BuildItem};
    /// use std::fs::File;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create a new model
    /// let mut model = Model::new();
    ///
    /// // Create a simple triangle mesh
    /// let mut mesh = Mesh::new();
    /// mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    /// mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
    /// mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));
    /// mesh.triangles.push(Triangle::new(0, 1, 2));
    ///
    /// // Add object with mesh
    /// let mut obj = Object::new(1);
    /// obj.mesh = Some(mesh);
    /// model.resources.objects.push(obj);
    ///
    /// // Add to build
    /// model.build.items.push(BuildItem::new(1));
    ///
    /// // Write to file
    /// let file = File::create("output.3mf")?;
    /// model.to_writer(file)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_writer<W: std::io::Write + std::io::Seek>(&self, writer: W) -> Result<()> {
        writer::write_3mf(self, writer)
    }
}
