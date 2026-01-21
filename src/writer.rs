//! XML writing for 3MF model files
//!
//! This module provides functionality to serialize Model structures back into
//! 3MF-compliant XML format.

use crate::error::{Error, Result};
use crate::model::*;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
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
    write_build(&mut xml_writer, &model.build)?;

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
        write_base_material_group(writer, group)?;
    }

    // Write texture2d resources
    for texture in &resources.texture2d_resources {
        write_texture2d(writer, texture)?;
    }

    // Write texture2dgroup resources
    for group in &resources.texture2d_groups {
        write_texture2d_group(writer, group)?;
    }

    // Write colorgroup resources
    for group in &resources.color_groups {
        write_color_group(writer, group)?;
    }

    // Write composite materials
    for composite in &resources.composite_materials {
        write_composite_materials(writer, composite)?;
    }

    // Write multiproperties resources
    for multi in &resources.multi_properties {
        write_multi_properties(writer, multi)?;
    }

    // Write normvectorgroup resources (displacement extension)
    for group in &resources.norm_vector_groups {
        write_normvector_group(writer, group)?;
    }

    // Write disp2dgroup resources (displacement extension)
    for group in &resources.disp2d_groups {
        write_disp2d_group(writer, group)?;
    }

    // Write objects
    for object in &resources.objects {
        write_object(writer, object)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("resources")))
        .map_err(|e| Error::xml_write(format!("Failed to close resources element: {}", e)))?;

    Ok(())
}

/// Write a base material group
fn write_base_material_group<W: IoWrite>(
    writer: &mut Writer<W>,
    group: &BaseMaterialGroup,
) -> Result<()> {
    let mut elem = BytesStart::new("m:basematerials");
    elem.push_attribute(("id", group.id.to_string().as_str()));

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write basematerials element: {}", e)))?;

    for material in &group.materials {
        let mut mat_elem = BytesStart::new("m:base");
        mat_elem.push_attribute(("name", material.name.as_str()));

        let color = format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            material.displaycolor.0,
            material.displaycolor.1,
            material.displaycolor.2,
            material.displaycolor.3
        );
        mat_elem.push_attribute(("displaycolor", color.as_str()));

        writer
            .write_event(Event::Empty(mat_elem))
            .map_err(|e| Error::xml_write(format!("Failed to write base material: {}", e)))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("m:basematerials")))
        .map_err(|e| Error::xml_write(format!("Failed to close basematerials element: {}", e)))?;

    Ok(())
}

