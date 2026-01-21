//! Integration tests for advanced material properties

use lib3mf::{Model, ParserConfig};

#[test]
fn test_texture2d_parsing() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" 
       xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:m="http://schemas.microsoft.com/3dmanufacturing/material/2015/02"
       requiredextensions="m">
  <resources>
    <m:texture2d id="1" path="/3D/Textures/color.png" contenttype="image/png" tilestyleu="wrap" tilestylev="clamp" filter="linear"/>
    <m:texture2d id="2" path="/3D/Textures/normal.jpg" contenttype="image/jpeg"/>
    <object id="10">
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
    <item objectid="10"/>
  </build>
</model>"#;

    let config = ParserConfig::with_all_extensions();
    let model = Model::from_xml_with_config(xml, config).unwrap();
    
    // Verify texture2d resources were parsed
    assert_eq!(model.resources.texture2d.len(), 2);
    
    let tex1 = &model.resources.texture2d[0];
    assert_eq!(tex1.id, 1);
    assert_eq!(tex1.path, "/3D/Textures/color.png");
    assert_eq!(tex1.contenttype, "image/png");
    assert_eq!(tex1.tilestyleu, Some(lib3mf::TileStyle::Wrap));
    assert_eq!(tex1.tilestylev, Some(lib3mf::TileStyle::Clamp));
    assert_eq!(tex1.filter, Some(lib3mf::TextureFilter::Linear));
    
    let tex2 = &model.resources.texture2d[1];
    assert_eq!(tex2.id, 2);
    assert_eq!(tex2.path, "/3D/Textures/normal.jpg");
    assert_eq!(tex2.contenttype, "image/jpeg");
    assert_eq!(tex2.tilestyleu, None);
    assert_eq!(tex2.tilestylev, None);
    assert_eq!(tex2.filter, None);
}

#[test]
fn test_texture2dgroup_parsing() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" 
       xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:m="http://schemas.microsoft.com/3dmanufacturing/material/2015/02"
       requiredextensions="m">
  <resources>
    <m:texture2d id="1" path="/3D/Textures/color.png" contenttype="image/png"/>
    <m:texture2dgroup id="2" texid="1">
      <m:tex2coord u="0.0" v="0.0"/>
      <m:tex2coord u="1.0" v="0.0"/>
      <m:tex2coord u="0.5" v="1.0"/>
    </m:texture2dgroup>
    <object id="10">
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
    <item objectid="10"/>
  </build>
</model>"#;

    let config = ParserConfig::with_all_extensions();
    let model = Model::from_xml_with_config(xml, config).unwrap();
    
    // Verify texture2dgroup was parsed
    assert_eq!(model.resources.texture2dgroups.len(), 1);
    
    let texgroup = &model.resources.texture2dgroups[0];
    assert_eq!(texgroup.id, 2);
    assert_eq!(texgroup.texid, 1);
    assert_eq!(texgroup.coords.len(), 3);
    
    assert_eq!(texgroup.coords[0].u, 0.0);
    assert_eq!(texgroup.coords[0].v, 0.0);
    
    assert_eq!(texgroup.coords[1].u, 1.0);
    assert_eq!(texgroup.coords[1].v, 0.0);
    
    assert_eq!(texgroup.coords[2].u, 0.5);
    assert_eq!(texgroup.coords[2].v, 1.0);
}

