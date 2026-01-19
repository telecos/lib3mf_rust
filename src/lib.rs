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

pub use error::{Error, Result};
pub use model::{Build, BuildItem, ColorGroup, Material, Mesh, Model, Object, Resources, Triangle, Vertex};

use std::io::Read;

impl Model {
    /// Parse a 3MF file from a reader
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
        parser::parse_3mf(reader)
    }
}
