use lib3mf::{Extension, ParserConfig};

fn main() {
    let config = ParserConfig::new().with_extension(Extension::Displacement);

    // Simple test with explicit namespace prefix
    let xml = r#"<?xml version="1.0"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:d="http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07" requiredextensions="d">
  <resources>
    <object id="4" type="model">
      <mesh>
        <vertices>
          <vertex x="0" y="0" z="0"/>
          <vertex x="10" y="0" z="0"/>
          <vertex x="5" y="10" z="0"/>
        </vertices>
        <triangles>
          <triangle v1="0" v2="1" v3="2"/>
        </triangles>
      </mesh>
      <d:displacementmesh>
        <d:triangles did="3">
          <d:triangle v1="0" v2="1" v3="2" d1="0" d2="0" d3="0"/>
        </d:triangles>
      </d:displacementmesh>
    </object>
  </resources>
  <build><item objectid="4"/></build>
</model>"#;

    match lib3mf::parser::parse_model_xml_with_config(xml, config) {
        Ok(_) => println!("✓ Parsed successfully"),
        Err(e) => println!("❌ Failed: {}", e),
    }
}
