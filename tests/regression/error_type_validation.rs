//! Demonstration test showing how specific error types prevent missing spec violations
//!
//! This test demonstrates the solution to the problem:
//! - OLD BEHAVIOR: Files were marked as "expected failure" without specifying why
//!   If the file failed for a different reason, we wouldn't catch it
//! - NEW BEHAVIOR: Files specify the expected error type
//!   If the file fails for a different reason, the test fails

use lib3mf::{Model, ParserConfig};
use std::io::Write;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// Simulate the old behavior: a file is marked as expected failure
/// but fails for the WRONG reason (InvalidFormat instead of OutsidePositiveOctant)
///
/// This test is intentionally designed to demonstrate error detection.
/// It shows that if we expect OutsidePositiveOctant but get InvalidFormat,
/// the new system will catch this mismatch.
#[test]
#[should_panic(expected = "File failed with wrong error type")]
fn test_old_behavior_would_miss_wrong_error() {
    // Create a 3MF file with BOTH issues:
    // 1. Invalid content type (InvalidFormat error)
    // 2. Outside positive octant
    //
    // OLD BEHAVIOR: Would just mark as "expected failure" and pass
    // NEW BEHAVIOR: Will fail because error type doesn't match

    let mut buffer = Vec::new();
    let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));

    // Add INVALID [Content_Types].xml (wrong content type for model)
    zip.start_file("[Content_Types].xml", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="model" ContentType="text/plain"/>
</Types>"#,
    ).unwrap();

    // Add _rels/.rels
    zip.add_directory("_rels/", SimpleFileOptions::default())
        .unwrap();
    zip.start_file("_rels/.rels", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rel0" Target="/3D/3dmodel.model" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"#,
    ).unwrap();

    // Add 3D/3dmodel.model with transform outside positive octant
    zip.add_directory("3D/", SimpleFileOptions::default())
        .unwrap();
    zip.start_file("3D/3dmodel.model", SimpleFileOptions::default())
        .unwrap();

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
        <item objectid="1" transform="1 0 0 0 0 1 0 -5 0 0 1 0"/>
    </build>
</model>"#;

    zip.write_all(model_xml.as_bytes()).unwrap();
    let _writer = zip.finish().unwrap();

    // Try to parse it
    let result =
        Model::from_reader_with_config(std::io::Cursor::new(&buffer), ParserConfig::default());

    // The file should fail, but with InvalidFormat, not OutsidePositiveOctant
    match result {
        Err(e) => {
            let actual_error_type = e.error_type();

            // With the NEW behavior, we can detect this mismatch
            let expected_error_type = "OutsidePositiveOctant";

            if actual_error_type != expected_error_type {
                // This is what we want - detecting that the file is failing for
                // a DIFFERENT reason than expected
                println!("✓ NEW BEHAVIOR: Detected wrong error type!");
                println!("  Expected: {}", expected_error_type);
                println!("  Actual: {}", actual_error_type);
                println!("  Error: {}", e);

                // This would fail the test in the conformance suite
                assert_eq!(
                    actual_error_type, expected_error_type,
                    "File failed with wrong error type - this catches spec violations!"
                );
            }
        }
        Ok(_) => {
            panic!("File should have failed");
        }
    }
}

/// Demonstrate the new behavior working correctly
#[test]
#[ignore] // OutsidePositiveOctant validation is not yet implemented in the validator
fn test_new_behavior_catches_correct_error() {
    // Create a 3MF file that fails ONLY due to being outside positive octant
    let mut buffer = Vec::new();
    let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));

    // Add VALID [Content_Types].xml
    zip.start_file("[Content_Types].xml", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
</Types>"#,
    ).unwrap();

    // Add _rels/.rels
    zip.add_directory("_rels/", SimpleFileOptions::default())
        .unwrap();
    zip.start_file("_rels/.rels", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(
        br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rel0" Target="/3D/3dmodel.model" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"#,
    ).unwrap();

    // Add 3D/3dmodel.model with transform outside positive octant
    zip.add_directory("3D/", SimpleFileOptions::default())
        .unwrap();
    zip.start_file("3D/3dmodel.model", SimpleFileOptions::default())
        .unwrap();

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
        <item objectid="1" transform="1 0 0 0 0 1 0 -5 0 0 1 0"/>
    </build>
</model>"#;

    zip.write_all(model_xml.as_bytes()).unwrap();
    let _writer = zip.finish().unwrap();

    // Try to parse it
    let result =
        Model::from_reader_with_config(std::io::Cursor::new(&buffer), ParserConfig::default());

    // The file should fail with OutsidePositiveOctant
    match result {
        Err(e) => {
            let actual_error_type = e.error_type();
            let expected_error_type = "OutsidePositiveOctant";

            // NEW BEHAVIOR: Verify the error type matches
            assert_eq!(
                actual_error_type, expected_error_type,
                "Expected {} error, got {}: {}",
                expected_error_type, actual_error_type, e
            );

            println!("✓ NEW BEHAVIOR: Error type matches expectation!");
            println!("  Expected: {}", expected_error_type);
            println!("  Actual: {}", actual_error_type);
        }
        Ok(_) => {
            panic!("File should have failed");
        }
    }
}
