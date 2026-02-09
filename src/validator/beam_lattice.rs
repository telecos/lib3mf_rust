//! Beam lattice extension validation

use crate::error::{Error, Result};
use crate::model::{Model, ObjectType};
use std::collections::HashSet;

/// Validates beam lattice structures in model objects
pub fn validate_beam_lattice(model: &Model) -> Result<()> {
    // Collect all valid resource IDs (objects, property groups, etc.)
    let mut valid_resource_ids = HashSet::new();

    for obj in &model.resources.objects {
        valid_resource_ids.insert(obj.id);
    }
    for cg in &model.resources.color_groups {
        valid_resource_ids.insert(cg.id);
    }
    for bg in &model.resources.base_material_groups {
        valid_resource_ids.insert(bg.id);
    }
    for tg in &model.resources.texture2d_groups {
        valid_resource_ids.insert(tg.id);
    }
    for c2d in &model.resources.composite_materials {
        valid_resource_ids.insert(c2d.id);
    }
    for mg in &model.resources.multi_properties {
        valid_resource_ids.insert(mg.id);
    }

    // Validate each object with beam lattice
    for (obj_position, object) in model.resources.objects.iter().enumerate() {
        if let Some(ref mesh) = object.mesh
            && let Some(ref beamset) = mesh.beamset
        {
            // Validate object type
            // Per spec: "A beamlattice MUST only be added to a mesh object of type 'model' or 'solidsupport'"
            if object.object_type != ObjectType::Model
                && object.object_type != ObjectType::SolidSupport
            {
                return Err(Error::InvalidModel(format!(
                    "Object {}: BeamLattice can only be added to objects of type 'model' or 'solidsupport'. \
                         This object has type '{:?}'. Per the Beam Lattice spec, types like 'support' or 'other' are not allowed.",
                    object.id, object.object_type
                )));
            }

            let vertex_count = mesh.vertices.len();

            // Validate beam vertex indices
            for (beam_idx, beam) in beamset.beams.iter().enumerate() {
                if beam.v1 >= vertex_count {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Beam {} references invalid vertex index v1={} \
                             (mesh has {} vertices). Beam vertex indices must be less than \
                             the number of vertices in the mesh.",
                        object.id, beam_idx, beam.v1, vertex_count
                    )));
                }
                if beam.v2 >= vertex_count {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Beam {} references invalid vertex index v2={} \
                             (mesh has {} vertices). Beam vertex indices must be less than \
                             the number of vertices in the mesh.",
                        object.id, beam_idx, beam.v2, vertex_count
                    )));
                }

                // Validate that beam is not self-referencing (v1 != v2)
                // A beam must connect two different vertices
                if beam.v1 == beam.v2 {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Beam {} is self-referencing (v1=v2={}). \
                             A beam must connect two different vertices.",
                        object.id, beam_idx, beam.v1
                    )));
                }

                // Validate beam material references
                if let Some(pid) = beam.property_id
                    && !valid_resource_ids.contains(&(pid as usize))
                {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Beam {} references non-existent property group ID {}. \
                                 Property group IDs must reference existing color groups, base material groups, \
                                 texture groups, composite materials, or multi-property groups.",
                        object.id, beam_idx, pid
                    )));
                }
            }

            // Validate no duplicate beams
            // Two beams are considered duplicates if they connect the same pair of vertices
            // (regardless of order: beam(v1,v2) equals beam(v2,v1))
            let mut seen_beams = HashSet::new();
            for (beam_idx, beam) in beamset.beams.iter().enumerate() {
                // Normalize beam to sorted order so (1,2) and (2,1) are treated as the same
                let normalized = if beam.v1 < beam.v2 {
                    (beam.v1, beam.v2)
                } else {
                    (beam.v2, beam.v1)
                };

                if !seen_beams.insert(normalized) {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Beam {} is a duplicate (connects vertices {} and {}). \
                             Each pair of vertices can only be connected by one beam.",
                        object.id, beam_idx, beam.v1, beam.v2
                    )));
                }
            }

            // Validate that if beamlattice has pid, object must also have pid
            // Per spec requirement: when beamlattice specifies pid, object level pid is required
            if beamset.property_id.is_some() && object.pid.is_none() {
                return Err(Error::InvalidModel(format!(
                    "Object {}: BeamLattice specifies pid but object does not have pid attribute. \
                         When beamlattice has pid, the object must also specify pid.",
                    object.id
                )));
            }

            // Validate that if beams or balls have property assignments,
            // then beamlattice or object must have a default pid
            // Per spec: "If this beam lattice contains any beam or ball with assigned properties,
            // the beam lattice or object MUST specify pid and pindex"
            let beams_have_properties = beamset
                .beams
                .iter()
                .any(|b| b.property_id.is_some() || b.p1.is_some() || b.p2.is_some());

            let balls_have_properties = beamset
                .balls
                .iter()
                .any(|b| b.property_id.is_some() || b.property_index.is_some());

            if beams_have_properties || balls_have_properties {
                let has_default_pid = beamset.property_id.is_some() || object.pid.is_some();
                if !has_default_pid {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: BeamLattice contains beams or balls with property assignments \
                             but neither the beamlattice nor the object specifies a default pid. \
                             Per the Beam Lattice spec, when beams or balls have assigned properties, \
                             the beamlattice or object MUST specify pid and pindex to act as default values.",
                        object.id
                    )));
                }
            }

            // Validate beamset references (if any)
            // Beamset refs are indices into the beams array and must be within bounds
            for ref_index in &beamset.beam_set_refs {
                if *ref_index >= beamset.beams.len() {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: BeamSet reference index {} is out of bounds. \
                             The beamlattice has {} beams (valid indices: 0-{}).",
                        object.id,
                        ref_index,
                        beamset.beams.len(),
                        beamset.beams.len().saturating_sub(1)
                    )));
                }
            }

            // Validate ball set references (if any)
            // Ball set refs are indices into the balls array and must be within bounds
            for ref_index in &beamset.ball_set_refs {
                if *ref_index >= beamset.balls.len() {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: BallSet reference index {} is out of bounds. \
                             The beamlattice has {} balls (valid indices: 0-{}).",
                        object.id,
                        ref_index,
                        beamset.balls.len(),
                        beamset.balls.len().saturating_sub(1)
                    )));
                }
            }

            // Validate balls (from balls sub-extension)
            // First, build set of beam endpoint vertices
            let mut beam_endpoints: HashSet<usize> = HashSet::new();
            for beam in &beamset.beams {
                beam_endpoints.insert(beam.v1);
                beam_endpoints.insert(beam.v2);
            }

            for (ball_idx, ball) in beamset.balls.iter().enumerate() {
                // Validate ball vertex index
                if ball.vindex >= vertex_count {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Ball {} references invalid vertex index {} \
                             (mesh has {} vertices). Ball vertex indices must be less than \
                             the number of vertices in the mesh.",
                        object.id, ball_idx, ball.vindex, vertex_count
                    )));
                }

                // Validate that ball vindex is at a beam endpoint
                // Per spec requirement: balls must be placed at beam endpoints
                if !beam_endpoints.contains(&ball.vindex) {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Ball {} at vertex {} is not at a beam endpoint. \
                             Balls must be placed at vertices that are endpoints of beams.",
                        object.id, ball_idx, ball.vindex
                    )));
                }

                // Validate ball material references
                if let Some(ball_pid) = ball.property_id {
                    if !valid_resource_ids.contains(&(ball_pid as usize)) {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Ball {} references non-existent property group ID {}. \
                                 Property group IDs must reference existing color groups, base material groups, \
                                 texture groups, composite materials, or multi-property groups.",
                            object.id, ball_idx, ball_pid
                        )));
                    }

                    // Validate ball property index if present
                    if let Some(ball_p) = ball.property_index {
                        // Check if it's a color group
                        if let Some(colorgroup) = model
                            .resources
                            .color_groups
                            .iter()
                            .find(|cg| cg.id == ball_pid as usize)
                        {
                            if ball_p as usize >= colorgroup.colors.len() {
                                let max_index = colorgroup.colors.len().saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Ball {} property index {} is out of bounds.\n\
                                         Color group {} has {} colors (valid indices: 0-{}).",
                                    object.id,
                                    ball_idx,
                                    ball_p,
                                    ball_pid,
                                    colorgroup.colors.len(),
                                    max_index
                                )));
                            }
                        }
                        // Check if it's a base material group
                        else if let Some(basematerialgroup) = model
                            .resources
                            .base_material_groups
                            .iter()
                            .find(|bg| bg.id == ball_pid as usize)
                            && ball_p as usize >= basematerialgroup.materials.len()
                        {
                            let max_index = basematerialgroup.materials.len().saturating_sub(1);
                            return Err(Error::InvalidModel(format!(
                                "Object {}: Ball {} property index {} is out of bounds.\n\
                                         Base material group {} has {} materials (valid indices: 0-{}).",
                                object.id,
                                ball_idx,
                                ball_p,
                                ball_pid,
                                basematerialgroup.materials.len(),
                                max_index
                            )));
                        }
                    }
                }
            }

            // Validate clipping mesh reference
            if let Some(clip_id) = beamset.clipping_mesh_id {
                if !valid_resource_ids.contains(&(clip_id as usize)) {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: BeamLattice references non-existent clippingmesh ID {}. \
                             The clippingmesh attribute must reference a valid object resource.",
                        object.id, clip_id
                    )));
                }

                // Check for self-reference (clipping mesh cannot be the same object)
                if clip_id as usize == object.id {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: BeamLattice clippingmesh references itself. \
                             The clippingmesh cannot be the same object that contains the beamlattice.",
                        object.id
                    )));
                }

                // Per spec: "The clippingmesh attribute MUST reference an object id earlier in the file"
                // This means clippingmesh must be a backward reference (earlier position in objects vector)
                if let Some(clip_obj_position) = model
                    .resources
                    .objects
                    .iter()
                    .position(|o| o.id == clip_id as usize)
                    && clip_obj_position >= obj_position
                {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: BeamLattice clippingmesh={} is not declared earlier in the file. \
                                 Per the Beam Lattice spec, clippingmesh MUST reference an object that appears earlier \
                                 in the resources section of the 3MF file.",
                        object.id, clip_id
                    )));
                }

                // Check that the referenced object is a mesh object (not a component-only object)
                // and does not contain a beamlattice
                if let Some(clip_obj) = model
                    .resources
                    .objects
                    .iter()
                    .find(|o| o.id == clip_id as usize)
                {
                    // Object must have a mesh, not just components
                    if clip_obj.mesh.is_none() && !clip_obj.components.is_empty() {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: BeamLattice clippingmesh references object {} which is a component object (no mesh). \
                                 The clippingmesh must reference an object that contains a mesh.",
                            object.id, clip_id
                        )));
                    }

                    // Clipping mesh MUST NOT contain a beamlattice
                    if let Some(ref clip_mesh) = clip_obj.mesh
                        && clip_mesh.beamset.is_some()
                    {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: BeamLattice clippingmesh references object {} which contains a beamlattice. \
                                     Per the Beam Lattice spec, clippingmesh objects MUST NOT contain a beamlattice.",
                            object.id, clip_id
                        )));
                    }
                }
            }

            // Validate representation mesh reference
            if let Some(rep_id) = beamset.representation_mesh_id {
                if !valid_resource_ids.contains(&(rep_id as usize)) {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: BeamLattice references non-existent representationmesh ID {}. \
                             The representationmesh attribute must reference a valid object resource.",
                        object.id, rep_id
                    )));
                }

                // Check for self-reference (representation mesh cannot be the same object)
                if rep_id as usize == object.id {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: BeamLattice representationmesh references itself. \
                             The representationmesh cannot be the same object that contains the beamlattice.",
                        object.id
                    )));
                }

                // Per spec: "The representationmesh attribute MUST reference an object id earlier in the file"
                // This means representationmesh must be a backward reference (earlier position in objects vector)
                if let Some(rep_obj_position) = model
                    .resources
                    .objects
                    .iter()
                    .position(|o| o.id == rep_id as usize)
                    && rep_obj_position >= obj_position
                {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: BeamLattice representationmesh={} is not declared earlier in the file. \
                                 Per the Beam Lattice spec, representationmesh MUST reference an object that appears earlier \
                                 in the resources section of the 3MF file.",
                        object.id, rep_id
                    )));
                }

                // Check that the referenced object is a mesh object (not a component-only object)
                // and does not contain a beamlattice
                if let Some(rep_obj) = model
                    .resources
                    .objects
                    .iter()
                    .find(|o| o.id == rep_id as usize)
                {
                    // Object must have a mesh, not just components
                    if rep_obj.mesh.is_none() && !rep_obj.components.is_empty() {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: BeamLattice representationmesh references object {} which is a component object (no mesh). \
                                 The representationmesh must reference an object that contains a mesh.",
                            object.id, rep_id
                        )));
                    }

                    // Representation mesh MUST NOT contain a beamlattice
                    if let Some(ref rep_mesh) = rep_obj.mesh
                        && rep_mesh.beamset.is_some()
                    {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: BeamLattice representationmesh references object {} which contains a beamlattice. \
                                     Per the Beam Lattice spec, representationmesh objects MUST NOT contain a beamlattice.",
                            object.id, rep_id
                        )));
                    }
                }
            }

            // Validate clipping mode
            if let Some(ref clip_mode) = beamset.clipping_mode {
                // Check that clipping mode has valid value
                if clip_mode != "none" && clip_mode != "inside" && clip_mode != "outside" {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: BeamLattice has invalid clippingmode '{}'. \
                             Valid values are: 'none', 'inside', 'outside'.",
                        object.id, clip_mode
                    )));
                }

                // If clipping mode is specified (and not 'none'), must have clipping mesh
                if clip_mode != "none" && beamset.clipping_mesh_id.is_none() {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: BeamLattice has clippingmode='{}' but no clippingmesh attribute. \
                             When clippingmode is specified (other than 'none'), a clippingmesh must be provided.",
                        object.id, clip_mode
                    )));
                }
            }

            // Validate ball mode - only check if value is valid
            // Valid values are: "none", "all", "mixed"
            // Per Beam Lattice Balls sub-extension spec
            if let Some(ref ball_mode) = beamset.ball_mode {
                if ball_mode != "none" && ball_mode != "all" && ball_mode != "mixed" {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: BeamLattice has invalid ballmode '{}'. \
                             Valid values are: 'none', 'all', 'mixed'.",
                        object.id, ball_mode
                    )));
                }

                // If ballmode is 'all' or 'mixed', ballradius must be specified
                // Per Beam Lattice Balls sub-extension spec
                if (ball_mode == "all" || ball_mode == "mixed") && beamset.ball_radius.is_none() {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: BeamLattice has ballmode='{}' but no ballradius attribute. \
                             When ballmode is 'all' or 'mixed', ballradius must be specified.",
                        object.id, ball_mode
                    )));
                }
            }

            // Validate beamset material reference and property index
            if let Some(pid) = beamset.property_id {
                if !valid_resource_ids.contains(&(pid as usize)) {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: BeamLattice references non-existent property group ID {}. \
                             Property group IDs must reference existing color groups, base material groups, \
                             texture groups, composite materials, or multi-property groups.",
                        object.id, pid
                    )));
                }

                // Validate beamset pindex if present
                if let Some(pindex) = beamset.property_index {
                    // Check if it's a color group
                    if let Some(colorgroup) = model
                        .resources
                        .color_groups
                        .iter()
                        .find(|cg| cg.id == pid as usize)
                    {
                        if pindex as usize >= colorgroup.colors.len() {
                            let max_index = colorgroup.colors.len().saturating_sub(1);
                            return Err(Error::InvalidModel(format!(
                                "Object {}: BeamLattice pindex {} is out of bounds.\n\
                                     Color group {} has {} colors (valid indices: 0-{}).",
                                object.id,
                                pindex,
                                pid,
                                colorgroup.colors.len(),
                                max_index
                            )));
                        }
                    }
                    // Check if it's a base material group
                    else if let Some(basematerialgroup) = model
                        .resources
                        .base_material_groups
                        .iter()
                        .find(|bg| bg.id == pid as usize)
                        && pindex as usize >= basematerialgroup.materials.len()
                    {
                        let max_index = basematerialgroup.materials.len().saturating_sub(1);
                        return Err(Error::InvalidModel(format!(
                            "Object {}: BeamLattice pindex {} is out of bounds.\n\
                                     Base material group {} has {} materials (valid indices: 0-{}).",
                            object.id,
                            pindex,
                            pid,
                            basematerialgroup.materials.len(),
                            max_index
                        )));
                    }
                }
            }

            // Validate beam-level property indices (p1, p2)
            for (beam_idx, beam) in beamset.beams.iter().enumerate() {
                // Determine which property group to use for validation
                let pid_to_check = beam.property_id.or(beamset.property_id);

                if let Some(pid) = pid_to_check {
                    // Check if it's a color group
                    if let Some(colorgroup) = model
                        .resources
                        .color_groups
                        .iter()
                        .find(|cg| cg.id == pid as usize)
                    {
                        let num_colors = colorgroup.colors.len();

                        // Validate p1
                        if let Some(p1) = beam.p1
                            && p1 as usize >= num_colors
                        {
                            let max_index = num_colors.saturating_sub(1);
                            return Err(Error::InvalidModel(format!(
                                "Object {}: Beam {} p1 {} is out of bounds.\n\
                                         Color group {} has {} colors (valid indices: 0-{}).",
                                object.id, beam_idx, p1, pid, num_colors, max_index
                            )));
                        }

                        // Validate p2
                        if let Some(p2) = beam.p2
                            && p2 as usize >= num_colors
                        {
                            let max_index = num_colors.saturating_sub(1);
                            return Err(Error::InvalidModel(format!(
                                "Object {}: Beam {} p2 {} is out of bounds.\n\
                                         Color group {} has {} colors (valid indices: 0-{}).",
                                object.id, beam_idx, p2, pid, num_colors, max_index
                            )));
                        }
                    }
                    // Check if it's a base material group
                    else if let Some(basematerialgroup) = model
                        .resources
                        .base_material_groups
                        .iter()
                        .find(|bg| bg.id == pid as usize)
                    {
                        let num_materials = basematerialgroup.materials.len();

                        // Validate p1
                        if let Some(p1) = beam.p1
                            && p1 as usize >= num_materials
                        {
                            let max_index = num_materials.saturating_sub(1);
                            return Err(Error::InvalidModel(format!(
                                "Object {}: Beam {} p1 {} is out of bounds.\n\
                                         Base material group {} has {} materials (valid indices: 0-{}).",
                                object.id, beam_idx, p1, pid, num_materials, max_index
                            )));
                        }

                        // Validate p2
                        if let Some(p2) = beam.p2
                            && p2 as usize >= num_materials
                        {
                            let max_index = num_materials.saturating_sub(1);
                            return Err(Error::InvalidModel(format!(
                                "Object {}: Beam {} p2 {} is out of bounds.\n\
                                         Base material group {} has {} materials (valid indices: 0-{}).",
                                object.id, beam_idx, p2, pid, num_materials, max_index
                            )));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
