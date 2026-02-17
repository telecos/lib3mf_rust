//! Color resolution module for the slicer
//!
//! Resolves per-vertex colors from 3MF material properties (ColorGroup,
//! BaseMaterialGroup, Texture2DGroup) and loads texture images from the
//! 3MF ZIP package.

use image::RgbaImage;
use lib3mf::{Model, Triangle};
use std::collections::HashMap;
use std::path::Path;

/// RGBA color as [r, g, b, a]
pub type Rgba = [u8; 4];

/// Default color when no material is assigned (mid-gray, fully opaque)
pub const DEFAULT_COLOR: Rgba = [128, 128, 128, 255];

/// Information about a property group
enum PropertyGroup {
    /// Direct color lookup
    Color { colors: Vec<Rgba> },
    /// Base material with display colors
    BaseMaterial { colors: Vec<Rgba> },
    /// Texture-mapped group: UV coords + texture image reference
    Texture2D {
        texid: usize,
        coords: Vec<(f32, f32)>,
    },
}

/// Resolves triangle material properties to RGBA colors.
///
/// Handles all three property group types:
/// - `ColorGroup`: direct per-vertex colors
/// - `BaseMaterialGroup`: uniform face color from display color
/// - `Texture2DGroup`: UV-mapped texture sampling
pub struct ColorResolver {
    /// Maps property group ID → property group info
    groups: HashMap<usize, PropertyGroup>,
    /// Loaded texture images: texture resource ID → decoded RGBA image
    textures: HashMap<usize, RgbaImage>,
}

impl ColorResolver {
    /// Build a ColorResolver from the parsed model and the 3MF file path.
    ///
    /// This opens the ZIP archive a second time to read texture image data.
    pub fn from_model(model: &Model, file_path: &Path) -> Self {
        let mut groups = HashMap::new();

        // Index ColorGroups
        for cg in &model.resources.color_groups {
            groups.insert(
                cg.id,
                PropertyGroup::Color {
                    colors: cg.colors.iter().map(|&(r, g, b, a)| [r, g, b, a]).collect(),
                },
            );
        }

        // Index BaseMaterialGroups
        for bmg in &model.resources.base_material_groups {
            groups.insert(
                bmg.id,
                PropertyGroup::BaseMaterial {
                    colors: bmg
                        .materials
                        .iter()
                        .map(|m| {
                            let (r, g, b, a) = m.displaycolor;
                            [r, g, b, a]
                        })
                        .collect(),
                },
            );
        }

        // Index Texture2DGroups
        for tg in &model.resources.texture2d_groups {
            groups.insert(
                tg.id,
                PropertyGroup::Texture2D {
                    texid: tg.texid,
                    coords: tg.tex2coords.iter().map(|tc| (tc.u, tc.v)).collect(),
                },
            );
        }

        // Load texture images from the ZIP archive
        let textures = Self::load_textures(model, file_path);

        Self { groups, textures }
    }

    /// Returns true if any property groups were found in the model.
    pub fn has_colors(&self) -> bool {
        !self.groups.is_empty()
    }

    /// Resolve a single property index within a group to an RGBA color.
    pub fn resolve_color(&self, pid: usize, prop_index: usize) -> Rgba {
        match self.groups.get(&pid) {
            Some(PropertyGroup::Color { colors }) => {
                colors.get(prop_index).copied().unwrap_or(DEFAULT_COLOR)
            }
            Some(PropertyGroup::BaseMaterial { colors }) => {
                colors.get(prop_index).copied().unwrap_or(DEFAULT_COLOR)
            }
            Some(PropertyGroup::Texture2D { texid, coords }) => {
                if let Some((u, v)) = coords.get(prop_index) {
                    self.sample_texture(*texid, *u, *v)
                } else {
                    DEFAULT_COLOR
                }
            }
            None => DEFAULT_COLOR,
        }
    }

