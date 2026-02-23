#[cfg(test)]
mod boolean_operations_test {
    use lib3mf::BooleanOpType;

    #[test]
    fn test_parse_boolean_union() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:bo="http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07" unit="millimeter" requiredextensions="bo">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0"/>
                    <vertex x="10" y="0" z="0"/>
                    <vertex x="5" y="10" z="0"/>
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2"/>
                </triangles>
            </mesh>
        </object>
        <object id="2" type="model">
            <mesh>
                <vertices>
                    <vertex x="5" y="0" z="0"/>
                    <vertex x="15" y="0" z="0"/>
                    <vertex x="10" y="10" z="0"/>
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2"/>
                </triangles>
            </mesh>
        </object>
        <object id="3" type="model">
            <bo:booleanshape objectid="1" operation="union">
                <bo:boolean objectid="2"/>
            </bo:booleanshape>
        </object>
    </resources>
    <build>
        <item objectid="3"/>
    </build>
</model>"#;

        let model = lib3mf::parser::parse_model_xml(xml).expect("Failed to parse model");

        // Verify the model was parsed
        assert_eq!(model.resources.objects.len(), 3);

        // Find the boolean object
        let boolean_obj = model.resources.objects.iter().find(|o| o.id == 3).unwrap();

        // Verify boolean shape exists
        assert!(boolean_obj.boolean_shape.is_some());

        let shape = boolean_obj.boolean_shape.as_ref().unwrap();
        assert_eq!(shape.objectid, 1);
        assert_eq!(shape.operation, BooleanOpType::Union);
        assert_eq!(shape.operands.len(), 1);
        assert_eq!(shape.operands[0].objectid, 2);
    }

    #[test]
    fn test_parse_boolean_difference() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:bo="http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07" unit="millimeter" requiredextensions="bo">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0"/>
                    <vertex x="10" y="0" z="0"/>
                    <vertex x="5" y="10" z="0"/>
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2"/>
                </triangles>
            </mesh>
        </object>
        <object id="2" type="model">
            <mesh>
                <vertices>
                    <vertex x="5" y="0" z="0"/>
                    <vertex x="15" y="0" z="0"/>
                    <vertex x="10" y="10" z="0"/>
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2"/>
                </triangles>
            </mesh>
        </object>
        <object id="3" type="model">
            <bo:booleanshape objectid="1" operation="difference">
                <bo:boolean objectid="2"/>
            </bo:booleanshape>
        </object>
    </resources>
    <build>
        <item objectid="3"/>
    </build>
</model>"#;

        let model = lib3mf::parser::parse_model_xml(xml).expect("Failed to parse model");

        let boolean_obj = model.resources.objects.iter().find(|o| o.id == 3).unwrap();
        let shape = boolean_obj.boolean_shape.as_ref().unwrap();

        assert_eq!(shape.operation, BooleanOpType::Difference);
    }

    #[test]
    fn test_parse_boolean_intersection() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:bo="http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07" unit="millimeter" requiredextensions="bo">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0"/>
                    <vertex x="10" y="0" z="0"/>
                    <vertex x="5" y="10" z="0"/>
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2"/>
                </triangles>
            </mesh>
        </object>
        <object id="2" type="model">
            <mesh>
                <vertices>
                    <vertex x="5" y="0" z="0"/>
                    <vertex x="15" y="0" z="0"/>
                    <vertex x="10" y="10" z="0"/>
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2"/>
                </triangles>
            </mesh>
        </object>
        <object id="3" type="model">
            <bo:booleanshape objectid="1" operation="intersection">
                <bo:boolean objectid="2"/>
            </bo:booleanshape>
        </object>
    </resources>
    <build>
        <item objectid="3"/>
    </build>
</model>"#;

        let model = lib3mf::parser::parse_model_xml(xml).expect("Failed to parse model");

        let boolean_obj = model.resources.objects.iter().find(|o| o.id == 3).unwrap();
        let shape = boolean_obj.boolean_shape.as_ref().unwrap();

        assert_eq!(shape.operation, BooleanOpType::Intersection);
    }

    #[test]
    fn test_parse_boolean_default_operation() {
        // Operation attribute is optional and defaults to union
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:bo="http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07" unit="millimeter" requiredextensions="bo">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0"/>
                    <vertex x="10" y="0" z="0"/>
                    <vertex x="5" y="10" z="0"/>
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2"/>
                </triangles>
            </mesh>
        </object>
        <object id="2" type="model">
            <mesh>
                <vertices>
                    <vertex x="5" y="0" z="0"/>
                    <vertex x="15" y="0" z="0"/>
                    <vertex x="10" y="10" z="0"/>
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2"/>
                </triangles>
            </mesh>
        </object>
        <object id="3" type="model">
            <bo:booleanshape objectid="1">
                <bo:boolean objectid="2"/>
            </bo:booleanshape>
        </object>
    </resources>
    <build>
        <item objectid="3"/>
    </build>
</model>"#;

        let model = lib3mf::parser::parse_model_xml(xml).expect("Failed to parse model");

        let boolean_obj = model.resources.objects.iter().find(|o| o.id == 3).unwrap();
        let shape = boolean_obj.boolean_shape.as_ref().unwrap();

        // Should default to union
        assert_eq!(shape.operation, BooleanOpType::Union);
    }
}

