//! Test for displacement rendering functionality
//!
//! This test verifies that the displacement data structures are correctly
//! exported and can be used programmatically.

use lib3mf::{
    Disp2DCoords, Disp2DGroup, Displacement2D, DisplacementMesh, DisplacementTriangle, Model,
    NormVector, NormVectorGroup, Object, Vertex,
};

#[test]
fn test_displacement_types_exported() {
    // This test verifies that DisplacementMesh and DisplacementTriangle
    // are properly exported from the lib3mf crate

    // Create a displacement mesh
    let mut disp_mesh = DisplacementMesh::new();

    // Add vertices
    disp_mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    disp_mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
    disp_mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));

    // Add a displacement triangle
    let mut triangle = DisplacementTriangle::new(0, 1, 2);
    triangle.did = Some(1);
    triangle.d1 = Some(0);
    triangle.d2 = Some(1);
    triangle.d3 = Some(2);

    disp_mesh.triangles.push(triangle);

    // Create an object with the displacement mesh
    let mut obj = Object::new(1);
    obj.displacement_mesh = Some(disp_mesh);

    // Create a model and add resources
    let mut model = Model::new();

    // Add displacement map
    model.resources.displacement_maps.push(Displacement2D::new(
        1,
        "/3D/Textures/displacement.png".to_string(),
    ));

    // Add normal vector group
    let mut norm_group = NormVectorGroup::new(2);
    norm_group.vectors.push(NormVector::new(0.0, 0.0, 1.0));
    model.resources.norm_vector_groups.push(norm_group);

    // Add displacement coordinate group
    let mut disp_group = Disp2DGroup::new(1, 1, 2, 1.0);
    disp_group.coords.push(Disp2DCoords::new(0.0, 0.0, 0));
    disp_group.coords.push(Disp2DCoords::new(1.0, 0.0, 0));
    disp_group.coords.push(Disp2DCoords::new(0.5, 1.0, 0));
    model.resources.disp2d_groups.push(disp_group);

    // Add object to model
    model.resources.objects.push(obj);

    // Verify the structure
    assert_eq!(model.resources.objects.len(), 1);
    assert!(model.resources.objects[0].displacement_mesh.is_some());

    let disp_mesh = model.resources.objects[0]
        .displacement_mesh
        .as_ref()
        .unwrap();
    assert_eq!(disp_mesh.vertices.len(), 3);
    assert_eq!(disp_mesh.triangles.len(), 1);

    let triangle = &disp_mesh.triangles[0];
    assert_eq!(triangle.did, Some(1));
    assert_eq!(triangle.d1, Some(0));
    assert_eq!(triangle.d2, Some(1));
    assert_eq!(triangle.d3, Some(2));

    assert_eq!(model.resources.displacement_maps.len(), 1);
    assert_eq!(model.resources.norm_vector_groups.len(), 1);
    assert_eq!(model.resources.disp2d_groups.len(), 1);
}

#[test]
fn test_displacement_mesh_default() {
    // Test that DisplacementMesh has a proper default/new implementation
    let mesh = DisplacementMesh::new();
    assert_eq!(mesh.vertices.len(), 0);
    assert_eq!(mesh.triangles.len(), 0);
}

#[test]
fn test_displacement_triangle_new() {
    // Test DisplacementTriangle creation
    let triangle = DisplacementTriangle::new(0, 1, 2);
    assert_eq!(triangle.v1, 0);
    assert_eq!(triangle.v2, 1);
    assert_eq!(triangle.v3, 2);
    assert_eq!(triangle.did, None);
    assert_eq!(triangle.d1, None);
    assert_eq!(triangle.d2, None);
    assert_eq!(triangle.d3, None);
}

