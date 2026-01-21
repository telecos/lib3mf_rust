//! Example demonstrating advanced material properties support
//!
//! This example shows how to parse 3MF files with advanced material properties
//! including textures, composite materials, and multi-properties.

use lib3mf::{Model, ParserConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Advanced Materials Example\n");
    println!("==========================\n");

    // Parse a 3MF file with advanced materials
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" 
       xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:m="http://schemas.microsoft.com/3dmanufacturing/material/2015/02"
       requiredextensions="m">
  <resources>
    <!-- Base materials -->
    <basematerials id="1">
      <base name="Red Plastic" displaycolor="#FF0000FF"/>
      <base name="Blue Plastic" displaycolor="#0000FFFF"/>
    </basematerials>
    
    <!-- Color group -->
    <m:colorgroup id="2">
      <m:color color="#00FF00FF"/>
      <m:color color="#FFFF00FF"/>
    </m:colorgroup>
    
    <!-- Texture2D resources -->
    <m:texture2d id="3" path="/3D/Textures/wood.png" contenttype="image/png" 
                 tilestyleu="wrap" tilestylev="wrap" filter="linear"/>
    
    <!-- Texture coordinates -->
    <m:texture2dgroup id="4" texid="3">
      <m:tex2coord u="0.0" v="0.0"/>
      <m:tex2coord u="1.0" v="0.0"/>
      <m:tex2coord u="0.5" v="1.0"/>
    </m:texture2dgroup>
    
    <!-- Composite materials (material mixtures) -->
    <m:compositematerials id="5" basematerialid="1">
      <m:composite>
        <m:component propertyid="0" proportion="0.7"/>
        <m:component propertyid="1" proportion="0.3"/>
      </m:composite>
      <m:composite>
        <m:component propertyid="0" proportion="0.5"/>
        <m:component propertyid="1" proportion="0.5"/>
      </m:composite>
    </m:compositematerials>
    
    <!-- Multi-properties (combining multiple property types) -->
    <m:multiproperties id="6" pids="1 2">
      <m:multi pindices="0 0"/>
      <m:multi pindices="1 1"/>
    </m:multiproperties>
    
    <object id="10">
      <mesh>
        <vertices>
          <vertex x="0" y="0" z="0"/>
          <vertex x="10" y="0" z="0"/>
          <vertex x="5" y="10" z="0"/>
          <vertex x="5" y="5" z="10"/>
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
    <item objectid="10"/>
  </build>
</model>"##;

    let config = ParserConfig::with_all_extensions();
    let model = Model::from_xml_with_config(xml, config)?;

    // Display material information
    println!("Base Materials: {}", model.resources.materials.len());
    for mat in &model.resources.materials {
        println!(
            "  Material {}: name={:?}, color={:?}",
            mat.id, mat.name, mat.color
        );
    }
    println!();

    println!("Color Groups: {}", model.resources.color_groups.len());
    for cgroup in &model.resources.color_groups {
        println!("  ColorGroup {}: {} colors", cgroup.id, cgroup.colors.len());
        for (i, color) in cgroup.colors.iter().enumerate() {
            println!(
                "    Color {}: #{:02X}{:02X}{:02X}{:02X}",
                i, color.0, color.1, color.2, color.3
            );
        }
    }
    println!();

    println!("Texture2D Resources: {}", model.resources.texture2d.len());
    for tex in &model.resources.texture2d {
        println!("  Texture {}: path={}", tex.id, tex.path);
        println!("    contenttype: {}", tex.contenttype);
        println!("    tilestyleu: {:?}", tex.tilestyleu);
        println!("    tilestylev: {:?}", tex.tilestylev);
        println!("    filter: {:?}", tex.filter);
    }
    println!();

    println!(
        "Texture2D Groups: {}",
        model.resources.texture2dgroups.len()
    );
    for texgroup in &model.resources.texture2dgroups {
        println!(
            "  Texture2DGroup {}: texid={}, {} coordinates",
            texgroup.id,
            texgroup.texid,
            texgroup.coords.len()
        );
        for (i, coord) in texgroup.coords.iter().enumerate() {
            println!("    Coord {}: u={}, v={}", i, coord.u, coord.v);
        }
    }
    println!();

    println!(
        "Composite Materials: {}",
        model.resources.composite_materials.len()
    );
    for comp_group in &model.resources.composite_materials {
        println!(
            "  CompositeMaterialGroup {}: basematerialid={}, {} composites",
            comp_group.id,
            comp_group.basematerialid,
            comp_group.composites.len()
        );
        for (i, comp) in comp_group.composites.iter().enumerate() {
            println!("    Composite {}:", i);
            for (j, component) in comp.components.iter().enumerate() {
                println!(
                    "      Component {}: material={}, proportion={}",
                    j, component.propertyid, component.proportion
                );
            }
        }
    }
    println!();

    println!(
        "Multi-Properties: {}",
        model.resources.multi_properties.len()
    );
    for multi in &model.resources.multi_properties {
        println!(
            "  MultiProperties {}: pids={:?}, {} entries",
            multi.id,
            multi.pids,
            multi.entries.len()
        );
        for (i, entry) in multi.entries.iter().enumerate() {
            println!("    Entry {}: pindices={:?}", i, entry.pids);
        }
    }
    println!();

    println!("Objects: {}", model.resources.objects.len());
    for obj in &model.resources.objects {
        if let Some(mesh) = &obj.mesh {
            println!(
                "  Object {}: {} vertices, {} triangles",
                obj.id,
                mesh.vertices.len(),
                mesh.triangles.len()
            );
        }
    }

    Ok(())
}
