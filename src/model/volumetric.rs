//! Volumetric extension types
//!
//! This module implements data structures for the 3MF Volumetric Extension,
//! which enables detailed volumetric (voxel-based or implicit) descriptions
//! of 3D models for additive manufacturing.

/// Volumetric data resource from volumetric extension
///
/// Represents a volumetric region that can contain either explicit voxel
/// grids or implicit (mathematical) volume definitions.
#[derive(Debug, Clone)]
pub struct VolumetricData {
    /// Volumetric data ID
    pub id: usize,
    /// Optional boundary defining the volumetric region
    pub boundary: Option<VolumetricBoundary>,
    /// Voxel grid data (if explicit)
    pub voxels: Option<VoxelGrid>,
    /// Implicit volume data (if mathematical/SDF)
    pub implicit: Option<ImplicitVolume>,
}

impl VolumetricData {
    /// Create a new volumetric data resource
    pub fn new(id: usize) -> Self {
        Self {
            id,
            boundary: None,
            voxels: None,
            implicit: None,
        }
    }
}

/// Bounding box defining the volumetric region
#[derive(Debug, Clone, Copy)]
pub struct VolumetricBoundary {
    /// Minimum coordinates (x, y, z)
    pub min: (f64, f64, f64),
    /// Maximum coordinates (x, y, z)
    pub max: (f64, f64, f64),
}

impl VolumetricBoundary {
    /// Create a new volumetric boundary
    pub fn new(min: (f64, f64, f64), max: (f64, f64, f64)) -> Self {
        Self { min, max }
    }
}

/// Explicit voxel grid data
#[derive(Debug, Clone)]
pub struct VoxelGrid {
    /// Grid dimensions (x, y, z)
    pub dimensions: (usize, usize, usize),
    /// Physical spacing between voxels (x, y, z)
    pub spacing: Option<(f64, f64, f64)>,
    /// Origin point in model coordinates
    pub origin: Option<(f64, f64, f64)>,
    /// List of voxels with their properties
    pub voxels: Vec<Voxel>,
}

impl VoxelGrid {
    /// Create a new voxel grid
    pub fn new(dimensions: (usize, usize, usize)) -> Self {
        Self {
            dimensions,
            spacing: None,
            origin: None,
            voxels: Vec::new(),
        }
    }
}

/// Individual voxel with position and properties
#[derive(Debug, Clone)]
pub struct Voxel {
    /// Voxel position (x, y, z)
    pub position: (usize, usize, usize),
    /// Property reference ID (optional)
    pub property_id: Option<usize>,
    /// Color reference ID (optional)
    pub color_id: Option<usize>,
}

impl Voxel {
    /// Create a new voxel at the specified position
    pub fn new(position: (usize, usize, usize)) -> Self {
        Self {
            position,
            property_id: None,
            color_id: None,
        }
    }
}

/// Implicit volume definition using mathematical functions
#[derive(Debug, Clone)]
pub struct ImplicitVolume {
    /// Type of implicit function (e.g., "sdf" for signed distance field)
    pub implicit_type: String,
    /// Additional parameters for the implicit function
    pub parameters: Vec<(String, String)>,
}

impl ImplicitVolume {
    /// Create a new implicit volume definition
    pub fn new(implicit_type: String) -> Self {
        Self {
            implicit_type,
            parameters: Vec::new(),
        }
    }
}

/// Volumetric property group
///
/// Maps property indices to property values for volumetric data
#[derive(Debug, Clone)]
pub struct VolumetricPropertyGroup {
    /// Property group ID
    pub id: usize,
    /// List of property values
    pub properties: Vec<VolumetricProperty>,
}

impl VolumetricPropertyGroup {
    /// Create a new volumetric property group
    pub fn new(id: usize) -> Self {
        Self {
            id,
            properties: Vec::new(),
        }
    }
}

/// Individual volumetric property value
#[derive(Debug, Clone)]
pub struct VolumetricProperty {
    /// Property index
    pub index: usize,
    /// Property value (can be various types)
    pub value: String,
}

impl VolumetricProperty {
    /// Create a new volumetric property
    pub fn new(index: usize, value: String) -> Self {
        Self { index, value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volumetric_data_creation() {
        let data = VolumetricData::new(1);
        assert_eq!(data.id, 1);
        assert!(data.boundary.is_none());
        assert!(data.voxels.is_none());
        assert!(data.implicit.is_none());
    }

    #[test]
    fn test_volumetric_boundary() {
        let boundary = VolumetricBoundary::new((0.0, 0.0, 0.0), (100.0, 100.0, 100.0));
        assert_eq!(boundary.min, (0.0, 0.0, 0.0));
        assert_eq!(boundary.max, (100.0, 100.0, 100.0));
    }

    #[test]
    fn test_voxel_grid() {
        let grid = VoxelGrid::new((10, 10, 10));
        assert_eq!(grid.dimensions, (10, 10, 10));
        assert!(grid.spacing.is_none());
        assert!(grid.origin.is_none());
        assert!(grid.voxels.is_empty());
    }

    #[test]
    fn test_voxel_creation() {
        let voxel = Voxel::new((5, 5, 5));
        assert_eq!(voxel.position, (5, 5, 5));
        assert!(voxel.property_id.is_none());
        assert!(voxel.color_id.is_none());
    }

    #[test]
    fn test_implicit_volume() {
        let mut implicit = ImplicitVolume::new("sdf".to_string());
        assert_eq!(implicit.implicit_type, "sdf");
        assert!(implicit.parameters.is_empty());

        implicit
            .parameters
            .push(("param1".to_string(), "value1".to_string()));
        assert_eq!(implicit.parameters.len(), 1);
    }

    #[test]
    fn test_volumetric_property_group() {
        let mut group = VolumetricPropertyGroup::new(1);
        assert_eq!(group.id, 1);
        assert!(group.properties.is_empty());

        group
            .properties
            .push(VolumetricProperty::new(0, "value".to_string()));
        assert_eq!(group.properties.len(), 1);
    }
}
