//! Integration tests for lib3mf
//!
//! These tests create actual 3MF files and test the parsing functionality

use lib3mf::{Model, Object, Vertex};
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Create a minimal valid 3MF file for testing
fn create_test_3mf() -> Vec<u8> {
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

    zip.start_file("[Content_Types].xml", options).unwrap();
    zip.write_all(content_types.as_bytes()).unwrap();

    // Add _rels/.rels
    let rels = r##"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Target="/3D/3dmodel.model" Id="rel0" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"##;

    zip.start_file("_rels/.rels", options).unwrap();
    zip.write_all(rels.as_bytes()).unwrap();

    // Add 3D/3dmodel.model
    let model = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <metadata name="Title">Test Model</metadata>
  <metadata name="Designer">lib3mf_rust</metadata>
  <resources>
    <object id="1" type="model">
      <mesh>
        <vertices>
          <vertex x="0.0" y="0.0" z="0.0"/>
          <vertex x="10.0" y="0.0" z="0.0"/>
          <vertex x="5.0" y="10.0" z="0.0"/>
          <vertex x="5.0" y="5.0" z="10.0"/>
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

    zip.start_file("3D/3dmodel.model", options).unwrap();
    zip.write_all(model.as_bytes()).unwrap();

    zip.finish().unwrap();
    buffer
}

#[test]
fn test_parse_minimal_3mf() {
    let data = create_test_3mf();
    let cursor = Cursor::new(data);

    let model = Model::from_reader(cursor).unwrap();

    assert_eq!(model.unit, "millimeter");
    assert_eq!(model.resources.objects.len(), 1);

    let obj = &model.resources.objects[0];
    assert_eq!(obj.id, 1);

    let mesh = obj.mesh.as_ref().unwrap();
    assert_eq!(mesh.vertices.len(), 4);
    assert_eq!(mesh.triangles.len(), 4);

    // Check first vertex
    let v0 = &mesh.vertices[0];
    assert_eq!(v0.x, 0.0);
    assert_eq!(v0.y, 0.0);
    assert_eq!(v0.z, 0.0);

    // Check first triangle
    let t0 = &mesh.triangles[0];
    assert_eq!(t0.v1, 0);
    assert_eq!(t0.v2, 1);
    assert_eq!(t0.v3, 2);

    // Check build items
    assert_eq!(model.build.items.len(), 1);
    assert_eq!(model.build.items[0].objectid, 1);

    // Check metadata
    assert_eq!(model.metadata.get("Title"), Some(&"Test Model".to_string()));
    assert_eq!(
        model.metadata.get("Designer"),
        Some(&"lib3mf_rust".to_string())
    );
}

#[test]
fn test_parse_3mf_with_materials() {
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    let mut zip = ZipWriter::new(cursor);

    let options = SimpleFileOptions::default();

    // Add [Content_Types].xml with required rels extension
    let content_types = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\n\
  <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\n\
  <Default Extension=\"model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\n\
</Types>";

    zip.start_file("[Content_Types].xml", options).unwrap();
    zip.write_all(content_types.as_bytes()).unwrap();

    // Add _rels/.rels with model relationship
    let rels = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n\
  <Relationship Id=\"rel0\" Target=\"/3D/3dmodel.model\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\n\
</Relationships>";

    zip.start_file("_rels/.rels", options).unwrap();
    zip.write_all(rels.as_bytes()).unwrap();

    // Add model with materials
    let model = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<model unit=\"millimeter\" xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\" xmlns:m=\"http://schemas.microsoft.com/3dmanufacturing/material/2015/02\">\n\
  <resources>\n\
    <basematerials id=\"1\">\n\
      <base name=\"Red\" displaycolor=\"#FF0000\"/>\n\
      <base name=\"Green\" displaycolor=\"#00FF00\"/>\n\
      <base name=\"Blue\" displaycolor=\"#0000FF\"/>\n\
    </basematerials>\n\
    <object id=\"2\" type=\"model\">\n\
      <mesh>\n\
        <vertices>\n\
          <vertex x=\"0.0\" y=\"0.0\" z=\"0.0\"/>\n\
          <vertex x=\"1.0\" y=\"0.0\" z=\"0.0\"/>\n\
          <vertex x=\"0.5\" y=\"1.0\" z=\"0.0\"/>\n\
        </vertices>\n\
        <triangles>\n\
          <triangle v1=\"0\" v2=\"1\" v3=\"2\" pid=\"0\"/>\n\
        </triangles>\n\
      </mesh>\n\
    </object>\n\
  </resources>\n\
  <build>\n\
    <item objectid=\"2\"/>\n\
  </build>\n\
</model>";

    zip.start_file("3D/3dmodel.model", options).unwrap();
    zip.write_all(model.as_bytes()).unwrap();

    zip.finish().unwrap();

    let cursor = Cursor::new(buffer);
    let model = Model::from_reader(cursor).unwrap();

    assert_eq!(model.resources.materials.len(), 3);

    // Check red material
    let red = &model.resources.materials[0];
    assert_eq!(red.name, Some("Red".to_string()));
    assert_eq!(red.color, Some((255, 0, 0, 255)));

    // Check green material
    let green = &model.resources.materials[1];
    assert_eq!(green.name, Some("Green".to_string()));
    assert_eq!(green.color, Some((0, 255, 0, 255)));

    // Check blue material
    let blue = &model.resources.materials[2];
    assert_eq!(blue.name, Some("Blue".to_string()));
    assert_eq!(blue.color, Some((0, 0, 255, 255)));

    // Check base material groups
    assert_eq!(model.resources.base_material_groups.len(), 1);
    let base_mat_group = &model.resources.base_material_groups[0];
    assert_eq!(base_mat_group.id, 1);
    assert_eq!(base_mat_group.materials.len(), 3);

    // Check red base material
    let red_base = &base_mat_group.materials[0];
    assert_eq!(red_base.name, "Red");
    assert_eq!(red_base.displaycolor, (255, 0, 0, 255));

    // Check green base material
    let green_base = &base_mat_group.materials[1];
    assert_eq!(green_base.name, "Green");
    assert_eq!(green_base.displaycolor, (0, 255, 0, 255));

    // Check blue base material
    let blue_base = &base_mat_group.materials[2];
    assert_eq!(blue_base.name, "Blue");
    assert_eq!(blue_base.displaycolor, (0, 0, 255, 255));
}

