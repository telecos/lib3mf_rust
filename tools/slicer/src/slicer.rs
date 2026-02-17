//! Main slicer logic for processing 3MF models

use crate::config::SlicerConfig;
use crate::renderer::{Point2D, SliceContour, SliceRenderer};
use lib3mf::{BuildItem, Mesh, Model, Object, assemble_contours, collect_intersection_segments};
use std::path::Path;
use thiserror::Error;

/// Slicer errors
#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)]
pub enum SlicerError {
    #[error("Failed to load 3MF file: {0}")]
    LoadError(#[from] lib3mf::Error),

    #[error("Failed to render slice: {0}")]
    RenderError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Apply a 3MF affine transform to a 3D point
/// Transform is stored as 12 values: [m00, m01, m02, m10, m11, m12, m20, m21, m22, m30, m31, m32]
/// This represents a 3x4 matrix in row-major order
fn apply_transform(point: &[f64; 3], transform: &[f64; 12]) -> [f64; 3] {
    let x =
        transform[0] * point[0] + transform[1] * point[1] + transform[2] * point[2] + transform[3];
    let y =
        transform[4] * point[0] + transform[5] * point[1] + transform[6] * point[2] + transform[7];
    let z = transform[8] * point[0]
        + transform[9] * point[1]
        + transform[10] * point[2]
        + transform[11];
    [x, y, z]
}

/// Transform an entire mesh by applying an affine transformation to all vertices
fn transform_mesh(mesh: &Mesh, transform: &[f64; 12]) -> Mesh {
    let mut transformed = mesh.clone();
    for vertex in &mut transformed.vertices {
        let transformed_point = apply_transform(&[vertex.x, vertex.y, vertex.z], transform);
        vertex.x = transformed_point[0];
        vertex.y = transformed_point[1];
        vertex.z = transformed_point[2];
    }
    transformed
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

        // Calculate image dimensions from printable box size and DPI
        let image_width = self.config.calculate_image_width();
        let image_height = self.config.calculate_image_height();
        println!(
            "  Resolution: {} DPI ({}x{} pixels)",
            self.config.resolution.dpi, image_width, image_height
        );

        // Process build items (objects that are actually part of the build)
        println!("  Build items: {}", model.build.items.len());

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
            image_width,
            image_height,
            self.config.printable_box.origin.x,
            self.config.printable_box.origin.y,
            box_width,
            box_height,
        );

        let mut output_files = Vec::new();

        // Generate each slice
        for layer_idx in 0..num_layers {
            let z = z_min + (layer_idx as f64) * layer_height;

            // Collect contours from all build items at this Z height
            let mut all_contours = Vec::new();

            for build_item in &model.build.items {
                // Find the referenced object
                let object = model
                    .resources
                    .objects
                    .iter()
                    .find(|obj| obj.id == build_item.objectid);

                if let Some(object) = object {
                    // Check if object intersects with current Z layer
                    if self.object_intersects_z_layer(object, build_item, z)? {
                        // Extract and transform contours for this object
                        if let Some(contours) = self.slice_object_at_z(object, build_item, z)? {
                            all_contours.extend(contours);
                        }
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

    /// Check if an object (with its transform) intersects a given Z layer
    fn object_intersects_z_layer(
        &self,
        object: &Object,
        build_item: &BuildItem,
        z: f64,
    ) -> Result<bool, SlicerError> {
        // Get the mesh from the object
        let mesh = match &object.mesh {
            Some(m) => m,
            None => return Ok(false), // No mesh, no intersection
        };

        if mesh.vertices.is_empty() {
            return Ok(false);
        }

        // Calculate bounding box in object's local space
        let mut min_z = f64::INFINITY;
        let mut max_z = f64::NEG_INFINITY;

        for vertex in &mesh.vertices {
            // Apply transform if present
            let transformed = if let Some(transform) = &build_item.transform {
                apply_transform(&[vertex.x, vertex.y, vertex.z], transform)
            } else {
                [vertex.x, vertex.y, vertex.z]
            };

            min_z = min_z.min(transformed[2]);
            max_z = max_z.max(transformed[2]);
        }

        // Check if Z layer intersects the object's Z bounds
        Ok(z >= min_z && z <= max_z)
    }

    /// Slice an object at a given Z height and return transformed contours
    fn slice_object_at_z(
        &self,
        object: &Object,
        build_item: &BuildItem,
        z: f64,
    ) -> Result<Option<Vec<SliceContour>>, SlicerError> {
        // Get the mesh from the object
        let mesh = match &object.mesh {
            Some(m) => m,
            None => return Ok(None),
        };

        // If there's a transform, we need to apply it to get the mesh in world space
        let transformed_mesh = if let Some(transform) = &build_item.transform {
            transform_mesh(mesh, transform)
        } else {
            mesh.clone()
        };

        // Slice the transformed mesh at the given Z
        let segments = collect_intersection_segments(&transformed_mesh, z);
        if segments.is_empty() {
            return Ok(None);
        }

        let contours = assemble_contours(segments, 1e-6);
        let mut result = Vec::new();

        for contour in contours {
            let points: Vec<Point2D> = contour.iter().map(|(x, y)| Point2D::new(*x, *y)).collect();
            result.push(SliceContour::new(points));
        }

        Ok(Some(result))
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
            resolution: Resolution { dpi: 300 },
            key_file: None,
            spec_support: None,
        };

        let slicer = Slicer::new(config);
        assert_eq!(slicer.config.slice_thickness_um, 50.0);
    }
}
