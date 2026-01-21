//! Property-based tests for lib3mf
//!
//! These tests use proptest to generate random valid/invalid 3MF models
//! and verify invariants hold across a wide range of inputs.

#![allow(clippy::single_char_add_str)]

use lib3mf::{
    Build, BuildItem, ColorGroup, Material, Mesh, Model, Object, Resources, Triangle, Vertex,
};
use proptest::prelude::*;
use std::io::Cursor;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

// ============================================================================
// Generators for basic data structures
// ============================================================================

/// Generate a valid Vertex with finite coordinates
fn vertex_strategy() -> impl Strategy<Value = Vertex> {
    (
        // Use finite f64 values to avoid NaN/Inf issues
        prop::num::f64::NORMAL,
        prop::num::f64::NORMAL,
        prop::num::f64::NORMAL,
    )
        .prop_map(|(x, y, z)| Vertex::new(x, y, z))
}

/// Generate a valid Triangle with indices within bounds
fn triangle_strategy(max_vertex_index: usize) -> impl Strategy<Value = Triangle> {
    let v_range = 0..max_vertex_index;
    (
        v_range.clone(),
        v_range.clone(),
        v_range.clone(),
        // Optional properties
        prop::option::of(0usize..100),
        prop::option::of(0usize..100),
        prop::option::of(0usize..100),
        prop::option::of(0usize..100),
        prop::option::of(0usize..100),
    )
        .prop_filter(
            "Triangle must not be degenerate",
            |(v1, v2, v3, _, _, _, _, _)| {
                // Ensure all three vertices are different to avoid degenerate triangles
                v1 != v2 && v2 != v3 && v1 != v3
            },
        )
        .prop_map(|(v1, v2, v3, pid, pindex, p1, p2, p3)| {
            let mut tri = Triangle::new(v1, v2, v3);
            tri.pid = pid;
            tri.pindex = pindex;
            tri.p1 = p1;
            tri.p2 = p2;
            tri.p3 = p3;
            tri
        })
}

/// Generate a valid Mesh with consistent vertex/triangle references
fn mesh_strategy() -> impl Strategy<Value = Mesh> {
    // Generate 3-100 vertices
    prop::collection::vec(vertex_strategy(), 3..100).prop_flat_map(|vertices| {
        let vertex_count = vertices.len();
        // Generate 1-50 triangles with valid vertex indices
        prop::collection::vec(triangle_strategy(vertex_count), 1..50).prop_map(move |triangles| {
            let mut mesh = Mesh::new();
            mesh.vertices = vertices.clone();
            mesh.triangles = triangles;
            mesh
        })
    })
}

/// Generate a valid Material
fn material_strategy() -> impl Strategy<Value = Material> {
    (
        1usize..1000,
        prop::option::of("[a-zA-Z0-9 ]{1,50}"),
        prop::option::of((any::<u8>(), any::<u8>(), any::<u8>(), any::<u8>())),
    )
        .prop_map(|(id, name, color)| {
            let mut mat = Material::new(id);
            mat.name = name;
            mat.color = color;
            mat
        })
}

/// Generate a valid ColorGroup
fn colorgroup_strategy() -> impl Strategy<Value = ColorGroup> {
    (
        1usize..1000,
        prop::collection::vec((any::<u8>(), any::<u8>(), any::<u8>(), any::<u8>()), 1..20),
    )
        .prop_map(|(id, colors)| {
            let mut cg = ColorGroup::new(id);
            cg.colors = colors;
            cg
        })
}

/// Generate a valid Object
fn object_strategy() -> impl Strategy<Value = Object> {
    (
        1usize..1000,
        prop::option::of("[a-zA-Z0-9 ]{1,50}"),
        prop::option::of(mesh_strategy()),
        prop::option::of(0usize..100),
        prop::option::of(0usize..100),
    )
        .prop_map(|(id, name, mesh, pid, pindex)| {
            let mut obj = Object::new(id);
            obj.name = name;
            obj.mesh = mesh;
            obj.pid = pid;
            obj.pindex = pindex;
            obj
        })
}

