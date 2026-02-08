//! Integration tests for the Volumetric extension

use lib3mf::{
    Extension, ImplicitVolume, Model, VolumetricBoundary, VolumetricData, VolumetricProperty,
    VolumetricPropertyGroup, Voxel, VoxelGrid,
};

#[test]
fn test_volumetric_extension_basic() {
    let mut model = Model::new();
    model.required_extensions.push(Extension::Volumetric);

    // Add volumetric data
    let mut vol_data = VolumetricData::new(1);
    vol_data.boundary = Some(VolumetricBoundary::new(
        (0.0, 0.0, 0.0),
        (100.0, 100.0, 100.0),
    ));

    let mut grid = VoxelGrid::new((10, 10, 10));
    grid.spacing = Some((1.0, 1.0, 1.0));
    grid.origin = Some((0.0, 0.0, 0.0));
    grid.voxels.push(Voxel::new((5, 5, 5)));

    vol_data.voxels = Some(grid);
    model.resources.volumetric_data.push(vol_data);

    // Validate model
    let result = lib3mf::validator::validate_volumetric_extension(&model);
    assert!(
        result.is_ok(),
        "Volumetric extension validation failed: {:?}",
        result.err()
    );
}

#[test]
fn test_volumetric_extension_with_properties() {
    let mut model = Model::new();
    model.required_extensions.push(Extension::Volumetric);

    // Add property group
    let prop_group = VolumetricPropertyGroup::new(1);
    model.resources.volumetric_property_groups.push(prop_group);

    // Add volumetric data with voxels referencing properties
    let mut vol_data = VolumetricData::new(2);
    let mut grid = VoxelGrid::new((5, 5, 5));
    let mut voxel = Voxel::new((2, 2, 2));
    voxel.property_id = Some(1); // Reference the property group
    grid.voxels.push(voxel);
    vol_data.voxels = Some(grid);

    model.resources.volumetric_data.push(vol_data);

    // Validate model
    let result = lib3mf::validator::validate_volumetric_extension(&model);
    assert!(
        result.is_ok(),
        "Volumetric extension validation failed: {:?}",
        result.err()
    );
}

#[test]
fn test_volumetric_extension_handler() {
    use lib3mf::extension::{ExtensionHandler, ExtensionRegistry};
    use lib3mf::extensions::VolumetricExtensionHandler;
    use std::sync::Arc;

    let handler = VolumetricExtensionHandler;

    // Test basic handler properties
    assert_eq!(handler.extension_type(), Extension::Volumetric);
    assert_eq!(handler.name(), "Volumetric");
    assert_eq!(
        handler.namespace(),
        "http://schemas.3mf.io/volumetric/2023/02"
    );

    // Test is_used_in_model
    let mut model = Model::new();
    assert!(!handler.is_used_in_model(&model));

    model.resources.volumetric_data.push(VolumetricData::new(1));
    assert!(handler.is_used_in_model(&model));

    // Test registration
    let mut registry = ExtensionRegistry::new();
    registry.register(Arc::new(handler));
    assert_eq!(registry.handlers().len(), 1);
    assert!(registry.get_handler(Extension::Volumetric).is_some());
}

#[test]
fn test_volumetric_in_default_registry() {
    use lib3mf::extensions::create_default_registry;

    let registry = create_default_registry();

    // Verify volumetric handler is in the default registry
    assert!(registry.get_handler(Extension::Volumetric).is_some());
}

#[test]
fn test_volumetric_parser_config() {
    use lib3mf::ParserConfig;

    // Test with_all_extensions includes volumetric
    let config = ParserConfig::with_all_extensions();
    assert!(
        config
            .registry()
            .get_handler(Extension::Volumetric)
            .is_some()
    );

    // Test with_extension for volumetric
    let config = ParserConfig::new().with_extension(Extension::Volumetric);
    assert!(
        config
            .registry()
            .get_handler(Extension::Volumetric)
            .is_some()
    );
}

#[test]
fn test_volumetric_namespace_parsing() {
    // Test that the namespace is correctly parsed
    let extension = Extension::from_namespace("http://schemas.3mf.io/volumetric/2023/02");
    assert_eq!(extension, Some(Extension::Volumetric));
}

// ===========================================================================
// Parsing tests – verify volumetric data is extracted from .3mf files
// ===========================================================================

