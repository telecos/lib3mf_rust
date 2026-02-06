//! Volumetric extension validation

use crate::error::{Error, Result};
use crate::model::{Extension, Model};
use std::collections::HashSet;

/// Validates volumetric extension resources and references
pub fn validate_volumetric_extension(model: &Model) -> Result<()> {
    // Check if volumetric resources/elements are used
    let has_volumetric_resources = !model.resources.volumetric_data.is_empty()
        || !model.resources.volumetric_property_groups.is_empty();

    if has_volumetric_resources {
        // Check if volumetric extension is declared in requiredextensions
        let has_volumetric_required = model
            .required_extensions
            .iter()
            .any(|ext| matches!(ext, Extension::Volumetric))
            || model
                .required_custom_extensions
                .iter()
                .any(|ns| ns.contains("volumetric/2023/02"));

        if !has_volumetric_required {
            return Err(Error::InvalidModel(
                "Model contains volumetric extension elements (volumetricdata or volumetricpropertygroup) \
                 but volumetric extension is not declared in requiredextensions attribute.\n\
                 Per 3MF Volumetric Extension spec, files using volumetric elements MUST declare the volumetric extension \
                 as a required extension in the <model> element's requiredextensions attribute.\n\
                 Add 'v' to requiredextensions and declare xmlns:v=\"http://schemas.3mf.io/volumetric/2023/02\"."
                    .to_string(),
            ));
        }
    }

    // Build sets of valid IDs for quick lookup
    let volumetric_property_group_ids: HashSet<usize> = model
        .resources
        .volumetric_property_groups
        .iter()
        .map(|g| g.id)
        .collect();

    // Validate that volumetric data boundaries are valid
    for vol_data in &model.resources.volumetric_data {
        if let Some(ref boundary) = vol_data.boundary {
            // Validate that min is less than max for all coordinates
            if boundary.min.0 >= boundary.max.0
                || boundary.min.1 >= boundary.max.1
                || boundary.min.2 >= boundary.max.2
            {
                return Err(Error::InvalidModel(format!(
                    "VolumetricData resource {}: Invalid boundary - min coordinates must be less than max coordinates.\n\
                     Found min: ({}, {}, {}), max: ({}, {}, {}).",
                    vol_data.id,
                    boundary.min.0,
                    boundary.min.1,
                    boundary.min.2,
                    boundary.max.0,
                    boundary.max.1,
                    boundary.max.2
                )));
            }
        }

        // Validate voxel grid if present
        if let Some(ref voxels) = vol_data.voxels {
            // Validate that dimensions are non-zero
            if voxels.dimensions.0 == 0 || voxels.dimensions.1 == 0 || voxels.dimensions.2 == 0 {
                return Err(Error::InvalidModel(format!(
                    "VolumetricData resource {}: Voxel grid dimensions must be greater than zero.\n\
                     Found dimensions: ({}, {}, {}).",
                    vol_data.id, voxels.dimensions.0, voxels.dimensions.1, voxels.dimensions.2
                )));
            }

            // Validate voxel positions
            for voxel in &voxels.voxels {
                if voxel.position.0 >= voxels.dimensions.0
                    || voxel.position.1 >= voxels.dimensions.1
                    || voxel.position.2 >= voxels.dimensions.2
                {
                    return Err(Error::InvalidModel(format!(
                        "VolumetricData resource {}: Voxel at position ({}, {}, {}) is outside grid dimensions ({}, {}, {}).",
                        vol_data.id,
                        voxel.position.0,
                        voxel.position.1,
                        voxel.position.2,
                        voxels.dimensions.0,
                        voxels.dimensions.1,
                        voxels.dimensions.2
                    )));
                }

                // Validate property references
                if let Some(prop_id) = voxel.property_id
                    && !volumetric_property_group_ids.contains(&prop_id)
                {
                    return Err(Error::InvalidModel(format!(
                        "VolumetricData resource {}: Voxel references non-existent property group ID {}.",
                        vol_data.id, prop_id
                    )));
                }
            }

            // Validate spacing if present
            if let Some(spacing) = voxels.spacing
                && (spacing.0 <= 0.0 || spacing.1 <= 0.0 || spacing.2 <= 0.0)
            {
                return Err(Error::InvalidModel(format!(
                    "VolumetricData resource {}: Voxel spacing must be positive.\n\
                     Found spacing: ({}, {}, {}).",
                    vol_data.id, spacing.0, spacing.1, spacing.2
                )));
            }
        }
    }

    // Validate volumetric property groups have unique IDs
    let mut seen_ids = HashSet::new();
    for group in &model.resources.volumetric_property_groups {
        if !seen_ids.insert(group.id) {
            return Err(Error::InvalidModel(format!(
                "Duplicate volumetric property group ID: {}",
                group.id
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        VolumetricBoundary, VolumetricData, VolumetricPropertyGroup, Voxel, VoxelGrid,
    };

    #[test]
    fn test_validate_empty_model() {
        let model = Model::new();
        assert!(validate_volumetric_extension(&model).is_ok());
    }

    #[test]
    fn test_validate_missing_extension_declaration() {
        let mut model = Model::new();
        model.resources.volumetric_data.push(VolumetricData::new(1));

        let result = validate_volumetric_extension(&model);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("volumetric extension is not declared"));
    }

    #[test]
    fn test_validate_with_extension_declared() {
        let mut model = Model::new();
        model.required_extensions.push(Extension::Volumetric);
        model.resources.volumetric_data.push(VolumetricData::new(1));

        assert!(validate_volumetric_extension(&model).is_ok());
    }

    #[test]
    fn test_validate_invalid_boundary() {
        let mut model = Model::new();
        model.required_extensions.push(Extension::Volumetric);

        let mut vol_data = VolumetricData::new(1);
        // Invalid boundary: min >= max
        vol_data.boundary = Some(VolumetricBoundary::new((10.0, 10.0, 10.0), (5.0, 5.0, 5.0)));
        model.resources.volumetric_data.push(vol_data);

        let result = validate_volumetric_extension(&model);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Invalid boundary"));
    }

    #[test]
    fn test_validate_zero_dimensions() {
        let mut model = Model::new();
        model.required_extensions.push(Extension::Volumetric);

        let mut vol_data = VolumetricData::new(1);
        vol_data.voxels = Some(VoxelGrid::new((0, 10, 10))); // Zero dimension
        model.resources.volumetric_data.push(vol_data);

        let result = validate_volumetric_extension(&model);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("dimensions must be greater than zero"));
    }

    #[test]
    fn test_validate_voxel_out_of_bounds() {
        let mut model = Model::new();
        model.required_extensions.push(Extension::Volumetric);

        let mut vol_data = VolumetricData::new(1);
        let mut grid = VoxelGrid::new((10, 10, 10));
        grid.voxels.push(Voxel::new((15, 5, 5))); // Out of bounds
        vol_data.voxels = Some(grid);
        model.resources.volumetric_data.push(vol_data);

        let result = validate_volumetric_extension(&model);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("outside grid dimensions"));
    }

    #[test]
    fn test_validate_invalid_spacing() {
        let mut model = Model::new();
        model.required_extensions.push(Extension::Volumetric);

        let mut vol_data = VolumetricData::new(1);
        let mut grid = VoxelGrid::new((10, 10, 10));
        grid.spacing = Some((1.0, -1.0, 1.0)); // Negative spacing
        vol_data.voxels = Some(grid);
        model.resources.volumetric_data.push(vol_data);

        let result = validate_volumetric_extension(&model);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("spacing must be positive"));
    }

    #[test]
    fn test_validate_duplicate_property_group_ids() {
        let mut model = Model::new();
        model.required_extensions.push(Extension::Volumetric);

        model
            .resources
            .volumetric_property_groups
            .push(VolumetricPropertyGroup::new(1));
        model
            .resources
            .volumetric_property_groups
            .push(VolumetricPropertyGroup::new(1)); // Duplicate ID

        let result = validate_volumetric_extension(&model);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Duplicate volumetric property group ID"));
    }
}