#[test]
fn test_displacement_writer_round_trip() {
    use lib3mf::model::{
        Channel, Disp2DCoords, Disp2DGroup, Displacement2D, Extension, FilterMode, NormVector,
        NormVectorGroup, TileStyle,
    };
    use lib3mf::{BuildItem, Model, Object};
    use std::io::Cursor;

    let mut model = Model::new();
    model.required_extensions.push(Extension::Displacement);

    // Add displacement2d resource with all channel/style combinations
    let mut disp = Displacement2D::new(1, "/3D/Textures/disp.png".to_string());
    disp.channel = Channel::G;
    disp.tilestyleu = TileStyle::Wrap;
    disp.tilestylev = TileStyle::Wrap;
    disp.filter = FilterMode::Auto;
    model.resources.displacement_maps.push(disp);

    // Add normvector group
    let mut ng = NormVectorGroup::new(10);
    ng.vectors.push(NormVector::new(0.0, 0.0, 1.0));
    model.resources.norm_vector_groups.push(ng);

    // Add disp2d group
    let mut dg = Disp2DGroup::new(20, 1, 10, 1.0);
    dg.coords.push(Disp2DCoords::new(0.0, 0.0, 0));
    model.resources.disp2d_groups.push(dg);

    // Add a simple object
    let obj = Object::new(1);
    model.resources.objects.push(obj);
    model.build.items.push(BuildItem::new(1));

    // Write and read back
    let buffer = Vec::new();
    let cursor = Cursor::new(buffer);
    let result = model.to_writer(cursor);
    assert!(result.is_ok(), "Failed to write displacement model");

    let cursor = result.unwrap();
    let config = lib3mf::ParserConfig::new().with_extension(Extension::Displacement);
    let parsed = Model::from_reader_with_config(Cursor::new(cursor.into_inner()), config);
    assert!(parsed.is_ok(), "Failed to parse written displacement model");
    let parsed = parsed.unwrap();
    assert_eq!(parsed.resources.displacement_maps.len(), 1);
    assert_eq!(parsed.resources.norm_vector_groups.len(), 1);
    assert_eq!(parsed.resources.disp2d_groups.len(), 1);
}

/// Helper: minimal displacement XML wrapper with the given resources block
fn displacement_xml(resources_inner: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US"
    xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
    xmlns:d="http://schemas.3mf.io/3dmanufacturing/displacement/2023/10"
    requiredextensions="d">
  <resources>
{}
    <object id="99" type="model">
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
  <build><item objectid="99"/></build>
</model>"#,
        resources_inner
    )
}

#[test]
fn test_displacement2d_invalid_channel() {
    let xml =
        displacement_xml(r#"<d:displacement2d id="1" path="/3D/Textures/t.png" channel="X"/>"#);
    let config = lib3mf::ParserConfig::new().with_extension(lib3mf::Extension::Displacement);
    let result = lib3mf::parser::parse_model_xml_with_config(&xml, config);
    assert!(result.is_err(), "Expected error for invalid channel value");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("channel") || msg.contains("Channel"),
        "Error should mention channel, got: {}",
        msg
    );
}

#[test]
fn test_displacement2d_invalid_tilestyleu() {
    let xml = displacement_xml(
        r#"<d:displacement2d id="1" path="/3D/Textures/t.png" tilestyleu="invalid"/>"#,
    );
    let config = lib3mf::ParserConfig::new().with_extension(lib3mf::Extension::Displacement);
    let result = lib3mf::parser::parse_model_xml_with_config(&xml, config);
    assert!(
        result.is_err(),
        "Expected error for invalid tilestyleu value"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("tilestyleu"),
        "Error should mention tilestyleu, got: {}",
        msg
    );
}

#[test]
fn test_displacement2d_invalid_tilestylev() {
    let xml = displacement_xml(
        r#"<d:displacement2d id="1" path="/3D/Textures/t.png" tilestylev="bad"/>"#,
    );
    let config = lib3mf::ParserConfig::new().with_extension(lib3mf::Extension::Displacement);
    let result = lib3mf::parser::parse_model_xml_with_config(&xml, config);
    assert!(
        result.is_err(),
        "Expected error for invalid tilestylev value"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("tilestylev"),
        "Error should mention tilestylev, got: {}",
        msg
    );
}

