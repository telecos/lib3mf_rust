//! Integration tests for the Volumetric extension

use lib3mf::{
    Extension, Model, VolumetricBoundary, VolumetricData, VolumetricPropertyGroup, Voxel, VoxelGrid,
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