/// Write a texture2d resource
fn write_texture2d<W: IoWrite>(writer: &mut Writer<W>, texture: &Texture2D) -> Result<()> {
    let mut elem = BytesStart::new("m:texture2d");
    elem.push_attribute(("id", texture.id.to_string().as_str()));
    elem.push_attribute(("path", texture.path.as_str()));
    elem.push_attribute(("contenttype", texture.contenttype.as_str()));

    let tilestyleu = match texture.tilestyleu {
        TileStyle::Wrap => "wrap",
        TileStyle::Mirror => "mirror",
        TileStyle::Clamp => "clamp",
        TileStyle::None => "none",
    };
    elem.push_attribute(("tilestyleu", tilestyleu));

    let tilestylev = match texture.tilestylev {
        TileStyle::Wrap => "wrap",
        TileStyle::Mirror => "mirror",
        TileStyle::Clamp => "clamp",
        TileStyle::None => "none",
    };
    elem.push_attribute(("tilestylev", tilestylev));

    let filter = match texture.filter {
        FilterMode::Auto => "auto",
        FilterMode::Linear => "linear",
        FilterMode::Nearest => "nearest",
    };
    elem.push_attribute(("filter", filter));

    writer
        .write_event(Event::Empty(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write texture2d: {}", e)))?;

    Ok(())
}

/// Write a texture2dgroup resource
fn write_texture2d_group<W: IoWrite>(writer: &mut Writer<W>, group: &Texture2DGroup) -> Result<()> {
    let mut elem = BytesStart::new("m:texture2dgroup");
    elem.push_attribute(("id", group.id.to_string().as_str()));
    elem.push_attribute(("texid", group.texid.to_string().as_str()));

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write texture2dgroup element: {}", e)))?;

    for coord in &group.tex2coords {
        let mut coord_elem = BytesStart::new("m:tex2coord");
        coord_elem.push_attribute(("u", coord.u.to_string().as_str()));
        coord_elem.push_attribute(("v", coord.v.to_string().as_str()));

        writer
            .write_event(Event::Empty(coord_elem))
            .map_err(|e| Error::xml_write(format!("Failed to write tex2coord: {}", e)))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("m:texture2dgroup")))
        .map_err(|e| Error::xml_write(format!("Failed to close texture2dgroup element: {}", e)))?;

    Ok(())
}

/// Write a color group resource
fn write_color_group<W: IoWrite>(writer: &mut Writer<W>, group: &ColorGroup) -> Result<()> {
    let mut elem = BytesStart::new("m:colorgroup");
    elem.push_attribute(("id", group.id.to_string().as_str()));

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write colorgroup element: {}", e)))?;

    for color in &group.colors {
        let mut color_elem = BytesStart::new("m:color");
        let color_str = format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            color.0, color.1, color.2, color.3
        );
        color_elem.push_attribute(("color", color_str.as_str()));

        writer
            .write_event(Event::Empty(color_elem))
            .map_err(|e| Error::xml_write(format!("Failed to write color: {}", e)))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("m:colorgroup")))
        .map_err(|e| Error::xml_write(format!("Failed to close colorgroup element: {}", e)))?;

    Ok(())
}

/// Write composite materials
fn write_composite_materials<W: IoWrite>(
    writer: &mut Writer<W>,
    composite: &CompositeMaterials,
) -> Result<()> {
    let mut elem = BytesStart::new("m:compositematerials");
    elem.push_attribute(("id", composite.id.to_string().as_str()));
    elem.push_attribute(("matid", composite.matid.to_string().as_str()));

    // Convert matindices to space-separated string
    let matindices_str = composite
        .matindices
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    elem.push_attribute(("matindices", matindices_str.as_str()));

    writer.write_event(Event::Start(elem)).map_err(|e| {
        Error::xml_write(format!("Failed to write compositematerials element: {}", e))
    })?;

    for comp in &composite.composites {
        write_composite(writer, comp)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("m:compositematerials")))
        .map_err(|e| {
            Error::xml_write(format!("Failed to close compositematerials element: {}", e))
        })?;

    Ok(())
}

