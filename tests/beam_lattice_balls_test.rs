//! Integration tests for Beam Lattice Balls extension support

use lib3mf::parser::parse_model_xml_with_config;
use lib3mf::{Extension, ParserConfig};

#[test]
fn test_balls_extension_namespace_recognized() {
    // Create a minimal 3MF XML with balls extension in requiredextensions
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" 
       xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:b="http://schemas.microsoft.com/3dmanufacturing/beamlattice/2017/02"
       xmlns:bb="http://schemas.microsoft.com/3dmanufacturing/beamlattice/balls/2020/07"
       requiredextensions="b bb">
  <resources>
    <object id="1" type="model">
      <mesh>
        <vertices>
          <vertex x="0" y="0" z="0"/>
          <vertex x="10" y="0" z="0"/>
          <vertex x="0" y="10" z="0"/>
        </vertices>
        <triangles>
          <triangle v1="0" v2="1" v3="2"/>
        </triangles>
        <b:beamlattice radius="1.0" minlength="0.1" cap="sphere">
          <b:beams>
            <b:beam v1="0" v2="1"/>
          </b:beams>
          <bb:balls>
            <bb:ball vindex="0"/>
            <bb:ball vindex="1"/>
          </bb:balls>
        </b:beamlattice>
      </mesh>
    </object>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"#;

    // Create a parser config that supports BeamLattice extension
    let config = ParserConfig::new().with_extension(Extension::BeamLattice);

    // Parse the model - should succeed without errors
    let result = parse_model_xml_with_config(xml, config);
    assert!(
        result.is_ok(),
        "Failed to parse model with balls extension: {:?}",
        result.err()
    );

    let model = result.unwrap();

    // Verify the model has the required extension
    assert!(model.required_extensions.contains(&Extension::BeamLattice));

    // Verify the beamset with balls was parsed
    let object = &model.resources.objects[0];
    assert!(object.mesh.is_some());

    let mesh = object.mesh.as_ref().unwrap();
    assert!(mesh.beamset.is_some());

    let beamset = mesh.beamset.as_ref().unwrap();
    assert_eq!(beamset.beams.len(), 1);
    assert_eq!(beamset.balls.len(), 2);

    // Verify ball properties
    assert_eq!(beamset.balls[0].vindex, 0);
    assert_eq!(beamset.balls[1].vindex, 1);
}

#[test]
fn test_balls_extension_with_full_namespace() {
    // Test using the full namespace URI instead of prefix
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" 
       xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       requiredextensions="http://schemas.microsoft.com/3dmanufacturing/beamlattice/2017/02 http://schemas.microsoft.com/3dmanufacturing/beamlattice/balls/2020/07">
  <resources>
    <object id="1" type="model">
      <mesh>
        <vertices>
          <vertex x="0" y="0" z="0"/>
          <vertex x="10" y="0" z="0"/>
          <vertex x="0" y="10" z="0"/>
        </vertices>
        <triangles>
          <triangle v1="0" v2="1" v3="2"/>
        </triangles>
      </mesh>
    </object>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"#;

    let config = ParserConfig::new().with_extension(Extension::BeamLattice);
    let result = parse_model_xml_with_config(xml, config);
    
    assert!(
        result.is_ok(),
        "Failed to parse model with full balls namespace URI: {:?}",
        result.err()
    );
}

#[test]
fn test_balls_extension_fails_without_beamlattice_support() {
    // If BeamLattice extension is not supported, parsing should fail
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" 
       xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:bb="http://schemas.microsoft.com/3dmanufacturing/beamlattice/balls/2020/07"
       requiredextensions="bb">
  <resources>
    <object id="1" type="model">
      <mesh>
        <vertices>
          <vertex x="0" y="0" z="0"/>
          <vertex x="10" y="0" z="0"/>
          <vertex x="0" y="10" z="0"/>
        </vertices>
        <triangles>
          <triangle v1="0" v2="1" v3="2"/>
        </triangles>
      </mesh>
    </object>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"#;

    // Don't support BeamLattice extension
    let config = ParserConfig::new();
    let result = parse_model_xml_with_config(xml, config);
    
    // Should fail because balls extension requires BeamLattice support
    assert!(
        result.is_err(),
        "Should have failed when BeamLattice extension is not supported"
    );
}
