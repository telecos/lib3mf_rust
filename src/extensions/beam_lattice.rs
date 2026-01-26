//! BeamLattice extension handler implementation

use crate::error::Result;
use crate::extension::ExtensionHandler;
use crate::model::{Extension, Model};
use crate::validator;

/// Extension handler for the BeamLattice extension
///
/// This handler provides validation and processing for 3MF files using the
/// Beam Lattice extension, which defines lattice structures made of beams
/// connecting mesh vertices.
///
/// # Example
///
/// ```
/// use lib3mf::extensions::BeamLatticeExtensionHandler;
/// use lib3mf::{ExtensionHandler, ExtensionRegistry, Model};
///
/// let handler = BeamLatticeExtensionHandler;
/// let mut registry = ExtensionRegistry::new();
/// registry.register(Box::new(handler));
///
/// let model = Model::new();
/// // Use registry.validate_all(&model) to validate
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct BeamLatticeExtensionHandler;

impl BeamLatticeExtensionHandler {
    /// Create a new BeamLattice extension handler
    pub fn new() -> Self {
        Self
    }
}

impl ExtensionHandler for BeamLatticeExtensionHandler {
    fn extension_type(&self) -> Extension {
        Extension::BeamLattice
    }

    fn validate(&self, model: &Model) -> Result<()> {
        // Delegate to the existing validator function
        validator::validate_beam_lattice(model)
    }

    fn is_used_in_model(&self, model: &Model) -> bool {
        // Check if any mesh objects have a beamset
        model
            .resources
            .objects
            .iter()
            .filter_map(|obj| obj.mesh.as_ref())
            .any(|mesh| mesh.beamset.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Beam, BeamSet, Mesh, Object};

    #[test]
    fn test_extension_type() {
        let handler = BeamLatticeExtensionHandler::new();
        assert_eq!(handler.extension_type(), Extension::BeamLattice);
    }

    #[test]
    fn test_namespace() {
        let handler = BeamLatticeExtensionHandler::new();
        assert_eq!(
            handler.namespace(),
            "http://schemas.microsoft.com/3dmanufacturing/beamlattice/2017/02"
        );
    }

    #[test]
    fn test_name() {
        let handler = BeamLatticeExtensionHandler::new();
        assert_eq!(handler.name(), "BeamLattice");
    }

    #[test]
    fn test_is_used_in_model_empty() {
        let handler = BeamLatticeExtensionHandler::new();
        let model = Model::new();
        assert!(!handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_no_beamset() {
        let handler = BeamLatticeExtensionHandler::new();
        let mut model = Model::new();

        // Add an object with a mesh but no beamset
        let mut mesh = Mesh::new();
        mesh.vertices.push(crate::model::Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(crate::model::Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(crate::model::Vertex::new(0.0, 1.0, 0.0));
        mesh.triangles
            .push(crate::model::Triangle::new(0, 1, 2));

        let mut object = Object::new(1);
        object.mesh = Some(mesh);

        model.resources.objects.push(object);
        assert!(!handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_with_beamset() {
        let handler = BeamLatticeExtensionHandler::new();
        let mut model = Model::new();

        // Add an object with a mesh that has a beamset
        let mut mesh = Mesh::new();
        mesh.vertices.push(crate::model::Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(crate::model::Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(crate::model::Vertex::new(0.0, 1.0, 0.0));
        mesh.triangles
            .push(crate::model::Triangle::new(0, 1, 2));

        let mut beamset = BeamSet::new();
        beamset.beams.push(Beam::new(0, 1));
        mesh.beamset = Some(beamset);

        let mut object = Object::new(1);
        object.mesh = Some(mesh);

        model.resources.objects.push(object);
        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_validate_empty_model() {
        let handler = BeamLatticeExtensionHandler::new();
        let model = Model::new();
        // Empty model should be valid (no beamsets to validate)
        assert!(handler.validate(&model).is_ok());
    }

    #[test]
    fn test_validate_valid_beamset() {
        let handler = BeamLatticeExtensionHandler::new();
        let mut model = Model::new();

        // Create a valid object with beamset
        let mut mesh = Mesh::new();
        mesh.vertices.push(crate::model::Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(crate::model::Vertex::new(1.0, 0.0, 0.0));
        mesh.vertices.push(crate::model::Vertex::new(0.0, 1.0, 0.0));
        mesh.triangles
            .push(crate::model::Triangle::new(0, 1, 2));

        let mut beamset = BeamSet::new();
        beamset.beams.push(Beam::new(0, 1)); // Valid beam between vertices 0 and 1
        mesh.beamset = Some(beamset);

        let mut object = Object::new(1);
        object.mesh = Some(mesh);

        model.resources.objects.push(object);
        assert!(handler.validate(&model).is_ok());
    }

    #[test]
    fn test_validate_invalid_beamset() {
        let handler = BeamLatticeExtensionHandler::new();
        let mut model = Model::new();

        // Create an invalid object with beamset (beam references non-existent vertex)
        let mut mesh = Mesh::new();
        mesh.vertices.push(crate::model::Vertex::new(0.0, 0.0, 0.0));
        mesh.vertices.push(crate::model::Vertex::new(1.0, 0.0, 0.0));
        mesh.triangles
            .push(crate::model::Triangle::new(0, 1, 0)); // Degenerate, but that's not what we're testing

        let mut beamset = BeamSet::new();
        beamset.beams.push(Beam::new(0, 5)); // Invalid: vertex 5 doesn't exist
        mesh.beamset = Some(beamset);

        let mut object = Object::new(1);
        object.mesh = Some(mesh);

        model.resources.objects.push(object);
        assert!(handler.validate(&model).is_err());
    }

    #[test]
    fn test_default_implementation() {
        let handler = BeamLatticeExtensionHandler::default();
        assert_eq!(handler.extension_type(), Extension::BeamLattice);
    }
}
