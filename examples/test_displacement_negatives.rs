use lib3mf::{Extension, ParserConfig};

fn main() {
    println!("Testing displacement validation with minimal negative test cases...\n");

    // Test 1: Invalid dispid reference
    let test1 = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:d="http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07" requiredextensions="d">
  <resources>
    <d:normvectorgroup id="1">
      <d:normvector x="0.0" y="0.0" z="1.0"/>
    </d:normvectorgroup>
    <d:disp2dgroup id="2" dispid="999" nid="1" height="1.0" offset="0.0">
      <d:disp2dcoord u="0.0" v="0.0" n="0"/>
    </d:disp2dgroup>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"#;

    let config = ParserConfig::new().with_extension(Extension::Displacement);
    match lib3mf::parser::parse_model_xml_with_config(test1, config.clone()) {
        Ok(_) => println!("❌ Test 1 FAILED: Should reject invalid dispid reference"),
        Err(e) => println!("✓ Test 1 PASSED: {}", e),
    }

    // Test 2: Invalid nid reference
    let test2 = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:d="http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07" requiredextensions="d">
  <resources>
    <d:displacement2d id="1" path="/3D/Textures/disp.png" channel="red" filter="linear" tilestyleu="wrap" tilestylev="wrap"/>
    <d:disp2dgroup id="2" dispid="1" nid="999" height="1.0" offset="0.0">
      <d:disp2dcoord u="0.0" v="0.0" n="0"/>
    </d:disp2dgroup>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"#;

    match lib3mf::parser::parse_model_xml_with_config(test2, config.clone()) {
        Ok(_) => println!("❌ Test 2 FAILED: Should reject invalid nid reference"),
        Err(e) => println!("✓ Test 2 PASSED: {}", e),
    }

    // Test 3: Invalid normvector index
    let test3 = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:d="http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07" requiredextensions="d">
  <resources>
    <d:displacement2d id="1" path="/3D/Textures/disp.png" channel="red" filter="linear" tilestyleu="wrap" tilestylev="wrap"/>
    <d:normvectorgroup id="2">
      <d:normvector x="0.0" y="0.0" z="1.0"/>
    </d:normvectorgroup>
    <d:disp2dgroup id="3" dispid="1" nid="2" height="1.0" offset="0.0">
      <d:disp2dcoord u="0.0" v="0.0" n="999"/>
    </d:disp2dgroup>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"#;

    match lib3mf::parser::parse_model_xml_with_config(test3, config.clone()) {
        Ok(_) => println!("❌ Test 3 FAILED: Should reject invalid normvector index"),
        Err(e) => println!("✓ Test 3 PASSED: {}", e),
    }

    // Test 4: Invalid displacement coordinate index in triangle
    let test4 = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:d="http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07" requiredextensions="d">
  <resources>
    <d:displacement2d id="1" path="/3D/Textures/disp.png" channel="red" filter="linear" tilestyleu="wrap" tilestylev="wrap"/>
    <d:normvectorgroup id="2">
      <d:normvector x="0.0" y="0.0" z="1.0"/>
    </d:normvectorgroup>
    <d:disp2dgroup id="3" dispid="1" nid="2" height="1.0" offset="0.0">
      <d:disp2dcoord u="0.0" v="0.0" n="0"/>
    </d:disp2dgroup>
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
          <d:triangle v1="0" v2="1" v3="2" d1="999" d2="0" d3="0"/>
        </d:triangles>
      </d:displacementmesh>
    </object>
  </resources>
  <build>
    <item objectid="4"/>
  </build>
</model>"#;

    match lib3mf::parser::parse_model_xml_with_config(test4, config) {
        Ok(_) => println!("❌ Test 4 FAILED: Should reject invalid displacement coordinate index"),
        Err(e) => println!("✓ Test 4 PASSED: {}", e),
    }
}
