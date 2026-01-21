use lib3mf::{Extension, ParserConfig};

fn main() {
    // Test that p:path and p:UUID attributes don't cause validation errors
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06">
  <resources>
    <object id="1">
      <mesh><vertices><vertex x="0" y="0" z="0"/><vertex x="1" y="0" z="0"/><vertex x="0" y="1" z="0"/></vertices><triangles><triangle v1="0" v2="1" v3="2"/></triangles></mesh>
    </object>
    <object id="2">
      <components>
        <component objectid="1" p:UUID="test-uuid" p:path="/some/path.model"/>
      </components>
    </object>
  </resources>
  <build p:UUID="build-uuid">
    <item objectid="2" p:UUID="item-uuid"/>
  </build>
</model>"#;

    let config = ParserConfig::new().with_extension(Extension::Production);
    match lib3mf::parser::parse_model_xml_with_config(xml, config) {
        Ok(model) => {
            println!("✓ SUCCESS - p:path and p:UUID accepted");
            if let Some(comp) = model
                .resources
                .objects
                .get(1)
                .and_then(|o| o.components.first())
            {
                println!("  Component path: {:?}", comp.path);
            }
        }
        Err(e) => println!("✗ FAILED: {}", e),
    }
}
