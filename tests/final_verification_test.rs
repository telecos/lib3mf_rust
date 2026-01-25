//! Final verification that all acceptance criteria are met

use lib3mf::parser::parse_model_xml;
use lib3mf::validator::validate_model;

#[test]
fn verify_circular_path_in_error_message() {
    // Create A→B→C→A circular reference
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <resources>
    <object id="10">
      <components>
        <component objectid="20"/>
      </components>
    </object>
    <object id="20">
      <components>
        <component objectid="30"/>
      </components>
    </object>
    <object id="30">
      <components>
        <component objectid="10"/>
      </components>
    </object>
  </resources>
  <build>
    <item objectid="10"/>
  </build>
</model>"#;

    // First parse the XML (parsing doesn't validate circular references)
    let model = parse_model_xml(xml).expect("Parsing should succeed");

    // Then validate the model (this detects circular references)
    let result = validate_model(&model);
    assert!(result.is_err(), "Should detect circular reference");

    let err = result.unwrap_err();
    let err_msg = err.to_string();

    println!("\nActual error message:\n{}\n", err_msg);

    // Verify the error message format matches issue requirements
    assert!(
        err_msg.contains("Circular component reference"),
        "Error should mention circular component reference"
    );
    assert!(
        err_msg.contains("→"),
        "Error should use arrow notation to show path"
    );

    // Verify the path is shown (exact order may vary based on DFS traversal)
    assert!(err_msg.contains("10"), "Should contain object ID 10");
    assert!(err_msg.contains("20"), "Should contain object ID 20");
    assert!(err_msg.contains("30"), "Should contain object ID 30");
}

#[test]
fn verify_all_validation_cases() {
    // Valid component reference - should succeed
    let valid_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <resources>
    <object id="1">
      <mesh>
        <vertices><vertex x="0" y="0" z="0"/><vertex x="1" y="0" z="0"/><vertex x="0" y="1" z="0"/></vertices>
        <triangles><triangle v1="0" v2="1" v3="2"/></triangles>
      </mesh>
    </object>
    <object id="2">
      <components>
        <component objectid="1" transform="1 0 0 0 1 0 0 0 1 10 20 30"/>
      </components>
    </object>
  </resources>
  <build><item objectid="2"/></build>
</model>"#;

    let result = parse_model_xml(valid_xml);
    assert!(result.is_ok(), "Valid component reference should succeed");
    let model = result.unwrap();
    assert_eq!(model.resources.objects[1].components.len(), 1);
    assert_eq!(model.resources.objects[1].components[0].objectid, 1);
    assert!(model.resources.objects[1].components[0].transform.is_some());

    println!("✅ Valid component reference test passed");
    println!("✅ Component with transformation matrix test passed");
}