#[test]
fn test_boolean_ops_writer_round_trip() {
    use lib3mf::model::{BooleanOpType, BooleanRef, BooleanShape, Extension};
    use lib3mf::{BuildItem, Mesh, Model, Object, Triangle, Vertex};
    use std::io::Cursor;

    let mut model = Model::new();
    model.required_extensions.push(Extension::BooleanOperations);

    // Base mesh object
    let mut mesh1 = Mesh::new();
    mesh1.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh1.vertices.push(Vertex::new(10.0, 0.0, 0.0));
    mesh1.vertices.push(Vertex::new(0.0, 10.0, 0.0));
    mesh1.triangles.push(Triangle::new(0, 1, 2));
    let mut obj1 = Object::new(1);
    obj1.mesh = Some(mesh1);
    model.resources.objects.push(obj1);

    // Operand mesh object
    let mut mesh2 = Mesh::new();
    mesh2.vertices.push(Vertex::new(5.0, 0.0, 0.0));
    mesh2.vertices.push(Vertex::new(15.0, 0.0, 0.0));
    mesh2.vertices.push(Vertex::new(5.0, 10.0, 0.0));
    mesh2.triangles.push(Triangle::new(0, 1, 2));
    let mut obj2 = Object::new(2);
    obj2.mesh = Some(mesh2);
    model.resources.objects.push(obj2);

    // Boolean union object
    let mut boolean_shape = BooleanShape::new(1, BooleanOpType::Union);
    boolean_shape.operands.push(BooleanRef::new(2));
    let mut obj3 = Object::new(3);
    obj3.boolean_shape = Some(boolean_shape);
    model.resources.objects.push(obj3);

    model.build.items.push(BuildItem::new(3));

    let buffer = Vec::new();
    let cursor = Cursor::new(buffer);
    let result = model.to_writer(cursor);
    assert!(result.is_ok(), "Failed to write boolean ops model");

    let cursor = result.unwrap();
    let config = lib3mf::ParserConfig::new().with_extension(Extension::BooleanOperations);
    let parsed = Model::from_reader_with_config(Cursor::new(cursor.into_inner()), config);
    assert!(
        parsed.is_ok(),
        "Failed to parse written boolean ops model: {:?}",
        parsed.err()
    );
    let parsed = parsed.unwrap();
    let obj3 = parsed.resources.objects.iter().find(|o| o.id == 3).unwrap();
    let shape = obj3.boolean_shape.as_ref().unwrap();
    assert_eq!(shape.operation, BooleanOpType::Union);
    assert_eq!(shape.operands.len(), 1);
}

#[test]
fn test_boolean_ops_invalid_operation_defaults_to_union() {
    use lib3mf::BooleanOpType;

    // Per parser implementation, invalid operations default to Union
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:bo="http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07"
       unit="millimeter" requiredextensions="bo">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0"/>
                    <vertex x="10" y="0" z="0"/>
                    <vertex x="5" y="10" z="0"/>
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2"/>
                </triangles>
            </mesh>
        </object>
        <object id="2" type="model">
            <bo:booleanshape objectid="1" operation="invalid_op">
                <bo:boolean objectid="1"/>
            </bo:booleanshape>
        </object>
    </resources>
    <build>
        <item objectid="2"/>
    </build>
</model>"#;

    let result = lib3mf::parser::parse_model_xml(xml);
    // Invalid operation defaults to Union per spec (unknown operations treated as union)
    assert!(
        result.is_ok(),
        "Expected invalid operation to default to union, got: {:?}",
        result.err()
    );
    let model = result.unwrap();
    let obj = model.resources.objects.iter().find(|o| o.id == 2).unwrap();
    let shape = obj.boolean_shape.as_ref().unwrap();
    assert_eq!(
        shape.operation,
        BooleanOpType::Union,
        "Invalid operation should default to union"
    );
}

#[test]
fn test_boolean_ops_missing_objectid() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:bo="http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07"
       unit="millimeter" requiredextensions="bo">
    <resources>
        <object id="2" type="model">
            <bo:booleanshape operation="union">
                <bo:boolean objectid="1"/>
            </bo:booleanshape>
        </object>
    </resources>
    <build>
        <item objectid="2"/>
    </build>
</model>"#;

    let result = lib3mf::parser::parse_model_xml(xml);
    // booleanshape missing objectid should fail
    assert!(
        result.is_err(),
        "Expected error for booleanshape missing objectid"
    );
}

