use lib3mf::parser::parse_model_xml;

#[test]
fn test_circular_reference_error_shows_path() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <resources>
    <object id="1">
      <components>
        <component objectid="2"/>
      </components>
    </object>
    <object id="2">
      <components>
        <component objectid="3"/>
      </components>
    </object>
    <object id="3">
      <components>
        <component objectid="1"/>
      </components>
    </object>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"#;

    let result = parse_model_xml(xml);
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    println!("Error: {}", err_msg);

    // Should contain the arrow notation
    assert!(err_msg.contains("→"));
    // Should contain circular reference
    assert!(err_msg.contains("Circular component reference"));
}