#[test]
fn test_parse_simple_volumetric_file() {
    let file = std::fs::File::open("test_files/volumetric/simple_volumetric.3mf")
        .expect("test file should exist");
    let model = Model::from_reader(file).expect("should parse successfully");

    // Volumetric data should be populated
    assert_eq!(model.resources.volumetric_data.len(), 1);
    let vol = &model.resources.volumetric_data[0];
    assert_eq!(vol.id, 1);

    // Boundary
    let boundary = vol.boundary.as_ref().expect("boundary should be present");
    assert_eq!(boundary.min, (0.0, 0.0, 0.0));
    assert_eq!(boundary.max, (10.0, 10.0, 10.0));

    // Voxel grid
    let grid = vol.voxels.as_ref().expect("voxels should be present");
    assert_eq!(grid.dimensions, (10, 10, 10));
    assert_eq!(grid.spacing, Some((1.0, 1.0, 1.0)));
    assert_eq!(grid.voxels.len(), 3);
    assert_eq!(grid.voxels[0].position, (5, 5, 5));
    assert_eq!(grid.voxels[1].position, (6, 5, 5));
    assert_eq!(grid.voxels[2].position, (5, 6, 5));
}

#[test]
fn test_parse_volumetric_with_properties_file() {
    let file = std::fs::File::open("test_files/volumetric/volumetric_with_properties.3mf")
        .expect("test file should exist");
    let model = Model::from_reader(file).expect("should parse successfully");

    // Property groups
    assert_eq!(model.resources.volumetric_property_groups.len(), 1);
    let group = &model.resources.volumetric_property_groups[0];
    assert_eq!(group.id, 1);
    assert_eq!(group.properties.len(), 1);
    assert_eq!(group.properties[0].index, 0);
    assert_eq!(group.properties[0].value, "solid");

    // Volumetric data
    assert_eq!(model.resources.volumetric_data.len(), 1);
    let vol = &model.resources.volumetric_data[0];
    assert_eq!(vol.id, 2);

    let grid = vol.voxels.as_ref().expect("voxels should be present");
    assert_eq!(grid.dimensions, (5, 5, 5));
    assert_eq!(grid.voxels.len(), 1);
    assert_eq!(grid.voxels[0].position, (2, 2, 2));
    assert_eq!(grid.voxels[0].property_id, Some(1));
}

// ===========================================================================
// Write + round-trip tests
// ===========================================================================

