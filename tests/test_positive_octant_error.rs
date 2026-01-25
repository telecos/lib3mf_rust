//! Test to verify OutsidePositiveOctant error is properly detected

use lib3mf::{Error, Model, ParserConfig};
use std::io::Write;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Helper function to create a minimal 3MF file with a transformed object
fn create_3mf_with_transform(transform: [f64; 12]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));

    // Add [Content_Types].xml
    zip.start_file("[Content_Types].xml", SimpleFileOptions::default())?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
</Types>"#,
    )?;

    // Add _rels/.rels
    zip.add_directory("_rels/", SimpleFileOptions::default())?;
    zip.start_file("_rels/.rels", SimpleFileOptions::default())?;
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rel0" Target="/3D/3dmodel.model" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"#,
    )?;

    // Add 3D/3dmodel.model with transform
    zip.add_directory("3D/", SimpleFileOptions::default())?;
    zip.start_file("3D/3dmodel.model", SimpleFileOptions::default())?;
    
    let transform_str = format!(
        "{} {} {} {} {} {} {} {} {} {} {} {}",
        transform[0], transform[1], transform[2], transform[3],
        transform[4], transform[5], transform[6], transform[7],
        transform[8], transform[9], transform[10], transform[11]
    );
    
    let model_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0"/>
                    <vertex x="10" y="0" z="0"/>
                    <vertex x="5" y="10" z="0"/>
                    <vertex x="5" y="5" z="10"/>
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2"/>
                    <triangle v1="0" v2="1" v3="3"/>
                    <triangle v1="1" v2="2" v3="3"/>
                    <triangle v1="2" v2="0" v3="3"/>
                </triangles>
            </mesh>
        </object>
    </resources>
    <build>
        <item objectid="1" transform="{}"/>
    </build>
</model>"#,
        transform_str
    );
    
    zip.write_all(model_xml.as_bytes())?;
    let _writer = zip.finish()?;

    Ok(buffer)
}

#[test]
fn test_outside_positive_octant_error_detection() {
    // Create a transform that moves object to negative Y (outside positive octant)
    // Identity matrix except for Y translation = -5
    let transform_negative_y = [
        1.0, 0.0, 0.0, 0.0,   // X translation = 0
        0.0, 1.0, 0.0, -5.0,  // Y translation = -5
        0.0, 0.0, 1.0, 0.0,   // Z translation = 0
    ];

    let buffer = create_3mf_with_transform(transform_negative_y)
        .expect("Failed to create 3MF file");

    // Try to parse it
    let result = Model::from_reader_with_config(
        std::io::Cursor::new(&buffer),
        ParserConfig::default(),
    );

    // Should fail with OutsidePositiveOctant error
    match result {
        Err(e) => {
            let error_type = e.error_type();
            assert_eq!(
                error_type, "OutsidePositiveOctant",
                "Expected OutsidePositiveOctant error, got: {} (type: {})",
                e, error_type
            );
            
            // Verify the error message contains relevant information
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("E3003"),
                "Error should contain error code E3003"
            );
            assert!(
                error_msg.contains("Object 1"),
                "Error should mention the object ID"
            );
        }
        Ok(_) => {
            panic!("Expected parsing to fail with OutsidePositiveOctant error, but it succeeded");
        }
    }
}

#[test]
fn test_inside_positive_octant_succeeds() {
    // Create a 3MF file without transform (object remains in positive octant)
    let mut buffer = Vec::new();
    let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));

    // Add [Content_Types].xml
    zip.start_file("[Content_Types].xml", SimpleFileOptions::default()).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
</Types>"#,
    ).unwrap();

    // Add _rels/.rels
    zip.add_directory("_rels/", SimpleFileOptions::default()).unwrap();
    zip.start_file("_rels/.rels", SimpleFileOptions::default()).unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rel0" Target="/3D/3dmodel.model" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"#,
    ).unwrap();

    // Add 3D/3dmodel.model without transform
    zip.add_directory("3D/", SimpleFileOptions::default()).unwrap();
    zip.start_file("3D/3dmodel.model", SimpleFileOptions::default()).unwrap();
    
    let model_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0"/>
                    <vertex x="10" y="0" z="0"/>
                    <vertex x="5" y="10" z="0"/>
                    <vertex x="5" y="5" z="10"/>
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2"/>
                    <triangle v1="0" v2="1" v3="3"/>
                    <triangle v1="1" v2="2" v3="3"/>
                    <triangle v1="2" v2="0" v3="3"/>
                </triangles>
            </mesh>
        </object>
    </resources>
    <build>
        <item objectid="1"/>
    </build>
</model>"#;
    
    zip.write_all(model_xml.as_bytes()).unwrap();
    let _writer = zip.finish().unwrap();

    // Try to parse it
    let result = Model::from_reader_with_config(
        std::io::Cursor::new(&buffer),
        ParserConfig::default(),
    );

    // Should succeed
    assert!(
        result.is_ok(),
        "Expected parsing to succeed for object in positive octant, but got error: {:?}",
        result.err()
    );
}

#[test]
fn test_error_type_method_coverage() {
    // Test that error_type() works for various error types
    let errors = vec![
        (Error::InvalidModel("test".to_string()), "InvalidModel"),
        (Error::OutsidePositiveOctant(1, -1.0, 0.0, 0.0), "OutsidePositiveOctant"),
        (Error::ParseError("test".to_string()), "ParseError"),
        (Error::InvalidFormat("test".to_string()), "InvalidFormat"),
        (Error::UnsupportedExtension("test".to_string()), "UnsupportedExtension"),
    ];

    for (error, expected_type) in errors {
        assert_eq!(
            error.error_type(),
            expected_type,
            "error_type() returned wrong type for error: {}",
            error
        );
    }
}
