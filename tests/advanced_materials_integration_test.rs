//! Integration tests for advanced materials extension
//!
//! Tests end-to-end parsing and writing of Texture2D, composite materials,
//! and multi-properties from the Materials Extension 1.2.1

use lib3mf::*;
use std::io::Cursor;

#[test]
fn test_parse_and_write_texture2d() {
    // Create a model with Texture2D resource
    let mut model = Model::new();
    model.unit = "millimeter".to_string();

    // Add a Texture2D resource
    let mut texture = Texture2D::new(
        1,
        "/3D/Textures/wood.png".to_string(),
        "image/png".to_string(),
    );
    texture.tilestyleu = TileStyle::Mirror;
    texture.tilestylev = TileStyle::Wrap;
    texture.filter = FilterMode::Linear;
    model.resources.texture2d_resources.push(texture);

    // Add a Texture2DGroup with coordinates
    let mut tex_group = Texture2DGroup::new(2, 1); // References texture ID 1
    tex_group.tex2coords.push(Tex2Coord::new(0.0, 0.0));
    tex_group.tex2coords.push(Tex2Coord::new(1.0, 0.0));
    tex_group.tex2coords.push(Tex2Coord::new(0.5, 1.0));
    model.resources.texture2d_groups.push(tex_group);

    // Create a simple mesh
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));

    let mut object = Object::new(1);
    object.mesh = Some(mesh);
    model.resources.objects.push(object);
    model.build.items.push(BuildItem::new(1));

    // Write to buffer
    let cursor = Cursor::new(Vec::new());
    let cursor = model.to_writer(cursor).expect("Failed to write model");

    // Read back
    let data = cursor.into_inner();
    let cursor = Cursor::new(data);
    let parsed_model = Model::from_reader(cursor).expect("Failed to parse model");

    // Verify Texture2D
    assert_eq!(parsed_model.resources.texture2d_resources.len(), 1);
    let parsed_texture = &parsed_model.resources.texture2d_resources[0];
    assert_eq!(parsed_texture.id, 1);
    assert_eq!(parsed_texture.path, "/3D/Textures/wood.png");
    assert_eq!(parsed_texture.contenttype, "image/png");
    assert_eq!(parsed_texture.tilestyleu, TileStyle::Mirror);
    assert_eq!(parsed_texture.tilestylev, TileStyle::Wrap);
    assert_eq!(parsed_texture.filter, FilterMode::Linear);

    // Verify Texture2DGroup
    assert_eq!(parsed_model.resources.texture2d_groups.len(), 1);
    let parsed_group = &parsed_model.resources.texture2d_groups[0];
    assert_eq!(parsed_group.id, 2);
    assert_eq!(parsed_group.texid, 1);
    assert_eq!(parsed_group.tex2coords.len(), 3);
    assert_eq!(parsed_group.tex2coords[0].u, 0.0);
    assert_eq!(parsed_group.tex2coords[0].v, 0.0);
    assert_eq!(parsed_group.tex2coords[1].u, 1.0);
    assert_eq!(parsed_group.tex2coords[2].v, 1.0);
}