#[test]
fn test_composite_materials_parsing() {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" 
       xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:m="http://schemas.microsoft.com/3dmanufacturing/material/2015/02"
       requiredextensions="m">
  <resources>
    <basematerials id="1">
      <base name="Red" displaycolor="#FF0000"/>
      <base name="Blue" displaycolor="#0000FF"/>
    </basematerials>
    <m:compositematerials id="2" basematerialid="1">
      <m:composite>
        <m:component propertyid="0" proportion="0.7"/>
        <m:component propertyid="1" proportion="0.3"/>
      </m:composite>
      <m:composite>
        <m:component propertyid="0" proportion="0.5"/>
        <m:component propertyid="1" proportion="0.5"/>
      </m:composite>
    </m:compositematerials>
    <object id="10">
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
    <item objectid="10"/>
  </build>
</model>"##;

    let config = ParserConfig::with_all_extensions();
    let model = Model::from_xml_with_config(xml, config).unwrap();
    
    // Verify composite materials were parsed
    assert_eq!(model.resources.composite_materials.len(), 1);
    
    let comp_group = &model.resources.composite_materials[0];
    assert_eq!(comp_group.id, 2);
    assert_eq!(comp_group.basematerialid, 1);
    assert_eq!(comp_group.composites.len(), 2);
    
    // First composite: 70% red, 30% blue
    let comp1 = &comp_group.composites[0];
    assert_eq!(comp1.components.len(), 2);
    assert_eq!(comp1.components[0].propertyid, 0);
    assert_eq!(comp1.components[0].proportion, 0.7);
    assert_eq!(comp1.components[1].propertyid, 1);
    assert_eq!(comp1.components[1].proportion, 0.3);
    
    // Second composite: 50% red, 50% blue
    let comp2 = &comp_group.composites[1];
    assert_eq!(comp2.components.len(), 2);
    assert_eq!(comp2.components[0].propertyid, 0);
    assert_eq!(comp2.components[0].proportion, 0.5);
    assert_eq!(comp2.components[1].propertyid, 1);
    assert_eq!(comp2.components[1].proportion, 0.5);
}

#[test]
fn test_multiproperties_parsing() {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" 
       xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:m="http://schemas.microsoft.com/3dmanufacturing/material/2015/02"
       requiredextensions="m">
  <resources>
    <basematerials id="1">
      <base name="Red" displaycolor="#FF0000"/>
    </basematerials>
    <m:colorgroup id="2">
      <m:color color="#00FF00"/>
      <m:color color="#0000FF"/>
    </m:colorgroup>
    <m:multiproperties id="3" pids="1 2">
      <m:multi pindices="0 0"/>
      <m:multi pindices="0 1"/>
    </m:multiproperties>
    <object id="10">
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
    <item objectid="10"/>
  </build>
</model>"##;

    let config = ParserConfig::with_all_extensions();
    let model = Model::from_xml_with_config(xml, config).unwrap();
    
    // Verify multiproperties were parsed
    assert_eq!(model.resources.multi_properties.len(), 1);
    
    let multi = &model.resources.multi_properties[0];
    assert_eq!(multi.id, 3);
    assert_eq!(multi.pids, vec![1, 2]);
    assert_eq!(multi.entries.len(), 2);
    
    // First entry: material 0, color 0
    assert_eq!(multi.entries[0].pids, vec![0, 0]);
    
    // Second entry: material 0, color 1
    assert_eq!(multi.entries[1].pids, vec![0, 1]);
}

#[test]
fn test_all_advanced_materials_together() {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" 
       xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:m="http://schemas.microsoft.com/3dmanufacturing/material/2015/02"
       requiredextensions="m">
  <resources>
    <basematerials id="1">
      <base name="Red" displaycolor="#FF0000"/>
      <base name="Blue" displaycolor="#0000FF"/>
    </basematerials>
    <m:colorgroup id="2">
      <m:color color="#00FF00"/>
    </m:colorgroup>
    <m:texture2d id="3" path="/3D/Textures/color.png" contenttype="image/png"/>
    <m:texture2dgroup id="4" texid="3">
      <m:tex2coord u="0.0" v="0.0"/>
    </m:texture2dgroup>
    <m:compositematerials id="5" basematerialid="1">
      <m:composite>
        <m:component propertyid="0" proportion="0.5"/>
        <m:component propertyid="1" proportion="0.5"/>
      </m:composite>
    </m:compositematerials>
    <m:multiproperties id="6" pids="1 2">
      <m:multi pindices="0 0"/>
    </m:multiproperties>
    <object id="10">
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
    <item objectid="10"/>
  </build>
</model>"##;

    let config = ParserConfig::with_all_extensions();
    let model = Model::from_xml_with_config(xml, config).unwrap();
    
    // Verify all advanced material types are parsed
    assert_eq!(model.resources.materials.len(), 2); // Base materials
    assert_eq!(model.resources.color_groups.len(), 1);
    assert_eq!(model.resources.texture2d.len(), 1);
    assert_eq!(model.resources.texture2dgroups.len(), 1);
    assert_eq!(model.resources.composite_materials.len(), 1);
    assert_eq!(model.resources.multi_properties.len(), 1);
}
