//! XML writing for 3MF model files
//!
//! This module provides functionality to serialize Model structures to XML format
//! following the 3MF specification.

use crate::error::Result;
use crate::model::*;
use std::io::Write;

/// Write a Model to XML string
pub fn write_model_xml(model: &Model) -> Result<String> {
    let mut xml = String::new();
    
    // XML declaration
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push('\n');
    
    // Model element with namespaces
    xml.push_str("<model unit=\"");
    xml.push_str(&escape_xml(&model.unit));
    xml.push_str("\" xml:lang=\"en-US\" xmlns=\"");
    xml.push_str(&escape_xml(&model.xmlns));
    xml.push('"');
    
    // Add namespace declarations for extensions
    let mut has_material_extension = false;
    for ext in &model.required_extensions {
        match ext {
            Extension::Material => {
                xml.push_str(" xmlns:m=\"");
                xml.push_str(Extension::Material.namespace());
                xml.push('"');
                has_material_extension = true;
            }
            Extension::Production => {
                xml.push_str(" xmlns:p=\"");
                xml.push_str(Extension::Production.namespace());
                xml.push('"');
            }
            Extension::Slice => {
                xml.push_str(" xmlns:s=\"");
                xml.push_str(Extension::Slice.namespace());
                xml.push('"');
            }
            Extension::BeamLattice => {
                xml.push_str(" xmlns:b=\"");
                xml.push_str(Extension::BeamLattice.namespace());
                xml.push('"');
            }
            Extension::SecureContent => {
                xml.push_str(" xmlns:sc=\"");
                xml.push_str(Extension::SecureContent.namespace());
                xml.push('"');
            }
            Extension::BooleanOperations => {
                xml.push_str(" xmlns:v=\"");
                xml.push_str(Extension::BooleanOperations.namespace());
                xml.push('"');
            }
            Extension::Displacement => {
                xml.push_str(" xmlns:d=\"");
                xml.push_str(Extension::Displacement.namespace());
                xml.push('"');
            }
            Extension::Core => {
                // Core namespace is already the default xmlns
            }
        }
    }
    
    // Add requiredextensions attribute if needed
    if !model.required_extensions.is_empty() {
        let mut req_exts = Vec::new();
        for ext in &model.required_extensions {
            match ext {
                Extension::Material => req_exts.push("m"),
                Extension::Production => req_exts.push("p"),
                Extension::Slice => req_exts.push("s"),
                Extension::BeamLattice => req_exts.push("b"),
                Extension::SecureContent => req_exts.push("sc"),
                Extension::BooleanOperations => req_exts.push("v"),
                Extension::Displacement => req_exts.push("d"),
                Extension::Core => {
                    // Core is not included in requiredextensions
                }
            }
        }
        if !req_exts.is_empty() {
            xml.push_str(" requiredextensions=\"");
            xml.push_str(&req_exts.join(" "));
            xml.push('"');
        }
    }
    
    xml.push_str(">\n");
    
    // Metadata
    for (name, value) in &model.metadata {
        xml.push_str("  <metadata name=\"");
        xml.push_str(&escape_xml(name));
        xml.push_str("\">");
        xml.push_str(&escape_xml(value));
        xml.push_str("</metadata>\n");
    }
    
    // Resources
    xml.push_str("  <resources>\n");
    
    // Materials (basematerials)
    if !model.resources.materials.is_empty() {
        xml.push_str("    <basematerials id=\"1\">\n");
        for material in &model.resources.materials {
            xml.push_str("      <base name=\"");
            xml.push_str(&escape_xml(material.name.as_deref().unwrap_or("")));
            xml.push_str("\" displaycolor=\"");
            if let Some((r, g, b, a)) = material.color {
                xml.push_str(&format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a));
            } else {
                xml.push_str("#FFFFFFFF");
            }
            xml.push_str("\"/>\n");
        }
        xml.push_str("    </basematerials>\n");
    }
    
    // Color groups (materials extension)
    if has_material_extension {
        for colorgroup in &model.resources.color_groups {
            xml.push_str("    <m:colorgroup id=\"");
            xml.push_str(&colorgroup.id.to_string());
            xml.push_str("\">\n");
            for (r, g, b, a) in &colorgroup.colors {
                xml.push_str("      <m:color color=\"#");
                xml.push_str(&format!("{:02X}{:02X}{:02X}{:02X}", r, g, b, a));
                xml.push_str("\"/>\n");
            }
            xml.push_str("    </m:colorgroup>\n");
        }
    }
    
    // Objects
    for object in &model.resources.objects {
        xml.push_str("    <object id=\"");
        xml.push_str(&object.id.to_string());
        xml.push('"');
        
        if let Some(ref name) = object.name {
            xml.push_str(" name=\"");
            xml.push_str(&escape_xml(name));
            xml.push('"');
        }
        
        xml.push_str(" type=\"");
        xml.push_str(match object.object_type {
            ObjectType::Model => "model",
            ObjectType::Support => "support",
            ObjectType::SolidSupport => "solidsupport",
            ObjectType::Surface => "surface",
            ObjectType::Other => "other",
        });
        xml.push('"');
        
        // Add pid and pindex if present
        if let Some(pid) = object.pid {
            xml.push_str(" pid=\"");
            xml.push_str(&pid.to_string());
            xml.push('"');
        }
        if let Some(pindex) = object.pindex {
            xml.push_str(" pindex=\"");
            xml.push_str(&pindex.to_string());
            xml.push('"');
        }
        
        xml.push_str(">\n");
        
        // Mesh
        if let Some(ref mesh) = object.mesh {
            xml.push_str("      <mesh>\n");
            
            // Vertices
            xml.push_str("        <vertices>\n");
            for vertex in &mesh.vertices {
                xml.push_str("          <vertex x=\"");
                xml.push_str(&vertex.x.to_string());
                xml.push_str("\" y=\"");
                xml.push_str(&vertex.y.to_string());
                xml.push_str("\" z=\"");
                xml.push_str(&vertex.z.to_string());
                xml.push_str("\"/>\n");
            }
            xml.push_str("        </vertices>\n");
            
            // Triangles
            xml.push_str("        <triangles>\n");
            for triangle in &mesh.triangles {
                xml.push_str("          <triangle v1=\"");
                xml.push_str(&triangle.v1.to_string());
                xml.push_str("\" v2=\"");
                xml.push_str(&triangle.v2.to_string());
                xml.push_str("\" v3=\"");
                xml.push_str(&triangle.v3.to_string());
                xml.push('"');
                
                // Add material properties
                if let Some(pid) = triangle.pid {
                    xml.push_str(" pid=\"");
                    xml.push_str(&pid.to_string());
                    xml.push('"');
                }
                if let Some(pindex) = triangle.pindex {
                    xml.push_str(" pindex=\"");
                    xml.push_str(&pindex.to_string());
                    xml.push('"');
                }
                if let Some(p1) = triangle.p1 {
                    xml.push_str(" p1=\"");
                    xml.push_str(&p1.to_string());
                    xml.push('"');
                }
                if let Some(p2) = triangle.p2 {
                    xml.push_str(" p2=\"");
                    xml.push_str(&p2.to_string());
                    xml.push('"');
                }
                if let Some(p3) = triangle.p3 {
                    xml.push_str(" p3=\"");
                    xml.push_str(&p3.to_string());
                    xml.push('"');
                }
                
                xml.push_str("/>\n");
            }
            xml.push_str("        </triangles>\n");
            
            xml.push_str("      </mesh>\n");
        }
        
        xml.push_str("    </object>\n");
    }
    
    xml.push_str("  </resources>\n");
    
    // Build
    xml.push_str("  <build>\n");
    for item in &model.build.items {
        xml.push_str("    <item objectid=\"");
        xml.push_str(&item.objectid.to_string());
        xml.push('"');
        
        if let Some(transform) = item.transform {
            xml.push_str(" transform=\"");
            let transform_str: Vec<String> = transform.iter().map(|v| v.to_string()).collect();
            xml.push_str(&transform_str.join(" "));
            xml.push('"');
        }
        
        xml.push_str("/>\n");
    }
    xml.push_str("  </build>\n");
    
    xml.push_str("</model>\n");
    
    Ok(xml)
}