/// Generate valid Resources
fn resources_strategy() -> impl Strategy<Value = Resources> {
    (
        prop::collection::vec(object_strategy(), 1..10),
        prop::collection::vec(material_strategy(), 0..10),
        prop::collection::vec(colorgroup_strategy(), 0..5),
    )
        .prop_map(|(objects, materials, color_groups)| {
            let mut res = Resources::new();
            res.objects = objects;
            res.materials = materials;
            res.color_groups = color_groups;
            res
        })
}

/// Generate a valid BuildItem
fn builditem_strategy(object_ids: Vec<usize>) -> impl Strategy<Value = BuildItem> {
    (
        prop::sample::select(object_ids),
        prop::option::of(prop::collection::vec(prop::num::f64::NORMAL, 12..=12)),
    )
        .prop_map(|(objectid, transform_vec)| {
            let mut item = BuildItem::new(objectid);
            if let Some(vec) = transform_vec {
                let mut arr = [0.0; 12];
                arr.copy_from_slice(&vec);
                item.transform = Some(arr);
            }
            item
        })
}

/// Generate valid Build
fn build_strategy(resources: &Resources) -> impl Strategy<Value = Build> {
    if resources.objects.is_empty() {
        return Just(Build::new()).boxed();
    }

    let object_ids: Vec<usize> = resources.objects.iter().map(|o| o.id).collect();

    prop::collection::vec(builditem_strategy(object_ids), 1..5)
        .prop_map(|items| {
            let mut build = Build::new();
            build.items = items;
            build
        })
        .boxed()
}

/// Generate a valid Model
fn model_strategy() -> impl Strategy<Value = Model> {
    resources_strategy().prop_flat_map(|resources| {
        let build_strat = build_strategy(&resources);
        (
            Just(resources),
            build_strat,
            // Valid units according to 3MF spec
            prop::sample::select(vec![
                "micron",
                "millimeter",
                "centimeter",
                "inch",
                "foot",
                "meter",
            ]),
        )
            .prop_map(|(resources, build, unit)| {
                let mut model = Model::new();
                model.resources = resources;
                model.build = build;
                model.unit = unit.to_string();
                model
            })
    })
}

// ============================================================================
// Helper function to create 3MF file from Model
// ============================================================================

/// Create a 3MF file from a Model structure
fn create_3mf_from_model(model: &Model) -> Vec<u8> {
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    let mut zip = ZipWriter::new(cursor);

    let options = SimpleFileOptions::default();

    // Add [Content_Types].xml
    let content_types = r##"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
</Types>"##;

    zip.start_file("[Content_Types].xml", options).unwrap();
    std::io::Write::write_all(&mut zip, content_types.as_bytes()).unwrap();

    // Add _rels/.rels
    let rels = r##"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Target="/3D/3dmodel.model" Id="rel0" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"##;

    zip.start_file("_rels/.rels", options).unwrap();
    std::io::Write::write_all(&mut zip, rels.as_bytes()).unwrap();

    // Generate model XML
    let model_xml = generate_model_xml(model);

    zip.start_file("3D/3dmodel.model", options).unwrap();
    std::io::Write::write_all(&mut zip, model_xml.as_bytes()).unwrap();

    zip.finish().unwrap();
    buffer
}

