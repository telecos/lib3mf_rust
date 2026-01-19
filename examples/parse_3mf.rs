//! Example of creating and parsing a 3MF file
//!
//! This example demonstrates how to:
//! 1. Create a simple 3MF file in memory
//! 2. Parse it using lib3mf
//! 3. Inspect the parsed model data

use lib3mf::Model;
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating a simple 3MF file...\n");

    // Create a 3MF file containing a tetrahedron
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    let mut zip = ZipWriter::new(cursor);

    let options = SimpleFileOptions::default();

    // Add [Content_Types].xml
    let content_types = r##"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
</Types>"##;

    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(content_types.as_bytes())?;

    // Add _rels/.rels
    let rels = r##"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Target="/3D/3dmodel.model" Id="rel0" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"##;

    zip.start_file("_rels/.rels", options)?;
    zip.write_all(rels.as_bytes())?;

    // Add 3D/3dmodel.model with a tetrahedron
    let model = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <metadata name="Title">Tetrahedron Example</metadata>
  <metadata name="Designer">lib3mf_rust example</metadata>
  <metadata name="Description">A simple tetrahedron shape</metadata>
  <resources>
    <object id="1" type="model">
      <mesh>
        <vertices>
          <vertex x="0.0" y="0.0" z="0.0"/>
          <vertex x="10.0" y="0.0" z="0.0"/>
          <vertex x="5.0" y="8.66" z="0.0"/>
          <vertex x="5.0" y="2.89" z="8.16"/>
        </vertices>
        <triangles>
          <triangle v1="0" v2="1" v3="2"/>
          <triangle v1="0" v2="1" v3="3"/>
          <triangle v1="1" v2="2" v3="3"/>
          <triangle v1="2" v2="0" v3="3"/>
        </triangles>
      </mesh>
    </object>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"##;

    zip.start_file("3D/3dmodel.model", options)?;
    zip.write_all(model.as_bytes())?;

    zip.finish()?;

    println!("3MF file created successfully!\n");

    // Now parse the 3MF file
    println!("Parsing the 3MF file...\n");
    let cursor = Cursor::new(buffer);
    let parsed_model = Model::from_reader(cursor)?;

    // Display model information
    println!("Model Information:");
    println!("  Unit: {}", parsed_model.unit);
    println!("  Namespace: {}", parsed_model.xmlns);
    println!();

    println!("Metadata:");
    for (key, value) in &parsed_model.metadata {
        println!("  {}: {}", key, value);
    }
    println!();

    println!("Resources:");
    println!("  Objects: {}", parsed_model.resources.objects.len());
    println!("  Materials: {}", parsed_model.resources.materials.len());
    println!();

    // Display object details
    for obj in &parsed_model.resources.objects {
        println!("Object {}:", obj.id);
        if let Some(ref name) = obj.name {
            println!("  Name: {}", name);
        }
        println!("  Type: {:?}", obj.object_type);

        if let Some(ref mesh) = obj.mesh {
            println!("  Mesh:");
            println!("    Vertices: {}", mesh.vertices.len());
            println!("    Triangles: {}", mesh.triangles.len());

            println!("\n  Vertex coordinates:");
            for (i, vertex) in mesh.vertices.iter().enumerate() {
                println!(
                    "    Vertex {}: ({:.2}, {:.2}, {:.2})",
                    i, vertex.x, vertex.y, vertex.z
                );
            }

            println!("\n  Triangle indices:");
            for (i, triangle) in mesh.triangles.iter().enumerate() {
                println!(
                    "    Triangle {}: ({}, {}, {})",
                    i, triangle.v1, triangle.v2, triangle.v3
                );
            }
        }
        println!();
    }

    println!("Build:");
    println!("  Items: {}", parsed_model.build.items.len());
    for (i, item) in parsed_model.build.items.iter().enumerate() {
        println!("    Item {}: object ID {}", i, item.objectid);
        if let Some(transform) = item.transform {
            println!("      Has transformation matrix");
            println!("        [{:.2}, {:.2}, {:.2}, {:.2}]", transform[0], transform[1], transform[2], transform[3]);
            println!("        [{:.2}, {:.2}, {:.2}, {:.2}]", transform[4], transform[5], transform[6], transform[7]);
            println!("        [{:.2}, {:.2}, {:.2}, {:.2}]", transform[8], transform[9], transform[10], transform[11]);
        }
    }
    println!();

    println!("✓ Successfully parsed the 3MF file!");

    Ok(())
}