/// Write a composite entry
fn write_composite<W: IoWrite>(writer: &mut Writer<W>, composite: &Composite) -> Result<()> {
    let mut elem = BytesStart::new("m:composite");

    // Convert values to space-separated string
    let values_str = composite
        .values
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    elem.push_attribute(("values", values_str.as_str()));

    writer
        .write_event(Event::Empty(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write composite: {}", e)))?;

    Ok(())
}

/// Write multi properties resource
fn write_multi_properties<W: IoWrite>(
    writer: &mut Writer<W>,
    multi: &MultiProperties,
) -> Result<()> {
    let mut elem = BytesStart::new("m:multiproperties");
    elem.push_attribute(("id", multi.id.to_string().as_str()));

    // Convert pids to space-separated string
    let pids_str = multi
        .pids
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    elem.push_attribute(("pids", pids_str.as_str()));

    // Convert blendmethods to space-separated string
    if !multi.blendmethods.is_empty() {
        let blendmethods_str = multi
            .blendmethods
            .iter()
            .map(|b| match b {
                BlendMethod::Mix => "mix",
                BlendMethod::Multiply => "multiply",
            })
            .collect::<Vec<_>>()
            .join(" ");
        elem.push_attribute(("blendmethods", blendmethods_str.as_str()));
    }

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write multiproperties element: {}", e)))?;

    for multi_elem in &multi.multis {
        write_multi(writer, multi_elem)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("m:multiproperties")))
        .map_err(|e| Error::xml_write(format!("Failed to close multiproperties element: {}", e)))?;

    Ok(())
}

/// Write a multi entry
fn write_multi<W: IoWrite>(writer: &mut Writer<W>, multi: &Multi) -> Result<()> {
    let mut elem = BytesStart::new("m:multi");

    // Convert pindices to space-separated string
    let pindices_str = multi
        .pindices
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    elem.push_attribute(("pindices", pindices_str.as_str()));

    writer
        .write_event(Event::Empty(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write multi: {}", e)))?;

    Ok(())
}

/// Write normvector group (displacement extension)
fn write_normvector_group<W: IoWrite>(
    writer: &mut Writer<W>,
    group: &NormVectorGroup,
) -> Result<()> {
    let mut elem = BytesStart::new("d:normvectorgroup");
    elem.push_attribute(("id", group.id.to_string().as_str()));

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write normvectorgroup element: {}", e)))?;

    for vector in &group.vectors {
        let mut vec_elem = BytesStart::new("d:normvector");
        vec_elem.push_attribute(("x", vector.x.to_string().as_str()));
        vec_elem.push_attribute(("y", vector.y.to_string().as_str()));
        vec_elem.push_attribute(("z", vector.z.to_string().as_str()));

        writer
            .write_event(Event::Empty(vec_elem))
            .map_err(|e| Error::xml_write(format!("Failed to write normvector: {}", e)))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("d:normvectorgroup")))
        .map_err(|e| Error::xml_write(format!("Failed to close normvectorgroup element: {}", e)))?;

    Ok(())
}

/// Write disp2dgroup (displacement extension)
fn write_disp2d_group<W: IoWrite>(writer: &mut Writer<W>, group: &Disp2DGroup) -> Result<()> {
    let mut elem = BytesStart::new("d:disp2dgroup");
    elem.push_attribute(("id", group.id.to_string().as_str()));
    elem.push_attribute(("dispid", group.dispid.to_string().as_str()));
    elem.push_attribute(("nid", group.nid.to_string().as_str()));
    elem.push_attribute(("height", group.height.to_string().as_str()));
    elem.push_attribute(("offset", group.offset.to_string().as_str()));

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write disp2dgroup element: {}", e)))?;

    for coord in &group.coords {
        let mut coord_elem = BytesStart::new("d:disp2d");
        coord_elem.push_attribute(("u", coord.u.to_string().as_str()));
        coord_elem.push_attribute(("v", coord.v.to_string().as_str()));

        writer
            .write_event(Event::Empty(coord_elem))
            .map_err(|e| Error::xml_write(format!("Failed to write disp2d: {}", e)))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("d:disp2dgroup")))
        .map_err(|e| Error::xml_write(format!("Failed to close disp2dgroup element: {}", e)))?;

    Ok(())
}

/// Write an object
fn write_object<W: IoWrite>(writer: &mut Writer<W>, object: &Object) -> Result<()> {
    let mut elem = BytesStart::new("object");
    elem.push_attribute(("id", object.id.to_string().as_str()));

    let obj_type = match object.object_type {
        ObjectType::Model => "model",
        ObjectType::Support => "support",
        ObjectType::SolidSupport => "solidsupport",
        ObjectType::Surface => "surface",
        ObjectType::Other => "other",
    };
    elem.push_attribute(("type", obj_type));

    if let Some(ref name) = object.name {
        elem.push_attribute(("name", name.as_str()));
    }

    if let Some(pid) = object.pid {
        elem.push_attribute(("pid", pid.to_string().as_str()));
    }

    if let Some(pindex) = object.pindex {
        elem.push_attribute(("pindex", pindex.to_string().as_str()));
    }

    // Production extension attributes
    if let Some(ref production) = object.production {
        if let Some(ref uuid) = production.uuid {
            elem.push_attribute(("p:UUID", uuid.as_str()));
        }

        if let Some(ref path) = production.path {
            elem.push_attribute(("p:path", path.as_str()));
        }
    }

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write object element: {}", e)))?;

    // Write mesh if present
    if let Some(ref mesh) = object.mesh {
        write_mesh(writer, mesh)?;
    }

    // Write components if present
    if !object.components.is_empty() {
        write_components(writer, &object.components)?;
    }

    // Write boolean shape if present (boolean operations extension)
    if let Some(ref boolean_shape) = object.boolean_shape {
        write_boolean_shape(writer, boolean_shape)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("object")))
        .map_err(|e| Error::xml_write(format!("Failed to close object element: {}", e)))?;

    Ok(())
}

