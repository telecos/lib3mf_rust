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
