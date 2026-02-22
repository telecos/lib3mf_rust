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

/// Default color when no material is assigned (black, fully opaque)
pub const DEFAULT_COLOR: Rgba = [0, 0, 0, 255];

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

#[cfg(test)]
mod tests {
    use super::*;
    use lib3mf::{BaseMaterial, BaseMaterialGroup, ColorGroup, Model, Tex2Coord, Texture2DGroup};
    use std::path::Path;

    // --- lerp_color tests ---

    #[test]
    fn test_lerp_color_at_zero() {
        let c0: Rgba = [255, 0, 0, 255];
        let c1: Rgba = [0, 255, 0, 128];
        assert_eq!(lerp_color(c0, c1, 0.0), c0);
    }

    #[test]
    fn test_lerp_color_at_one() {
        let c0: Rgba = [255, 0, 0, 255];
        let c1: Rgba = [0, 255, 0, 128];
        assert_eq!(lerp_color(c0, c1, 1.0), c1);
    }

    #[test]
    fn test_lerp_color_midpoint() {
        let c0: Rgba = [0, 0, 0, 0];
        let c1: Rgba = [200, 100, 50, 255];
        let result = lerp_color(c0, c1, 0.5);
        assert_eq!(result[0], 100);
        assert_eq!(result[1], 50);
        assert_eq!(result[2], 25);
        assert_eq!(result[3], 128);
    }

    #[test]
    fn test_lerp_color_clamp_below_zero() {
        let c0: Rgba = [100, 100, 100, 100];
        let c1: Rgba = [200, 200, 200, 200];
        // t < 0.0 should be clamped to 0.0 → result == c0
        assert_eq!(lerp_color(c0, c1, -1.0), c0);
    }

    #[test]
    fn test_lerp_color_clamp_above_one() {
        let c0: Rgba = [100, 100, 100, 100];
        let c1: Rgba = [200, 200, 200, 200];
        // t > 1.0 should be clamped to 1.0 → result == c1
        assert_eq!(lerp_color(c0, c1, 2.0), c1);
    }

    // --- ColorResolver tests ---

    fn make_model_with_color_group() -> Model {
        let mut model = Model::default();
        let mut cg = ColorGroup::new(1);
        cg.colors = vec![(255, 0, 0, 255), (0, 255, 0, 255), (0, 0, 255, 255)];
        model.resources.color_groups.push(cg);
        model
    }

    fn make_model_with_base_material_group() -> Model {
        let mut model = Model::default();
        let mut bmg = BaseMaterialGroup::new(2);
        bmg.materials = vec![
            BaseMaterial {
                name: "Red".to_string(),
                displaycolor: (200, 0, 0, 255),
            },
            BaseMaterial {
                name: "Blue".to_string(),
                displaycolor: (0, 0, 200, 255),
            },
        ];
        model.resources.base_material_groups.push(bmg);
        model
    }

    fn make_model_with_texture_group() -> Model {
        let mut model = Model::default();
        let mut tg = Texture2DGroup::new(3, 10);
        tg.tex2coords = vec![Tex2Coord { u: 0.0, v: 0.0 }, Tex2Coord { u: 1.0, v: 0.0 }];
        model.resources.texture2d_groups.push(tg);
        model
    }

    #[test]
    fn test_has_colors_empty_model() {
        let model = Model::default();
        let resolver = ColorResolver::from_model(&model, Path::new("/nonexistent.3mf"));
        assert!(!resolver.has_colors());
    }

    #[test]
    fn test_has_colors_with_color_group() {
        let model = make_model_with_color_group();
        let resolver = ColorResolver::from_model(&model, Path::new("/nonexistent.3mf"));
        assert!(resolver.has_colors());
    }

    #[test]
    fn test_resolve_color_from_color_group() {
        let model = make_model_with_color_group();
        let resolver = ColorResolver::from_model(&model, Path::new("/nonexistent.3mf"));
        // Index 0 → red
        assert_eq!(resolver.resolve_color(1, 0), [255, 0, 0, 255]);
        // Index 1 → green
        assert_eq!(resolver.resolve_color(1, 1), [0, 255, 0, 255]);
        // Index 2 → blue
        assert_eq!(resolver.resolve_color(1, 2), [0, 0, 255, 255]);
    }

