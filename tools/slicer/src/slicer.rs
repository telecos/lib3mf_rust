//! Main slicer logic for processing 3MF models

use crate::color::{ColorResolver, Rgba, lerp_color};
use crate::config::SlicerConfig;
use crate::displacement::DisplacementHandler;
use crate::renderer::{ColoredContour, Point2D, SliceContour, SliceRenderer};
use lib3mf::{BuildItem, Mesh, Model, Object, Vertex, assemble_contours};
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
///
/// Transform is stored as 12 values: [m00, m01, m02, m10, m11, m12, m20, m21, m22, tx, ty, tz]
///
/// The 3MF spec uses row-vector × matrix convention:
///   [x', y', z'] = [x, y, z] × [[m00, m01, m02], [m10, m11, m12], [m20, m21, m22]] + [tx, ty, tz]
///
/// Which gives:
///   x' = x*m00 + y*m10 + z*m20 + tx
///   y' = x*m01 + y*m11 + z*m21 + ty
///   z' = x*m02 + y*m12 + z*m22 + tz
fn apply_transform(point: &[f64; 3], transform: &[f64; 12]) -> [f64; 3] {
    let (x, y, z) = (point[0], point[1], point[2]);

    let new_x = x * transform[0] + y * transform[3] + z * transform[6] + transform[9];
    let new_y = x * transform[1] + y * transform[4] + z * transform[7] + transform[10];
    let new_z = x * transform[2] + y * transform[5] + z * transform[8] + transform[11];

    [new_x, new_y, new_z]
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

/// Compose two 3x4 affine transforms: result = parent * child
///
/// In 3MF, transforms are applied from right to left: to transform a point,
/// first apply the child transform, then the parent transform.
///
/// Transform matrices are stored as [m00, m01, m02, m10, m11, m12, m20, m21, m22, tx, ty, tz]
/// representing:
/// ```text
/// | m00 m01 m02 tx |
/// | m10 m11 m12 ty |
/// | m20 m21 m22 tz |
/// |   0   0   0  1 |
/// ```
fn compose_transforms(parent: &[f64; 12], child: &[f64; 12]) -> [f64; 12] {
    // Extract parent matrix components
    let p = parent;
    // Extract child matrix components
    let c = child;

    // Multiply the 3x3 rotation/scale parts: result_rot = parent_rot * child_rot
    let m00 = p[0] * c[0] + p[1] * c[3] + p[2] * c[6];
    let m01 = p[0] * c[1] + p[1] * c[4] + p[2] * c[7];
    let m02 = p[0] * c[2] + p[1] * c[5] + p[2] * c[8];

    let m10 = p[3] * c[0] + p[4] * c[3] + p[5] * c[6];
    let m11 = p[3] * c[1] + p[4] * c[4] + p[5] * c[7];
    let m12 = p[3] * c[2] + p[4] * c[5] + p[5] * c[8];

    let m20 = p[6] * c[0] + p[7] * c[3] + p[8] * c[6];
    let m21 = p[6] * c[1] + p[7] * c[4] + p[8] * c[7];
    let m22 = p[6] * c[2] + p[7] * c[5] + p[8] * c[8];

    // Transform child's translation by parent's rotation/scale, then add parent's translation
    let tx = p[0] * c[9] + p[1] * c[10] + p[2] * c[11] + p[9];
    let ty = p[3] * c[9] + p[4] * c[10] + p[5] * c[11] + p[10];
    let tz = p[6] * c[9] + p[7] * c[10] + p[8] * c[11] + p[11];

    [m00, m01, m02, m10, m11, m12, m20, m21, m22, tx, ty, tz]
}

/// Merge another mesh into this mesh, combining vertices and triangles
fn merge_meshes(base: &mut Mesh, other: &Mesh) {
    let vertex_offset = base.vertices.len();

    // Add vertices from other mesh
    base.vertices.extend(other.vertices.iter().cloned());

    // Add triangles from other mesh, adjusting indices
    for tri in &other.triangles {
        base.triangles.push(lib3mf::Triangle {
            v1: tri.v1 + vertex_offset,
            v2: tri.v2 + vertex_offset,
            v3: tri.v3 + vertex_offset,
            pid: tri.pid,
            p1: tri.p1,
            p2: tri.p2,
            p3: tri.p3,
            pindex: tri.pindex,
        });
    }
}

/// Compute beam-plane intersection for a cylindrical beam
/// Returns the circle center in 2D and interpolated radius if the beam crosses the Z plane
///
/// # Arguments
/// * `p1` - First endpoint of the beam (x, y, z)
/// * `p2` - Second endpoint of the beam (x, y, z)
/// * `r1` - Radius at p1 (must be positive, supports tapered beams)
/// * `r2` - Radius at p2 (must be positive, supports tapered beams)
/// * `z_height` - Z coordinate of the cutting plane
fn beam_plane_intersection(
    p1: (f64, f64, f64),
    p2: (f64, f64, f64),
    r1: f64,
    r2: f64,
    z_height: f64,
) -> Option<(Point2D, f64)> {
    let (x1, y1, z1) = p1;
    let (x2, y2, z2) = p2;

    // Validate radii
    if r1 <= 0.0 || r2 <= 0.0 {
        return None; // Invalid radius
    }

    // Check if beam crosses Z plane (endpoints on different sides)
    if (z1 - z_height) * (z2 - z_height) > 0.0 {
        return None; // Both endpoints on same side
    }

    // Handle edge case where beam is exactly on the plane
    // Use epsilon appropriate for f64 precision
    let epsilon = 1e-9;
    if (z1 - z_height).abs() < epsilon && (z2 - z_height).abs() < epsilon {
        return None; // Beam lies in plane - degenerate case
    }

    // Find intersection point along beam axis
    let t = (z_height - z1) / (z2 - z1);

    // Clamp t to [0, 1] to handle numerical precision issues
    let t = t.clamp(0.0, 1.0);

    let center_x = x1 + t * (x2 - x1);
    let center_y = y1 + t * (y2 - y1);

    // Interpolate radius for tapered beams
    let radius = r1 + t * (r2 - r1);

    Some((Point2D::new(center_x, center_y), radius))
}

/// Compute ball-plane intersection for a spherical ball joint
/// Returns a circle in 2D if the sphere intersects the Z plane
///
/// # Arguments
/// * `center` - Center of the sphere (x, y, z)
/// * `radius` - Radius of the sphere (must be positive)
/// * `z_height` - Z coordinate of the plane
fn ball_plane_intersection(
    center: (f64, f64, f64),
    radius: f64,
    z_height: f64,
) -> Option<(Point2D, f64)> {
    let (x, y, z) = center;

    // Validate radius
    if radius <= 0.0 {
        return None; // Invalid radius
    }

    let dz = (z - z_height).abs();

    if dz > radius {
        return None; // Plane doesn't intersect sphere
    }

    // Circle radius at slice height (from sphere geometry: r^2 = r_slice^2 + dz^2)
    let slice_radius = (radius * radius - dz * dz).sqrt();

    Some((Point2D::new(x, y), slice_radius))
}

/// Convert a circle to a polygon approximation with line segments
///
/// # Arguments
/// * `center` - Center of the circle
/// * `radius` - Radius of the circle (should be positive)
/// * `segments` - Number of segments (must be >= 3)
fn circle_to_line_segments(center: Point2D, radius: f64, segments: u32) -> Vec<(Point2D, Point2D)> {
    // Validate input
    if segments < 3 {
        return Vec::new();
    }

    let mut line_segments = Vec::with_capacity(segments as usize);
    let two_pi = 2.0 * std::f64::consts::PI;

    for i in 0..segments {
        let angle1 = two_pi * (i as f64) / (segments as f64);
        let angle2 = two_pi * ((i + 1) as f64) / (segments as f64);

        let p1 = Point2D::new(
            center.x + radius * angle1.cos(),
            center.y + radius * angle1.sin(),
        );
        let p2 = Point2D::new(
            center.x + radius * angle2.cos(),
            center.y + radius * angle2.sin(),
        );

        line_segments.push((p1, p2));
    }

    line_segments
}

/// Main slicer structure
pub struct Slicer {
    config: SlicerConfig,
}

/// A line segment with per-endpoint colors from a triangle intersection
struct ColoredSegment {
    p0: Point2D,
    p1: Point2D,
    color0: Rgba,
    color1: Rgba,
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

    /// Recursively resolve mesh from an object, handling component hierarchies
    ///
    /// This function traverses component references and combines meshes from the entire
    /// hierarchy, applying transforms at each level.
    ///
    /// # Arguments
    /// * `object` - The object to resolve
    /// * `model` - The model containing all objects
    /// * `accumulated_transform` - Transform accumulated from parent levels
    /// * `visited` - Set of visited object IDs to detect circular references
    /// * `displacement_handler` - Handler for displacement mesh conversion
    /// * `warned_booleans` - Set of object IDs for which boolean warnings have been shown
    fn resolve_object_mesh_recursive(
        &self,
        object: &Object,
        model: &Model,
        accumulated_transform: &Option<[f64; 12]>,
        visited: &mut std::collections::HashSet<usize>,
        displacement_handler: &DisplacementHandler,
        warned_booleans: &mut std::collections::HashSet<usize>,
    ) -> Result<Option<Mesh>, SlicerError> {
        // Detect circular references
        if visited.contains(&object.id) {
            return Ok(None); // Skip circular references
        }
        visited.insert(object.id);

        // Check if this object has a boolean shape definition
        // Boolean operations (CSG) require advanced mesh processing capabilities
        // that are not yet implemented in this slicer
        if let Some(ref bool_shape) = object.boolean_shape {
            if !warned_booleans.contains(&object.id) {
                println!(
                    "  Warning: Object {} has boolean shape (operation: {:?}) which is not yet supported by the slicer.",
                    object.id, bool_shape.operation
                );
                println!(
                    "  Boolean operations will be ignored and only the base mesh will be sliced."
                );
                warned_booleans.insert(object.id);
            }

            // For now, just resolve the base object mesh without boolean operations
            if let Some(base_obj) = model
                .resources
                .objects
                .iter()
                .find(|obj| obj.id == bool_shape.objectid)
            {
                visited.remove(&object.id);
                return self.resolve_object_mesh_recursive(
                    base_obj,
                    model,
                    accumulated_transform,
                    visited,
                    displacement_handler,
                    warned_booleans,
                );
            }
        }

        // If the object has a direct mesh, use it
        if let Some(ref mesh) = object.mesh {
            let mesh_to_use = mesh.clone();
            // Apply accumulated transform if present
            let final_mesh = if let Some(transform) = accumulated_transform {
                transform_mesh(&mesh_to_use, transform)
            } else {
                mesh_to_use
            };
            visited.remove(&object.id);
            return Ok(Some(final_mesh));
        }

        // If the object has a displacement mesh, convert and use it
        if let Some(ref disp_mesh) = object.displacement_mesh {
            let mesh_to_use = displacement_handler.apply_displacement(disp_mesh);
            // Apply accumulated transform if present
            let final_mesh = if let Some(transform) = accumulated_transform {
                transform_mesh(&mesh_to_use, transform)
            } else {
                mesh_to_use
            };
            visited.remove(&object.id);
            return Ok(Some(final_mesh));
        }

        // Otherwise, recursively resolve and combine meshes from all components
        if !object.components.is_empty() {
            let mut combined_mesh = Mesh {
                vertices: Vec::new(),
                triangles: Vec::new(),
                beamset: None,
            };

            for component in &object.components {
                // Find the referenced object
                if let Some(ref_object) = model
                    .resources
                    .objects
                    .iter()
                    .find(|obj| obj.id == component.objectid)
                {
                    // Compose transforms: accumulated * component
                    let new_transform = match (accumulated_transform, &component.transform) {
                        (Some(acc), Some(comp)) => Some(compose_transforms(acc, comp)),
                        (Some(acc), None) => Some(*acc),
                        (None, Some(comp)) => Some(*comp),
                        (None, None) => None,
                    };

                    // Recursively resolve the component's mesh
                    if let Some(comp_mesh) = self.resolve_object_mesh_recursive(
                        ref_object,
                        model,
                        &new_transform,
                        visited,
                        displacement_handler,
                        warned_booleans,
                    )? {
                        merge_meshes(&mut combined_mesh, &comp_mesh);
                    }
                }
            }

            visited.remove(&object.id);
            if combined_mesh.vertices.is_empty() {
                return Ok(None);
            }
            return Ok(Some(combined_mesh));
        }

        visited.remove(&object.id);
        Ok(None)
    }

    /// Slice a model and generate output images
    pub fn slice_model(
        &self,
        model: &Model,
        input_path: &Path,
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

        // Build color resolver
        let color_resolver = ColorResolver::from_model(model, input_path);
        let has_colors = color_resolver.has_colors();
        if has_colors {
            println!("  Color/material information detected — rendering colored borders");
        }

        // Build displacement handler
        let displacement_handler = DisplacementHandler::from_model(model, input_path);

        // Border width in pixels for colored surface rendering
        let border_width: u32 = 5;

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

        // Track which objects have shown boolean warnings to avoid duplicates
        let mut warned_booleans = std::collections::HashSet::new();

        // Generate each slice
        for layer_idx in 0..num_layers {
            let z = z_min + (layer_idx as f64) * layer_height;

            // Collect contours from all build items at this Z height
            let mut all_contours = Vec::new();
            let mut all_colored_contours: Vec<ColoredContour> = Vec::new();

            for build_item in &model.build.items {
                // Find the referenced object
                let object = model
                    .resources
                    .objects
                    .iter()
                    .find(|obj| obj.id == build_item.objectid);

                if let Some(object) = object {
                    // Check if object intersects with current Z layer
                    if self.object_intersects_z_layer(
                        object,
                        build_item,
                        z,
                        &displacement_handler,
                        model,
                        &mut warned_booleans,
                    )? {
                        // Extract and transform contours for this object
                        let (contours, colored) = self.slice_object_at_z_with_color(
                            object,
                            build_item,
                            z,
                            &color_resolver,
                            &displacement_handler,
                            model,
                            &mut warned_booleans,
                        )?;
                        all_contours.extend(contours);
                        all_colored_contours.extend(colored);
                    }
                }
            }

            // Render the slice
            let output_filename = format!("slice_{:05}_z{:.3}mm.png", layer_idx, z);
            let output_path = output_dir.join(&output_filename);

            renderer
                .render_to_file(
                    &all_contours,
                    &all_colored_contours,
                    border_width,
                    has_colors,
                    &output_path,
                )
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
        displacement_handler: &DisplacementHandler,
        model: &Model,
        warned_booleans: &mut std::collections::HashSet<usize>,
    ) -> Result<bool, SlicerError> {
        // Check if object has a slice stack reference
        if let Some(slicestackid) = object.slicestackid {
            // Find the slice stack in the model
            if let Some(slice_stack) = model
                .resources
                .slice_stacks
                .iter()
                .find(|ss| ss.id == slicestackid)
            {
                // For slice stacks, check if Z is within the stack's range
                if slice_stack.slices.is_empty() {
                    return Ok(false);
                }
                
                // Transform the zbottom and ztop to world space
                let zbottom = slice_stack.zbottom;
                let ztop = slice_stack
                    .slices
                    .last()
                    .map(|s| s.ztop)
                    .unwrap_or(zbottom);
                
                // Apply transform to get world space Z bounds
                let (world_zbottom, world_ztop) = if let Some(t) = &build_item.transform {
                    // Transform a point at object's zbottom and ztop
                    let bottom_point = apply_transform(&[0.0, 0.0, zbottom], t);
                    let top_point = apply_transform(&[0.0, 0.0, ztop], t);
                    (bottom_point[2], top_point[2])
                } else {
                    (zbottom, ztop)
                };
                
                return Ok(z >= world_zbottom && z <= world_ztop);
            }
        }

        // No slice stack - check mesh intersection
        // Recursively resolve mesh from object (handles components and hierarchy)
        let mut visited = std::collections::HashSet::new();
        let mesh_option = self.resolve_object_mesh_recursive(
            object,
            model,
            &build_item.transform,
            &mut visited,
            displacement_handler,
            warned_booleans,
        )?;

        let mesh = match mesh_option {
            Some(m) => m,
            None => return Ok(false), // No mesh, no intersection
        };

        if mesh.vertices.is_empty() {
            return Ok(false);
        }

        // Calculate bounding box in world space (mesh is already transformed)
        let mut min_z = f64::INFINITY;
        let mut max_z = f64::NEG_INFINITY;

        for vertex in &mesh.vertices {
            min_z = min_z.min(vertex.z);
            max_z = max_z.max(vertex.z);
        }

        // Check if Z layer intersects the object's Z bounds
        Ok(z >= min_z && z <= max_z)
    }

    /// Slice an object at a given Z height and return transformed contours plus colored contours
    #[allow(clippy::too_many_arguments)]
    fn slice_object_at_z_with_color(
        &self,
        object: &Object,
        build_item: &BuildItem,
        z: f64,
        color_resolver: &ColorResolver,
        displacement_handler: &DisplacementHandler,
        model: &Model,
        warned_booleans: &mut std::collections::HashSet<usize>,
    ) -> Result<(Vec<SliceContour>, Vec<ColoredContour>), SlicerError> {
        // Check if object has a slice stack reference
        if let Some(slicestackid) = object.slicestackid {
            // Find the slice stack in the model
            if let Some(slice_stack) = model
                .resources
                .slice_stacks
                .iter()
                .find(|ss| ss.id == slicestackid)
            {
                // Extract contours from slice stack at this Z height
                return self.extract_slice_stack_contours(
                    slice_stack,
                    z,
                    &build_item.transform,
                    object,
                    color_resolver,
                );
            }
        }

        // No slice stack - use mesh-based slicing
        // Recursively resolve mesh from object (handles components and hierarchy)
        let mut visited = std::collections::HashSet::new();
        let mesh_option = self.resolve_object_mesh_recursive(
            object,
            model,
            &build_item.transform,
            &mut visited,
            displacement_handler,
            warned_booleans,
        )?;

        let mesh = match mesh_option {
            Some(m) => m,
            None => return Ok((Vec::new(), Vec::new())),
        };

        // Mesh is already transformed by resolve_object_mesh_recursive
        let transformed_mesh = mesh.clone();

        // Collect plain segments for fill rendering (uses lib3mf)
        let mut plain_segments = lib3mf::collect_intersection_segments(&transformed_mesh, z);

        // Add beam lattice segments to plain segments
        if let Some(ref beamset) = transformed_mesh.beamset {
            let beam_segments = self.collect_beam_segments(&transformed_mesh, beamset, z);
            plain_segments.extend(beam_segments);
        }

        let assembled = assemble_contours(plain_segments, 1e-6);
        let fill_contours: Vec<SliceContour> = assembled
            .into_iter()
            .map(|pts| SliceContour::new(pts.iter().map(|(x, y)| Point2D::new(*x, *y)).collect()))
            .collect();

        // Collect colored segments (iterate triangles directly)
        let colored_segments = self.collect_colored_segments(
            &transformed_mesh,
            &transformed_mesh, // use transformed mesh for both
            z,
            object,
            color_resolver,
        );

        // Assemble colored segments into colored contours
        let colored_contours = assemble_colored_contours(colored_segments, 1e-6);

        Ok((fill_contours, colored_contours))
    }

    /// Extract contours from a slice stack at a given Z height
    ///
    /// Finds the appropriate slice from the stack and converts its polygons into contours.
    /// Applies the build item transform to all vertices.
    fn extract_slice_stack_contours(
        &self,
        slice_stack: &lib3mf::SliceStack,
        world_z: f64,
        transform: &Option<[f64; 12]>,
        _object: &Object,
        _color_resolver: &ColorResolver,
    ) -> Result<(Vec<SliceContour>, Vec<ColoredContour>), SlicerError> {
        // Convert world space Z to object space Z
        // Transform format: [m00, m01, m02, m10, m11, m12, m20, m21, m22, tx, ty, tz]
        // For Z coordinate: world_z = object_x * m02 + object_y * m12 + object_z * m22 + tz
        // For a typical case where X,Y don't affect Z (m02=0, m12=0, m22=1), this simplifies to:
        // object_z = (world_z - tz) / m22
        let object_z = if let Some(t) = transform {
            let tz = t[11];  // Z translation
            let m22 = t[8];   // Z scale/rotation component
            if m22.abs() < 1e-10 {
                // Degenerate transform - can't compute object Z
                return Ok((Vec::new(), Vec::new()));
            }
            (world_z - tz) / m22
        } else {
            world_z
        };
        
        // Find the slice at or just above this object-space Z height
        // Slices are stored in ascending ztop order
        let mut prev_ztop = slice_stack.zbottom;
        let slice_opt = slice_stack.slices.iter().find(|slice| {
            // Check if object_z is between zbottom and ztop of this slice
            let zbottom = prev_ztop;
            prev_ztop = slice.ztop;
            object_z >= zbottom && object_z <= slice.ztop
        });

        let slice = match slice_opt {
            Some(s) => s,
            None => return Ok((Vec::new(), Vec::new())), // Z is outside slice stack range
        };

        // Convert slice polygons to contours
        let mut fill_contours = Vec::new();
        
        for polygon in &slice.polygons {
            let mut points = Vec::new();
            
            // Start with the initial vertex
            if polygon.startv >= slice.vertices.len() {
                eprintln!(
                    "Warning: Slice polygon startv={} out of bounds (vertices={})",
                    polygon.startv,
                    slice.vertices.len()
                );
                continue;
            }
            
            let v = &slice.vertices[polygon.startv];
            let point = if let Some(t) = transform {
                // Apply transform to 2D vertex (use object_z for the Z coordinate)
                let transformed = apply_transform(&[v.x, v.y, object_z], t);
                Point2D::new(transformed[0], transformed[1])
            } else {
                Point2D::new(v.x, v.y)
            };
            points.push(point);
            
            // Add points from segments
            for segment in &polygon.segments {
                if segment.v2 >= slice.vertices.len() {
                    eprintln!(
                        "Warning: Slice segment v2={} out of bounds (vertices={})",
                        segment.v2,
                        slice.vertices.len()
                    );
                    continue;
                }
                
                let v = &slice.vertices[segment.v2];
                let point = if let Some(t) = transform {
                    let transformed = apply_transform(&[v.x, v.y, object_z], t);
                    Point2D::new(transformed[0], transformed[1])
                } else {
                    Point2D::new(v.x, v.y)
                };
                points.push(point);
            }
            
            // Only add non-empty contours
            if !points.is_empty() {
                fill_contours.push(SliceContour::new(points));
            }
        }

        // For now, we don't generate colored contours from slice stacks
        // as they don't carry per-vertex color information
        // TODO: Could use object's material properties for border coloring
        let colored_contours = Vec::new();

        Ok((fill_contours, colored_contours))
    }

    /// Collect intersection segments with per-endpoint color from triangles
    fn collect_colored_segments(
        &self,
        transformed_mesh: &Mesh,
        original_mesh: &Mesh,
        z: f64,
        object: &Object,
        color_resolver: &ColorResolver,
    ) -> Vec<ColoredSegment> {
        let mut segments = Vec::new();

        for tri in original_mesh.triangles.iter() {
            if tri.v1 >= transformed_mesh.vertices.len()
                || tri.v2 >= transformed_mesh.vertices.len()
                || tri.v3 >= transformed_mesh.vertices.len()
            {
                continue;
            }

            let v0 = &transformed_mesh.vertices[tri.v1];
            let v1 = &transformed_mesh.vertices[tri.v2];
            let v2 = &transformed_mesh.vertices[tri.v3];

            if let Some(seg) =
                colored_triangle_intersection(v0, v1, v2, z, tri, object, color_resolver)
            {
                segments.push(seg);
            }
        }

        segments
    }

    /// Collect beam lattice intersection segments at a given Z height
    /// Returns line segments approximating circles where beams intersect the Z plane
    fn collect_beam_segments(
        &self,
        mesh: &Mesh,
        beamset: &lib3mf::BeamSet,
        z: f64,
    ) -> Vec<((f64, f64), (f64, f64))> {
        let mut segments = Vec::new();
        const CIRCLE_SEGMENTS: u32 = 16; // Number of line segments to approximate each beam circle

        // Process beams
        for beam in &beamset.beams {
            // Validate vertex indices
            if beam.v1 >= mesh.vertices.len() || beam.v2 >= mesh.vertices.len() {
                continue; // Skip invalid beams
            }

            let v1 = &mesh.vertices[beam.v1];
            let v2 = &mesh.vertices[beam.v2];

            let p1 = (v1.x, v1.y, v1.z);
            let p2 = (v2.x, v2.y, v2.z);

            // Get beam radii (with fallbacks to beamset defaults)
            let r1 = beam.r1.unwrap_or(beamset.radius);
            let r2 = beam.r2.or(beam.r1).unwrap_or(beamset.radius);

            if let Some((center, radius)) = beam_plane_intersection(p1, p2, r1, r2, z) {
                // Convert circle to polygon segments
                let circle_segments = circle_to_line_segments(center, radius, CIRCLE_SEGMENTS);
                for (p1, p2) in circle_segments {
                    segments.push(((p1.x, p1.y), (p2.x, p2.y)));
                }
            }
        }

        // Process ball joints (if present)
        for ball in &beamset.balls {
            // Validate vertex index
            if ball.vindex >= mesh.vertices.len() {
                continue; // Skip invalid balls
            }

            let vertex = &mesh.vertices[ball.vindex];
            let center = (vertex.x, vertex.y, vertex.z);

            // Get ball radius (with fallback to beamset ball_radius or default radius)
            let radius = ball
                .radius
                .or(beamset.ball_radius)
                .unwrap_or(beamset.radius);

            if let Some((center_2d, slice_radius)) = ball_plane_intersection(center, radius, z) {
                // Convert circle to polygon segments
                let circle_segments =
                    circle_to_line_segments(center_2d, slice_radius, CIRCLE_SEGMENTS);
                for (p1, p2) in circle_segments {
                    segments.push(((p1.x, p1.y), (p2.x, p2.y)));
                }
            }
        }

        segments
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

/// Intersect a single triangle with a Z plane and return a colored segment.
///
/// Each intersection endpoint gets an interpolated color based on the
/// triangle's material properties and the interpolation parameter along
/// the intersected edges.
fn colored_triangle_intersection(
    v0: &Vertex,
    v1: &Vertex,
    v2: &Vertex,
    z: f64,
    tri: &lib3mf::Triangle,
    object: &Object,
    color_resolver: &ColorResolver,
) -> Option<ColoredSegment> {
    // Resolve per-vertex colors for this triangle
    let (c0, c1, c2) = color_resolver
        .resolve_triangle_colors(tri, object.pid, object.pindex)
        .unwrap_or((
            crate::color::DEFAULT_COLOR,
            crate::color::DEFAULT_COLOR,
            crate::color::DEFAULT_COLOR,
        ));

    let vertices = [v0, v1, v2];
    let vertex_colors = [c0, c1, c2];

    // Find edge intersections with the z-plane
    // Each intersection records: (x, y, interpolated_color)
    let mut intersections: Vec<(f64, f64, Rgba)> = Vec::with_capacity(2);

    for i in 0..3 {
        let va = vertices[i];
        let vb = vertices[(i + 1) % 3];
        let ca = vertex_colors[i];
        let cb = vertex_colors[(i + 1) % 3];

        let za = va.z;
        let zb = vb.z;

        if (za - z) * (zb - z) > 0.0 {
            continue; // both on same side
        }

        if (za - z).abs() < 1e-10 && (zb - z).abs() < 1e-10 {
            // Both on plane
            intersections.push((va.x, va.y, ca));
            intersections.push((vb.x, vb.y, cb));
            break;
        }

        if (za - z).abs() < 1e-10 {
            intersections.push((va.x, va.y, ca));
            continue;
        }
        if (zb - z).abs() < 1e-10 {
            intersections.push((vb.x, vb.y, cb));
            continue;
        }

        // Interpolation parameter along the edge
        let t = (z - za) / (zb - za);
        let x = va.x + t * (vb.x - va.x);
        let y = va.y + t * (vb.y - va.y);

        if x.is_finite() && y.is_finite() {
            let color = lerp_color(ca, cb, t);
            intersections.push((x, y, color));
        }
    }

    if intersections.len() < 2 {
        return None;
    }

    // Deduplicate if more than 2
    if intersections.len() > 2 {
        intersections.sort_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        });
        intersections.dedup_by(|a, b| (a.0 - b.0).abs() < 1e-10 && (a.1 - b.1).abs() < 1e-10);
    }

    if intersections.len() >= 2 {
        Some(ColoredSegment {
            p0: Point2D::new(intersections[0].0, intersections[0].1),
            p1: Point2D::new(intersections[1].0, intersections[1].1),
            color0: intersections[0].2,
            color1: intersections[1].2,
        })
    } else {
        None
    }
}

/// Assemble colored segments into closed colored contours
fn assemble_colored_contours(segments: Vec<ColoredSegment>, tolerance: f64) -> Vec<ColoredContour> {
    if segments.is_empty() {
        return Vec::new();
    }

    let mut remaining: Vec<ColoredSegment> = segments;
    let mut contours = Vec::new();

    while !remaining.is_empty() {
        let first = remaining.remove(0);
        let mut points = vec![first.p0, first.p1];
        let mut colors = vec![first.color0, first.color1];
        let start = first.p0;
        let mut current = first.p1;

        let mut found = true;
        while found && !remaining.is_empty() {
            found = false;

            for i in 0..remaining.len() {
                let seg = &remaining[i];
                let d0 = dist(current, seg.p0);
                let d1 = dist(current, seg.p1);

                if d0 <= tolerance {
                    current = seg.p1;
                    points.push(current);
                    colors.push(seg.color1);
                    remaining.remove(i);
                    found = true;
                    break;
                } else if d1 <= tolerance {
                    current = seg.p0;
                    points.push(current);
                    colors.push(seg.color0);
                    remaining.remove(i);
                    found = true;
                    break;
                }
            }

            if dist(current, start) <= tolerance {
                points.pop();
                colors.pop();
                break;
            }
        }

        if points.len() >= 3 {
            contours.push(ColoredContour { points, colors });
        }
    }

    contours
}

#[inline]
fn dist(a: Point2D, b: Point2D) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
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