#[test]
fn test_displacement2d_invalid_filter() {
    let xml =
        displacement_xml(r#"<d:displacement2d id="1" path="/3D/Textures/t.png" filter="bad"/>"#);
    let config = lib3mf::ParserConfig::new().with_extension(lib3mf::Extension::Displacement);
    let result = lib3mf::parser::parse_model_xml_with_config(&xml, config);
    assert!(result.is_err(), "Expected error for invalid filter value");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("filter"),
        "Error should mention filter, got: {}",
        msg
    );
}

#[test]
fn test_displacement2d_all_channel_values_valid() {
    for channel in &["R", "G", "B", "A"] {
        let xml = displacement_xml(&format!(
            r#"<d:displacement2d id="1" path="/3D/Textures/t.png" channel="{}"/>"#,
            channel
        ));
        let config = lib3mf::ParserConfig::new().with_extension(lib3mf::Extension::Displacement);
        let result = lib3mf::parser::parse_model_xml_with_config(&xml, config);
        assert!(
            result.is_ok(),
            "Expected channel '{}' to be valid, got: {:?}",
            channel,
            result.err()
        );
    }
}

#[test]
fn test_displacement2d_all_tilestyle_values_valid() {
    for style in &["wrap", "mirror", "clamp", "none"] {
        let xml = displacement_xml(&format!(
            r#"<d:displacement2d id="1" path="/3D/Textures/t.png" tilestyleu="{}" tilestylev="{}"/>"#,
            style, style
        ));
        let config = lib3mf::ParserConfig::new().with_extension(lib3mf::Extension::Displacement);
        let result = lib3mf::parser::parse_model_xml_with_config(&xml, config);
        assert!(
            result.is_ok(),
            "Expected tilestyle '{}' to be valid, got: {:?}",
            style,
            result.err()
        );
    }
}

#[test]
fn test_displacement2d_all_filter_values_valid() {
    for filter in &["auto", "linear", "nearest"] {
        let xml = displacement_xml(&format!(
            r#"<d:displacement2d id="1" path="/3D/Textures/t.png" filter="{}"/>"#,
            filter
        ));
        let config = lib3mf::ParserConfig::new().with_extension(lib3mf::Extension::Displacement);
        let result = lib3mf::parser::parse_model_xml_with_config(&xml, config);
        assert!(
            result.is_ok(),
            "Expected filter '{}' to be valid, got: {:?}",
            filter,
            result.err()
        );
    }
}

#[test]
fn test_normvector_missing_x_attribute() {
    let xml = displacement_xml(
        r#"<d:normvectorgroup id="2">
          <d:normvector y="0.0" z="1.0"/>
        </d:normvectorgroup>"#,
    );
    let config = lib3mf::ParserConfig::new().with_extension(lib3mf::Extension::Displacement);
    let result = lib3mf::parser::parse_model_xml_with_config(&xml, config);
    assert!(result.is_err(), "Expected error for missing x attribute");
}

#[test]
fn test_normvectorgroup_missing_id() {
    let xml = displacement_xml(
        r#"<d:normvectorgroup>
          <d:normvector x="0.0" y="0.0" z="1.0"/>
        </d:normvectorgroup>"#,
    );
    let config = lib3mf::ParserConfig::new().with_extension(lib3mf::Extension::Displacement);
    let result = lib3mf::parser::parse_model_xml_with_config(&xml, config);
    assert!(
        result.is_err(),
        "Expected error for normvectorgroup missing id"
    );
}

#[test]
fn test_disp2dgroup_invalid_dispid_reference() {
    // dispid=99 does not reference a declared displacement2d
    let xml = displacement_xml(
        r#"<d:displacement2d id="1" path="/3D/Textures/t.png"/>
        <d:normvectorgroup id="2">
          <d:normvector x="0.0" y="0.0" z="1.0"/>
        </d:normvectorgroup>
        <d:disp2dgroup id="3" dispid="99" nid="2" height="1.0">
          <d:disp2dcoord u="0.0" v="0.0" n="0"/>
        </d:disp2dgroup>"#,
    );
    let config = lib3mf::ParserConfig::new().with_extension(lib3mf::Extension::Displacement);
    let result = lib3mf::parser::parse_model_xml_with_config(&xml, config);
    assert!(
        result.is_err(),
        "Expected error for invalid dispid reference"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Displacement2D") || msg.contains("dispid") || msg.contains("99"),
        "Error should mention Displacement2D reference, got: {}",
        msg
    );
}

