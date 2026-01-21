//! Streaming parser for large 3MF files
//!
//! This module provides an iterator-based streaming API that allows parsing
//! large 3MF files without loading the entire model into memory. This is useful
//! for processing files with millions of vertices or triangles where memory
//! consumption is a concern.
//!
//! # Example
//!
//! ```no_run
//! use lib3mf::streaming::StreamingParser;
//! use std::fs::File;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let file = File::open("large_model.3mf")?;
//! let mut parser = StreamingParser::new(file)?;
//!
//! // Iterate through objects one at a time
//! for object in parser.objects() {
//!     let object = object?;
//!     println!("Processing object {} with {} vertices",
//!         object.id,
//!         object.mesh.as_ref().map(|m| m.vertices.len()).unwrap_or(0)
//!     );
//! }
//! # Ok(())
//! # }
//! ```

use crate::error::{Error, Result};
use crate::model::*;
use crate::opc::Package;
use crate::parser;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::{BufReader, Read, Seek};

/// Streaming parser for 3MF files
///
/// This parser processes 3MF files incrementally, allowing you to iterate through
/// objects without loading the entire model into memory at once. This is particularly
/// useful for large files with millions of vertices or triangles.
///
/// # Memory Usage
///
/// Unlike the standard parser which loads all data into a `Model` structure,
/// the streaming parser:
/// - Reads one object at a time
/// - Allows processing objects sequentially
/// - Reduces peak memory usage for large files
/// - Suitable for conversion, validation, or analysis workflows
pub struct StreamingParser<R: Read + Seek> {
    package: Package<R>,
    config: ParserConfig,
}

impl<R: Read + Seek> StreamingParser<R> {
    /// Create a new streaming parser
    ///
    /// # Arguments
    ///
    /// * `reader` - A reader containing the 3MF file data
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lib3mf::streaming::StreamingParser;
    /// use std::fs::File;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = File::open("model.3mf")?;
    /// let parser = StreamingParser::new(file)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(reader: R) -> Result<Self> {
        Self::new_with_config(reader, ParserConfig::with_all_extensions())
    }

    /// Create a new streaming parser with custom configuration
    ///
    /// # Arguments
    ///
    /// * `reader` - A reader containing the 3MF file data
    /// * `config` - Parser configuration specifying supported extensions
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lib3mf::streaming::StreamingParser;
    /// use lib3mf::{ParserConfig, Extension};
    /// use std::fs::File;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = File::open("model.3mf")?;
    /// let config = ParserConfig::new().with_extension(Extension::Material);
    /// let parser = StreamingParser::new_with_config(file, config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_with_config(reader: R, config: ParserConfig) -> Result<Self> {
        let package = Package::open(reader)?;
        Ok(Self { package, config })
    }

    /// Get an iterator over objects in the 3MF file
    ///
    /// This returns an iterator that yields each object one at a time,
    /// allowing you to process large files without loading everything into memory.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lib3mf::streaming::StreamingParser;
    /// use std::fs::File;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = File::open("model.3mf")?;
    /// let mut parser = StreamingParser::new(file)?;
    ///
    /// for object in parser.objects() {
    ///     let object = object?;
    ///     println!("Object {}: {} vertices",
    ///         object.id,
    ///         object.mesh.as_ref().map(|m| m.vertices.len()).unwrap_or(0)
    ///     );
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn objects(&mut self) -> ObjectIterator {
        let model_xml = match self.package.get_model() {
            Ok(xml) => xml,
            Err(e) => return ObjectIterator::with_error(e),
        };

        ObjectIterator::new(model_xml, self.config.clone())
    }

    /// Parse the entire model into memory
    ///
    /// This is equivalent to using the standard parser, but allows you to use
    /// the same API for both streaming and non-streaming workflows.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lib3mf::streaming::StreamingParser;
    /// use std::fs::File;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = File::open("model.3mf")?;
    /// let mut parser = StreamingParser::new(file)?;
    /// let model = parser.parse_full()?;
    /// println!("Loaded {} objects", model.resources.objects.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse_full(mut self) -> Result<Model> {
        let model_xml = self.package.get_model()?;
        crate::parser::parse_model_xml_with_config(&model_xml, self.config)
    }
}

/// Iterator over objects in a 3MF file
///
/// This iterator processes the 3MF XML incrementally, yielding one object at a time.
/// Objects are parsed on-demand, minimizing memory usage for large files.
pub struct ObjectIterator {
    reader: Option<Reader<BufReader<std::io::Cursor<String>>>>,
    #[allow(dead_code)] // Reserved for future use to validate extensions
    config: ParserConfig,
    buf: Vec<u8>,
    in_resources: bool,
    error: Option<Error>,
    done: bool,
}

impl ObjectIterator {
    fn new(xml: String, config: ParserConfig) -> Self {
        let cursor = std::io::Cursor::new(xml);
        let buf_reader = BufReader::new(cursor);
        let mut reader = Reader::from_reader(buf_reader);
        reader.config_mut().trim_text(true);

        Self {
            reader: Some(reader),
            config,
            buf: Vec::with_capacity(4096),
            in_resources: false,
            error: None,
            done: false,
        }
    }

    fn with_error(error: Error) -> Self {
        Self {
            reader: None,
            config: ParserConfig::new(),
            buf: Vec::new(),
            in_resources: false,
            error: Some(error),
            done: true,
        }
    }