    /// Resolve the three per-vertex colors for a triangle.
    ///
    /// Uses the 3MF property inheritance rules:
    /// - `pid` from triangle, falling back to object-level `pid`
    /// - `p1` from triangle, falling back to `pindex`, then object-level `pindex`
    /// - `p2` falls back to effective `p1`
    /// - `p3` falls back to effective `p1`
    pub fn resolve_triangle_colors(
        &self,
        tri: &Triangle,
        object_pid: Option<usize>,
        object_pindex: Option<usize>,
    ) -> Option<(Rgba, Rgba, Rgba)> {
        // Determine effective pid
        let pid = tri.pid.or(object_pid)?;

        // Determine effective property indices
        let p1 = tri.p1.or(tri.pindex).or(object_pindex).unwrap_or(0);
        let p2 = tri.p2.unwrap_or(p1);
        let p3 = tri.p3.unwrap_or(p1);

        let c1 = self.resolve_color(pid, p1);
        let c2 = self.resolve_color(pid, p2);
        let c3 = self.resolve_color(pid, p3);

        Some((c1, c2, c3))
    }

    /// Sample a texture at the given UV coordinates.
    fn sample_texture(&self, texid: usize, u: f32, v: f32) -> Rgba {
        if let Some(img) = self.textures.get(&texid) {
            let w = img.width() as f32;
            let h = img.height() as f32;

            // Wrap UV coordinates (3MF default tile style)
            let u_wrapped = u.rem_euclid(1.0);
            // 3MF: V=0 is bottom, image y=0 is top → flip
            let v_wrapped = 1.0 - v.rem_euclid(1.0);

            let px = ((u_wrapped * w) as u32).min(img.width().saturating_sub(1));
            let py = ((v_wrapped * h) as u32).min(img.height().saturating_sub(1));

            let pixel = img.get_pixel(px, py);
            [pixel[0], pixel[1], pixel[2], pixel[3]]
        } else {
            DEFAULT_COLOR
        }
    }

    /// Load texture images from the 3MF ZIP archive.
    fn load_textures(model: &Model, file_path: &Path) -> HashMap<usize, RgbaImage> {
        let mut textures = HashMap::new();

        if model.resources.texture2d_resources.is_empty() {
            return textures;
        }

        let file = match std::fs::File::open(file_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Warning: Cannot open 3MF for texture loading: {}", e);
                return textures;
            }
        };

        let mut archive = match zip::ZipArchive::new(file) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Warning: Cannot open 3MF as ZIP for textures: {}", e);
                return textures;
            }
        };

        for tex in &model.resources.texture2d_resources {
            let normalized = tex.path.trim_start_matches('/');

            let image_data = {
                let mut buf = Vec::new();
                let mut found = false;

                if let Ok(mut entry) = archive.by_name(normalized)
                    && std::io::Read::read_to_end(&mut entry, &mut buf).is_ok()
                {
                    found = true;
                }

                if !found {
                    buf.clear();
                    if let Ok(mut entry) = archive.by_name(&tex.path)
                        && std::io::Read::read_to_end(&mut entry, &mut buf).is_ok()
                    {
                        found = true;
                    }
                }

                if !found {
                    eprintln!("Warning: Texture '{}' not found in 3MF package", tex.path);
                    continue;
                }
                buf
            };

            match image::load_from_memory(&image_data) {
                Ok(img) => {
                    println!(
                        "  Loaded texture ID {}: {} ({}x{})",
                        tex.id,
                        tex.path,
                        img.width(),
                        img.height()
                    );
                    textures.insert(tex.id, img.to_rgba8());
                }
                Err(e) => {
                    eprintln!("Warning: Failed to decode texture '{}': {}", tex.path, e);
                }
            }
        }

        textures
    }
}

/// Linearly interpolate between two RGBA colors.
pub fn lerp_color(c0: Rgba, c1: Rgba, t: f64) -> Rgba {
    let t = t.clamp(0.0, 1.0) as f32;
    let inv = 1.0 - t;
    [
        (c0[0] as f32 * inv + c1[0] as f32 * t).round() as u8,
        (c0[1] as f32 * inv + c1[1] as f32 * t).round() as u8,
        (c0[2] as f32 * inv + c1[2] as f32 * t).round() as u8,
        (c0[3] as f32 * inv + c1[3] as f32 * t).round() as u8,
    ]
}