#[test]
fn test_parse_and_write_composite_materials() {
    // Create a model with composite materials
    let mut model = Model::new();
    model.unit = "millimeter".to_string();

    // Add base materials group
    let mut base_group = BaseMaterialGroup::new(1);
    base_group
        .materials
        .push(BaseMaterial::new("Red".to_string(), (255, 0, 0, 255)));
    base_group
        .materials
        .push(BaseMaterial::new("Blue".to_string(), (0, 0, 255, 255)));
    base_group
        .materials
        .push(BaseMaterial::new("Green".to_string(), (0, 255, 0, 255)));
    model.resources.base_material_groups.push(base_group);

    // Add composite materials group
    let matindices = vec![0, 1, 2];
    let mut comp_group = CompositeMaterials::new(2, 1, matindices);

    // Add composite definitions (mixing ratios)
    comp_group
        .composites
        .push(Composite::new(vec![0.5, 0.3, 0.2])); // Purple-ish
    comp_group
        .composites
        .push(Composite::new(vec![0.7, 0.2, 0.1])); // More red
    comp_group
        .composites
        .push(Composite::new(vec![0.1, 0.8, 0.1])); // Mostly blue
    model.resources.composite_materials.push(comp_group);

    // Create a simple mesh
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));

    let mut object = Object::new(1);
    object.mesh = Some(mesh);
    model.resources.objects.push(object);
    model.build.items.push(BuildItem::new(1));

    // Write to buffer
    let cursor = Cursor::new(Vec::new());
    let cursor = model.to_writer(cursor).expect("Failed to write model");

    // Read back
    let data = cursor.into_inner();
    let cursor = Cursor::new(data);
    let parsed_model = Model::from_reader(cursor).expect("Failed to parse model");

    // Verify base materials
    assert_eq!(parsed_model.resources.base_material_groups.len(), 1);
    let parsed_base = &parsed_model.resources.base_material_groups[0];
    assert_eq!(parsed_base.id, 1);
    assert_eq!(parsed_base.materials.len(), 3);

    // Verify composite materials
    assert_eq!(parsed_model.resources.composite_materials.len(), 1);
    let parsed_comp = &parsed_model.resources.composite_materials[0];
    assert_eq!(parsed_comp.id, 2);
    assert_eq!(parsed_comp.matid, 1);
    assert_eq!(parsed_comp.matindices, vec![0, 1, 2]);
    assert_eq!(parsed_comp.composites.len(), 3);

    // Verify composite values
    assert_eq!(parsed_comp.composites[0].values, vec![0.5, 0.3, 0.2]);
    assert_eq!(parsed_comp.composites[1].values, vec![0.7, 0.2, 0.1]);
    assert_eq!(parsed_comp.composites[2].values, vec![0.1, 0.8, 0.1]);
}

#[test]
fn test_parse_and_write_multi_properties() {
    // Create a model with multi-properties
    let mut model = Model::new();
    model.unit = "millimeter".to_string();

    // Add color groups that will be layered
    let mut color_group1 = ColorGroup::new(1);
    color_group1.colors.push((255, 0, 0, 255)); // Red
    color_group1.colors.push((0, 255, 0, 255)); // Green
    model.resources.color_groups.push(color_group1);

    let mut color_group2 = ColorGroup::new(2);
    color_group2.colors.push((0, 0, 255, 128)); // Semi-transparent blue
    color_group2.colors.push((255, 255, 0, 128)); // Semi-transparent yellow
    model.resources.color_groups.push(color_group2);

    // Add multi-properties group
    let mut multi = MultiProperties::new(3, vec![1, 2]);
    multi.blendmethods.push(BlendMethod::Mix);

    // Add multi elements (combinations of property indices)
    multi.multis.push(Multi::new(vec![0, 0])); // First from each
    multi.multis.push(Multi::new(vec![1, 1])); // Second from each
    multi.multis.push(Multi::new(vec![0, 1])); // Mixed indices
    model.resources.multi_properties.push(multi);

    // Create a simple mesh
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));

    let mut object = Object::new(1);
    object.mesh = Some(mesh);
    model.resources.objects.push(object);
    model.build.items.push(BuildItem::new(1));

    // Write to buffer
    let cursor = Cursor::new(Vec::new());
    let cursor = model.to_writer(cursor).expect("Failed to write model");

    // Read back
    let data = cursor.into_inner();
    let cursor = Cursor::new(data);
    let parsed_model = Model::from_reader(cursor).expect("Failed to parse model");

    // Verify color groups
    assert_eq!(parsed_model.resources.color_groups.len(), 2);

    // Verify multi-properties
    assert_eq!(parsed_model.resources.multi_properties.len(), 1);
    let parsed_multi = &parsed_model.resources.multi_properties[0];
    assert_eq!(parsed_multi.id, 3);
    assert_eq!(parsed_multi.pids, vec![1, 2]);
    assert_eq!(parsed_multi.blendmethods.len(), 1);
    assert_eq!(parsed_multi.blendmethods[0], BlendMethod::Mix);
    assert_eq!(parsed_multi.multis.len(), 3);

    // Verify multi elements
    assert_eq!(parsed_multi.multis[0].pindices, vec![0, 0]);
    assert_eq!(parsed_multi.multis[1].pindices, vec![1, 1]);
    assert_eq!(parsed_multi.multis[2].pindices, vec![0, 1]);
}
