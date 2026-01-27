//! Test for Displacement extension namespace support
//!
//! This test verifies that both the older 2022/07 and newer 2023/10
//! displacement namespaces are properly supported.

use lib3mf::{Extension, ParserConfig};

#[test]
fn test_parse_displacement_2023_10_namespace() {
    // Create a simple 3MF model XML that uses the 2023/10 displacement namespace
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:d="http://schemas.3mf.io/3dmanufacturing/displacement/2023/10"
       requiredextensions="d">
    <metadata name="Application">Test</metadata>
    <resources>
        <d:displacement2d id="1" path="/3D/Textures/test.png"/>
        <d:normvectorgroup id="2">
            <d:normvector x="0.0" y="0.0" z="1.0"/>
        </d:normvectorgroup>
        <d:disp2dgroup id="3" dispid="1" nid="2" height="1.0">
            <d:disp2dcoord u="0.0" v="0.0" n="0"/>
        </d:disp2dgroup>
        <object id="4" type="model">
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
        <item objectid="4"/>
    </build>
</model>"#;

    // Create parser config with displacement support
    let config = ParserConfig::new().with_extension(Extension::Displacement);

    // Parse the XML - this should succeed with the 2023/10 namespace
    let result = lib3mf::parser::parse_model_xml_with_config(xml, config);

    // Verify parsing succeeded
    assert!(
        result.is_ok(),
        "Failed to parse displacement with 2023/10 namespace: {:?}",
        result.err()
    );

    let model = result.unwrap();

    // Verify the displacement resources were parsed
    assert_eq!(model.resources.displacement_maps.len(), 1);
    assert_eq!(model.resources.norm_vector_groups.len(), 1);
    assert_eq!(model.resources.disp2d_groups.len(), 1);

    // Verify the extension is in required extensions
    assert!(model.required_extensions.contains(&Extension::Displacement));
}

#[test]
fn test_parse_displacement_2022_07_namespace_still_works() {
    // Create a simple 3MF model XML that uses the older 2022/07 displacement namespace
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:d="http://schemas.microsoft.com/3dmanufacturing/displacement/2022/07"
       requiredextensions="d">
    <metadata name="Application">Test</metadata>
    <resources>
        <d:displacement2d id="1" path="/3D/Textures/test.png"/>
        <object id="2" type="model">
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
        <item objectid="2"/>
    </build>
</model>"#;

    // Create parser config with displacement support
    let config = ParserConfig::new().with_extension(Extension::Displacement);

    // Parse the XML - this should succeed with the 2022/07 namespace
    let result = lib3mf::parser::parse_model_xml_with_config(xml, config);

    // Verify parsing succeeded
    assert!(
        result.is_ok(),
        "Failed to parse displacement with 2022/07 namespace: {:?}",
        result.err()
    );

    let model = result.unwrap();

    // Verify the displacement resources were parsed
    assert_eq!(model.resources.displacement_maps.len(), 1);

    // Verify the extension is in required extensions
    assert!(model.required_extensions.contains(&Extension::Displacement));
}
