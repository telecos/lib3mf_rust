//! Configuration structures for the lib3mf-slicer tool

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Configuration file structure for the slicer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlicerConfig {
    /// Slice thickness in micrometers
    pub slice_thickness_um: f64,

    /// Printable box definition
    pub printable_box: PrintableBox,

    /// Resolution for generated slice images
    pub resolution: Resolution,

    /// Optional path to public key file for encrypted 3MF files
    #[serde(default)]
    pub key_file: Option<String>,

    /// Optional specification support configuration
    #[serde(default)]
    pub spec_support: Option<SpecSupport>,
}

/// Printable box definition using two 3D points (origin and end)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintableBox {
    /// Origin point (min corner) in mm
    pub origin: Point3D,

    /// End point (max corner) in mm
    pub end: Point3D,
}

/// 3D point in millimeters
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Resolution for generated images in DPI (dots per inch)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Resolution {
    /// DPI (dots per inch) for image generation
    pub dpi: u32,
}

/// Specification support configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpecSupport {
    /// Enable mesh support (default: true)
    #[serde(default = "default_true")]
    pub meshes: bool,

    /// Enable materials support (default: true)
    #[serde(default = "default_true")]
    pub materials: bool,

    /// Enable beam lattice support (default: true)
    #[serde(default = "default_true")]
    pub beam_lattice: bool,

    /// Enable boolean operations support (default: true)
    #[serde(default = "default_true")]
    pub boolean_ops: bool,

    /// Enable displacement maps support (default: true)
    #[serde(default = "default_true")]
    pub displacement: bool,

    /// Enable slice extension support (default: true)
    #[serde(default = "default_true")]
    pub slice_extension: bool,
}

fn default_true() -> bool {
    true
}

impl PrintableBox {
    /// Get the dimensions of the printable box in mm
    pub fn dimensions(&self) -> (f64, f64, f64) {
        (
            self.end.x - self.origin.x,
            self.end.y - self.origin.y,
            self.end.z - self.origin.z,
        )
    }

    /// Get the Z range (min, max) in mm
    pub fn z_range(&self) -> (f64, f64) {
        (self.origin.z, self.end.z)
    }
}

/// Configuration validation errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Invalid slice thickness: {0} (must be positive)")]
    InvalidSliceThickness(f64),

    #[error("Invalid printable box: origin must be less than end in all dimensions")]
    InvalidPrintableBox,

    #[error("Invalid resolution: {0} DPI (must be positive)")]
    InvalidResolution(u32, u32),

    #[error("Failed to parse JSON config: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Failed to read config file: {0}")]
    IoError(#[from] std::io::Error),
}

impl SlicerConfig {
    /// Load configuration from a JSON file
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate slice thickness
        if self.slice_thickness_um <= 0.0 {
            return Err(ConfigError::InvalidSliceThickness(self.slice_thickness_um));
        }

        // Validate printable box
        if self.printable_box.origin.x >= self.printable_box.end.x
            || self.printable_box.origin.y >= self.printable_box.end.y
            || self.printable_box.origin.z >= self.printable_box.end.z
        {
            return Err(ConfigError::InvalidPrintableBox);
        }

        // Validate resolution
        if self.resolution.dpi == 0 {
            return Err(ConfigError::InvalidResolution(self.resolution.dpi, 0));
        }

        Ok(())
    }

    /// Get slice thickness in millimeters
    pub fn slice_thickness_mm(&self) -> f64 {
        self.slice_thickness_um / 1000.0
    }

    /// Calculate image width in pixels from printable box width (mm) and DPI
    pub fn calculate_image_width(&self) -> u32 {
        let width_mm = self.printable_box.end.x - self.printable_box.origin.x;
        let width_inches = width_mm / 25.4; // Convert mm to inches
        (width_inches * self.resolution.dpi as f64).round() as u32
    }

    /// Calculate image height in pixels from printable box height (mm) and DPI
    pub fn calculate_image_height(&self) -> u32 {
        let height_mm = self.printable_box.end.y - self.printable_box.origin.y;
        let height_inches = height_mm / 25.4; // Convert mm to inches
        (height_inches * self.resolution.dpi as f64).round() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let config = SlicerConfig {
            slice_thickness_um: 50.0,
            printable_box: PrintableBox {
                origin: Point3D {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                end: Point3D {
                    x: 200.0,
                    y: 200.0,
                    z: 200.0,
                },
            },
            resolution: Resolution { dpi: 300 },
            key_file: None,
            spec_support: None,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_slice_thickness() {
        let config = SlicerConfig {
            slice_thickness_um: -10.0,
            printable_box: PrintableBox {
                origin: Point3D {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                end: Point3D {
                    x: 200.0,
                    y: 200.0,
                    z: 200.0,
                },
            },
            resolution: Resolution { dpi: 300 },
            key_file: None,
            spec_support: None,
        };

        assert!(config.validate().is_err());
    }
}
