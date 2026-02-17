//! Main slicer logic for processing 3MF models

use crate::config::SlicerConfig;
use crate::renderer::{Point2D, SliceContour, SliceRenderer};
use lib3mf::{Mesh, Model, Object, assemble_contours, collect_intersection_segments};
use std::path::Path;
use thiserror::Error;

/// Slicer errors
#[derive(Debug, Error)]
pub enum SlicerError {
    #[error("No objects found in 3MF model")]
    NoObjects,

    #[error("Failed to load 3MF file: {0}")]
    LoadError(#[from] lib3mf::Error),

    #[error("Failed to render slice: {0}")]
    RenderError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Main slicer structure
pub struct Slicer {
    config: SlicerConfig,
}

impl Slicer {
    /// Create a new slicer with the given configuration
    pub fn new(config: SlicerConfig) -> Self {
        Self { config }
    }

    /// Load a 3MF model from file
    pub fn load_model(&self, path: &Path) -> Result<Model, SlicerError> {
        let file = std::fs::File::open(path)?;
        let model = Model::from_reader(file)?;
        Ok(model)
    }

    /// Slice a model and generate output images
    pub fn slice_model(
        &self,
        model: &Model,
        output_dir: &Path,
    ) -> Result<Vec<String>, SlicerError> {
        println!("Starting slicing process...");
        println!(
            "  Slice thickness: {:.3} mm ({} μm)",
            self.config.slice_thickness_mm(),
            self.config.slice_thickness_um
        );
        println!(
            "  Printable box: ({:.1}, {:.1}, {:.1}) to ({:.1}, {:.1}, {:.1}) mm",
            self.config.printable_box.origin.x,
            self.config.printable_box.origin.y,
            self.config.printable_box.origin.z,
            self.config.printable_box.end.x,
            self.config.printable_box.end.y,
            self.config.printable_box.end.z
        );
        println!(
            "  Resolution: {}x{} pixels",
            self.config.resolution.width, self.config.resolution.height
        );

        // Extract all meshes from the model
        let meshes = self.extract_meshes(model)?;
        println!("  Found {} mesh(es)", meshes.len());

        if meshes.is_empty() {
            return Err(SlicerError::NoObjects);
        }

        // Calculate z layers
        let (z_min, z_max) = self.config.printable_box.z_range();
        let layer_height = self.config.slice_thickness_mm();
        let num_layers = ((z_max - z_min) / layer_height).ceil() as usize;

        println!(
            "  Generating {} layers from Z={:.2} to Z={:.2} mm",
            num_layers, z_min, z_max
        );

        // Create output directory if it doesn't exist
        std::fs::create_dir_all(output_dir)?;

        // Create renderer
        let (box_width, box_height, _) = self.config.printable_box.dimensions();
        let renderer = SliceRenderer::new(
            self.config.resolution.width,
            self.config.resolution.height,
            self.config.printable_box.origin.x,
            self.config.printable_box.origin.y,
            box_width,
            box_height,
        );

        let mut output_files = Vec::new();

        // Generate each slice
        for layer_idx in 0..num_layers {
            let z = z_min + (layer_idx as f64) * layer_height;

            // Collect contours from all meshes at this Z height
            let mut all_contours = Vec::new();

            for mesh in &meshes {
                let segments = collect_intersection_segments(mesh, z);
                if !segments.is_empty() {
                    let contours = assemble_contours(segments, 1e-6);

                    for contour in contours {
                        let points: Vec<Point2D> =
                            contour.iter().map(|(x, y)| Point2D::new(*x, *y)).collect();
                        all_contours.push(SliceContour::new(points));
                    }
                }
            }

            // Render the slice
            let output_filename = format!("slice_{:05}_z{:.3}mm.png", layer_idx, z);
            let output_path = output_dir.join(&output_filename);

            renderer
                .render_to_file(&all_contours, &output_path)
                .map_err(|e| SlicerError::RenderError(e.to_string()))?;

            if layer_idx % 10 == 0 || layer_idx == num_layers - 1 {
                println!(
                    "  Progress: {}/{} layers ({} contours at Z={:.3} mm)",
                    layer_idx + 1,
                    num_layers,
                    all_contours.len(),
                    z
                );
            }

            output_files.push(output_filename);
        }

        println!(
            "Slicing complete! Generated {} slice images.",
            output_files.len()
        );

        Ok(output_files)
    }

    /// Extract all meshes from the model, respecting spec support configuration
    fn extract_meshes(&self, model: &Model) -> Result<Vec<Mesh>, SlicerError> {
        let mut meshes = Vec::new();
        let spec_support = self.config.spec_support.as_ref();

        // Check if meshes are enabled
        let meshes_enabled = spec_support.is_none_or(|s| s.meshes);

        if !meshes_enabled {
            println!("  Mesh support disabled in configuration");
            return Ok(meshes);
        }

        // Extract meshes from all objects
        for object in &model.resources.objects {
            if let Some(mesh) = self.extract_mesh_from_object(object) {
                meshes.push(mesh);
            }
        }

        // Note: Support for other formats (boolean ops, beam lattice, etc.)
        // would be added here in future iterations. For now, we focus on basic meshes.

        Ok(meshes)
    }

    /// Extract mesh from an object
    fn extract_mesh_from_object(&self, object: &Object) -> Option<Mesh> {
        object.mesh.clone()
    }

    /// Print model statistics
    pub fn print_model_info(&self, model: &Model) {
        println!("\n=== Model Information ===");
        println!("  Objects: {}", model.resources.objects.len());
        println!(
            "  Base material groups: {}",
            model.resources.base_material_groups.len()
        );
        println!("  Color groups: {}", model.resources.color_groups.len());
        println!(
            "  Texture 2D groups: {}",
            model.resources.texture2d_groups.len()
        );
        println!("  Materials: {}", model.resources.materials.len());
        println!("  Slice stacks: {}", model.resources.slice_stacks.len());

        // Count total vertices and triangles
        let mut total_vertices = 0;
        let mut total_triangles = 0;

        for object in &model.resources.objects {
            if let Some(mesh) = &object.mesh {
                total_vertices += mesh.vertices.len();
                total_triangles += mesh.triangles.len();
            }
        }

        println!("  Total vertices: {}", total_vertices);
        println!("  Total triangles: {}", total_triangles);

        // Calculate overall bounding box for each object
        if total_vertices > 0 {
            for object in &model.resources.objects {
                if let Some(mesh) = &object.mesh {
                    // Calculate simple bounding box from vertices
                    if !mesh.vertices.is_empty() {
                        let mut min_x = f64::INFINITY;
                        let mut min_y = f64::INFINITY;
                        let mut min_z = f64::INFINITY;
                        let mut max_x = f64::NEG_INFINITY;
                        let mut max_y = f64::NEG_INFINITY;
                        let mut max_z = f64::NEG_INFINITY;

                        for vertex in &mesh.vertices {
                            min_x = min_x.min(vertex.x);
                            min_y = min_y.min(vertex.y);
                            min_z = min_z.min(vertex.z);
                            max_x = max_x.max(vertex.x);
                            max_y = max_y.max(vertex.y);
                            max_z = max_z.max(vertex.z);
                        }

                        println!(
                            "  Object {} AABB: ({:.2}, {:.2}, {:.2}) to ({:.2}, {:.2}, {:.2})",
                            object.id, min_x, min_y, min_z, max_x, max_y, max_z
                        );
                    }
                }
            }
        }

        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Point3D, PrintableBox, Resolution};

    #[test]
    fn test_slicer_creation() {
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
            resolution: Resolution {
                width: 1920,
                height: 1080,
            },
            key_file: None,
            spec_support: None,
        };

        let slicer = Slicer::new(config);
        assert_eq!(slicer.config.slice_thickness_um, 50.0);
    }
}
