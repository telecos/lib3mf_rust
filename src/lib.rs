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
mod validator;

pub use error::{Error, Result};
pub use model::{
    BaseMaterial, BaseMaterialGroup, Beam, BeamCapMode, BeamSet, BlendMethod, BooleanOpType,
    BooleanRef, BooleanShape, Build, BuildItem, Channel, ColorGroup, Composite,
    CompositeMaterials, Disp2DCoords, Disp2DGroup, Displacement2D, Extension, FilterMode,
    Material, Mesh, Model, Multi, MultiProperties, NormVector, NormVectorGroup, Object,
    ParserConfig, Resources, SecureContentInfo, Tex2Coord, Texture2D, Texture2DGroup, TileStyle,
    Triangle, Vertex,
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
}
