//! XML writing for 3MF model files
//!
//! This module provides functionality to serialize Model structures back into
//! 3MF-compliant XML format.

mod beam_lattice;
mod boolean_ops;
mod core;
mod displacement;
mod material;
mod production;
mod volumetric;

use crate::error::{Error, Result};
use crate::model::*;
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use std::io::Write as IoWrite;

/// Write a Model to XML format
///
/// Serializes a Model struct to 3MF-compliant XML.
/// This generates the 3dmodel.model file content.
pub fn write_model_xml<W: IoWrite>(model: &Model, writer: W) -> Result<()> {
    let mut xml_writer = Writer::new_with_indent(writer, b' ', 2);

    // Write XML declaration
    xml_writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(|e| Error::xml_write(format!("Failed to write XML declaration: {}", e)))?;

    // Build model element with attributes and namespaces
    let mut model_elem = BytesStart::new("model");
    model_elem.push_attribute(("unit", model.unit.as_str()));
    model_elem.push_attribute(("xml:lang", "en-US"));
    model_elem.push_attribute(("xmlns", model.xmlns.as_str()));

    // Add extension namespaces
    let mut ns_attrs = Vec::new();
    for ext in &model.required_extensions {
        if *ext == Extension::Core {
            continue; // Core is already added as default xmlns
        }

        let (prefix, namespace) = match ext {
            Extension::Material => ("m", ext.namespace()),
            Extension::Production => ("p", ext.namespace()),
            Extension::Slice => ("s", ext.namespace()),
            Extension::BeamLattice => ("b", ext.namespace()),
            Extension::SecureContent => ("sc", ext.namespace()),
            Extension::BooleanOperations => ("bool", ext.namespace()),
            Extension::Displacement => ("d", ext.namespace()),
            Extension::Volumetric => ("v", ext.namespace()),
            Extension::Core => continue,
        };

        ns_attrs.push((format!("xmlns:{}", prefix), namespace));
    }

    for (name, value) in &ns_attrs {
        model_elem.push_attribute((name.as_str(), *value));
    }

    // Add requiredextensions attribute if needed
    if !model.required_extensions.is_empty() {
        let ext_names: Vec<String> = model
            .required_extensions
            .iter()
            .filter(|e| **e != Extension::Core)
            .map(|e| match e {
                Extension::Material => "m",
                Extension::Production => "p",
                Extension::Slice => "s",
                Extension::BeamLattice => "b",
                Extension::SecureContent => "sc",
                Extension::BooleanOperations => "bool",
                Extension::Displacement => "d",
                Extension::Volumetric => "v",
                Extension::Core => "",
            })
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();

        if !ext_names.is_empty() {
            model_elem.push_attribute(("requiredextensions", ext_names.join(" ").as_str()));
        }
    }

    xml_writer
        .write_event(Event::Start(model_elem))
        .map_err(|e| Error::xml_write(format!("Failed to write model element: {}", e)))?;

    // Write metadata
    for entry in &model.metadata {
        write_metadata(&mut xml_writer, entry)?;
    }

    // Write resources
    write_resources(&mut xml_writer, &model.resources)?;

    // Write build
    production::write_build(&mut xml_writer, &model.build)?;

    // Close model element
    xml_writer
        .write_event(Event::End(BytesEnd::new("model")))
        .map_err(|e| Error::xml_write(format!("Failed to close model element: {}", e)))?;

    Ok(())
}

/// Write a metadata entry
fn write_metadata<W: IoWrite>(writer: &mut Writer<W>, entry: &MetadataEntry) -> Result<()> {
    let mut elem = BytesStart::new("metadata");
    elem.push_attribute(("name", entry.name.as_str()));

    if let Some(preserve) = entry.preserve {
        elem.push_attribute(("preserve", if preserve { "1" } else { "0" }));
    }

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write metadata element: {}", e)))?;

    writer
        .write_event(Event::Text(BytesText::new(&entry.value)))
        .map_err(|e| Error::xml_write(format!("Failed to write metadata value: {}", e)))?;

    writer
        .write_event(Event::End(BytesEnd::new("metadata")))
        .map_err(|e| Error::xml_write(format!("Failed to close metadata element: {}", e)))?;

    Ok(())
}