#[test]
fn test_write_volumetric_model() {
    let mut model = Model::new();
    model.required_extensions.push(Extension::Volumetric);

    // Add a property group
    let mut prop_group = VolumetricPropertyGroup::new(1);
    prop_group
        .properties
        .push(VolumetricProperty::new(0, "density".to_string()));
    model.resources.volumetric_property_groups.push(prop_group);

    // Add volumetric data
    let mut vol_data = VolumetricData::new(2);
    vol_data.boundary = Some(VolumetricBoundary::new(
        (0.0, 0.0, 0.0),
        (50.0, 50.0, 50.0),
    ));
    let mut grid = VoxelGrid::new((5, 5, 5));
    grid.spacing = Some((10.0, 10.0, 10.0));
    let mut voxel = Voxel::new((1, 2, 3));
    voxel.property_id = Some(1);
    grid.voxels.push(voxel);
    vol_data.voxels = Some(grid);
    model.resources.volumetric_data.push(vol_data);

    // Add a minimal mesh object so the model is valid
    let mut mesh = lib3mf::Mesh::new();
    mesh.vertices.push(lib3mf::Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(lib3mf::Vertex::new(10.0, 0.0, 0.0));
    mesh.vertices.push(lib3mf::Vertex::new(5.0, 10.0, 0.0));
    mesh.triangles.push(lib3mf::Triangle::new(0, 1, 2));
    let mut obj = lib3mf::Object::new(3);
    obj.mesh = Some(mesh);
    model.resources.objects.push(obj);
    model.build.items.push(lib3mf::BuildItem::new(3));

    // Write to XML and re-parse to verify
    let mut out_buf = Vec::new();
    let cursor = std::io::Cursor::new(&mut out_buf);
    model.to_writer(cursor).expect("write should succeed");

    // Re-parse and verify volumetric data survived round-trip
    let model2 = Model::from_reader(std::io::Cursor::new(&out_buf)).expect("re-parse should succeed");

    // Verify key data is present
    assert_eq!(model2.resources.volumetric_property_groups.len(), 1);
    assert_eq!(model2.resources.volumetric_data.len(), 1);
    assert!(model2.required_extensions.contains(&Extension::Volumetric));
}

#[test]
fn test_roundtrip_simple_volumetric() {
    // Parse the test file
    let file = std::fs::File::open("test_files/volumetric/simple_volumetric.3mf")
        .expect("test file should exist");
    let model = Model::from_reader(file).expect("should parse successfully");

    // Write to a 3MF package in memory
    let mut out_buf = Vec::new();
    model
        .clone()
        .to_writer(std::io::Cursor::new(&mut out_buf))
        .expect("write should succeed");

    // Re-parse the written output
    let model2 =
        Model::from_reader(std::io::Cursor::new(&out_buf)).expect("re-parse should succeed");

    // Compare volumetric data
    assert_eq!(
        model.resources.volumetric_data.len(),
        model2.resources.volumetric_data.len()
    );
    let v1 = &model.resources.volumetric_data[0];
    let v2 = &model2.resources.volumetric_data[0];
    assert_eq!(v1.id, v2.id);

    let b1 = v1.boundary.as_ref().unwrap();
    let b2 = v2.boundary.as_ref().unwrap();
    assert_eq!(b1.min, b2.min);
    assert_eq!(b1.max, b2.max);

    let g1 = v1.voxels.as_ref().unwrap();
    let g2 = v2.voxels.as_ref().unwrap();
    assert_eq!(g1.dimensions, g2.dimensions);
    assert_eq!(g1.voxels.len(), g2.voxels.len());
    for (vx1, vx2) in g1.voxels.iter().zip(g2.voxels.iter()) {
        assert_eq!(vx1.position, vx2.position);
    }
}

#[test]
fn test_roundtrip_volumetric_with_properties() {
    // Parse the test file
    let file = std::fs::File::open("test_files/volumetric/volumetric_with_properties.3mf")
        .expect("test file should exist");
    let model = Model::from_reader(file).expect("should parse successfully");

    // Write to a 3MF package in memory
    let mut out_buf = Vec::new();
    model
        .clone()
        .to_writer(std::io::Cursor::new(&mut out_buf))
        .expect("write should succeed");

    // Re-parse the written output
    let model2 =
        Model::from_reader(std::io::Cursor::new(&out_buf)).expect("re-parse should succeed");

    // Compare property groups
    assert_eq!(
        model.resources.volumetric_property_groups.len(),
        model2.resources.volumetric_property_groups.len()
    );
    let pg1 = &model.resources.volumetric_property_groups[0];
    let pg2 = &model2.resources.volumetric_property_groups[0];
    assert_eq!(pg1.id, pg2.id);
    assert_eq!(pg1.properties.len(), pg2.properties.len());
    assert_eq!(pg1.properties[0].index, pg2.properties[0].index);
    assert_eq!(pg1.properties[0].value, pg2.properties[0].value);

    // Compare volumetric data
    let v1 = &model.resources.volumetric_data[0];
    let v2 = &model2.resources.volumetric_data[0];
    assert_eq!(v1.id, v2.id);

    let g1 = v1.voxels.as_ref().unwrap();
    let g2 = v2.voxels.as_ref().unwrap();
    assert_eq!(g1.dimensions, g2.dimensions);
    assert_eq!(g1.voxels[0].position, g2.voxels[0].position);
    assert_eq!(g1.voxels[0].property_id, g2.voxels[0].property_id);
}

#[test]
fn test_write_volumetric_with_implicit() {
    let mut model = Model::new();
    model.required_extensions.push(Extension::Volumetric);

    let mut vol_data = VolumetricData::new(1);
    let mut implicit = ImplicitVolume::new("sdf".to_string());
    implicit
        .parameters
        .push(("radius".to_string(), "5.0".to_string()));
    vol_data.implicit = Some(implicit);
    model.resources.volumetric_data.push(vol_data);

    // Add a minimal mesh object
    let mut mesh = lib3mf::Mesh::new();
    mesh.vertices.push(lib3mf::Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(lib3mf::Vertex::new(10.0, 0.0, 0.0));
    mesh.vertices.push(lib3mf::Vertex::new(5.0, 10.0, 0.0));
    mesh.triangles.push(lib3mf::Triangle::new(0, 1, 2));
    let mut obj = lib3mf::Object::new(2);
    obj.mesh = Some(mesh);
    model.resources.objects.push(obj);
    model.build.items.push(lib3mf::BuildItem::new(2));

    // Write to 3MF and re-parse
    let mut out_buf = Vec::new();
    model.to_writer(std::io::Cursor::new(&mut out_buf)).expect("write should succeed");

    let model2 = Model::from_reader(std::io::Cursor::new(&out_buf)).expect("re-parse should succeed");

    // Verify implicit volume survived
    assert_eq!(model2.resources.volumetric_data.len(), 1);
    let vol = &model2.resources.volumetric_data[0];
    let implicit2 = vol.implicit.as_ref().expect("implicit should be present");
    assert_eq!(implicit2.implicit_type, "sdf");
    assert!(implicit2.parameters.iter().any(|(k, v)| k == "radius" && v == "5.0"));
}
