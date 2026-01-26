//! Extension handler implementations
//!
//! This module contains concrete implementations of the `ExtensionHandler` trait
//! for various 3MF extensions. It also provides factory functions to create
//! pre-configured extension registries with all standard handlers.
//!
//! # Standard Extension Handlers
//!
//! The following extension handlers are provided:
//! - [`MaterialExtensionHandler`] - Material & Properties extension
//! - [`ProductionExtensionHandler`] - Production extension
//! - [`BeamLatticeExtensionHandler`] - Beam Lattice extension
//! - [`SliceExtensionHandler`] - Slice extension
//! - [`BooleanOperationsExtensionHandler`] - Boolean Operations extension
//! - [`DisplacementExtensionHandler`] - Displacement extension
//! - [`SecureContentExtensionHandler`] - Secure Content extension
//!
//! # Creating a Default Registry
//!
//! The easiest way to use all standard extension handlers is through the
//! [`create_default_registry`] function:
//!
//! ```
//! use lib3mf::extensions::create_default_registry;
//!
//! // Create a registry with all standard handlers
//! let registry = create_default_registry();
//!
//! // Use the registry for validation
//! # let model = lib3mf::Model::new();
//! registry.validate_all(&model).unwrap();
//! ```
//!
//! # Manual Registration
//!
//! You can also create your own registry and register handlers individually:
//!
//! ```
//! use lib3mf::extension::ExtensionRegistry;
//! use lib3mf::extensions::{MaterialExtensionHandler, ProductionExtensionHandler};
//!
//! let mut registry = ExtensionRegistry::new();
//! registry.register(Box::new(MaterialExtensionHandler));
//! registry.register(Box::new(ProductionExtensionHandler));
//! ```
//!
//! # Using the Helper Function
//!
//! To add all standard handlers to an existing registry, use [`register_all_handlers`]:
//!
//! ```
//! use lib3mf::extension::ExtensionRegistry;
//! use lib3mf::extensions::register_all_handlers;
//!
//! let mut registry = ExtensionRegistry::new();
//! register_all_handlers(&mut registry);
//! ```

pub mod material;
pub mod production;
pub mod beam_lattice;
pub mod slice;
pub mod boolean_ops;
pub mod displacement;
pub mod secure_content;

pub use material::MaterialExtensionHandler;
pub use production::ProductionExtensionHandler;
pub use beam_lattice::BeamLatticeExtensionHandler;
pub use slice::SliceExtensionHandler;
pub use boolean_ops::BooleanOperationsExtensionHandler;
pub use displacement::DisplacementExtensionHandler;
pub use secure_content::SecureContentExtensionHandler;

use crate::extension::ExtensionRegistry;

/// Create a default extension registry with all standard extension handlers
///
/// This function creates and returns an [`ExtensionRegistry`] pre-populated with
/// all standard 3MF extension handlers. This is the recommended way to get started
/// with extension validation.
///
/// # Returns
///
/// An [`ExtensionRegistry`] containing all standard extension handlers:
/// - Material & Properties
/// - Production
/// - Beam Lattice
/// - Slice
/// - Boolean Operations
/// - Displacement
/// - Secure Content
///
/// # Example
///
/// ```
/// use lib3mf::extensions::create_default_registry;
/// use lib3mf::Model;
///
/// # let model = Model::new();
/// let registry = create_default_registry();
/// registry.validate_all(&model).unwrap();
/// ```
pub fn create_default_registry() -> ExtensionRegistry {
    let mut registry = ExtensionRegistry::new();
    register_all_handlers(&mut registry);
    registry
}

/// Register all standard extension handlers to an existing registry
///
/// This function adds all standard 3MF extension handlers to the provided
/// registry. This is useful if you already have a registry and want to add
/// all standard handlers to it.
///
/// # Arguments
///
/// * `registry` - The registry to which handlers will be added
///
/// # Example
///
/// ```
/// use lib3mf::extension::ExtensionRegistry;
/// use lib3mf::extensions::register_all_handlers;
///
/// let mut registry = ExtensionRegistry::new();
/// register_all_handlers(&mut registry);
/// ```
pub fn register_all_handlers(registry: &mut ExtensionRegistry) {
    registry.register(Box::new(MaterialExtensionHandler));
    registry.register(Box::new(ProductionExtensionHandler));
    registry.register(Box::new(BeamLatticeExtensionHandler));
    registry.register(Box::new(SliceExtensionHandler));
    registry.register(Box::new(BooleanOperationsExtensionHandler));
    registry.register(Box::new(DisplacementExtensionHandler));
    registry.register(Box::new(SecureContentExtensionHandler));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Extension;

    #[test]
    fn test_create_default_registry() {
        let registry = create_default_registry();
        // Verify all handlers are registered
        assert!(registry.get_handler(Extension::Material).is_some());
        assert!(registry.get_handler(Extension::Production).is_some());
        assert!(registry.get_handler(Extension::BeamLattice).is_some());
        assert!(registry.get_handler(Extension::Slice).is_some());
        assert!(registry.get_handler(Extension::BooleanOperations).is_some());
        assert!(registry.get_handler(Extension::Displacement).is_some());
        assert!(registry.get_handler(Extension::SecureContent).is_some());
        assert_eq!(registry.handlers().len(), 7);
    }

    #[test]
    fn test_register_all_handlers() {
        let mut registry = ExtensionRegistry::new();
        assert_eq!(registry.handlers().len(), 0);

        register_all_handlers(&mut registry);
        assert_eq!(registry.handlers().len(), 7);

        // Verify all handlers are registered
        assert!(registry.get_handler(Extension::Material).is_some());
        assert!(registry.get_handler(Extension::Production).is_some());
        assert!(registry.get_handler(Extension::BeamLattice).is_some());
        assert!(registry.get_handler(Extension::Slice).is_some());
        assert!(registry.get_handler(Extension::BooleanOperations).is_some());
        assert!(registry.get_handler(Extension::Displacement).is_some());
        assert!(registry.get_handler(Extension::SecureContent).is_some());
    }

    #[test]
    fn test_validate_all_with_default_registry() {
        let registry = create_default_registry();
        let model = crate::Model::new();
        // Should pass validation on empty model
        assert!(registry.validate_all(&model).is_ok());
    }
}