/// Write resources section
fn write_resources<W: IoWrite>(writer: &mut Writer<W>, resources: &Resources) -> Result<()> {
    writer
        .write_event(Event::Start(BytesStart::new("resources")))
        .map_err(|e| Error::xml_write(format!("Failed to write resources element: {}", e)))?;

    // Write base material groups
    for group in &resources.base_material_groups {
        material::write_base_material_group(writer, group)?;
    }

    // Write texture2d resources
    for texture in &resources.texture2d_resources {
        material::write_texture2d(writer, texture)?;
    }

    // Write texture2dgroup resources
    for group in &resources.texture2d_groups {
        material::write_texture2d_group(writer, group)?;
    }

    // Write colorgroup resources
    for group in &resources.color_groups {
        material::write_color_group(writer, group)?;
    }

    // Write composite materials
    for composite in &resources.composite_materials {
        material::write_composite_materials(writer, composite)?;
    }

    // Write multiproperties resources
    for multi in &resources.multi_properties {
        material::write_multi_properties(writer, multi)?;
    }

    // Write displacement2d resources (displacement extension) - MUST come before disp2dgroup
    for disp_map in &resources.displacement_maps {
        displacement::write_displacement2d(writer, disp_map)?;
    }

    // Write normvectorgroup resources (displacement extension)
    for group in &resources.norm_vector_groups {
        displacement::write_normvector_group(writer, group)?;
    }

    // Write disp2dgroup resources (displacement extension)
    for group in &resources.disp2d_groups {
        displacement::write_disp2d_group(writer, group)?;
    }

    // Write volumetric property groups (volumetric extension)
    for group in &resources.volumetric_property_groups {
        volumetric::write_volumetric_property_group(writer, group)?;
    }

    // Write volumetric data resources (volumetric extension)
    for vol_data in &resources.volumetric_data {
        volumetric::write_volumetric_data(writer, vol_data)?;
    }

    // Write objects
    for object in &resources.objects {
        core::write_object(writer, object)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("resources")))
        .map_err(|e| Error::xml_write(format!("Failed to close resources element: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_minimal_model() {
        let mut model = Model::new();
        model.unit = "millimeter".to_string();

        let mut buffer = Vec::new();
        write_model_xml(&model, &mut buffer).unwrap();

        let xml = String::from_utf8(buffer).unwrap();
        assert!(xml.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<model"));
        assert!(xml.contains("unit=\"millimeter\""));
        assert!(xml.contains("<resources"));
        assert!(xml.contains("<build"));
    }

    #[test]
    fn test_write_model_with_metadata() {
        let mut model = Model::new();
        model.metadata.push(MetadataEntry::new(
            "Title".to_string(),
            "Test Model".to_string(),
        ));
        model.metadata.push(MetadataEntry::new(
            "Designer".to_string(),
            "lib3mf_rust".to_string(),
        ));

        let mut buffer = Vec::new();
        write_model_xml(&model, &mut buffer).unwrap();

        let xml = String::from_utf8(buffer).unwrap();
        assert!(xml.contains("<metadata name=\"Title\">Test Model</metadata>"));
        assert!(xml.contains("<metadata name=\"Designer\">lib3mf_rust</metadata>"));
    }

    #[test]
    fn test_write_model_with_simple_mesh() {
        let mut model = Model::new();

        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));

        let mut object = Object::new(1);
        object.mesh = Some(mesh);

        model.resources.objects.push(object);
        model.build.items.push(BuildItem::new(1));

        let mut buffer = Vec::new();
        write_model_xml(&model, &mut buffer).unwrap();

        let xml = String::from_utf8(buffer).unwrap();
        assert!(xml.contains("<object id=\"1\""));
        assert!(xml.contains("<mesh>"));
        assert!(xml.contains("<vertices>"));
        assert!(xml.contains("<triangles>"));
        assert!(xml.contains("v1=\"0\" v2=\"1\" v3=\"2\""));
    }

    #[test]
    fn test_write_object_with_basematerialid() {
        let mut model = Model::new();

        // Create a base material group
        let mut base_group = BaseMaterialGroup::new(5);
        base_group.materials.push(BaseMaterial::new(
            "Red Plastic".to_string(),
            (255, 0, 0, 255),
        ));
        model.resources.base_material_groups.push(base_group);

        // Create object with basematerialid
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(5.0, 10.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));

        let mut object = Object::new(1);
        object.basematerialid = Some(5);
        object.mesh = Some(mesh);

        model.resources.objects.push(object);
        model.build.items.push(BuildItem::new(1));

        let mut buffer = Vec::new();
        write_model_xml(&model, &mut buffer).unwrap();

        let xml = String::from_utf8(buffer).unwrap();
        assert!(xml.contains("basematerialid=\"5\""));
    }

    #[test]
    fn test_write_model_with_beamset() {
        use crate::model::{Beam, BeamCapMode, BeamSet, Extension};

        let mut model = Model::new();
        model.required_extensions.push(Extension::BeamLattice);

        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(10.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.0, 10.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));

        let mut beamset = BeamSet::new();
        beamset.radius = 1.5;
        beamset.cap_mode = BeamCapMode::Butt;
        let mut beam = Beam::new(0, 1);
        beam.r1 = Some(1.0);
        beam.r2 = Some(2.0);
        beam.cap1 = Some(BeamCapMode::Sphere);
        beam.cap2 = Some(BeamCapMode::Hemisphere);
        beamset.beams.push(beam);
        beamset.beams.push(Beam::with_radius(1, 2, 0.5));
        mesh.beamset = Some(beamset);

        let mut object = Object::new(1);
        object.mesh = Some(mesh);
        model.resources.objects.push(object);
        model.build.items.push(BuildItem::new(1));

        let mut buffer = Vec::new();
        write_model_xml(&model, &mut buffer).unwrap();

        let xml = String::from_utf8(buffer).unwrap();
        assert!(xml.contains("b:beamset"), "Should have beamset element");
        assert!(xml.contains("radius=\"1.5\""), "Should have beamset radius");
        assert!(xml.contains("capmode=\"butt\""), "Should have cap mode");
        assert!(xml.contains("b:beam"), "Should have beam elements");
        assert!(xml.contains("r1=\"1\""), "Should have r1");
        assert!(xml.contains("r2=\"2\""), "Should have r2");
        assert!(xml.contains("cap1=\"sphere\""), "Should have cap1");
        assert!(xml.contains("cap2=\"hemisphere\""), "Should have cap2");
    }

    #[test]
    fn test_write_model_with_boolean_shape() {
        use crate::model::{BooleanOpType, BooleanRef, BooleanShape, Extension};

        let mut model = Model::new();
        model.required_extensions.push(Extension::BooleanOperations);

        // Object 1: base mesh
        let mut mesh1 = Mesh::new();
        mesh1.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh1.vertices.push(Vertex::new(10.0, 0.0, 0.0));
        mesh1.vertices.push(Vertex::new(0.0, 10.0, 0.0));
        mesh1.triangles.push(Triangle::new(0, 1, 2));
        let mut obj1 = Object::new(1);
        obj1.mesh = Some(mesh1);
        model.resources.objects.push(obj1);

        // Object 2: operand mesh
        let mut mesh2 = Mesh::new();
        mesh2.vertices.push(Vertex::new(5.0, 0.0, 0.0));
        mesh2.vertices.push(Vertex::new(15.0, 0.0, 0.0));
        mesh2.vertices.push(Vertex::new(5.0, 10.0, 0.0));
        mesh2.triangles.push(Triangle::new(0, 1, 2));
        let mut obj2 = Object::new(2);
        obj2.mesh = Some(mesh2);
        model.resources.objects.push(obj2);

        // Object 3: boolean shape with union
        let mut boolean_shape = BooleanShape::new(1, BooleanOpType::Union);
        boolean_shape.operands.push(BooleanRef::new(2));
        let mut ref_with_path = BooleanRef::new(2);
        ref_with_path.path = Some("/3D/other.model".to_string());
        boolean_shape.operands.push(ref_with_path);
        boolean_shape.path = Some("/3D/base.model".to_string());
        let mut obj3 = Object::new(3);
        obj3.boolean_shape = Some(boolean_shape);
        model.resources.objects.push(obj3);

        // Also test difference operation
        let mut obj4 = Object::new(4);
        let mut diff_shape = BooleanShape::new(1, BooleanOpType::Difference);
        diff_shape.operands.push(BooleanRef::new(2));
        obj4.boolean_shape = Some(diff_shape);
        model.resources.objects.push(obj4);

        // And intersection
        let mut obj5 = Object::new(5);
        let mut inter_shape = BooleanShape::new(1, BooleanOpType::Intersection);
        inter_shape.operands.push(BooleanRef::new(2));
        obj5.boolean_shape = Some(inter_shape);
        model.resources.objects.push(obj5);

        model.build.items.push(BuildItem::new(3));

        let mut buffer = Vec::new();
        write_model_xml(&model, &mut buffer).unwrap();

        let xml = String::from_utf8(buffer).unwrap();
        assert!(
            xml.contains("bool:booleanshape"),
            "Should have booleanshape element"
        );
        assert!(
            xml.contains("operation=\"union\""),
            "Should have union operation"
        );
        assert!(
            xml.contains("operation=\"difference\""),
            "Should have difference operation"
        );
        assert!(
            xml.contains("operation=\"intersection\""),
            "Should have intersection operation"
        );
        assert!(
            xml.contains("bool:boolean"),
            "Should have boolean reference"
        );
    }

    #[test]
    fn test_write_model_with_displacement() {
        use crate::model::{
            Channel, Disp2DCoords, Disp2DGroup, Displacement2D, Extension, FilterMode, NormVector,
            NormVectorGroup, TileStyle,
        };

        let mut model = Model::new();
        model.required_extensions.push(Extension::Displacement);

        // Add displacement2d resource
        let mut disp = Displacement2D::new(1, "/3D/Textures/disp.png".to_string());
        disp.channel = Channel::R;
        disp.tilestyleu = TileStyle::Mirror;
        disp.tilestylev = TileStyle::Clamp;
        disp.filter = FilterMode::Linear;
        model.resources.displacement_maps.push(disp);

        // Also test other channel/style combinations for coverage
        let mut disp2 = Displacement2D::new(2, "/3D/Textures/disp2.png".to_string());
        disp2.channel = Channel::B;
        disp2.tilestyleu = TileStyle::None;
        disp2.filter = FilterMode::Nearest;
        model.resources.displacement_maps.push(disp2);

        let mut disp3 = Displacement2D::new(3, "/3D/Textures/disp3.png".to_string());
        disp3.channel = Channel::A;
        disp3.filter = FilterMode::Auto;
        model.resources.displacement_maps.push(disp3);

        // Add normvector group
        let mut ng = NormVectorGroup::new(10);
        ng.vectors.push(NormVector::new(0.0, 0.0, 1.0));
        ng.vectors.push(NormVector::new(0.0, 1.0, 0.0));
        model.resources.norm_vector_groups.push(ng);

        // Add disp2d group
        let mut dg = Disp2DGroup::new(20, 1, 10, 0.5);
        dg.coords.push(Disp2DCoords::new(0.0, 0.0, 0));
        dg.coords.push(Disp2DCoords::new(0.5, 0.5, 1));
        model.resources.disp2d_groups.push(dg);

        let mut buffer = Vec::new();
        write_model_xml(&model, &mut buffer).unwrap();

        let xml = String::from_utf8(buffer).unwrap();
        assert!(
            xml.contains("d:displacement2d"),
            "Should have displacement2d element"
        );
        assert!(xml.contains("channel=\"r\""), "Should have channel r");
        assert!(xml.contains("channel=\"b\""), "Should have channel b");
        assert!(xml.contains("channel=\"a\""), "Should have channel a");
        assert!(
            xml.contains("tilestyleu=\"mirror\""),
            "Should have tilestyleu"
        );
        assert!(
            xml.contains("tilestylev=\"clamp\""),
            "Should have tilestylev"
        );
        assert!(
            xml.contains("filter=\"linear\""),
            "Should have filter linear"
        );
        assert!(
            xml.contains("filter=\"nearest\""),
            "Should have filter nearest"
        );
        assert!(
            xml.contains("d:normvectorgroup"),
            "Should have normvectorgroup element"
        );
        assert!(
            xml.contains("d:normvector"),
            "Should have normvector elements"
        );
        assert!(
            xml.contains("d:disp2dgroup"),
            "Should have disp2dgroup element"
        );
        assert!(xml.contains("d:disp2d"), "Should have disp2d coordinates");
    }
}
