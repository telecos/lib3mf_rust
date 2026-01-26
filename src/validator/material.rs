//! Material validation functions

use crate::error::{Error, Result};
use crate::model::Model;
use std::collections::{HashMap, HashSet};

use super::sorted_ids_from_set;

/// Validates material property group references and uniqueness
pub fn validate_material_references(model: &Model) -> Result<()> {
    // Validate that all property group IDs are unique across all property group types
    // Property groups include: color groups, base material groups, multiproperties,
    // texture2d groups, and composite materials
    let mut seen_property_group_ids: HashMap<usize, String> = HashMap::new();

    // Check color group IDs
    for colorgroup in &model.resources.color_groups {
        if let Some(existing_type) =
            seen_property_group_ids.insert(colorgroup.id, "colorgroup".to_string())
        {
            return Err(Error::InvalidModel(format!(
                "Duplicate resource ID: {}. \
                 This ID is used by both a {} and a colorgroup. \
                 Each resource must have a unique id attribute. \
                 Check your material definitions for duplicate IDs.",
                colorgroup.id, existing_type
            )));
        }
    }

    // Check base material group IDs
    for basematerialgroup in &model.resources.base_material_groups {
        if let Some(existing_type) =
            seen_property_group_ids.insert(basematerialgroup.id, "basematerials".to_string())
        {
            return Err(Error::InvalidModel(format!(
                "Duplicate resource ID: {}. \
                 This ID is used by both a {} and a basematerials group. \
                 Each resource must have a unique id attribute. \
                 Check your material definitions for duplicate IDs.",
                basematerialgroup.id, existing_type
            )));
        }
    }

    // Check multiproperties IDs
    for multiprop in &model.resources.multi_properties {
        if let Some(existing_type) =
            seen_property_group_ids.insert(multiprop.id, "multiproperties".to_string())
        {
            return Err(Error::InvalidModel(format!(
                "Duplicate resource ID: {}. \
                 This ID is used by both a {} and a multiproperties group. \
                 Each resource must have a unique id attribute. \
                 Check your material definitions for duplicate IDs.",
                multiprop.id, existing_type
            )));
        }
    }

    // Check texture2d group IDs
    for tex2dgroup in &model.resources.texture2d_groups {
        if let Some(existing_type) =
            seen_property_group_ids.insert(tex2dgroup.id, "texture2dgroup".to_string())
        {
            return Err(Error::InvalidModel(format!(
                "Duplicate resource ID: {}. \
                 This ID is used by both a {} and a texture2dgroup. \
                 Each resource must have a unique id attribute. \
                 Check your material definitions for duplicate IDs.",
                tex2dgroup.id, existing_type
            )));
        }
    }

    // Check composite materials IDs
    for composite in &model.resources.composite_materials {
        if let Some(existing_type) =
            seen_property_group_ids.insert(composite.id, "compositematerials".to_string())
        {
            return Err(Error::InvalidModel(format!(
                "Duplicate resource ID: {}. \
                 This ID is used by both a {} and a compositematerials group. \
                 Each resource must have a unique id attribute. \
                 Check your material definitions for duplicate IDs.",
                composite.id, existing_type
            )));
        }
    }

    // Keep separate base material IDs set for basematerialid validation
    let valid_basematerial_ids: HashSet<usize> = model
        .resources
        .base_material_groups
        .iter()
        .map(|bg| bg.id)
        .collect();

    // Validate multiproperties: each multi element's pindices must be valid for the referenced property groups
    for multiprop in &model.resources.multi_properties {
        for (multi_idx, multi) in multiprop.multis.iter().enumerate() {
            // Validate each pindex against the corresponding property group
            // Note: pindices.len() can be less than pids.len() - unspecified indices default to 0
            for (layer_idx, (&pid, &pindex)) in
                multiprop.pids.iter().zip(multi.pindices.iter()).enumerate()
            {
                // Check if it's a color group
                if let Some(colorgroup) =
                    model.resources.color_groups.iter().find(|cg| cg.id == pid)
                {
                    if pindex >= colorgroup.colors.len() {
                        let max_index = colorgroup.colors.len().saturating_sub(1);
                        return Err(Error::InvalidModel(format!(
                            "MultiProperties group {}: Multi element {} layer {} references pindex {} which is out of bounds.\n\
                             Color group {} has {} colors (valid indices: 0-{}).\n\
                             Hint: Each pindex in a multi element must be less than the number of items in the corresponding property group.",
                            multiprop.id,
                            multi_idx,
                            layer_idx,
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
                    .find(|bg| bg.id == pid)
                {
                    if pindex >= basematerialgroup.materials.len() {
                        let max_index = basematerialgroup.materials.len().saturating_sub(1);
                        return Err(Error::InvalidModel(format!(
                            "MultiProperties group {}: Multi element {} layer {} references pindex {} which is out of bounds.\n\
                             Base material group {} has {} materials (valid indices: 0-{}).\n\
                             Hint: Each pindex in a multi element must be less than the number of items in the corresponding property group.",
                            multiprop.id,
                            multi_idx,
                            layer_idx,
                            pindex,
                            pid,
                            basematerialgroup.materials.len(),
                            max_index
                        )));
                    }
                }
                // Check if it's a texture2d group
                else if let Some(tex2dgroup) = model
                    .resources
                    .texture2d_groups
                    .iter()
                    .find(|tg| tg.id == pid)
                {
                    if pindex >= tex2dgroup.tex2coords.len() {
                        let max_index = tex2dgroup.tex2coords.len().saturating_sub(1);
                        return Err(Error::InvalidModel(format!(
                            "MultiProperties group {}: Multi element {} layer {} references pindex {} which is out of bounds.\n\
                             Texture2D group {} has {} texture coordinates (valid indices: 0-{}).\n\
                             Hint: Each pindex in a multi element must be less than the number of items in the corresponding property group.",
                            multiprop.id,
                            multi_idx,
                            layer_idx,
                            pindex,
                            pid,
                            tex2dgroup.tex2coords.len(),
                            max_index
                        )));
                    }
                }
                // Check if it's a composite materials group
                else if let Some(composite) = model
                    .resources
                    .composite_materials
                    .iter()
                    .find(|cm| cm.id == pid)
                {
                    if pindex >= composite.composites.len() {
                        let max_index = composite.composites.len().saturating_sub(1);
                        return Err(Error::InvalidModel(format!(
                            "MultiProperties group {}: Multi element {} layer {} references pindex {} which is out of bounds.\n\
                             Composite materials group {} has {} composite elements (valid indices: 0-{}).\n\
                             Hint: Each pindex in a multi element must be less than the number of items in the corresponding property group.",
                            multiprop.id,
                            multi_idx,
                            layer_idx,
                            pindex,
                            pid,
                            composite.composites.len(),
                            max_index
                        )));
                    }
                }
                // If the pid is another multiproperties, we don't need to validate here
                // as nested multiproperties would be validated separately
            }
        }
    }

    for object in &model.resources.objects {
        if let Some(pid) = object.pid {
            // If object has a pid, it should reference a valid property group
            // Only validate if there are property groups defined, otherwise pid might be unused
            if !seen_property_group_ids.is_empty() && !seen_property_group_ids.contains_key(&pid) {
                let available_ids: Vec<usize> = {
                    let mut ids: Vec<usize> = seen_property_group_ids.keys().copied().collect();
                    ids.sort();
                    ids
                };
                return Err(Error::InvalidModel(format!(
                    "Object {} references non-existent property group ID: {}.\n\
                     Available property group IDs: {:?}\n\
                     Hint: Check that all referenced property groups are defined in the <resources> section.",
                    object.id, pid, available_ids
                )));
            }
        }

        // Validate basematerialid references
        if let Some(basematerialid) = object.basematerialid {
            // basematerialid should reference a valid base material group
            if !valid_basematerial_ids.contains(&basematerialid) {
                let available_ids = sorted_ids_from_set(&valid_basematerial_ids);
                return Err(Error::InvalidModel(format!(
                    "Object {} references non-existent base material group ID: {}.\n\
                     Available base material group IDs: {:?}\n\
                     Hint: Check that a basematerials group with this ID exists in the <resources> section.",
                    object.id, basematerialid, available_ids
                )));
            }
        }

        // Validate object pindex references for color groups
        if let Some(obj_pid) = object.pid {
            if let Some(colorgroup) = model
                .resources
                .color_groups
                .iter()
                .find(|cg| cg.id == obj_pid)
            {
                // Validate object-level pindex
                if let Some(pindex) = object.pindex {
                    if pindex >= colorgroup.colors.len() {
                        let max_index = colorgroup.colors.len().saturating_sub(1);
                        return Err(Error::InvalidModel(format!(
                            "Object {}: pindex {} is out of bounds.\n\
                             Color group {} has {} colors (valid indices: 0-{}).\n\
                             Hint: pindex must be less than the number of colors in the color group.",
                            object.id,
                            pindex,
                            obj_pid,
                            colorgroup.colors.len(),
                            max_index
                        )));
                    }
                }
            }
            // Validate object pindex references for base material groups
            else if let Some(basematerialgroup) = model
                .resources
                .base_material_groups
                .iter()
                .find(|bg| bg.id == obj_pid)
            {
                // Validate object-level pindex
                if let Some(pindex) = object.pindex {
                    if pindex >= basematerialgroup.materials.len() {
                        let max_index = basematerialgroup.materials.len().saturating_sub(1);
                        return Err(Error::InvalidModel(format!(
                            "Object {}: pindex {} is out of bounds.\n\
                             Base material group {} has {} materials (valid indices: 0-{}).\n\
                             Hint: pindex must be less than the number of materials in the base material group.",
                            object.id,
                            pindex,
                            obj_pid,
                            basematerialgroup.materials.len(),
                            max_index
                        )));
                    }
                }
            }
            // Validate object pindex references for texture2d groups
            else if let Some(tex2dgroup) = model
                .resources
                .texture2d_groups
                .iter()
                .find(|tg| tg.id == obj_pid)
            {
                // Validate object-level pindex
                if let Some(pindex) = object.pindex {
                    if pindex >= tex2dgroup.tex2coords.len() {
                        let max_index = tex2dgroup.tex2coords.len().saturating_sub(1);
                        return Err(Error::InvalidModel(format!(
                            "Object {}: pindex {} is out of bounds.\n\
                             Texture2D group {} has {} texture coordinates (valid indices: 0-{}).\n\
                             Hint: pindex must be less than the number of texture coordinates in the texture2d group.",
                            object.id,
                            pindex,
                            obj_pid,
                            tex2dgroup.tex2coords.len(),
                            max_index
                        )));
                    }
                }
            }
            // Validate object pindex references for multiproperties
            else if let Some(multiprop) = model
                .resources
                .multi_properties
                .iter()
                .find(|mp| mp.id == obj_pid)
            {
                // Validate object-level pindex
                if let Some(pindex) = object.pindex {
                    if pindex >= multiprop.multis.len() {
                        let max_index = multiprop.multis.len().saturating_sub(1);
                        return Err(Error::InvalidModel(format!(
                            "Object {}: pindex {} is out of bounds.\n\
                             MultiProperties group {} has {} multi elements (valid indices: 0-{}).\n\
                             Hint: pindex must be less than the number of multi elements in the multiproperties group.",
                            object.id,
                            pindex,
                            obj_pid,
                            multiprop.multis.len(),
                            max_index
                        )));
                    }
                }
            }
            // Validate object pindex references for composite materials
            else if let Some(composite) = model
                .resources
                .composite_materials
                .iter()
                .find(|cm| cm.id == obj_pid)
            {
                // Validate object-level pindex
                if let Some(pindex) = object.pindex {
                    if pindex >= composite.composites.len() {
                        let max_index = composite.composites.len().saturating_sub(1);
                        return Err(Error::InvalidModel(format!(
                            "Object {}: pindex {} is out of bounds.\n\
                             Composite materials group {} has {} composite elements (valid indices: 0-{}).\n\
                             Hint: pindex must be less than the number of composite elements in the composite materials group.",
                            object.id,
                            pindex,
                            obj_pid,
                            composite.composites.len(),
                            max_index
                        )));
                    }
                }
            }
        }

        // Validate triangle property index references for color groups and base materials
        if let Some(ref mesh) = object.mesh {
            for (tri_idx, triangle) in mesh.triangles.iter().enumerate() {
                // Determine which color group or base material to use for validation
                let pid_to_check = triangle.pid.or(object.pid);

                if let Some(pid) = pid_to_check {
                    // Check if it's a color group
                    if let Some(colorgroup) =
                        model.resources.color_groups.iter().find(|cg| cg.id == pid)
                    {
                        let num_colors = colorgroup.colors.len();

                        // Validate triangle-level pindex
                        if let Some(pindex) = triangle.pindex {
                            if pindex >= num_colors {
                                let max_index = num_colors.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} pindex {} is out of bounds.\n\
                                     Color group {} has {} colors (valid indices: 0-{}).\n\
                                     Hint: pindex must be less than the number of colors in the color group.",
                                    object.id, tri_idx, pindex, pid, num_colors, max_index
                                )));
                            }
                        }

                        // Validate per-vertex property indices (p1, p2, p3)
                        if let Some(p1) = triangle.p1 {
                            if p1 >= num_colors {
                                let max_index = num_colors.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p1 {} is out of bounds.\n\
                                     Color group {} has {} colors (valid indices: 0-{}).\n\
                                     Hint: p1 must be less than the number of colors in the color group.",
                                    object.id, tri_idx, p1, pid, num_colors, max_index
                                )));
                            }
                        }

                        if let Some(p2) = triangle.p2 {
                            if p2 >= num_colors {
                                let max_index = num_colors.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p2 {} is out of bounds.\n\
                                     Color group {} has {} colors (valid indices: 0-{}).\n\
                                     Hint: p2 must be less than the number of colors in the color group.",
                                    object.id, tri_idx, p2, pid, num_colors, max_index
                                )));
                            }
                        }

                        if let Some(p3) = triangle.p3 {
                            if p3 >= num_colors {
                                let max_index = num_colors.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p3 {} is out of bounds.\n\
                                     Color group {} has {} colors (valid indices: 0-{}).\n\
                                     Hint: p3 must be less than the number of colors in the color group.",
                                    object.id, tri_idx, p3, pid, num_colors, max_index
                                )));
                            }
                        }
                    }
                    // Check if it's a base material group
                    else if let Some(basematerialgroup) = model
                        .resources
                        .base_material_groups
                        .iter()
                        .find(|bg| bg.id == pid)
                    {
                        let num_materials = basematerialgroup.materials.len();

                        // Validate triangle-level pindex
                        if let Some(pindex) = triangle.pindex {
                            if pindex >= num_materials {
                                let max_index = num_materials.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} pindex {} is out of bounds.\n\
                                     Base material group {} has {} materials (valid indices: 0-{}).\n\
                                     Hint: pindex must be less than the number of materials in the base material group.",
                                    object.id, tri_idx, pindex, pid, num_materials, max_index
                                )));
                            }
                        }

                        // Validate per-vertex property indices (p1, p2, p3)
                        if let Some(p1) = triangle.p1 {
                            if p1 >= num_materials {
                                let max_index = num_materials.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p1 {} is out of bounds.\n\
                                     Base material group {} has {} materials (valid indices: 0-{}).\n\
                                     Hint: p1 must be less than the number of materials in the base material group.",
                                    object.id, tri_idx, p1, pid, num_materials, max_index
                                )));
                            }
                        }

                        if let Some(p2) = triangle.p2 {
                            if p2 >= num_materials {
                                let max_index = num_materials.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p2 {} is out of bounds.\n\
                                     Base material group {} has {} materials (valid indices: 0-{}).\n\
                                     Hint: p2 must be less than the number of materials in the base material group.",
                                    object.id, tri_idx, p2, pid, num_materials, max_index
                                )));
                            }
                        }

                        if let Some(p3) = triangle.p3 {
                            if p3 >= num_materials {
                                let max_index = num_materials.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p3 {} is out of bounds.\n\
                                     Base material group {} has {} materials (valid indices: 0-{}).\n\
                                     Hint: p3 must be less than the number of materials in the base material group.",
                                    object.id, tri_idx, p3, pid, num_materials, max_index
                                )));
                            }
                        }
                    }
                    // Check if it's a texture2d group
                    else if let Some(tex2dgroup) = model
                        .resources
                        .texture2d_groups
                        .iter()
                        .find(|tg| tg.id == pid)
                    {
                        let num_coords = tex2dgroup.tex2coords.len();

                        // Validate triangle-level pindex
                        if let Some(pindex) = triangle.pindex {
                            if pindex >= num_coords {
                                let max_index = num_coords.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} pindex {} is out of bounds.\n\
                                     Texture2D group {} has {} texture coordinates (valid indices: 0-{}).\n\
                                     Hint: pindex must be less than the number of texture coordinates in the texture2d group.",
                                    object.id, tri_idx, pindex, pid, num_coords, max_index
                                )));
                            }
                        }

                        // Validate per-vertex property indices (p1, p2, p3)
                        if let Some(p1) = triangle.p1 {
                            if p1 >= num_coords {
                                let max_index = num_coords.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p1 {} is out of bounds.\n\
                                     Texture2D group {} has {} texture coordinates (valid indices: 0-{}).\n\
                                     Hint: p1 must be less than the number of texture coordinates in the texture2d group.",
                                    object.id, tri_idx, p1, pid, num_coords, max_index
                                )));
                            }
                        }

                        if let Some(p2) = triangle.p2 {
                            if p2 >= num_coords {
                                let max_index = num_coords.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p2 {} is out of bounds.\n\
                                     Texture2D group {} has {} texture coordinates (valid indices: 0-{}).\n\
                                     Hint: p2 must be less than the number of texture coordinates in the texture2d group.",
                                    object.id, tri_idx, p2, pid, num_coords, max_index
                                )));
                            }
                        }

                        if let Some(p3) = triangle.p3 {
                            if p3 >= num_coords {
                                let max_index = num_coords.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p3 {} is out of bounds.\n\
                                     Texture2D group {} has {} texture coordinates (valid indices: 0-{}).\n\
                                     Hint: p3 must be less than the number of texture coordinates in the texture2d group.",
                                    object.id, tri_idx, p3, pid, num_coords, max_index
                                )));
                            }
                        }
                    }
                    // Check if it's a multiproperties group
                    else if let Some(multiprop) = model
                        .resources
                        .multi_properties
                        .iter()
                        .find(|mp| mp.id == pid)
                    {
                        let num_multis = multiprop.multis.len();

                        // Validate triangle-level pindex
                        if let Some(pindex) = triangle.pindex {
                            if pindex >= num_multis {
                                let max_index = num_multis.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} pindex {} is out of bounds.\n\
                                     MultiProperties group {} has {} multi elements (valid indices: 0-{}).\n\
                                     Hint: pindex must be less than the number of multi elements in the multiproperties group.",
                                    object.id, tri_idx, pindex, pid, num_multis, max_index
                                )));
                            }
                        }

                        // Validate per-vertex property indices (p1, p2, p3)
                        if let Some(p1) = triangle.p1 {
                            if p1 >= num_multis {
                                let max_index = num_multis.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p1 {} is out of bounds.\n\
                                     MultiProperties group {} has {} multi elements (valid indices: 0-{}).\n\
                                     Hint: p1 must be less than the number of multi elements in the multiproperties group.",
                                    object.id, tri_idx, p1, pid, num_multis, max_index
                                )));
                            }
                        }

                        if let Some(p2) = triangle.p2 {
                            if p2 >= num_multis {
                                let max_index = num_multis.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p2 {} is out of bounds.\n\
                                     MultiProperties group {} has {} multi elements (valid indices: 0-{}).\n\
                                     Hint: p2 must be less than the number of multi elements in the multiproperties group.",
                                    object.id, tri_idx, p2, pid, num_multis, max_index
                                )));
                            }
                        }

                        if let Some(p3) = triangle.p3 {
                            if p3 >= num_multis {
                                let max_index = num_multis.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p3 {} is out of bounds.\n\
                                     MultiProperties group {} has {} multi elements (valid indices: 0-{}).\n\
                                     Hint: p3 must be less than the number of multi elements in the multiproperties group.",
                                    object.id, tri_idx, p3, pid, num_multis, max_index
                                )));
                            }
                        }
                    }
                    // Check if it's a composite materials group
                    else if let Some(composite) = model
                        .resources
                        .composite_materials
                        .iter()
                        .find(|cm| cm.id == pid)
                    {
                        let num_composites = composite.composites.len();

                        // Validate triangle-level pindex
                        if let Some(pindex) = triangle.pindex {
                            if pindex >= num_composites {
                                let max_index = num_composites.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} pindex {} is out of bounds.\n\
                                     Composite materials group {} has {} composite elements (valid indices: 0-{}).\n\
                                     Hint: pindex must be less than the number of composite elements in the composite materials group.",
                                    object.id, tri_idx, pindex, pid, num_composites, max_index
                                )));
                            }
                        }

                        // Validate per-vertex property indices (p1, p2, p3)
                        if let Some(p1) = triangle.p1 {
                            if p1 >= num_composites {
                                let max_index = num_composites.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p1 {} is out of bounds.\n\
                                     Composite materials group {} has {} composite elements (valid indices: 0-{}).\n\
                                     Hint: p1 must be less than the number of composite elements in the composite materials group.",
                                    object.id, tri_idx, p1, pid, num_composites, max_index
                                )));
                            }
                        }

                        if let Some(p2) = triangle.p2 {
                            if p2 >= num_composites {
                                let max_index = num_composites.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p2 {} is out of bounds.\n\
                                     Composite materials group {} has {} composite elements (valid indices: 0-{}).\n\
                                     Hint: p2 must be less than the number of composite elements in the composite materials group.",
                                    object.id, tri_idx, p2, pid, num_composites, max_index
                                )));
                            }
                        }

                        if let Some(p3) = triangle.p3 {
                            if p3 >= num_composites {
                                let max_index = num_composites.saturating_sub(1);
                                return Err(Error::InvalidModel(format!(
                                    "Object {}: Triangle {} p3 {} is out of bounds.\n\
                                     Composite materials group {} has {} composite elements (valid indices: 0-{}).\n\
                                     Hint: p3 must be less than the number of composite elements in the composite materials group.",
                                    object.id, tri_idx, p3, pid, num_composites, max_index
                                )));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Validate boolean operation references
///
/// Per 3MF Boolean Operations Extension spec:
/// - Objects referenced in boolean operations exist (unless they're external via path attribute)
/// - Objects don't have more than one booleanshape element (checked during parsing)
/// - Boolean operand objects exist (unless they're external via path attribute)
/// - Base object and operand objects must be of type "model" (not support, solidsupport, etc.)
/// - Referenced objects must be defined before the object containing the booleanshape (forward reference rule)
/// - Base object must define a shape (mesh or booleanshape), not components
/// - Operand objects must be triangle meshes only
///
/// Validates that texture paths are in /3D/Textures/ directory
pub fn validate_texture_paths(model: &Model) -> Result<()> {
    // Get list of encrypted files to skip validation for them
    let encrypted_files: Vec<String> = model
        .secure_content
        .as_ref()
        .map(|sc| sc.encrypted_files.clone())
        .unwrap_or_default();

    for texture in &model.resources.texture2d_resources {
        // Skip validation for encrypted files (they may not follow standard paths)
        if encrypted_files.contains(&texture.path) {
            continue;
        }

        // N_XXM_0610_01: Check for empty or invalid texture paths
        if texture.path.is_empty() {
            return Err(Error::InvalidModel(format!(
                "Texture2D resource {}: Path is empty.\n\
                 Per 3MF Material Extension spec, texture path must reference a valid file in the package.",
                texture.id
            )));
        }

        // Check for obviously invalid path patterns (e.g., paths with null bytes, backslashes)
        if texture.path.contains('\0') {
            return Err(Error::InvalidModel(format!(
                "Texture2D resource {}: Path '{}' contains null bytes.\n\
                 Per 3MF Material Extension spec, texture paths must be valid OPC part names.",
                texture.id, texture.path
            )));
        }

        // Per OPC spec, part names should use forward slashes, not backslashes
        if texture.path.contains('\\') {
            return Err(Error::InvalidModel(format!(
                "Texture2D resource {}: Path '{}' contains backslashes.\n\
                 Per OPC specification, part names must use forward slashes ('/') as path separators, not backslashes ('\\').",
                texture.id, texture.path
            )));
        }

        // Check that the path contains only ASCII characters
        if !texture.path.is_ascii() {
            return Err(Error::InvalidModel(format!(
                "Texture2D resource {}: Path '{}' contains non-ASCII characters.\n\
                 Per 3MF Material Extension specification, texture paths must contain only ASCII characters.\n\
                 Hint: Remove Unicode or special characters from the texture path.",
                texture.id, texture.path
            )));
        }

        // Note: The 3MF Materials Extension spec does NOT require texture paths to be in
        // /3D/Texture/ or /3D/Textures/ directories. The spec only requires that:
        // 1. The path attribute specifies the part name of the texture data
        // 2. The texture must be the target of a 3D Texture relationship from the 3D Model part
        // Therefore, we do not validate the directory path here.

        // Validate content type
        let valid_content_types = ["image/png", "image/jpeg"];
        if !valid_content_types.contains(&texture.contenttype.as_str()) {
            return Err(Error::InvalidModel(format!(
                "Texture2D resource {}: Invalid contenttype '{}'.\n\
                 Per 3MF Material Extension spec, texture content type must be 'image/png' or 'image/jpeg'.\n\
                 Update the contenttype attribute to one of the supported values.",
                texture.id, texture.contenttype
            )));
        }
    }
    Ok(())
}

/// Validate color formats in color groups
///
/// Per 3MF Material Extension spec, colors are stored as RGBA tuples (u8, u8, u8, u8).
/// The parser already validates format during parsing, but this provides an additional check.
/// Validates that multiproperties reference valid property groups
pub fn validate_multiproperties_references(model: &Model) -> Result<()> {
    // Build sets of valid resource IDs
    let base_mat_ids: HashSet<usize> = model
        .resources
        .base_material_groups
        .iter()
        .map(|b| b.id)
        .collect();

    let color_group_ids: HashSet<usize> =
        model.resources.color_groups.iter().map(|c| c.id).collect();

    let tex_group_ids: HashSet<usize> = model
        .resources
        .texture2d_groups
        .iter()
        .map(|t| t.id)
        .collect();

    let composite_ids: HashSet<usize> = model
        .resources
        .composite_materials
        .iter()
        .map(|c| c.id)
        .collect();

    // Validate each multiproperties group
    for multi_props in &model.resources.multi_properties {
        // Track basematerials and colorgroup IDs to detect duplicates
        let mut base_mat_count: HashMap<usize, usize> = HashMap::new();
        let mut color_group_count: HashMap<usize, usize> = HashMap::new();

        for (idx, &pid) in multi_props.pids.iter().enumerate() {
            // Check if PID references a valid resource
            let is_valid = base_mat_ids.contains(&pid)
                || color_group_ids.contains(&pid)
                || tex_group_ids.contains(&pid)
                || composite_ids.contains(&pid);

            if !is_valid {
                return Err(Error::InvalidModel(format!(
                    "MultiProperties {}: PID {} at index {} does not reference a valid resource.\n\
                     Per 3MF spec, multiproperties pids must reference existing basematerials, \
                     colorgroup, texture2dgroup, or compositematerials resources.\n\
                     Ensure resource with ID {} exists in the <resources> section.",
                    multi_props.id, pid, idx, pid
                )));
            }

            // Track basematerials references
            if base_mat_ids.contains(&pid) {
                *base_mat_count.entry(pid).or_insert(0) += 1;

                // N_XXM_0604_03: basematerials MUST be at layer 0 (first position) if included
                // Per 3MF Material Extension spec Chapter 5: "A material, if included, MUST be
                // positioned as the first element in the list forming the first layer"
                if idx != 0 {
                    return Err(Error::InvalidModel(format!(
                        "MultiProperties {}: basematerials group {} referenced at layer {}.\n\
                         Per 3MF Material Extension spec, basematerials MUST be positioned as the first element \
                         (layer 0) in multiproperties pids when included.\n\
                         Move the basematerials reference to layer 0.",
                        multi_props.id, pid, idx
                    )));
                }
            }

            // Track colorgroup references
            if color_group_ids.contains(&pid) {
                *color_group_count.entry(pid).or_insert(0) += 1;
            }
        }

        // N_XXM_0604_01: Check that at most one colorgroup is referenced
        // Per 3MF Material Extension spec Chapter 5: "The pids list MUST NOT contain
        // more than one reference to a colorgroup"
        if color_group_count.len() > 1 {
            let color_ids: Vec<usize> = color_group_count.keys().copied().collect();
            return Err(Error::InvalidModel(format!(
                "MultiProperties {}: References multiple colorgroups {:?} in pids.\n\
                 Per 3MF Material Extension spec, multiproperties pids list MUST NOT contain \
                 more than one reference to a colorgroup.\n\
                 Remove all but one colorgroup reference from the pids list.",
                multi_props.id, color_ids
            )));
        }

        // Also check for duplicate references to the same colorgroup
        for (&color_id, &count) in &color_group_count {
            if count > 1 {
                return Err(Error::InvalidModel(format!(
                    "MultiProperties {}: References colorgroup {} multiple times in pids.\n\
                     Per 3MF Material Extension spec, multiproperties cannot reference the same colorgroup \
                     more than once in the pids list.",
                    multi_props.id, color_id
                )));
            }
        }

        // Check for duplicate basematerials references
        for (&base_id, &count) in &base_mat_count {
            if count > 1 {
                return Err(Error::InvalidModel(format!(
                    "MultiProperties {}: References basematerials group {} multiple times in pids.\n\
                     Per 3MF spec, multiproperties cannot reference the same basematerials group \
                     more than once in the pids list.",
                    multi_props.id, base_id
                )));
            }
        }
    }

    Ok(())
}

/// Validate triangle property attributes
///
/// Per 3MF Materials Extension spec section 4.1.1 (Triangle Properties):
/// - Triangles can have per-vertex properties (p1/p2/p3) to specify different properties for each vertex
/// - Partial specification (e.g., only p1 or only p1 and p2) is allowed and commonly used
/// - When unspecified, vertices inherit the default property from pid/pindex or object-level properties
///
/// Real-world usage: Files like kinect_scan.3mf use partial specification extensively (8,682 triangles
/// with only p1 specified), demonstrating this is valid and intentional usage per the spec.
///
/// Note: Earlier interpretation that ALL THREE must be specified was too strict and rejected
/// valid real-world files.
/// Helper function to validate triangle material properties for a single object
///
/// Checks:
/// - N_XXM_0601_02: Mixed material assignment requires default pid
/// - N_XXM_0601_01: Per-vertex properties require pid context
///
/// # Arguments
/// * `object_id` - The ID of the object being validated
/// * `object_pid` - The object's default pid (if any)
/// * `mesh` - The mesh containing triangles to validate
/// * `context` - Context string for error messages (e.g., "Object 1" or "External file 'x.model': Object 1")
///
/// # Returns
/// `Ok(())` if validation passes, `Err` with detailed message if validation fails
/// Validates that triangle material properties are correctly assigned
pub fn validate_triangle_properties(model: &Model) -> Result<()> {
    // Per 3MF Materials Extension spec:
    // - Triangles can have triangle-level properties (pid and/or pindex)
    // - Triangles can have per-vertex properties (p1, p2, p3) which work WITH pid for interpolation
    // - Having both pid and p1/p2/p3 is ALLOWED and is used for per-vertex material interpolation
    //
    // NOTE: After testing against positive test cases, we found that partial per-vertex
    // properties are actually allowed in some scenarios. The validation has been relaxed.

    // Validate object-level properties
    for object in &model.resources.objects {
        // Validate object-level pid/pindex
        if let (Some(pid), Some(pindex)) = (object.pid, object.pindex) {
            // Get the size of the property resource
            let property_size = get_property_resource_size(model, pid)?;

            // Validate pindex is within bounds
            if pindex >= property_size {
                return Err(Error::InvalidModel(format!(
                    "Object {} has pindex {} which is out of bounds. \
                     Property resource {} has only {} elements (valid indices: 0-{}).",
                    object.id,
                    pindex,
                    pid,
                    property_size,
                    property_size - 1
                )));
            }
        }

        // Validate triangle-level properties
        if let Some(ref mesh) = object.mesh {
            // Use helper function for triangle material validation
            validate_object_triangle_materials(
                object.id,
                object.pid,
                mesh,
                &format!("Object {}", object.id),
            )?;

            for triangle in &mesh.triangles {
                // Validate triangle pindex is within bounds for multi-properties
                if let (Some(pid), Some(pindex)) = (triangle.pid, triangle.pindex) {
                    // Check if pid references a multiproperties resource
                    if let Some(multi_props) = model
                        .resources
                        .multi_properties
                        .iter()
                        .find(|m| m.id == pid)
                    {
                        // pindex must be within bounds of the multiproperties entries
                        if pindex >= multi_props.multis.len() {
                            return Err(Error::InvalidModel(format!(
                                "Triangle in object {} has pindex {} which is out of bounds. \
                                 MultiProperties resource {} has only {} entries (valid indices: 0-{}).",
                                object.id, pindex, pid, multi_props.multis.len(), multi_props.multis.len() - 1
                            )));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Validates triangle material properties for a specific object
pub fn validate_object_triangle_materials(
    object_id: usize,
    object_pid: Option<usize>,
    mesh: &crate::model::Mesh,
    context: &str,
) -> Result<()> {
    // N_XXM_0601_02: If some triangles have material properties (pid or per-vertex)
    // and others don't, object must have a default pid to provide material for
    // triangles without explicit material properties
    let mut has_triangles_with_material = false;
    let mut has_triangles_without_material = false;

    for triangle in &mesh.triangles {
        let triangle_has_material = triangle.pid.is_some()
            || triangle.p1.is_some()
            || triangle.p2.is_some()
            || triangle.p3.is_some();

        if triangle_has_material {
            has_triangles_with_material = true;
        } else {
            has_triangles_without_material = true;
        }
    }

    // If we have mixed material assignment and no default pid on object, this is invalid
    let has_mixed_assignment_without_default_pid =
        has_triangles_with_material && has_triangles_without_material && object_pid.is_none();

    if has_mixed_assignment_without_default_pid {
        return Err(Error::InvalidModel(format!(
            "{} has some triangles with material properties and some without. \
             When triangles in an object have mixed material assignment, \
             the object must have a default pid attribute to provide material \
             for triangles without explicit material properties. \
             Add a pid attribute to object {}.",
            context, object_id
        )));
    }

    // N_XXM_0601_01: Validate per-vertex properties
    for triangle in &mesh.triangles {
        let has_per_vertex_properties =
            triangle.p1.is_some() || triangle.p2.is_some() || triangle.p3.is_some();

        if has_per_vertex_properties && triangle.pid.is_none() && object_pid.is_none() {
            return Err(Error::InvalidModel(format!(
                "{} has a triangle with per-vertex material properties (p1/p2/p3) \
                 but neither the triangle nor the object has a pid to provide material context.\n\
                 Per 3MF Material Extension spec, per-vertex properties require a pid, \
                 either on the triangle or as a default on the object.\n\
                 Add a pid attribute to either the triangle or object {}.",
                context, object_id
            )));
        }
    }

    Ok(())
}

/// Gets the size (number of properties) in a property resource group
pub fn get_property_resource_size(model: &Model, resource_id: usize) -> Result<usize> {
    // Check colorgroup
    if let Some(color_group) = model
        .resources
        .color_groups
        .iter()
        .find(|c| c.id == resource_id)
    {
        if color_group.colors.is_empty() {
            return Err(Error::InvalidModel(format!(
                "ColorGroup {} has no colors. Per 3MF Materials Extension spec, \
                 color groups must contain at least one color element.",
                resource_id
            )));
        }
        return Ok(color_group.colors.len());
    }

    // Check texture2dgroup
    if let Some(tex_group) = model
        .resources
        .texture2d_groups
        .iter()
        .find(|t| t.id == resource_id)
    {
        if tex_group.tex2coords.is_empty() {
            return Err(Error::InvalidModel(format!(
                "Texture2DGroup {} has no texture coordinates. Per 3MF Materials Extension spec, \
                 texture2dgroup must contain at least one tex2coord element.",
                resource_id
            )));
        }
        return Ok(tex_group.tex2coords.len());
    }

    // Check compositematerials
    if let Some(composite) = model
        .resources
        .composite_materials
        .iter()
        .find(|c| c.id == resource_id)
    {
        if composite.composites.is_empty() {
            return Err(Error::InvalidModel(format!(
                "CompositeMaterials {} has no composite elements. Per 3MF Materials Extension spec, \
                 compositematerials must contain at least one composite element.",
                resource_id
            )));
        }
        return Ok(composite.composites.len());
    }

    // Check basematerials
    if let Some(base_mat) = model
        .resources
        .base_material_groups
        .iter()
        .find(|b| b.id == resource_id)
    {
        if base_mat.materials.is_empty() {
            return Err(Error::InvalidModel(format!(
                "BaseMaterials {} has no base material elements. Per 3MF spec, \
                 basematerials must contain at least one base element.",
                resource_id
            )));
        }
        return Ok(base_mat.materials.len());
    }

    // Check multiproperties
    if let Some(multi_props) = model
        .resources
        .multi_properties
        .iter()
        .find(|m| m.id == resource_id)
    {
        if multi_props.multis.is_empty() {
            return Err(Error::InvalidModel(format!(
                "MultiProperties {} has no multi elements. Per 3MF Materials Extension spec, \
                 multiproperties must contain at least one multi element.",
                resource_id
            )));
        }
        return Ok(multi_props.multis.len());
    }

    // Resource not found or not a property resource
    Err(Error::InvalidModel(format!(
        "Property resource {} not found or is not a valid property resource type",
        resource_id
    )))
}