#[test]
fn test_disp2dgroup_invalid_nid_reference() {
    // nid=99 does not reference a declared normvectorgroup
    let xml = displacement_xml(
        r#"<d:displacement2d id="1" path="/3D/Textures/t.png"/>
        <d:normvectorgroup id="2">
          <d:normvector x="0.0" y="0.0" z="1.0"/>
        </d:normvectorgroup>
        <d:disp2dgroup id="3" dispid="1" nid="99" height="1.0">
          <d:disp2dcoord u="0.0" v="0.0" n="0"/>
        </d:disp2dgroup>"#,
    );
    let config = lib3mf::ParserConfig::new().with_extension(lib3mf::Extension::Displacement);
    let result = lib3mf::parser::parse_model_xml_with_config(&xml, config);
    assert!(result.is_err(), "Expected error for invalid nid reference");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("NormVectorGroup") || msg.contains("nid") || msg.contains("99"),
        "Error should mention NormVectorGroup reference, got: {}",
        msg
    );
}

#[test]
fn test_disp2dgroup_missing_id() {
    let xml = displacement_xml(
        r#"<d:displacement2d id="1" path="/3D/Textures/t.png"/>
        <d:normvectorgroup id="2">
          <d:normvector x="0.0" y="0.0" z="1.0"/>
        </d:normvectorgroup>
        <d:disp2dgroup dispid="1" nid="2" height="1.0">
          <d:disp2dcoord u="0.0" v="0.0" n="0"/>
        </d:disp2dgroup>"#,
    );
    let config = lib3mf::ParserConfig::new().with_extension(lib3mf::Extension::Displacement);
    let result = lib3mf::parser::parse_model_xml_with_config(&xml, config);
    assert!(result.is_err(), "Expected error for disp2dgroup missing id");
}

#[test]
fn test_disp2dgroup_missing_height() {
    let xml = displacement_xml(
        r#"<d:displacement2d id="1" path="/3D/Textures/t.png"/>
        <d:normvectorgroup id="2">
          <d:normvector x="0.0" y="0.0" z="1.0"/>
        </d:normvectorgroup>
        <d:disp2dgroup id="3" dispid="1" nid="2">
          <d:disp2dcoord u="0.0" v="0.0" n="0"/>
        </d:disp2dgroup>"#,
    );
    let config = lib3mf::ParserConfig::new().with_extension(lib3mf::Extension::Displacement);
    let result = lib3mf::parser::parse_model_xml_with_config(&xml, config);
    assert!(
        result.is_err(),
        "Expected error for disp2dgroup missing height"
    );
}

#[test]
fn test_disp2dcoord_missing_u_attribute() {
    let xml = displacement_xml(
        r#"<d:displacement2d id="1" path="/3D/Textures/t.png"/>
        <d:normvectorgroup id="2">
          <d:normvector x="0.0" y="0.0" z="1.0"/>
        </d:normvectorgroup>
        <d:disp2dgroup id="3" dispid="1" nid="2" height="1.0">
          <d:disp2dcoord v="0.0" n="0"/>
        </d:disp2dgroup>"#,
    );
    let config = lib3mf::ParserConfig::new().with_extension(lib3mf::Extension::Displacement);
    let result = lib3mf::parser::parse_model_xml_with_config(&xml, config);
    assert!(
        result.is_err(),
        "Expected error for disp2dcoord missing u attribute"
    );
}

