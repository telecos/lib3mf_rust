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
                if let Some(&base_idx) = object_indices.get(&boolean_shape.objectid)
                    && base_idx >= current_idx
                {
                    return Err(Error::InvalidModel(format!(
                        "Object {}: Boolean shape references object {} which is defined after it.\n\
                             Per 3MF spec, objects must be defined before they are referenced.\n\
                             Move object {} definition before object {} in the file.",
                        object.id, boolean_shape.objectid, boolean_shape.objectid, object.id
                    )));
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
                    if let Some(&operand_idx) = object_indices.get(&operand.objectid)
                        && operand_idx >= current_idx
                    {
                        return Err(Error::InvalidModel(format!(
                            "Object {}: Boolean operand references object {} which is defined after it.\n\
                                 Per 3MF spec, objects must be defined before they are referenced.\n\
                                 Move object {} definition before object {} in the file.",
                            object.id, operand.objectid, operand.objectid, object.id
                        )));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        BooleanOpType, BooleanRef, BooleanShape, Component, Mesh, Model, Object, ObjectType,
        Triangle, Vertex,
    };

    fn make_simple_mesh() -> Mesh {
        let mut mesh = Mesh::new();
        mesh.vertices.push(Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(Vertex::new(0.0, 1.0, 0.0));
        mesh.triangles.push(Triangle::new(0, 1, 2));
        mesh
    }

    /// A minimal valid boolean model:
    /// - object 1: mesh (base)
    /// - object 2: mesh (operand)
    /// - object 3: boolean shape referencing 1 as base and 2 as operand
    fn make_valid_boolean_model() -> Model {
        let mut model = Model::new();
        model
            .required_extensions
            .push(crate::model::Extension::BooleanOperations);

        let mut base_obj = Object::new(1);
        base_obj.mesh = Some(make_simple_mesh());
        model.resources.objects.push(base_obj);

        let mut operand_obj = Object::new(2);
        operand_obj.mesh = Some(make_simple_mesh());
        model.resources.objects.push(operand_obj);

        let mut bool_obj = Object::new(3);
        let mut shape = BooleanShape::new(1, BooleanOpType::Union);
        shape.operands.push(BooleanRef::new(2));
        bool_obj.boolean_shape = Some(shape);
        model.resources.objects.push(bool_obj);

        model
    }

    #[test]
    fn test_empty_model_passes() {
        let model = Model::new();
        assert!(validate_boolean_operations(&model).is_ok());
    }

    #[test]
    fn test_object_without_boolean_shape_passes() {
        let mut model = Model::new();
        let mut obj = Object::new(1);
        obj.mesh = Some(make_simple_mesh());
        model.resources.objects.push(obj);
        assert!(validate_boolean_operations(&model).is_ok());
    }

    #[test]
    fn test_valid_boolean_operation_passes() {
        let model = make_valid_boolean_model();
        assert!(validate_boolean_operations(&model).is_ok());
    }

    #[test]
    fn test_boolean_shape_on_non_model_type_fails() {
        let mut model = make_valid_boolean_model();
        // Change the boolean object type to Support
        model.resources.objects[2].object_type = ObjectType::Support;
        let result = validate_boolean_operations(&model);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("type \"model\""));
    }

    #[test]
    fn test_boolean_shape_with_components_fails() {
        let mut model = make_valid_boolean_model();
        // Add a component to the boolean object
        model.resources.objects[2]
            .components
            .push(Component::new(1));
        let result = validate_boolean_operations(&model);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("<booleanshape> and <components>")
        );
    }

    #[test]
    fn test_boolean_shape_with_mesh_fails() {
        let mut model = make_valid_boolean_model();
        // Add a mesh to the boolean object
        model.resources.objects[2].mesh = Some(make_simple_mesh());
        let result = validate_boolean_operations(&model);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("<booleanshape> and <mesh>")
        );
    }

    #[test]
    fn test_boolean_shape_no_operands_fails() {
        let mut model = Model::new();
        model
            .required_extensions
            .push(crate::model::Extension::BooleanOperations);

        let mut base_obj = Object::new(1);
        base_obj.mesh = Some(make_simple_mesh());
        model.resources.objects.push(base_obj);

        let mut bool_obj = Object::new(2);
        // Shape with no operands
        bool_obj.boolean_shape = Some(BooleanShape::new(1, BooleanOpType::Union));
        model.resources.objects.push(bool_obj);

        let result = validate_boolean_operations(&model);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no operands"));
    }

    #[test]
    fn test_base_object_nonexistent_fails() {
        let mut model = Model::new();
        model
            .required_extensions
            .push(crate::model::Extension::BooleanOperations);

        let mut operand_obj = Object::new(2);
        operand_obj.mesh = Some(make_simple_mesh());
        model.resources.objects.push(operand_obj);

        let mut bool_obj = Object::new(3);
        let mut shape = BooleanShape::new(99, BooleanOpType::Union); // 99 doesn't exist
        shape.operands.push(BooleanRef::new(2));
        bool_obj.boolean_shape = Some(shape);
        model.resources.objects.push(bool_obj);

        let result = validate_boolean_operations(&model);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("non-existent object ID 99")
        );
    }

    #[test]
    fn test_base_forward_reference_fails() {
        let mut model = Model::new();
        model
            .required_extensions
            .push(crate::model::Extension::BooleanOperations);

        let mut bool_obj = Object::new(1); // defined first
        let mut operand_obj = Object::new(2);
        operand_obj.mesh = Some(make_simple_mesh());

        let mut base_obj = Object::new(3); // defined after the boolean object (forward reference)
        base_obj.mesh = Some(make_simple_mesh());

        let mut shape = BooleanShape::new(3, BooleanOpType::Union); // references obj 3 which comes later
        shape.operands.push(BooleanRef::new(2));
        bool_obj.boolean_shape = Some(shape);

        model.resources.objects.push(bool_obj); // index 0
        model.resources.objects.push(operand_obj); // index 1
        model.resources.objects.push(base_obj); // index 2

        let result = validate_boolean_operations(&model);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("defined after it"));
    }

    #[test]
    fn test_base_object_not_model_type_fails() {
        let mut model = make_valid_boolean_model();
        // Change the base object type to Support
        model.resources.objects[0].object_type = ObjectType::Support;
        let result = validate_boolean_operations(&model);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("base object"));
        assert!(err.contains("type \"model\""));
    }

    #[test]
    fn test_base_object_with_only_components_fails() {
        let mut model = Model::new();
        model
            .required_extensions
            .push(crate::model::Extension::BooleanOperations);

        // base object with only components (no mesh/boolean_shape)
        let mut base_obj = Object::new(1);
        base_obj.components.push(Component::new(2));
        model.resources.objects.push(base_obj);

        let mut leaf_obj = Object::new(2);
        leaf_obj.mesh = Some(make_simple_mesh());
        model.resources.objects.push(leaf_obj);

        let mut operand_obj = Object::new(3);
        operand_obj.mesh = Some(make_simple_mesh());
        model.resources.objects.push(operand_obj);

        let mut bool_obj = Object::new(4);
        let mut shape = BooleanShape::new(1, BooleanOpType::Union);
        shape.operands.push(BooleanRef::new(3));
        bool_obj.boolean_shape = Some(shape);
        model.resources.objects.push(bool_obj);

        // Base object has components (not empty), no mesh, no boolean_shape → no shape
        // has_shape = mesh.is_some() (false) || boolean_shape.is_some() (false) || has_extension_shapes (false)
        // || (components.is_empty()) (false, because components has 1 entry) = false
        let result = validate_boolean_operations(&model);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("does not define a shape")
        );
    }

    #[test]
    fn test_operand_nonexistent_fails() {
        let mut model = Model::new();
        model
            .required_extensions
            .push(crate::model::Extension::BooleanOperations);

        let mut base_obj = Object::new(1);
        base_obj.mesh = Some(make_simple_mesh());
        model.resources.objects.push(base_obj);

        let mut bool_obj = Object::new(2);
        let mut shape = BooleanShape::new(1, BooleanOpType::Union);
        shape.operands.push(BooleanRef::new(99)); // 99 doesn't exist
        bool_obj.boolean_shape = Some(shape);
        model.resources.objects.push(bool_obj);

        let result = validate_boolean_operations(&model);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Boolean operand references non-existent")
        );
    }

    #[test]
    fn test_operand_forward_reference_fails() {
        let mut model = Model::new();
        model
            .required_extensions
            .push(crate::model::Extension::BooleanOperations);

        let mut base_obj = Object::new(1);
        base_obj.mesh = Some(make_simple_mesh());
        model.resources.objects.push(base_obj); // index 0

        let mut bool_obj = Object::new(2); // index 1
        let mut shape = BooleanShape::new(1, BooleanOpType::Union);
        shape.operands.push(BooleanRef::new(3)); // references obj 3, which comes later
        bool_obj.boolean_shape = Some(shape);
        model.resources.objects.push(bool_obj);

        let mut operand_obj = Object::new(3); // index 2 - defined after bool_obj
        operand_obj.mesh = Some(make_simple_mesh());
        model.resources.objects.push(operand_obj);

        let result = validate_boolean_operations(&model);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("defined after it"));
    }

    #[test]
    fn test_operand_not_model_type_fails() {
        let mut model = make_valid_boolean_model();
        // Change the operand object type to Support
        model.resources.objects[1].object_type = ObjectType::Support;
        let result = validate_boolean_operations(&model);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Boolean operand object"));
        assert!(err.contains("type \"model\""));
    }

    #[test]
    fn test_operand_no_mesh_fails() {
        let mut model = make_valid_boolean_model();
        // Remove the mesh from the operand object
        model.resources.objects[1].mesh = None;
        let result = validate_boolean_operations(&model);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("must be a triangle mesh")
        );
    }

    #[test]
    fn test_operand_has_boolean_shape_fails() {
        // When an operand object has a boolean_shape, the outer loop catches it first
        // with "Contains both <booleanshape> and <mesh>" error (since it also has a mesh).
        // This tests that operand objects with boolean shapes are rejected.
        let mut model = make_valid_boolean_model();
        // Add a boolean shape to the operand object (which also keeps its mesh)
        model.resources.objects[1].boolean_shape = Some(BooleanShape::new(1, BooleanOpType::Union));
        let result = validate_boolean_operations(&model);
        assert!(result.is_err());
        // The outer loop catches the object with both booleanshape and mesh first
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("<booleanshape> and <mesh>")
        );
    }

    #[test]
    fn test_operand_has_components_fails() {
        let mut model = make_valid_boolean_model();
        // Add components to the operand object
        model.resources.objects[1]
            .components
            .push(Component::new(1));
        let result = validate_boolean_operations(&model);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("has components"));
    }

    #[test]
    fn test_operand_has_extension_shapes_fails() {
        let mut model = make_valid_boolean_model();
        // Mark the operand object as having extension shapes
        model.resources.objects[1].has_extension_shapes = true;
        let result = validate_boolean_operations(&model);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("extension shape elements")
        );
    }

    #[test]
    fn test_external_base_path_skips_validation() {
        // When booleanshape has a path, base object validation is skipped
        let mut model = Model::new();
        model
            .required_extensions
            .push(crate::model::Extension::BooleanOperations);

        let mut operand_obj = Object::new(1);
        operand_obj.mesh = Some(make_simple_mesh());
        model.resources.objects.push(operand_obj);

        let mut bool_obj = Object::new(2);
        let mut shape = BooleanShape::new(99, BooleanOpType::Union); // nonexistent base id
        shape.path = Some("/path/to/external.model".to_string()); // but has external path
        shape.operands.push(BooleanRef::new(1));
        bool_obj.boolean_shape = Some(shape);
        model.resources.objects.push(bool_obj);

        // Should pass because base has an external path
        assert!(validate_boolean_operations(&model).is_ok());
    }

    #[test]
    fn test_external_operand_path_skips_validation() {
        // When a BooleanRef has a path, operand validation is skipped
        let mut model = Model::new();
        model
            .required_extensions
            .push(crate::model::Extension::BooleanOperations);

        let mut base_obj = Object::new(1);
        base_obj.mesh = Some(make_simple_mesh());
        model.resources.objects.push(base_obj);

        let mut bool_obj = Object::new(2);
        let mut shape = BooleanShape::new(1, BooleanOpType::Union);
        let mut ext_ref = BooleanRef::new(99); // nonexistent operand id
        ext_ref.path = Some("/path/to/external.model".to_string()); // but has external path
        shape.operands.push(ext_ref);
        bool_obj.boolean_shape = Some(shape);
        model.resources.objects.push(bool_obj);

        // Should pass because operand has an external path
        assert!(validate_boolean_operations(&model).is_ok());
    }
}