/// Write a mesh
fn write_mesh<W: IoWrite>(writer: &mut Writer<W>, mesh: &Mesh) -> Result<()> {
    writer
        .write_event(Event::Start(BytesStart::new("mesh")))
        .map_err(|e| Error::xml_write(format!("Failed to write mesh element: {}", e)))?;

    // Write vertices
    writer
        .write_event(Event::Start(BytesStart::new("vertices")))
        .map_err(|e| Error::xml_write(format!("Failed to write vertices element: {}", e)))?;

    for vertex in &mesh.vertices {
        let mut v_elem = BytesStart::new("vertex");
        v_elem.push_attribute(("x", vertex.x.to_string().as_str()));
        v_elem.push_attribute(("y", vertex.y.to_string().as_str()));
        v_elem.push_attribute(("z", vertex.z.to_string().as_str()));

        writer
            .write_event(Event::Empty(v_elem))
            .map_err(|e| Error::xml_write(format!("Failed to write vertex: {}", e)))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("vertices")))
        .map_err(|e| Error::xml_write(format!("Failed to close vertices element: {}", e)))?;

    // Write triangles
    writer
        .write_event(Event::Start(BytesStart::new("triangles")))
        .map_err(|e| Error::xml_write(format!("Failed to write triangles element: {}", e)))?;

    for triangle in &mesh.triangles {
        let mut t_elem = BytesStart::new("triangle");
        t_elem.push_attribute(("v1", triangle.v1.to_string().as_str()));
        t_elem.push_attribute(("v2", triangle.v2.to_string().as_str()));
        t_elem.push_attribute(("v3", triangle.v3.to_string().as_str()));

        if let Some(pid) = triangle.pid {
            t_elem.push_attribute(("pid", pid.to_string().as_str()));
        }

        if let Some(pindex) = triangle.pindex {
            t_elem.push_attribute(("pindex", pindex.to_string().as_str()));
        }

        if let Some(p1) = triangle.p1 {
            t_elem.push_attribute(("p1", p1.to_string().as_str()));
        }

        if let Some(p2) = triangle.p2 {
            t_elem.push_attribute(("p2", p2.to_string().as_str()));
        }

        if let Some(p3) = triangle.p3 {
            t_elem.push_attribute(("p3", p3.to_string().as_str()));
        }

        writer
            .write_event(Event::Empty(t_elem))
            .map_err(|e| Error::xml_write(format!("Failed to write triangle: {}", e)))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("triangles")))
        .map_err(|e| Error::xml_write(format!("Failed to close triangles element: {}", e)))?;

    // Write beamset if present (beam lattice extension)
    if let Some(ref beamset) = mesh.beamset {
        write_beamset(writer, beamset)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("mesh")))
        .map_err(|e| Error::xml_write(format!("Failed to close mesh element: {}", e)))?;

    Ok(())
}

/// Write beamset (beam lattice extension)
fn write_beamset<W: IoWrite>(writer: &mut Writer<W>, beamset: &BeamSet) -> Result<()> {
    let mut elem = BytesStart::new("b:beamset");

    elem.push_attribute(("radius", beamset.radius.to_string().as_str()));
    elem.push_attribute(("minlength", beamset.min_length.to_string().as_str()));

    let cap_mode = match beamset.cap_mode {
        BeamCapMode::Sphere => "sphere",
        BeamCapMode::Butt => "butt",
    };
    elem.push_attribute(("capmode", cap_mode));

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write beamset element: {}", e)))?;

    for beam in &beamset.beams {
        let mut beam_elem = BytesStart::new("b:beam");
        beam_elem.push_attribute(("v1", beam.v1.to_string().as_str()));
        beam_elem.push_attribute(("v2", beam.v2.to_string().as_str()));

        if let Some(r1) = beam.r1 {
            beam_elem.push_attribute(("r1", r1.to_string().as_str()));
        }

        if let Some(r2) = beam.r2 {
            beam_elem.push_attribute(("r2", r2.to_string().as_str()));
        }

        writer
            .write_event(Event::Empty(beam_elem))
            .map_err(|e| Error::xml_write(format!("Failed to write beam: {}", e)))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("b:beamset")))
        .map_err(|e| Error::xml_write(format!("Failed to close beamset element: {}", e)))?;

    Ok(())
}

