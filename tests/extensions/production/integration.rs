//! Integration tests for the Production extension parser
//!
//! Tests that cover the validate_production_external_paths function
//! and related production extension parsing code paths.

use lib3mf::{Extension, Model, ParserConfig};
use std::io::Write;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

/// Helper: create a minimal 3MF package with optional extra files
fn create_production_3mf(root_model_xml: &str, extra_files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));

    zip.start_file("[Content_Types].xml", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
</Types>"#,
    )
    .unwrap();

    zip.add_directory("_rels/", SimpleFileOptions::default())
        .unwrap();
    zip.start_file("_rels/.rels", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rel0" Target="/3D/3dmodel.model" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"#,
    )
    .unwrap();

    zip.add_directory("3D/", SimpleFileOptions::default())
        .unwrap();
    zip.start_file("3D/3dmodel.model", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(root_model_xml.as_bytes()).unwrap();

    for (path, data) in extra_files {
        zip.start_file(path, SimpleFileOptions::default()).unwrap();
        zip.write_all(data).unwrap();
    }

    zip.finish().unwrap();
    buffer
}

const EXTERNAL_OBJECT_XML: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
    <resources>
        <object id="5" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0"/>
                    <vertex x="1" y="0" z="0"/>
                    <vertex x="0" y="1" z="0"/>
                    <vertex x="0" y="0" z="1"/>
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
    <build/>
</model>"#;

#[test]
fn test_production_build_item_external_path_valid() {
    // Build item references an object in an external file
    let root_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US"
    xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
    xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06"
    requiredextensions="p">
    <resources/>
    <build p:UUID="00000000-0000-0000-0000-000000000001">
        <item objectid="5" p:path="/3D/external.model" p:UUID="00000000-0000-0000-0000-000000000002"/>
    </build>
</model>"#;

    let buffer = create_production_3mf(root_xml, &[("3D/external.model", EXTERNAL_OBJECT_XML)]);
    let config = ParserConfig::new().with_extension(Extension::Production);
    let result = Model::from_reader_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_ok(),
        "Expected valid external build item to succeed, got: {:?}",
        result.err()
    );
}

#[test]
fn test_production_build_item_nonexistent_external_file() {
    // Build item references a non-existent external file
    let root_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US"
    xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
    xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06"
    requiredextensions="p">
    <resources/>
    <build p:UUID="00000000-0000-0000-0000-000000000001">
        <item objectid="5" p:path="/3D/nonexistent.model" p:UUID="00000000-0000-0000-0000-000000000002"/>
    </build>
</model>"#;

    let buffer = create_production_3mf(root_xml, &[]);
    let config = ParserConfig::new().with_extension(Extension::Production);
    let result = Model::from_reader_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_err(),
        "Expected error for non-existent external file in build item"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("nonexistent") || msg.contains("external"),
        "Error should mention the missing file, got: {}",
        msg
    );
}

#[test]
fn test_production_build_item_nonexistent_object_in_external_file() {
    // Build item references a valid file but non-existent object ID
    let root_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US"
    xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
    xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06"
    requiredextensions="p">
    <resources/>
    <build p:UUID="00000000-0000-0000-0000-000000000001">
        <item objectid="999" p:path="/3D/external.model" p:UUID="00000000-0000-0000-0000-000000000002"/>
    </build>
</model>"#;

    let buffer = create_production_3mf(root_xml, &[("3D/external.model", EXTERNAL_OBJECT_XML)]);
    let config = ParserConfig::new().with_extension(Extension::Production);
    let result = Model::from_reader_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_err(),
        "Expected error for non-existent object ID in external file"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("999") || msg.contains("object"),
        "Error should mention the missing object ID, got: {}",
        msg
    );
}

#[test]
fn test_production_component_external_path_valid() {
    // Component references an object in an external file
    let root_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US"
    xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
    xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06"
    requiredextensions="p">
    <resources>
        <object id="1" type="model" p:UUID="00000000-0000-0000-0000-000000000003">
            <components>
                <component objectid="5" p:path="/3D/external.model" p:UUID="00000000-0000-0000-0000-000000000004"/>
            </components>
        </object>
    </resources>
    <build p:UUID="00000000-0000-0000-0000-000000000001">
        <item objectid="1" p:UUID="00000000-0000-0000-0000-000000000002"/>
    </build>
</model>"#;

    let buffer = create_production_3mf(root_xml, &[("3D/external.model", EXTERNAL_OBJECT_XML)]);
    let config = ParserConfig::new().with_extension(Extension::Production);
    let result = Model::from_reader_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_ok(),
        "Expected valid external component to succeed, got: {:?}",
        result.err()
    );
}

