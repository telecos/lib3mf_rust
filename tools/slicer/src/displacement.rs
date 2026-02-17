//! Displacement map support for the slicer
//!
//! Handles loading and applying displacement maps to convert DisplacementMesh
//! into regular Mesh with displaced vertices.

use image::GrayImage;
use lib3mf::{
    Disp2DGroup, Displacement2D, DisplacementMesh, Mesh, Model, NormVectorGroup, Triangle, Vertex,
};
use std::collections::HashMap;
use std::path::Path;

/// Displacement map handler that loads textures and applies displacement
pub struct DisplacementHandler {
    /// Loaded displacement textures: displacement map ID → grayscale image
    textures: HashMap<usize, GrayImage>,
    /// Displacement map resources by ID
    displacement_maps: HashMap<usize, Displacement2D>,
    /// Displacement coordinate groups by ID
    disp2d_groups: HashMap<usize, Disp2DGroup>,
    /// Normal vector groups by ID
    norm_vector_groups: HashMap<usize, NormVectorGroup>,
}

impl DisplacementHandler {
    /// Create a new displacement handler from a model and 3MF file path
    pub fn from_model(model: &Model, file_path: &Path) -> Self {
        let textures = Self::load_displacement_textures(model, file_path);

        let mut displacement_maps = HashMap::new();
        for disp_map in &model.resources.displacement_maps {
            displacement_maps.insert(disp_map.id, disp_map.clone());
        }

        let mut disp2d_groups = HashMap::new();
        for group in &model.resources.disp2d_groups {
            disp2d_groups.insert(group.id, group.clone());
        }

        let mut norm_vector_groups = HashMap::new();
        for group in &model.resources.norm_vector_groups {
            norm_vector_groups.insert(group.id, group.clone());
        }

        Self {
            textures,
            displacement_maps,
            disp2d_groups,
            norm_vector_groups,
        }
    }

    /// Convert a DisplacementMesh to a regular Mesh by applying displacement
    pub fn apply_displacement(&self, disp_mesh: &DisplacementMesh) -> Mesh {
        // Start with a copy of vertices
        let mut displaced_vertices = disp_mesh.vertices.clone();

        // Process each triangle to apply displacement to its vertices
        for tri in &disp_mesh.triangles {
            if let Some(did) = tri.did {
                // Get the displacement group
                if let Some(disp_group) = self.disp2d_groups.get(&did) {
                    // Apply displacement to each vertex
                    if let Some(d1) = tri.d1 {
                        self.apply_vertex_displacement(
                            &mut displaced_vertices,
                            tri.v1,
                            disp_group,
                            d1,
                        );
                    }
                    if let Some(d2) = tri.d2 {
                        self.apply_vertex_displacement(
                            &mut displaced_vertices,
                            tri.v2,
                            disp_group,
                            d2,
                        );
                    }
                    if let Some(d3) = tri.d3 {
                        self.apply_vertex_displacement(
                            &mut displaced_vertices,
                            tri.v3,
                            disp_group,
                            d3,
                        );
                    }
                }
            }
        }

        // Convert displacement triangles to regular triangles
        let mut mesh = Mesh::new();
        mesh.vertices = displaced_vertices;

        for tri in &disp_mesh.triangles {
            let mut regular_tri = Triangle::new(tri.v1, tri.v2, tri.v3);
            regular_tri.pid = tri.pid;
            regular_tri.pindex = tri.pindex;
            regular_tri.p1 = tri.p1;
            regular_tri.p2 = tri.p2;
            regular_tri.p3 = tri.p3;
            mesh.triangles.push(regular_tri);
        }

        mesh
    }