/// Write components
fn write_components<W: IoWrite>(writer: &mut Writer<W>, components: &[Component]) -> Result<()> {
    writer
        .write_event(Event::Start(BytesStart::new("components")))
        .map_err(|e| Error::xml_write(format!("Failed to write components element: {}", e)))?;

    for component in components {
        let mut elem = BytesStart::new("component");
        elem.push_attribute(("objectid", component.objectid.to_string().as_str()));

        if let Some(transform) = component.transform {
            let transform_str = transform
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            elem.push_attribute(("transform", transform_str.as_str()));
        }

        writer
            .write_event(Event::Empty(elem))
            .map_err(|e| Error::xml_write(format!("Failed to write component: {}", e)))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("components")))
        .map_err(|e| Error::xml_write(format!("Failed to close components element: {}", e)))?;

    Ok(())
}

/// Write boolean shape (boolean operations extension)
fn write_boolean_shape<W: IoWrite>(writer: &mut Writer<W>, shape: &BooleanShape) -> Result<()> {
    let mut elem = BytesStart::new("bool:booleanshape");
    elem.push_attribute(("objectid", shape.objectid.to_string().as_str()));

    let op_type = match shape.operation {
        BooleanOpType::Union => "union",
        BooleanOpType::Intersection => "intersection",
        BooleanOpType::Difference => "difference",
    };
    elem.push_attribute(("op", op_type));

    if let Some(ref path) = shape.path {
        elem.push_attribute(("path", path.as_str()));
    }

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write booleanshape element: {}", e)))?;

    // Write boolean references (operands)
    for boolean_ref in &shape.operands {
        let mut ref_elem = BytesStart::new("bool:booleanref");
        ref_elem.push_attribute(("objectid", boolean_ref.objectid.to_string().as_str()));

        if let Some(ref path) = boolean_ref.path {
            ref_elem.push_attribute(("path", path.as_str()));
        }

        writer
            .write_event(Event::Empty(ref_elem))
            .map_err(|e| Error::xml_write(format!("Failed to write booleanref: {}", e)))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("bool:booleanshape")))
        .map_err(|e| Error::xml_write(format!("Failed to close booleanshape element: {}", e)))?;

    Ok(())
}

/// Write build section
fn write_build<W: IoWrite>(writer: &mut Writer<W>, build: &Build) -> Result<()> {
    let mut elem = BytesStart::new("build");

    if let Some(ref uuid) = build.production_uuid {
        elem.push_attribute(("p:UUID", uuid.as_str()));
    }

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write build element: {}", e)))?;

    for item in &build.items {
        write_build_item(writer, item)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("build")))
        .map_err(|e| Error::xml_write(format!("Failed to close build element: {}", e)))?;

    Ok(())
}

/// Write a build item
fn write_build_item<W: IoWrite>(writer: &mut Writer<W>, item: &BuildItem) -> Result<()> {
    let mut elem = BytesStart::new("item");
    elem.push_attribute(("objectid", item.objectid.to_string().as_str()));

    if let Some(transform) = item.transform {
        let transform_str = transform
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        elem.push_attribute(("transform", transform_str.as_str()));
    }

    // Production extension attributes
    if let Some(ref uuid) = item.production_uuid {
        elem.push_attribute(("p:UUID", uuid.as_str()));
    }

    writer
        .write_event(Event::Empty(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write item: {}", e)))?;

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
}