/// Write a Model to a writer in 3MF format (ZIP archive)
pub fn write_3mf<W: Write + std::io::Seek>(model: &Model, writer: W) -> Result<()> {
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;
    
    let mut zip = ZipWriter::new(writer);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    
    // Write [Content_Types].xml
    let content_types = r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
</Types>"#;
    
    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(content_types.as_bytes())?;
    
    // Write _rels/.rels
    let rels = r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Target="/3D/3dmodel.model" Id="rel0" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"#;
    
    zip.start_file("_rels/.rels", options)?;
    zip.write_all(rels.as_bytes())?;
    
    // Write 3D/3dmodel.model
    let model_xml = write_model_xml(model)?;
    
    zip.start_file("3D/3dmodel.model", options)?;
    zip.write_all(model_xml.as_bytes())?;
    
    zip.finish()?;
    
    Ok(())
}

/// Escape special XML characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    
    #[test]
    fn test_write_minimal_model() {
        let mut model = Model::new();
        model.metadata.insert("Title".to_string(), "Test Model".to_string());
        
        let mut obj = Object::new(1);
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));
        obj.mesh = Some(mesh);
        model.resources.objects.push(obj);
        
        model.build.items.push(BuildItem::new(1));
        
        let xml = write_model_xml(&model).unwrap();
        
        assert!(xml.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<model unit=\"millimeter\""));
        assert!(xml.contains("<metadata name=\"Title\">Test Model</metadata>"));
        assert!(xml.contains("<object id=\"1\""));
        assert!(xml.contains("<vertex x=\"0\" y=\"0\" z=\"0\"/>"));
        assert!(xml.contains("<triangle v1=\"0\" v2=\"1\" v3=\"2\"/>"));
        assert!(xml.contains("<item objectid=\"1\"/>"));
    }
    
    #[test]
    fn test_write_3mf_archive() {
        let mut model = Model::new();
        let mut obj = Object::new(1);
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));
        obj.mesh = Some(mesh);
        model.resources.objects.push(obj);
        model.build.items.push(BuildItem::new(1));
        
        let mut buffer = Vec::new();
        let cursor = Cursor::new(&mut buffer);
        write_3mf(&model, cursor).unwrap();
        
        // Verify it's a valid ZIP file
        assert!(!buffer.is_empty());
        assert_eq!(&buffer[0..2], b"PK");
    }
    
    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("normal text"), "normal text");
        assert_eq!(escape_xml("a&b"), "a&amp;b");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(escape_xml("'quoted'"), "&apos;quoted&apos;");
    }
}