    #[test]
    fn test_resolve_color_out_of_bounds_returns_default() {
        let model = make_model_with_color_group();
        let resolver = ColorResolver::from_model(&model, Path::new("/nonexistent.3mf"));
        assert_eq!(resolver.resolve_color(1, 99), DEFAULT_COLOR);
    }

    #[test]
    fn test_resolve_color_unknown_pid_returns_default() {
        let model = make_model_with_color_group();
        let resolver = ColorResolver::from_model(&model, Path::new("/nonexistent.3mf"));
        assert_eq!(resolver.resolve_color(99, 0), DEFAULT_COLOR);
    }

    #[test]
    fn test_resolve_color_from_base_material_group() {
        let model = make_model_with_base_material_group();
        let resolver = ColorResolver::from_model(&model, Path::new("/nonexistent.3mf"));
        assert_eq!(resolver.resolve_color(2, 0), [200, 0, 0, 255]);
        assert_eq!(resolver.resolve_color(2, 1), [0, 0, 200, 255]);
    }

    #[test]
    fn test_resolve_color_from_texture_group_no_texture() {
        // Texture group but no actual texture loaded → falls back to DEFAULT_COLOR
        let model = make_model_with_texture_group();
        let resolver = ColorResolver::from_model(&model, Path::new("/nonexistent.3mf"));
        assert_eq!(resolver.resolve_color(3, 0), DEFAULT_COLOR);
    }

    #[test]
    fn test_resolve_color_texture_group_out_of_bounds() {
        let model = make_model_with_texture_group();
        let resolver = ColorResolver::from_model(&model, Path::new("/nonexistent.3mf"));
        // Out-of-bounds coord index → DEFAULT_COLOR
        assert_eq!(resolver.resolve_color(3, 99), DEFAULT_COLOR);
    }

    #[test]
    fn test_resolve_triangle_colors_no_pid() {
        let model = make_model_with_color_group();
        let resolver = ColorResolver::from_model(&model, Path::new("/nonexistent.3mf"));
        let tri = lib3mf::Triangle::new(0, 1, 2);
        // No pid on triangle, no object pid → None
        assert!(resolver.resolve_triangle_colors(&tri, None, None).is_none());
    }

    #[test]
    fn test_resolve_triangle_colors_with_pid() {
        let model = make_model_with_color_group();
        let resolver = ColorResolver::from_model(&model, Path::new("/nonexistent.3mf"));
        let mut tri = lib3mf::Triangle::new(0, 1, 2);
        tri.pid = Some(1);
        tri.p1 = Some(0);
        tri.p2 = Some(1);
        tri.p3 = Some(2);
        let result = resolver.resolve_triangle_colors(&tri, None, None);
        assert!(result.is_some());
        let (c1, c2, c3) = result.unwrap();
        assert_eq!(c1, [255, 0, 0, 255]);
        assert_eq!(c2, [0, 255, 0, 255]);
        assert_eq!(c3, [0, 0, 255, 255]);
    }

    #[test]
    fn test_resolve_triangle_colors_fallback_to_object_pid() {
        let model = make_model_with_color_group();
        let resolver = ColorResolver::from_model(&model, Path::new("/nonexistent.3mf"));
        // Triangle has no pid, but object has pid=1, pindex=0
        let tri = lib3mf::Triangle::new(0, 1, 2);
        let result = resolver.resolve_triangle_colors(&tri, Some(1), Some(0));
        assert!(result.is_some());
        let (c1, c2, c3) = result.unwrap();
        // All fall back to index 0 (red)
        assert_eq!(c1, [255, 0, 0, 255]);
        assert_eq!(c2, [255, 0, 0, 255]);
        assert_eq!(c3, [255, 0, 0, 255]);
    }

    #[test]
    fn test_resolve_triangle_colors_p2_p3_fallback_to_p1() {
        let model = make_model_with_color_group();
        let resolver = ColorResolver::from_model(&model, Path::new("/nonexistent.3mf"));
        let mut tri = lib3mf::Triangle::new(0, 1, 2);
        tri.pid = Some(1);
        tri.p1 = Some(2); // Blue
        // p2, p3 not set → fall back to p1 (index 2)
        let result = resolver.resolve_triangle_colors(&tri, None, None);
        assert!(result.is_some());
        let (c1, c2, c3) = result.unwrap();
        assert_eq!(c1, [0, 0, 255, 255]);
        assert_eq!(c2, [0, 0, 255, 255]);
        assert_eq!(c3, [0, 0, 255, 255]);
    }
}