#[test]
fn test_boolean_ops_multiple_operands() {
    use lib3mf::BooleanOpType;

    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:bo="http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07"
       unit="millimeter" requiredextensions="bo">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0"/>
                    <vertex x="10" y="0" z="0"/>
                    <vertex x="5" y="10" z="0"/>
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2"/>
                </triangles>
            </mesh>
        </object>
        <object id="2" type="model">
            <mesh>
                <vertices>
                    <vertex x="5" y="0" z="0"/>
                    <vertex x="15" y="0" z="0"/>
                    <vertex x="10" y="10" z="0"/>
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2"/>
                </triangles>
            </mesh>
        </object>
        <object id="3" type="model">
            <mesh>
                <vertices>
                    <vertex x="2" y="0" z="0"/>
                    <vertex x="8" y="0" z="0"/>
                    <vertex x="5" y="8" z="0"/>
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2"/>
                </triangles>
            </mesh>
        </object>
        <object id="4" type="model">
            <bo:booleanshape objectid="1" operation="union">
                <bo:boolean objectid="2"/>
                <bo:boolean objectid="3"/>
            </bo:booleanshape>
        </object>
    </resources>
    <build>
        <item objectid="4"/>
    </build>
</model>"#;

    let result = lib3mf::parser::parse_model_xml(xml);
    assert!(
        result.is_ok(),
        "Expected multiple operands to parse successfully, got: {:?}",
        result.err()
    );
    let model = result.unwrap();
    let obj4 = model.resources.objects.iter().find(|o| o.id == 4).unwrap();
    let shape = obj4.boolean_shape.as_ref().unwrap();
    assert_eq!(shape.operation, BooleanOpType::Union);
    assert_eq!(shape.operands.len(), 2);
}

/// Helper: create a minimal 3MF ZIP with multiple model files for external reference testing
fn create_3mf_with_external_model(
    root_model_xml: &str,
    external_model_xml: Option<(&str, &str)>,
) -> Vec<u8> {
    use std::io::Write;
    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

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

    if let Some((path, xml)) = external_model_xml {
        // path should be relative, e.g. "3D/external.model"
        zip.start_file(path, SimpleFileOptions::default()).unwrap();
        zip.write_all(xml.as_bytes()).unwrap();
    }

    zip.finish().unwrap();
    buffer
}

#[test]
fn test_boolean_ops_external_path_nonexistent_file() {
    use lib3mf::{Extension, Model, ParserConfig};

    // Boolean shape references a non-existent external file
    let root_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:bo="http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07"
       unit="millimeter" requiredextensions="bo">
    <resources>
        <object id="1" type="model">
            <bo:booleanshape objectid="10" path="/3D/nonexistent.model" operation="union">
                <bo:boolean objectid="20"/>
            </bo:booleanshape>
        </object>
    </resources>
    <build>
        <item objectid="1"/>
    </build>
</model>"#;

    let buffer = create_3mf_with_external_model(root_xml, None);
    let config = ParserConfig::new().with_extension(Extension::BooleanOperations);
    let result = Model::from_reader_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_err(),
        "Expected error for non-existent external file in boolean shape"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("nonexistent") || msg.contains("external"),
        "Error should mention the missing file, got: {}",
        msg
    );
}

#[test]
fn test_boolean_ops_external_path_nonexistent_object() {
    use lib3mf::{Extension, Model, ParserConfig};

    let external_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" unit="millimeter">
    <resources>
        <object id="5" type="model">
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
    <build/>
</model>"#;

    // References object id=99 which doesn't exist in the external file (only id=5 exists)
    let root_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:bo="http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07"
       unit="millimeter" requiredextensions="bo">
    <resources>
        <object id="1" type="model">
            <bo:booleanshape objectid="99" path="/3D/external.model" operation="union">
                <bo:boolean objectid="5"/>
            </bo:booleanshape>
        </object>
    </resources>
    <build>
        <item objectid="1"/>
    </build>
</model>"#;

    let buffer =
        create_3mf_with_external_model(root_xml, Some(("3D/external.model", external_xml)));
    let config = ParserConfig::new().with_extension(Extension::BooleanOperations);
    let result = Model::from_reader_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_err(),
        "Expected error for non-existent object in external file"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("99") || msg.contains("object"),
        "Error should mention the missing object ID, got: {}",
        msg
    );
}

#[test]
fn test_boolean_ops_operand_external_path_nonexistent_file() {
    use lib3mf::{Extension, Model, ParserConfig};

    // Boolean operand references a non-existent external file
    let external_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" unit="millimeter">
    <resources>
        <object id="5" type="model">
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
    <build/>
</model>"#;

    let root_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:bo="http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07"
       unit="millimeter" requiredextensions="bo">
    <resources>
        <object id="1" type="model">
            <bo:booleanshape objectid="5" path="/3D/external.model" operation="union">
                <bo:boolean objectid="10" path="/3D/missing.model"/>
            </bo:booleanshape>
        </object>
    </resources>
    <build>
        <item objectid="1"/>
    </build>
</model>"#;

    let buffer =
        create_3mf_with_external_model(root_xml, Some(("3D/external.model", external_xml)));
    let config = ParserConfig::new().with_extension(Extension::BooleanOperations);
    let result = Model::from_reader_with_config(std::io::Cursor::new(&buffer), config);
    assert!(
        result.is_err(),
        "Expected error for non-existent external file in boolean operand"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("missing") || msg.contains("external"),
        "Error should mention the missing file, got: {}",
        msg
    );
}
