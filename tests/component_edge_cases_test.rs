//! Additional component validation edge case tests

use lib3mf::parser::parse_model_xml;

#[test]
fn test_component_three_way_circular_reference() {
    // Test A→B→C→A circular reference
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
    assert!(err.to_string().contains("Circular component reference"));
}

#[test]
fn test_component_deep_non_circular_hierarchy() {
    // Test deep but valid hierarchy: 1→2→3→4→5 (no cycle)
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <resources>
    <object id="5">
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
    <object id="4">
      <components>
        <component objectid="5"/>
      </components>
    </object>
    <object id="3">
      <components>
        <component objectid="4"/>
      </components>
    </object>
    <object id="2">
      <components>
        <component objectid="3"/>
      </components>
    </object>
    <object id="1">
      <components>
        <component objectid="2"/>
      </components>
    </object>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"#;

    let result = parse_model_xml(xml);
    assert!(result.is_ok(), "Deep non-circular hierarchy should be valid");
}

#[test]
fn test_component_with_transformation_matrix() {
    // Test component with transformation matrix
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <resources>
    <object id="1">
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
    <object id="2">
      <components>
        <component objectid="1" transform="1 0 0 0 1 0 0 0 1 10 20 30"/>
      </components>
    </object>
  </resources>
  <build>
    <item objectid="2"/>
  </build>
</model>"#;

    let result = parse_model_xml(xml);
    assert!(result.is_ok());
    let model = result.unwrap();
    let obj2 = &model.resources.objects[1];
    assert_eq!(obj2.components.len(), 1);
    assert!(obj2.components[0].transform.is_some());
    let transform = obj2.components[0].transform.unwrap();
    assert_eq!(transform[9], 10.0);  // tx
    assert_eq!(transform[10], 20.0); // ty
    assert_eq!(transform[11], 30.0); // tz
}

#[test]
fn test_component_invalid_transform_too_few_values() {
    // Test component with invalid transformation matrix (too few values)
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <resources>
    <object id="1">
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
    <object id="2">
      <components>
        <component objectid="1" transform="1 0 0 0 1 0"/>
      </components>
    </object>
  </resources>
  <build>
    <item objectid="2"/>
  </build>
</model>"#;

    let result = parse_model_xml(xml);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("12 values"));
}

#[test]
fn test_component_invalid_transform_infinity() {
    // Test component with invalid transformation matrix (infinity value)
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <resources>
    <object id="1">
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
    <object id="2">
      <components>
        <component objectid="1" transform="1 0 0 0 1 0 0 0 1 inf 20 30"/>
      </components>
    </object>
  </resources>
  <build>
    <item objectid="2"/>
  </build>
</model>"#;

    let result = parse_model_xml(xml);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("finite"));
}