    /// Apply displacement to a single vertex
    fn apply_vertex_displacement(
        &self,
        vertices: &mut [Vertex],
        vertex_idx: usize,
        disp_group: &Disp2DGroup,
        disp_coord_idx: usize,
    ) {
        if vertex_idx >= vertices.len() {
            return;
        }

        if disp_coord_idx >= disp_group.coords.len() {
            return;
        }

        let disp_coord = &disp_group.coords[disp_coord_idx];

        // Get the normal vector
        if let Some(norm_group) = self.norm_vector_groups.get(&disp_group.nid) {
            if disp_coord.n >= norm_group.vectors.len() {
                return;
            }

            let normal = &norm_group.vectors[disp_coord.n];

            // Get the displacement map
            if let Some(disp_map) = self.displacement_maps.get(&disp_group.dispid) {
                // Sample the displacement texture
                let displacement_value = self.sample_displacement_texture(
                    disp_map,
                    disp_coord.u as f32,
                    disp_coord.v as f32,
                );

                // Calculate displacement amount: offset + (height × texture_value × factor)
                // texture_value is normalized to 0-1
                let displacement_amount =
                    disp_group.offset + (disp_group.height * displacement_value * disp_coord.f);

                // Apply displacement along the normal vector
                let vertex = &mut vertices[vertex_idx];
                vertex.x += normal.x * displacement_amount;
                vertex.y += normal.y * displacement_amount;
                vertex.z += normal.z * displacement_amount;
            }
        }
    }

    /// Sample a displacement texture at the given UV coordinates
    /// Returns a value in range \[0.0, 1.0\]
    fn sample_displacement_texture(&self, disp_map: &Displacement2D, u: f32, v: f32) -> f64 {
        if let Some(img) = self.textures.get(&disp_map.id) {
            let w = img.width() as f32;
            let h = img.height() as f32;

            // Apply tile style for U coordinate
            let u_sampled = apply_tile_style(u, disp_map.tilestyleu);
            // Apply tile style for V coordinate, and flip V (3MF: V=0 is bottom, image y=0 is top)
            let v_sampled = 1.0 - apply_tile_style(v, disp_map.tilestylev);

            // Handle out-of-bounds after tiling (for None tile style)
            if !(0.0..=1.0).contains(&u_sampled) || !(0.0..=1.0).contains(&v_sampled) {
                return 0.0; // No displacement outside [0,1] for TileStyle::None
            }

            let px = ((u_sampled * w) as u32).min(img.width().saturating_sub(1));
            let py = ((v_sampled * h) as u32).min(img.height().saturating_sub(1));

            let pixel = img.get_pixel(px, py);
            // Normalize grayscale value to 0.0-1.0
            pixel[0] as f64 / 255.0
        } else {
            0.0 // No displacement if texture not found
        }
    }

    /// Load displacement textures from the 3MF ZIP archive
    fn load_displacement_textures(model: &Model, file_path: &Path) -> HashMap<usize, GrayImage> {
        let mut textures = HashMap::new();

        if model.resources.displacement_maps.is_empty() {
            return textures;
        }

        let file = match std::fs::File::open(file_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!(
                    "Warning: Cannot open 3MF for displacement texture loading: {}",
                    e
                );
                return textures;
            }
        };

        let mut archive = match zip::ZipArchive::new(file) {
            Ok(a) => a,
            Err(e) => {
                eprintln!(
                    "Warning: Cannot open 3MF as ZIP for displacement textures: {}",
                    e
                );
                return textures;
            }
        };

        for disp_map in &model.resources.displacement_maps {
            let normalized = disp_map.path.trim_start_matches('/');

            let image_data = {
                let mut buf = Vec::new();
                let mut found = false;

                if let Ok(mut entry) = archive.by_name(normalized)
                    && std::io::copy(&mut entry, &mut buf).is_ok()
                {
                    found = true;
                }

                if !found {
                    eprintln!(
                        "Warning: Displacement texture not found in archive: {}",
                        disp_map.path
                    );
                    continue;
                }

                buf
            };

            // Decode the image
            match image::load_from_memory(&image_data) {
                Ok(img) => {
                    // Convert to grayscale and store
                    let gray = img.to_luma8();
                    textures.insert(disp_map.id, gray);
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to decode displacement texture {}: {}",
                        disp_map.path, e
                    );
                }
            }
        }

        println!("  Loaded {} displacement texture(s)", textures.len());

        textures
    }
}

/// Apply tile style to a UV coordinate
fn apply_tile_style(coord: f32, tile_style: lib3mf::TileStyle) -> f32 {
    use lib3mf::TileStyle;

    match tile_style {
        TileStyle::Wrap => coord.rem_euclid(1.0),
        TileStyle::Mirror => {
            let wrapped = coord.rem_euclid(2.0);
            if wrapped > 1.0 {
                2.0 - wrapped
            } else {
                wrapped
            }
        }
        TileStyle::Clamp => coord.clamp(0.0, 1.0),
        TileStyle::None => coord, // Return as-is, caller will check bounds
    }
}