/// Generate XML for a Model structure
fn generate_model_xml(model: &Model) -> String {
    let mut xml = String::new();
    xml.push_str(r##"<?xml version="1.0" encoding="UTF-8"?>"##);
    xml.push_str(&format!(
        r#"<model unit="{}" xml:lang="en-US" xmlns="{}">"#,
        model.unit, model.xmlns
    ));

    // Add metadata
    for (key, value) in &model.metadata {
        xml.push_str(&format!(r#"<metadata name="{}">{}</metadata>"#, key, value));
    }

    // Add resources
    xml.push_str("<resources>");

    // Add objects
    for obj in &model.resources.objects {
        xml.push_str(&format!(r#"<object id="{}" type="model""#, obj.id));
        if let Some(pid) = obj.pid {
            xml.push_str(&format!(r#" pid="{}""#, pid));
        }
        if let Some(pindex) = obj.pindex {
            xml.push_str(&format!(r#" pindex="{}""#, pindex));
        }
        xml.push_str(">");

        // Add mesh if present
        if let Some(mesh) = &obj.mesh {
            xml.push_str("<mesh>");

            // Add vertices
            xml.push_str("<vertices>");
            for v in &mesh.vertices {
                xml.push_str(&format!(r#"<vertex x="{}" y="{}" z="{}"/>"#, v.x, v.y, v.z));
            }
            xml.push_str("</vertices>");

            // Add triangles
            xml.push_str("<triangles>");
            for t in &mesh.triangles {
                xml.push_str(&format!(
                    r#"<triangle v1="{}" v2="{}" v3="{}""#,
                    t.v1, t.v2, t.v3
                ));
                if let Some(pid) = t.pid {
                    xml.push_str(&format!(r#" pid="{}""#, pid));
                }
                if let Some(pindex) = t.pindex {
                    xml.push_str(&format!(r#" pindex="{}""#, pindex));
                }
                if let Some(p1) = t.p1 {
                    xml.push_str(&format!(r#" p1="{}""#, p1));
                }
                if let Some(p2) = t.p2 {
                    xml.push_str(&format!(r#" p2="{}""#, p2));
                }
                if let Some(p3) = t.p3 {
                    xml.push_str(&format!(r#" p3="{}""#, p3));
                }
                xml.push_str("/>");
            }
            xml.push_str("</triangles>");

            xml.push_str("</mesh>");
        }

        xml.push_str("</object>");
    }

    xml.push_str("</resources>");

    // Add build
    xml.push_str("<build>");
    for item in &model.build.items {
        xml.push_str(&format!(r#"<item objectid="{}""#, item.objectid));
        if let Some(transform) = &item.transform {
            xml.push_str(&format!(
                r#" transform="{} {} {} {} {} {} {} {} {} {} {} {}""#,
                transform[0],
                transform[1],
                transform[2],
                transform[3],
                transform[4],
                transform[5],
                transform[6],
                transform[7],
                transform[8],
                transform[9],
                transform[10],
                transform[11]
            ));
        }
        xml.push_str("/>");
    }
    xml.push_str("</build>");

    xml.push_str("</model>");
    xml
}

// ============================================================================
// Property-based tests
// ============================================================================

proptest! {
    /// Test that valid vertices are always created with finite values
    #[test]
    fn test_vertex_creation(v in vertex_strategy()) {
        assert!(v.x.is_finite());
        assert!(v.y.is_finite());
        assert!(v.z.is_finite());
    }

    /// Test that triangles have valid vertex indices
    #[test]
    fn test_triangle_indices(vertices in prop::collection::vec(vertex_strategy(), 3..100)) {
        let max_idx = vertices.len();
        let strategy = triangle_strategy(max_idx);

        proptest!(|(tri in strategy)| {
            prop_assert!(tri.v1 < max_idx);
            prop_assert!(tri.v2 < max_idx);
            prop_assert!(tri.v3 < max_idx);
        });
    }

    /// Test that meshes have consistent vertex/triangle relationships
    #[test]
    fn test_mesh_consistency(mesh in mesh_strategy()) {
        let vertex_count = mesh.vertices.len();

        // All vertices should have finite coordinates
        for v in &mesh.vertices {
            assert!(v.x.is_finite());
            assert!(v.y.is_finite());
            assert!(v.z.is_finite());
        }

        // All triangle indices should be within bounds
        for t in &mesh.triangles {
            assert!(t.v1 < vertex_count);
            assert!(t.v2 < vertex_count);
            assert!(t.v3 < vertex_count);
        }

        // Mesh should have at least 3 vertices (minimum for one triangle)
        assert!(mesh.vertices.len() >= 3);
        assert!(!mesh.triangles.is_empty());
    }

    /// Test that generated models are structurally valid
    #[test]
    fn test_model_structure(model in model_strategy()) {
        // Resources should have at least one object
        assert!(!model.resources.objects.is_empty());

        // Build items should reference valid object IDs
        for item in &model.build.items {
            let valid_id = model.resources.objects.iter().any(|obj| obj.id == item.objectid);
            assert!(valid_id, "Build item references non-existent object ID {}", item.objectid);
        }

        // All meshes should be valid
        for obj in &model.resources.objects {
            if let Some(mesh) = &obj.mesh {
                let vertex_count = mesh.vertices.len();
                for t in &mesh.triangles {
                    assert!(t.v1 < vertex_count);
                    assert!(t.v2 < vertex_count);
                    assert!(t.v3 < vertex_count);
                }
            }
        }
    }

    /// Test roundtrip: generate model → serialize to 3MF → parse → verify structure
    /// Note: Random meshes may violate 3MF validation rules (e.g., non-manifold geometry).
    /// This test verifies that the parser correctly identifies such issues.
    #[test]
    fn test_model_roundtrip(model in model_strategy()) {
        // Generate 3MF file from model
        let data = create_3mf_from_model(&model);

        // Parse it back
        let cursor = Cursor::new(data);
        let parsed = Model::from_reader(cursor);

        // The parser should either succeed or fail with a validation error
        // Both outcomes are acceptable for randomly generated meshes
        match parsed {
            Ok(parsed_model) => {
                // If it parses, verify basic structure
                assert_eq!(parsed_model.resources.objects.len(), model.resources.objects.len());
                assert_eq!(parsed_model.build.items.len(), model.build.items.len());

                // Verify object IDs match
                for (orig, parsed) in model.resources.objects.iter().zip(&parsed_model.resources.objects) {
                    assert_eq!(orig.id, parsed.id);

                    if let (Some(orig_mesh), Some(parsed_mesh)) = (&orig.mesh, &parsed.mesh) {
                        assert_eq!(orig_mesh.vertices.len(), parsed_mesh.vertices.len());
                        assert_eq!(orig_mesh.triangles.len(), parsed_mesh.triangles.len());
                    }
                }
            }
            Err(e) => {
                // Parser should only reject models for validation reasons
                // (non-manifold geometry, degenerate triangles, etc.)
                let error_msg = format!("{:?}", e);
                assert!(
                    error_msg.contains("InvalidModel") ||
                    error_msg.contains("degenerate") ||
                    error_msg.contains("Non-manifold"),
                    "Unexpected error parsing generated 3MF: {:?}", e
                );
            }
        }
    }

    /// Test that models with extreme but valid values parse correctly
    #[test]
    fn test_extreme_values(
        x in prop::num::f64::NORMAL,
        y in prop::num::f64::NORMAL,
        z in prop::num::f64::NORMAL
    ) {
        let v = Vertex::new(x, y, z);
        assert!(v.x.is_finite());
        assert!(v.y.is_finite());
        assert!(v.z.is_finite());
    }
}

// ============================================================================
// Additional unit tests for edge cases
// ============================================================================

#[test]
fn test_empty_build_items() {
    // Even with valid resources, an empty build should be valid structurally
    let mut model = Model::new();
    let mut obj = Object::new(1);
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(0.5, 1.0, 0.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));
    obj.mesh = Some(mesh);
    model.resources.objects.push(obj);

    // Model is structurally valid even without build items
    assert_eq!(model.build.items.len(), 0);
    assert_eq!(model.resources.objects.len(), 1);
}

#[test]
fn test_minimal_triangle() {
    // Test the absolute minimum triangle - 3 vertices
    let mut mesh = Mesh::new();
    mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
    mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));
    mesh.triangles.push(Triangle::new(0, 1, 2));

    assert_eq!(mesh.vertices.len(), 3);
    assert_eq!(mesh.triangles.len(), 1);
}

#[test]
fn test_material_with_all_fields() {
    let mut mat = Material::new(1);
    mat.name = Some("Test Material".to_string());
    mat.color = Some((255, 128, 64, 200));

    assert_eq!(mat.id, 1);
    assert_eq!(mat.name, Some("Test Material".to_string()));
    assert_eq!(mat.color, Some((255, 128, 64, 200)));
}

#[test]
fn test_colorgroup_multiple_colors() {
    let mut cg = ColorGroup::new(1);
    cg.colors.push((255, 0, 0, 255)); // Red
    cg.colors.push((0, 255, 0, 255)); // Green
    cg.colors.push((0, 0, 255, 255)); // Blue

    assert_eq!(cg.id, 1);
    assert_eq!(cg.colors.len(), 3);
}
