//! Test advanced materials data structures

#[cfg(test)]
mod tests {
    use lib3mf::*;

    #[test]
    fn test_texture2d_creation() {
        let texture = Texture2D::new(1, "texture.png".to_string(), "image/png".to_string());
        assert_eq!(texture.id, 1);
        assert_eq!(texture.path, "texture.png");
        assert_eq!(texture.contenttype, "image/png");
        assert_eq!(texture.tilestyleu, TileStyle::Wrap);
        assert_eq!(texture.tilestylev, TileStyle::Wrap);
        assert_eq!(texture.filter, FilterMode::Auto);
    }

    #[test]
    fn test_texture2d_group() {
        let mut group = Texture2DGroup::new(1, 5);
        assert_eq!(group.id, 1);
        assert_eq!(group.texid, 5);
        assert_eq!(group.tex2coords.len(), 0);

        group.tex2coords.push(Tex2Coord::new(0.0, 0.0));
        group.tex2coords.push(Tex2Coord::new(1.0, 1.0));
        assert_eq!(group.tex2coords.len(), 2);
    }

    #[test]
    fn test_composite_materials() {
        let matindices = vec![0, 1, 2];
        let mut comp = CompositeMaterials::new(1, 10, matindices.clone());
        assert_eq!(comp.id, 1);
        assert_eq!(comp.matid, 10);
        assert_eq!(comp.matindices, matindices);

        comp.composites.push(Composite::new(vec![0.5, 0.3, 0.2]));
        assert_eq!(comp.composites.len(), 1);
    }

    #[test]
    fn test_multi_properties() {
        let pids = vec![1, 2, 3];
        let mut multi = MultiProperties::new(1, pids.clone());
        assert_eq!(multi.id, 1);
        assert_eq!(multi.pids, pids);

        multi.blendmethods.push(BlendMethod::Mix);
        multi.blendmethods.push(BlendMethod::Multiply);
        assert_eq!(multi.blendmethods.len(), 2);

        multi.multis.push(Multi::new(vec![0, 1, 2]));
        assert_eq!(multi.multis.len(), 1);
    }

    #[test]
    fn test_resources_advanced_materials() {
        let resources = Resources::new();
        assert_eq!(resources.texture2d_resources.len(), 0);
        assert_eq!(resources.texture2d_groups.len(), 0);
        assert_eq!(resources.composite_materials.len(), 0);
        assert_eq!(resources.multi_properties.len(), 0);
    }
}
