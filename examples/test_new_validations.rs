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
    println!("=== Testing New Displacement Validations ===\n");

    let config = ParserConfig::new().with_extension(Extension::Displacement);

    // Test 1: Missing namespace prefix on vertex (DPX 3314_05)
    test_case(
        "Missing namespace on vertex",
        r#"<?xml version="1.0"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:d="http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07" requiredextensions="d">
  <resources>
    <object id="1" type="model">
      <mesh>
        <vertices><vertex x="0" y="0" z="0"/><vertex x="10" y="0" z="0"/><vertex x="5" y="10" z="0"/></vertices>
        <triangles><triangle v1="0" v2="1" v3="2"/></triangles>
      </mesh>
      <d:displacementmesh>
        <vertex x="0" y="0" z="0"/>
      </d:displacementmesh>
    </object>
  </resources>
  <build><item objectid="1"/></build>
</model>"#,
        &config,
        true,
    );

    // Test 2: Missing namespace prefix on triangles (DPX 3314_05)
    test_case(
        "Missing namespace on triangles",
        r#"<?xml version="1.0"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:d="http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07" requiredextensions="d">
  <resources>
    <object id="1" type="model">
      <mesh>
        <vertices><vertex x="0" y="0" z="0"/><vertex x="10" y="0" z="0"/><vertex x="5" y="10" z="0"/></vertices>
        <triangles><triangle v1="0" v2="1" v3="2"/></triangles>
      </mesh>
      <d:displacementmesh>
        <d:vertices><d:vertex x="0" y="0" z="0"/></d:vertices>
        <triangles did="1">
          <d:triangle v1="0" v2="1" v3="2"/>
        </triangles>
      </d:displacementmesh>
    </object>
  </resources>
  <build><item objectid="1"/></build>
</model>"#,
        &config,
        true,
    );

    // Test 3: Forward reference in triangle did (DPX 3314_01)
    test_case(
        "Forward ref in triangle did",
        r#"<?xml version="1.0"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:d="http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07" requiredextensions="d">
  <resources>
    <d:displacement2d id="1" path="/3D/Textures/disp.png" channel="R" filter="linear" tilestyleu="wrap" tilestylev="wrap"/>
    <d:normvectorgroup id="2">
      <d:normvector x="0.0" y="0.0" z="1.0"/>
    </d:normvectorgroup>
    <object id="4" type="model">
      <mesh>
        <vertices><vertex x="0" y="0" z="0"/><vertex x="10" y="0" z="0"/><vertex x="5" y="10" z="0"/></vertices>
        <triangles><triangle v1="0" v2="1" v3="2"/></triangles>
      </mesh>
      <d:displacementmesh>
        <d:vertices>
          <d:vertex x="0" y="0" z="0"/>
          <d:vertex x="10" y="0" z="0"/>
          <d:vertex x="5" y="10" z="0"/>
        </d:vertices>
        <d:triangles>
          <d:triangle v1="0" v2="1" v3="2" did="3" d1="0" d2="0" d3="0"/>
        </d:triangles>
      </d:displacementmesh>
    </object>
    <d:disp2dgroup id="3" dispid="1" nid="2" height="1.0" offset="0.0">
      <d:disp2dcoord u="0.0" v="0.0" n="0"/>
    </d:disp2dgroup>
  </resources>
  <build><item objectid="4"/></build>
</model>"#,
        &config,
        true,
    );

    // Test 4: Displacement extension not in requiredextensions (DPX 3312_03)
    test_case(
        "Displacement not in requiredextensions",
        r#"<?xml version="1.0"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:d="http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07">
  <resources>
    <d:displacement2d id="1" path="/3D/Textures/disp.png" channel="R" filter="linear" tilestyleu="wrap" tilestylev="wrap"/>
  </resources>
  <build><item objectid="1"/></build>
</model>"#,
        &config,
        true,
    );

    // Test 5: Degenerate triangle (DPX 3310_01)
    test_case(
        "Degenerate triangle in displacementmesh",
        r#"<?xml version="1.0"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:d="http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07" requiredextensions="d">
  <resources>
    <object id="1" type="model">
      <mesh>
        <vertices><vertex x="0" y="0" z="0"/><vertex x="10" y="0" z="0"/><vertex x="5" y="10" z="0"/><vertex x="5" y="5" z="5"/></vertices>
        <triangles><triangle v1="0" v2="1" v3="2"/><triangle v1="0" v2="1" v3="3"/><triangle v1="0" v2="2" v3="3"/><triangle v1="1" v2="2" v3="3"/></triangles>
      </mesh>
      <d:displacementmesh>
        <d:vertices>
          <d:vertex x="0" y="0" z="0"/>
          <d:vertex x="10" y="0" z="0"/>
          <d:vertex x="5" y="10" z="0"/>
          <d:vertex x="5" y="5" z="5"/>
        </d:vertices>
        <d:triangles>
          <d:triangle v1="0" v2="0" v3="2"/>
        </d:triangles>
      </d:displacementmesh>
    </object>
  </resources>
  <build><item objectid="1"/></build>
</model>"#,
        &config,
        true,
    );

    // Test 6: Only 3 triangles in displacementmesh (DPX 3308_02)
    test_case(
        "Only 3 triangles",
        r#"<?xml version="1.0"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:d="http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07" requiredextensions="d">
  <resources>
    <object id="1" type="model">
      <mesh>
        <vertices><vertex x="0" y="0" z="0"/><vertex x="10" y="0" z="0"/><vertex x="5" y="10" z="0"/><vertex x="5" y="5" z="5"/></vertices>
        <triangles><triangle v1="0" v2="1" v3="2"/><triangle v1="0" v2="1" v3="3"/><triangle v1="0" v2="2" v3="3"/><triangle v1="1" v2="2" v3="3"/></triangles>
      </mesh>
      <d:displacementmesh>
        <d:vertices>
          <d:vertex x="0" y="0" z="0"/>
          <d:vertex x="10" y="0" z="0"/>
          <d:vertex x="5" y="10" z="0"/>
          <d:vertex x="5" y="5" z="5"/>
        </d:vertices>
        <d:triangles>
          <d:triangle v1="0" v2="1" v3="2"/>
          <d:triangle v1="0" v2="1" v3="3"/>
          <d:triangle v1="0" v2="2" v3="3"/>
        </d:triangles>
      </d:displacementmesh>
    </object>
  </resources>
  <build><item objectid="1"/></build>
</model>"#,
        &config,
        true,
    );

    println!("\n=== All tests completed ===");
}
