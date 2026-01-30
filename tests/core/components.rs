//! Component tests for 3MF core specification
//!
//! Tests for component hierarchies, transformations, and validation

#[path = "../common/mod.rs"]
mod common;

use lib3mf::parser::parse_model_xml;
use lib3mf::Model;
use std::fs::File;

#[test]
fn test_parse_components_from_file() {
    let file = File::open("test_files/components/assembly.3mf").expect("Failed to open test file");
    let model = Model::from_reader(file).expect("Failed to parse 3MF file");

    // Should have 2 objects
    assert_eq!(model.resources.objects.len(), 2);

    // Object 1 is a base mesh with no components
    let obj1 = &model.resources.objects[0];
    assert_eq!(obj1.id, 1);
    assert!(obj1.mesh.is_some());
    assert_eq!(obj1.components.len(), 0);

    // Object 2 is an assembly with 3 component references to object 1
    let obj2 = &model.resources.objects[1];
    assert_eq!(obj2.id, 2);
    assert!(obj2.mesh.is_none());
    assert_eq!(obj2.components.len(), 3);

    // Check first component (identity transform at origin)
    assert_eq!(obj2.components[0].objectid, 1);
    assert!(obj2.components[0].transform.is_some());
    let t1 = obj2.components[0].transform.unwrap();
    assert_eq!(t1[9], 0.0); // x translation
    assert_eq!(t1[10], 0.0); // y translation
    assert_eq!(t1[11], 0.0); // z translation

    // Check second component (translated 20mm in x)
    assert_eq!(obj2.components[1].objectid, 1);
    assert!(obj2.components[1].transform.is_some());
    let t2 = obj2.components[1].transform.unwrap();
    assert_eq!(t2[9], 20.0); // x translation
    assert_eq!(t2[10], 0.0); // y translation
    assert_eq!(t2[11], 0.0); // z translation

    // Check third component (translated 20mm in y)
    assert_eq!(obj2.components[2].objectid, 1);
    assert!(obj2.components[2].transform.is_some());
    let t3 = obj2.components[2].transform.unwrap();
    assert_eq!(t3[9], 0.0); // x translation
    assert_eq!(t3[10], 20.0); // y translation
    assert_eq!(t3[11], 0.0); // z translation

    // Build should reference object 2 (the assembly)
    assert_eq!(model.build.items.len(), 1);
    assert_eq!(model.build.items[0].objectid, 2);
}

#[test]
fn test_component_validation_invalid_reference() {
    // Create a 3MF with invalid component reference
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
        <component objectid="99"/>
      </components>
    </object>
  </resources>
  <build>
    <item objectid="2"/>
  </build>
</model>"#;

    let result = common::parse_and_validate_components(xml);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("non-existent object"));
}

#[test]
fn test_component_validation_circular_reference() {
    // Create a 3MF with circular component reference
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

    let result = common::parse_and_validate_components(xml);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Circular component reference"));
}

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

    let result = common::parse_and_validate_components(xml);
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
    assert!(
        result.is_ok(),
        "Deep non-circular hierarchy should be valid"
    );
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
    assert_eq!(transform[9], 10.0); // tx
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
