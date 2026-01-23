use lib3mf::{Extension, ParserConfig};

fn test_case(name: &str, xml: &str, config: &ParserConfig, should_fail: bool) {
    match lib3mf::parser::parse_model_xml_with_config(xml, config.clone()) {
        Ok(_) => {
            if should_fail {
                println!("❌ {} FAILED: Should reject but succeeded", name);
            } else {
                println!("✓ {} PASSED: Correctly accepted", name);
            }
        }
        Err(e) => {
            if should_fail {
                println!("✓ {} PASSED: Correctly rejected - {}", name, e);
            } else {
                println!("❌ {} FAILED: Should accept but rejected - {}", name, e);
            }
        }
    }
}

fn main() {
    println!("=== Comprehensive Displacement Extension Validation Tests ===\n");

    let config = ParserConfig::new().with_extension(Extension::Displacement);

    // Test 1: Valid minimal displacement model (baseline)
    test_case(
        "Valid minimal model",
        r#"<?xml version="1.0"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:d="http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07" requiredextensions="d">
  <resources>
    <d:displacement2d id="1" path="/3D/Textures/disp.png" channel="R" filter="linear" tilestyleu="wrap" tilestylev="wrap"/>
    <d:normvectorgroup id="2">
      <d:normvector x="0.0" y="0.0" z="1.0"/>
    </d:normvectorgroup>
    <d:disp2dgroup id="3" dispid="1" nid="2" height="1.0" offset="0.0">
      <d:disp2dcoord u="0.0" v="0.0" n="0"/>
    </d:disp2dgroup>
    <object id="4" type="model">
      <mesh>
        <vertices><vertex x="0" y="0" z="0"/><vertex x="10" y="0" z="0"/><vertex x="5" y="10" z="0"/></vertices>
        <triangles><triangle v1="0" v2="1" v3="2"/></triangles>
      </mesh>
      <d:displacementmesh>
        <d:triangles did="3">
          <d:triangle v1="0" v2="1" v3="2" d1="0" d2="0" d3="0"/>
        </d:triangles>
      </d:displacementmesh>
    </object>
  </resources>
  <build><item objectid="4"/></build>
</model>"#,
        &config,
        false,
    );

    // Test 2: Invalid dispid forward reference (DPX 3312)
    test_case(
        "Invalid dispid forward ref",
        r#"<?xml version="1.0"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:d="http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07" requiredextensions="d">
  <resources>
    <d:normvectorgroup id="2">
      <d:normvector x="0.0" y="0.0" z="1.0"/>
    </d:normvectorgroup>
    <d:disp2dgroup id="3" dispid="1" nid="2" height="1.0" offset="0.0">
      <d:disp2dcoord u="0.0" v="0.0" n="0"/>
    </d:disp2dgroup>
    <d:displacement2d id="1" path="/3D/Textures/disp.png" channel="R" filter="linear" tilestyleu="wrap" tilestylev="wrap"/>
  </resources>
  <build><item objectid="1"/></build>
</model>"#,
        &config,
        true,
    );

    // Test 3: Invalid nid forward reference (DPX 3312)
    test_case(
        "Invalid nid forward ref",
        r#"<?xml version="1.0"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:d="http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07" requiredextensions="d">
  <resources>
    <d:displacement2d id="1" path="/3D/Textures/disp.png" channel="R" filter="linear" tilestyleu="wrap" tilestylev="wrap"/>
    <d:disp2dgroup id="3" dispid="1" nid="2" height="1.0" offset="0.0">
      <d:disp2dcoord u="0.0" v="0.0" n="0"/>
    </d:disp2dgroup>
    <d:normvectorgroup id="2">
      <d:normvector x="0.0" y="0.0" z="1.0"/>
    </d:normvectorgroup>
  </resources>
  <build><item objectid="1"/></build>
</model>"#,
        &config,
        true,
    );

    // Test 4: Multiple triangles elements (DPX 3314)
    test_case(
        "Multiple triangles in displacementmesh",
        r#"<?xml version="1.0"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:d="http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07" requiredextensions="d">
  <resources>
    <d:displacement2d id="1" path="/3D/Textures/disp.png" channel="R" filter="linear" tilestyleu="wrap" tilestylev="wrap"/>
    <d:normvectorgroup id="2">
      <d:normvector x="0.0" y="0.0" z="1.0"/>
    </d:normvectorgroup>
    <d:disp2dgroup id="3" dispid="1" nid="2" height="1.0" offset="0.0">
      <d:disp2dcoord u="0.0" v="0.0" n="0"/>
    </d:disp2dgroup>
    <object id="4" type="model">
      <mesh>
        <vertices><vertex x="0" y="0" z="0"/><vertex x="10" y="0" z="0"/><vertex x="5" y="10" z="0"/></vertices>
        <triangles><triangle v1="0" v2="1" v3="2"/></triangles>
      </mesh>
      <d:displacementmesh>
        <d:triangles did="3">
          <d:triangle v1="0" v2="1" v3="2" d1="0" d2="0" d3="0"/>
        </d:triangles>
        <d:triangles did="3">
          <d:triangle v1="0" v2="1" v3="2" d1="0" d2="0" d3="0"/>
        </d:triangles>
      </d:displacementmesh>
    </object>
  </resources>
  <build><item objectid="4"/></build>
</model>"#,
        &config,
        true,
    );

    // Test 5: Invalid displacement texture path (not in /3D/Textures/)
    test_case(
        "Invalid texture path",
        r#"<?xml version="1.0"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:d="http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07" requiredextensions="d">
  <resources>
    <d:displacement2d id="1" path="/Textures/disp.png" channel="R" filter="linear" tilestyleu="wrap" tilestylev="wrap"/>
  </resources>
  <build><item objectid="1"/></build>
</model>"#,
        &config,
        true,
    );

    // Test 6: Displacementmesh with non-model object type (DPX 4.0)
    test_case(
        "Displacementmesh on support object",
        r#"<?xml version="1.0"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:d="http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07" requiredextensions="d">
  <resources>
    <d:displacement2d id="1" path="/3D/Textures/disp.png" channel="R" filter="linear" tilestyleu="wrap" tilestylev="wrap"/>
    <d:normvectorgroup id="2">
      <d:normvector x="0.0" y="0.0" z="1.0"/>
    </d:normvectorgroup>
    <d:disp2dgroup id="3" dispid="1" nid="2" height="1.0" offset="0.0">
      <d:disp2dcoord u="0.0" v="0.0" n="0"/>
    </d:disp2dgroup>
    <object id="4" type="support">
      <mesh>
        <vertices><vertex x="0" y="0" z="0"/><vertex x="10" y="0" z="0"/><vertex x="5" y="10" z="0"/></vertices>
        <triangles><triangle v1="0" v2="1" v3="2"/></triangles>
      </mesh>
      <d:displacementmesh>
        <d:triangles did="3">
          <d:triangle v1="0" v2="1" v3="2" d1="0" d2="0" d3="0"/>
        </d:triangles>
      </d:displacementmesh>
    </object>
  </resources>
  <build><item objectid="4"/></build>
</model>"#,
        &config,
        true,
    );

    println!("\n=== All tests completed ===");
}