#[test]
fn test_production_component_nonexistent_external_file() {
    // Component references a non-existent external file
    let root_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US"
    xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
    xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06"
    requiredextensions="p">
    <resources>
        <object id="1" type="model" p:UUID="00000000-0000-0000-0000-000000000003">
            <components>
                <component objectid="5" p:path="/3D/nonexistent.model" p:UUID="00000000-0000-0000-0000-000000000004"/>
            </components>
        </object>
    </resources>
    <build p:UUID="00000000-0000-0000-0000-000000000001">
        <item objectid="1" p:UUID="00000000-0000-0000-0000-000000000002"/>
    </build>
</model>"#;

    let buffer = create_production_3mf(root_xml, &[]);
    let config = ParserConfig::new().with_extension(Extension::Production);
    let result = Model::from_reader_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_err(),
        "Expected error for non-existent external file in component"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("nonexistent") || msg.contains("external"),
        "Error should mention the missing file, got: {}",
        msg
    );
}

#[test]
fn test_production_component_nonexistent_object_in_external_file() {
    // Component references a valid file but non-existent object ID
    let root_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US"
    xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
    xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06"
    requiredextensions="p">
    <resources>
        <object id="1" type="model" p:UUID="00000000-0000-0000-0000-000000000003">
            <components>
                <component objectid="999" p:path="/3D/external.model" p:UUID="00000000-0000-0000-0000-000000000004"/>
            </components>
        </object>
    </resources>
    <build p:UUID="00000000-0000-0000-0000-000000000001">
        <item objectid="1" p:UUID="00000000-0000-0000-0000-000000000002"/>
    </build>
</model>"#;

    let buffer = create_production_3mf(root_xml, &[("3D/external.model", EXTERNAL_OBJECT_XML)]);
    let config = ParserConfig::new().with_extension(Extension::Production);
    let result = Model::from_reader_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_err(),
        "Expected error for non-existent object ID in external file"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("999") || msg.contains("object"),
        "Error should mention the missing object ID, got: {}",
        msg
    );
}

#[test]
fn test_production_uuid_in_build_item() {
    // Build item with UUID attribute (p:UUID)
    let root_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US"
    xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
    xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06"
    requiredextensions="p">
    <resources>
        <object id="1" type="model" p:UUID="00000000-0000-0000-0000-000000000003">
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
    </resources>
    <build p:UUID="12345678-1234-1234-1234-123456789abc">
        <item objectid="1" p:UUID="12345678-1234-1234-1234-123456789def"/>
    </build>
</model>"#;

    let buffer = create_production_3mf(root_xml, &[]);
    let config = ParserConfig::new().with_extension(Extension::Production);
    let result = Model::from_reader_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_ok(),
        "Expected build item with UUID to succeed, got: {:?}",
        result.err()
    );
}

#[test]
fn test_production_external_reference_cached() {
    // Two components referencing the same external file should use the cache
    // (validated once, not twice)
    let root_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US"
    xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
    xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06"
    requiredextensions="p">
    <resources>
        <object id="1" type="model" p:UUID="00000000-0000-0000-0000-000000000003">
            <components>
                <component objectid="5" p:path="/3D/external.model" p:UUID="00000000-0000-0000-0000-000000000004"/>
                <component objectid="5" p:path="/3D/external.model" p:UUID="00000000-0000-0000-0000-000000000005"/>
            </components>
        </object>
    </resources>
    <build p:UUID="00000000-0000-0000-0000-000000000001">
        <item objectid="1" p:UUID="00000000-0000-0000-0000-000000000002"/>
    </build>
</model>"#;

    let buffer = create_production_3mf(root_xml, &[("3D/external.model", EXTERNAL_OBJECT_XML)]);
    let config = ParserConfig::new().with_extension(Extension::Production);
    let result = Model::from_reader_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_ok(),
        "Expected duplicate external references to be cached and succeed, got: {:?}",
        result.err()
    );
}
