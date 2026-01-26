//! Boolean operations validation

use crate::error::{Error, Result};
use crate::model::Model;
use std::collections::{HashMap, HashSet};

use super::sorted_ids_from_set;

/// Validates boolean operation shapes and their references
pub fn validate_boolean_operations(model: &Model) -> Result<()> {
    // Build a set of valid object IDs for quick lookup
    let valid_object_ids: HashSet<usize> = model.resources.objects.iter().map(|o| o.id).collect();

    // Build a map of object ID to its index in the objects vector (for forward reference check)
    let object_indices: HashMap<usize, usize> = model
        .resources
        .objects
        .iter()
        .enumerate()
        .map(|(idx, obj)| (obj.id, idx))
        .collect();

    // Build a map of object ID to the object itself for type checking
    let object_map: HashMap<usize, &crate::model::Object> = model
        .resources
        .objects
        .iter()
        .map(|obj| (obj.id, obj))
        .collect();

    for (current_idx, object) in model.resources.objects.iter().enumerate() {
        if let Some(ref boolean_shape) = object.boolean_shape {
            // Per 3MF spec, objects containing booleanshape should be of type "model"
            // While the spec focuses on the BASE and OPERAND objects being type "model",
            // it's also implied that the containing object should be type="model" since
            // other types (support, solidsupport, surface, other) are for specific purposes
            if object.object_type != crate::model::ObjectType::Model {
                return Err(Error::InvalidModel(format!(
                    "Object {}: Object containing booleanshape must be of type \"model\", but is type \"{:?}\".\n\
                     Objects with boolean shapes should be model objects, not support, surface, or other types.",
                    object.id, object.object_type
                )));
            }

            // Per 3MF Boolean Operations Extension spec, an object can have
            // EITHER a mesh, components, OR booleanshape, but not multiple
            // Check that object with booleanshape doesn't also have components or mesh
            if !object.components.is_empty() {
                return Err(Error::InvalidModel(format!(
                    "Object {}: Contains both <booleanshape> and <components>.\n\
                     Per 3MF Boolean Operations spec, an object can contain either a mesh, \
                     components, or booleanshape, but not multiple of these.\n\
                     Remove either the booleanshape or the components from this object.",
                    object.id
                )));
            }

            if object.mesh.is_some() {
                return Err(Error::InvalidModel(format!(
                    "Object {}: Contains both <booleanshape> and <mesh>.\n\
                     Per 3MF Boolean Operations spec, an object can contain either a mesh, \
                     components, or booleanshape, but not multiple of these.\n\
                     Remove either the booleanshape or the mesh from this object.",
                    object.id
                )));
            }

            // Check that booleanshape has at least one operand
            if boolean_shape.operands.is_empty() {
                return Err(Error::InvalidModel(format!(
                    "Object {}: Boolean shape has no operands.\n\
                     Per 3MF Boolean Operations spec, <booleanshape> must contain \
                     one or more <boolean> elements to define the operands.\n\
                     Add at least one <boolean> element inside the <booleanshape>.",
                    object.id
                )));
            }

            // Validate the base object ID exists (skip if it has a path to an external file)
            if boolean_shape.path.is_none() {
                if !valid_object_ids.contains(&boolean_shape.objectid) {
                    let available_ids = sorted_ids_from_set(&valid_object_ids);
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Boolean shape references non-existent object ID {}.\n\
                         Available object IDs: {:?}\n\
                         Hint: Ensure the referenced object exists in the <resources> section.",
                        object.id, boolean_shape.objectid, available_ids
                    )));
                }

                // Check forward reference: base object must be defined before this object
                if let Some(&base_idx) = object_indices.get(&boolean_shape.objectid) {
                    if base_idx >= current_idx {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Boolean shape references object {} which is defined after it.\n\
                             Per 3MF spec, objects must be defined before they are referenced.\n\
                             Move object {} definition before object {} in the file.",
                            object.id, boolean_shape.objectid, boolean_shape.objectid, object.id
                        )));
                    }
                }

                // Check that base object is of type "model"
                if let Some(base_obj) = object_map.get(&boolean_shape.objectid) {
                    if base_obj.object_type != crate::model::ObjectType::Model {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Boolean shape base object {} must be of type \"model\", but is type \"{:?}\".\n\
                             Per 3MF Boolean Operations spec, only model objects can be used as base objects in boolean operations.\n\
                             Change the base object's type attribute to \"model\" or reference a different object.",
                            object.id, boolean_shape.objectid, base_obj.object_type
                        )));
                    }

                    // Check that base object has a shape (mesh, booleanshape, or extension shapes), not just components
                    // Per Boolean Operations spec, the base object "MUST NOT reference a components object"
                    // An object without mesh/boolean_shape but also without components is assumed to have extension shapes
                    let has_shape = base_obj.mesh.is_some()
                        || base_obj.boolean_shape.is_some()
                        || base_obj.has_extension_shapes
                        || (base_obj.components.is_empty()); // If no mesh/boolean and no components, assume extension shape

                    if !has_shape {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Boolean shape base object {} does not define a shape.\n\
                             Per 3MF Boolean Operations spec, the base object must define a shape \
                             (mesh, displacementmesh, booleanshape, or shapes from other extensions), not just an assembly of components.",
                            object.id, boolean_shape.objectid
                        )));
                    }
                }
            }

            // Validate all operand object IDs exist (skip external references)
            for operand in &boolean_shape.operands {
                if operand.path.is_none() {
                    if !valid_object_ids.contains(&operand.objectid) {
                        let available_ids = sorted_ids_from_set(&valid_object_ids);
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Boolean operand references non-existent object ID {}.\n\
                             Available object IDs: {:?}\n\
                             Hint: Ensure the referenced object exists in the <resources> section.",
                            object.id, operand.objectid, available_ids
                        )));
                    }

                    // Check forward reference: operand object must be defined before this object
                    if let Some(&operand_idx) = object_indices.get(&operand.objectid) {
                        if operand_idx >= current_idx {
                            return Err(Error::InvalidModel(format!(
                                "Object {}: Boolean operand references object {} which is defined after it.\n\
                                 Per 3MF spec, objects must be defined before they are referenced.\n\
                                 Move object {} definition before object {} in the file.",
                                object.id, operand.objectid, operand.objectid, object.id
                            )));
                        }
                    }

                    // Check that operand object is of type "model" and is a mesh
                    if let Some(operand_obj) = object_map.get(&operand.objectid) {
                        if operand_obj.object_type != crate::model::ObjectType::Model {
                            return Err(Error::InvalidModel(format!(
                                "Object {}: Boolean operand object {} must be of type \"model\", but is type \"{:?}\".\n\
                                 Per 3MF Boolean Operations spec, only model objects can be used in boolean operations.\n\
                                 Change the operand object's type attribute to \"model\" or reference a different object.",
                                object.id, operand.objectid, operand_obj.object_type
                            )));
                        }

                        // Per spec, operand must be a triangle mesh object only
                        // It MUST NOT contain shapes defined in any other extension
                        if operand_obj.mesh.is_none() {
                            return Err(Error::InvalidModel(format!(
                                "Object {}: Boolean operand object {} must be a triangle mesh.\n\
                                 Per 3MF Boolean Operations spec, operands must be mesh objects only.",
                                object.id, operand.objectid
                            )));
                        }

                        // Check that operand doesn't have booleanshape or components
                        // Per spec: "MUST be only a triangle mesh object" and
                        // "MUST NOT contain shapes defined in any other extension"
                        if operand_obj.boolean_shape.is_some() {
                            return Err(Error::InvalidModel(format!(
                                "Object {}: Boolean operand object {} has a boolean shape.\n\
                                 Per 3MF Boolean Operations spec, operands must be simple triangle meshes only.",
                                object.id, operand.objectid
                            )));
                        }

                        if !operand_obj.components.is_empty() {
                            return Err(Error::InvalidModel(format!(
                                "Object {}: Boolean operand object {} has components.\n\
                                 Per 3MF Boolean Operations spec, operands must be simple triangle meshes only.",
                                object.id, operand.objectid
                            )));
                        }

                        // Check for extension shape elements (beamlattice, displacement, etc.)
                        if operand_obj.has_extension_shapes {
                            return Err(Error::InvalidModel(format!(
                                "Object {}: Boolean operand object {} contains extension shape elements (e.g., beamlattice).\n\
                                 Per 3MF Boolean Operations spec, operands MUST be only triangle mesh objects \
                                 and MUST NOT contain shapes defined in any other extension.",
                                object.id, operand.objectid
                            )));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
