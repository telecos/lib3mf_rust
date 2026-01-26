//! Beam Lattice extension handler implementation

use crate::error::Result;
use crate::extension::ExtensionHandler;
use crate::model::{Extension, Model};

/// Extension handler for the Beam Lattice extension
///
/// This handler provides validation and processing for the Beam Lattice extension,
/// which enables representation of lattice structures using beams.
///
/// # Example
///
/// ```ignore
/// use lib3mf::extensions::BeamLatticeExtensionHandler;
/// use lib3mf::extension::{ExtensionHandler, ExtensionRegistry};
///
/// let handler = BeamLatticeExtensionHandler;
/// let mut registry = ExtensionRegistry::new();
/// registry.register(Box::new(handler));
/// ```
#[derive(Debug, Clone, Copy)]
pub struct BeamLatticeExtensionHandler;

impl ExtensionHandler for BeamLatticeExtensionHandler {
    fn extension_type(&self) -> Extension {
        Extension::BeamLattice
    }

    fn validate(&self, _model: &Model) -> Result<()> {
        // TODO: Implement beam lattice-specific validation
        // - Validate beam references
        // - Validate beam indices point to valid vertices
        // - Validate beam radii are positive
        Ok(())
    }

    fn is_used_in_model(&self, model: &Model) -> bool {
        // Check if any objects have beam sets (via mesh.beamset)
        model.resources.objects.iter().any(|obj| {
            obj.mesh
                .as_ref()
                .map(|mesh| mesh.beamset.is_some())
                .unwrap_or(false)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Beam, BeamSet, Object};

    #[test]
    fn test_extension_type() {
        let handler = BeamLatticeExtensionHandler;
        assert_eq!(handler.extension_type(), Extension::BeamLattice);
    }

    #[test]
    fn test_namespace() {
        let handler = BeamLatticeExtensionHandler;
        assert_eq!(
            handler.namespace(),
            "http://schemas.microsoft.com/3dmanufacturing/beamlattice/2017/02"
        );
    }

    #[test]
    fn test_name() {
        let handler = BeamLatticeExtensionHandler;
        assert_eq!(handler.name(), "BeamLattice");
    }

    #[test]
    fn test_is_used_in_model_empty() {
        let handler = BeamLatticeExtensionHandler;
        let model = Model::new();
        assert!(!handler.is_used_in_model(&model));
    }

    #[test]
    fn test_is_used_in_model_with_beams() {
        let handler = BeamLatticeExtensionHandler;
        let mut model = Model::new();

        let mut obj = Object::new(1);
        let mut mesh = crate::model::Mesh::new();
        let mut beam_set = BeamSet::new();
        beam_set.beams.push(Beam::new(0, 1));
        mesh.beamset = Some(beam_set);
        obj.mesh = Some(mesh);
        model.resources.objects.push(obj);

        assert!(handler.is_used_in_model(&model));
    }

    #[test]
    fn test_validate_empty_model() {
        let handler = BeamLatticeExtensionHandler;
        let model = Model::new();
        assert!(handler.validate(&model).is_ok());
    }
}
