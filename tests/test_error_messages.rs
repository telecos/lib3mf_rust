//! Test to verify error message quality

use lib3mf::parser::parse_model_xml;

#[test]
fn test_error_message_for_circular_reference() {
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
    println!("Error message: {}", err);
    // Verify the error message contains useful information
    assert!(err.to_string().contains("Circular component reference"));
}

#[test]
fn test_error_message_for_invalid_reference() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <resources>
    <object id="1">
      <components>
        <component objectid="999"/>
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
    println!("Error message: {}", err);
    // Verify the error message contains the object ID and referenced ID
    assert!(err.to_string().contains("999"));
}