#[test]
fn test_parse_3mf_with_basematerial_pid_reference() {
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    let mut zip = ZipWriter::new(cursor);

    let options = SimpleFileOptions::default();

    // Add [Content_Types].xml
    let content_types = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\n\
  <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\n\
  <Default Extension=\"model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\"/>\n\
</Types>";

    zip.start_file("[Content_Types].xml", options).unwrap();
    zip.write_all(content_types.as_bytes()).unwrap();

    // Add _rels/.rels
    let rels = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n\
  <Relationship Id=\"rel0\" Target=\"/3D/3dmodel.model\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\n\
</Relationships>";

    zip.start_file("_rels/.rels", options).unwrap();
    zip.write_all(rels.as_bytes()).unwrap();

    // Add model with base materials and pid reference
    let model = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<model unit=\"millimeter\" xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\">\n\
  <resources>\n\
    <basematerials id=\"5\">\n\
      <base name=\"Red\" displaycolor=\"#FF0000\"/>\n\
      <base name=\"Green\" displaycolor=\"#00FF00\"/>\n\
    </basematerials>\n\
    <object id=\"10\" type=\"model\" pid=\"5\" pindex=\"0\">\n\
      <mesh>\n\
        <vertices>\n\
          <vertex x=\"0.0\" y=\"0.0\" z=\"0.0\"/>\n\
          <vertex x=\"1.0\" y=\"0.0\" z=\"0.0\"/>\n\
          <vertex x=\"0.5\" y=\"1.0\" z=\"0.0\"/>\n\
        </vertices>\n\
        <triangles>\n\
          <triangle v1=\"0\" v2=\"1\" v3=\"2\" pindex=\"1\"/>\n\
        </triangles>\n\
      </mesh>\n\
    </object>\n\
  </resources>\n\
  <build>\n\
    <item objectid=\"10\"/>\n\
  </build>\n\
</model>";

    zip.start_file("3D/3dmodel.model", options).unwrap();
    zip.write_all(model.as_bytes()).unwrap();

    zip.finish().unwrap();

    let cursor = Cursor::new(buffer);
    let result = Model::from_reader(cursor);

    // Should succeed because pid=5 references a valid base material group
    assert!(result.is_ok());
    let model = result.unwrap();

    // Verify the base material group was parsed
    assert_eq!(model.resources.base_material_groups.len(), 1);
    assert_eq!(model.resources.base_material_groups[0].id, 5);

    // Verify the object references the correct pid
    assert_eq!(model.resources.objects.len(), 1);
    assert_eq!(model.resources.objects[0].pid, Some(5));
    assert_eq!(model.resources.objects[0].pindex, Some(0));
}

#[test]
fn test_vertex_creation() {
    let v = Vertex::new(1.0, 2.0, 3.0);
    assert_eq!(v.x, 1.0);
    assert_eq!(v.y, 2.0);
    assert_eq!(v.z, 3.0);
}

#[test]
fn test_object_creation() {
    let obj = Object::new(42);
    assert_eq!(obj.id, 42);
    assert!(obj.name.is_none());
    assert!(obj.mesh.is_none());
}

#[test]
fn test_empty_model() {
    let model = Model::new();
    assert_eq!(model.unit, "millimeter");
    assert_eq!(model.resources.objects.len(), 0);
    assert_eq!(model.build.items.len(), 0);
}