#[test]
fn test_disp2dcoord_missing_n_attribute() {
    let xml = displacement_xml(
        r#"<d:displacement2d id="1" path="/3D/Textures/t.png"/>
        <d:normvectorgroup id="2">
          <d:normvector x="0.0" y="0.0" z="1.0"/>
        </d:normvectorgroup>
        <d:disp2dgroup id="3" dispid="1" nid="2" height="1.0">
          <d:disp2dcoord u="0.5" v="0.5"/>
        </d:disp2dgroup>"#,
    );
    let config = lib3mf::ParserConfig::new().with_extension(lib3mf::Extension::Displacement);
    let result = lib3mf::parser::parse_model_xml_with_config(&xml, config);
    assert!(
        result.is_err(),
        "Expected error for disp2dcoord missing n attribute"
    );
}

#[test]
fn test_displacement_with_optional_offset() {
    // Test that the optional 'offset' attribute in disp2dgroup is parsed correctly
    let xml = displacement_xml(
        r#"<d:displacement2d id="1" path="/3D/Textures/t.png"/>
        <d:normvectorgroup id="2">
          <d:normvector x="0.0" y="0.0" z="1.0"/>
        </d:normvectorgroup>
        <d:disp2dgroup id="3" dispid="1" nid="2" height="1.0" offset="0.5">
          <d:disp2dcoord u="0.0" v="0.0" n="0"/>
        </d:disp2dgroup>"#,
    );
    let config = lib3mf::ParserConfig::new().with_extension(lib3mf::Extension::Displacement);
    let result = lib3mf::parser::parse_model_xml_with_config(&xml, config);
    assert!(
        result.is_ok(),
        "Expected offset attribute to be valid, got: {:?}",
        result.err()
    );
    let model = result.unwrap();
    assert_eq!(model.resources.disp2d_groups.len(), 1);
    assert_eq!(model.resources.disp2d_groups[0].offset, 0.5);
}

#[test]
fn test_displacement_triangle_with_displacement_coords() {
    // Parse a complete displacement mesh with triangle displacement coordinates
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US"
    xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
    xmlns:d="http://schemas.3mf.io/3dmanufacturing/displacement/2023/10"
    requiredextensions="d">
  <resources>
    <d:displacement2d id="1" path="/3D/Textures/t.png"/>
    <d:normvectorgroup id="2">
      <d:normvector x="0.0" y="0.0" z="1.0"/>
    </d:normvectorgroup>
    <d:disp2dgroup id="3" dispid="1" nid="2" height="1.0">
      <d:disp2dcoord u="0.0" v="0.0" n="0"/>
      <d:disp2dcoord u="1.0" v="0.0" n="0"/>
      <d:disp2dcoord u="0.5" v="1.0" n="0"/>
    </d:disp2dgroup>
    <object id="4" type="model">
      <d:displacementmesh>
        <d:vertices>
          <d:vertex x="0" y="0" z="0"/>
          <d:vertex x="1" y="0" z="0"/>
          <d:vertex x="0" y="1" z="0"/>
        </d:vertices>
        <d:triangles did="3">
          <d:triangle v1="0" v2="1" v3="2" d1="0" d2="1" d3="2"/>
        </d:triangles>
      </d:displacementmesh>
    </object>
  </resources>
  <build><item objectid="4"/></build>
</model>"#;

    let config = lib3mf::ParserConfig::new().with_extension(lib3mf::Extension::Displacement);
    let result = lib3mf::parser::parse_model_xml_with_config(xml, config);
    assert!(
        result.is_ok(),
        "Expected displacement mesh to parse successfully, got: {:?}",
        result.err()
    );
    let model = result.unwrap();
    let obj = model.resources.objects.iter().find(|o| o.id == 4).unwrap();
    assert!(
        obj.displacement_mesh.is_some(),
        "Object should have displacement mesh"
    );
    let dm = obj.displacement_mesh.as_ref().unwrap();
    assert_eq!(dm.vertices.len(), 3);
    assert_eq!(dm.triangles.len(), 1);
    assert_eq!(dm.triangles[0].d1, Some(0));
    assert_eq!(dm.triangles[0].d2, Some(1));
    assert_eq!(dm.triangles[0].d3, Some(2));
}