    fn parse_next_object(&mut self) -> Result<Option<Object>> {
        if self.done {
            return Ok(None);
        }

        if let Some(error) = self.error.take() {
            return Err(error);
        }

        let reader = match &mut self.reader {
            Some(r) => r,
            None => return Ok(None),
        };

        loop {
            match reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let name_str =
                        std::str::from_utf8(name.as_ref()).map_err(|e| Error::InvalidXml(e.to_string()))?;

                    let local_name = parser::get_local_name(name_str);

                    match local_name {
                        "resources" => {
                            self.in_resources = true;
                        }
                        "object" if self.in_resources => {
                            // Parse the complete object including its mesh
                            let mut object = parser::parse_object(reader, e)?;
                            let mut current_mesh: Option<Mesh> = None;
                            let mut depth = 1; // We're inside the object element

                            // Continue parsing until we hit the closing </object> tag
                            loop {
                                match reader.read_event_into(&mut self.buf) {
                                    Ok(Event::Start(ref e)) => {
                                        depth += 1;
                                        let name = e.name();
                                        let name_str = std::str::from_utf8(name.as_ref())
                                            .map_err(|e| Error::InvalidXml(e.to_string()))?;
                                        let local_name = parser::get_local_name(name_str);

                                        if local_name == "mesh" {
                                            current_mesh = Some(Mesh::new());
                                        }
                                    }
                                    Ok(Event::Empty(ref e)) => {
                                        let name = e.name();
                                        let name_str = std::str::from_utf8(name.as_ref())
                                            .map_err(|e| Error::InvalidXml(e.to_string()))?;
                                        let local_name = parser::get_local_name(name_str);

                                        match local_name {
                                            "vertex" => {
                                                if let Some(ref mut mesh) = current_mesh {
                                                    let vertex = parser::parse_vertex(reader, e)?;
                                                    mesh.vertices.push(vertex);
                                                }
                                            }
                                            "triangle" => {
                                                if let Some(ref mut mesh) = current_mesh {
                                                    let triangle = parser::parse_triangle(reader, e)?;
                                                    mesh.triangles.push(triangle);
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    Ok(Event::End(_)) => {
                                        depth -= 1;
                                        if depth == 0 {
                                            // End of object element
                                            object.mesh = current_mesh;
                                            self.buf.clear();
                                            return Ok(Some(object));
                                        }
                                    }
                                    Ok(Event::Eof) => {
                                        self.done = true;
                                        return Err(Error::InvalidXml("Unexpected EOF while parsing object".to_string()));
                                    }
                                    Err(e) => {
                                        self.done = true;
                                        return Err(Error::Xml(e));
                                    }
                                    _ => {}
                                }
                                self.buf.clear();
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    let name_str =
                        std::str::from_utf8(name.as_ref()).map_err(|e| Error::InvalidXml(e.to_string()))?;

                    let local_name = parser::get_local_name(name_str);

                    if local_name == "resources" {
                        self.in_resources = true;
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = e.name();
                    let name_str =
                        std::str::from_utf8(name.as_ref()).map_err(|e| Error::InvalidXml(e.to_string()))?;

                    let local_name = parser::get_local_name(name_str);

                    if local_name == "resources" {
                        self.in_resources = false;
                        self.done = true;
                        return Ok(None);
                    }
                }
                Ok(Event::Eof) => {
                    self.done = true;
                    return Ok(None);
                }
                Err(e) => {
                    self.done = true;
                    return Err(Error::Xml(e));
                }
                _ => {}
            }
            self.buf.clear();
        }
    }
}

impl Iterator for ObjectIterator {
    type Item = Result<Object>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parse_next_object() {
            Ok(Some(obj)) => Some(Ok(obj)),
            Ok(None) => None,
            Err(e) => {
                self.done = true;
                Some(Err(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_streaming_parser_basic() {
        // Create a minimal 3MF file in memory
        let file_data = create_test_3mf();
        let cursor = Cursor::new(file_data);
        
        let mut parser = StreamingParser::new(cursor).unwrap();
        let objects: Vec<_> = parser.objects().collect::<Result<Vec<_>>>().unwrap();
        
        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].id, 1);
        
        // Verify mesh data is loaded
        assert!(objects[0].mesh.is_some());
        let mesh = objects[0].mesh.as_ref().unwrap();
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.triangles.len(), 1);
    }

    fn create_test_3mf() -> Vec<u8> {
        use std::io::Write;
        use zip::write::SimpleFileOptions;
        use zip::ZipWriter;

        let mut buf = Vec::new();
        let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buf));

        let content_types = r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
</Types>"#;

        zip.start_file("[Content_Types].xml", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(content_types.as_bytes()).unwrap();

        let rels = r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rel0" Target="/3D/3dmodel.model" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"#;

        zip.start_file("_rels/.rels", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(rels.as_bytes()).unwrap();

        let model = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0"/>
                    <vertex x="1" y="0" z="0"/>
                    <vertex x="0" y="1" z="0"/>
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2"/>
                </triangles>
            </mesh>
        </object>
    </resources>
    <build>
        <item objectid="1"/>
    </build>
</model>"#;

        zip.start_file("3D/3dmodel.model", SimpleFileOptions::default())
            .unwrap();
        zip.write_all(model.as_bytes()).unwrap();

        let writer = zip.finish().unwrap();
        writer.into_inner().clone()
    }
}
