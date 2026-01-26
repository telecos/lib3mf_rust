//! Concrete implementations of ExtensionHandler trait
//!
//! This module contains concrete implementations of the ExtensionHandler trait
//! for each supported 3MF extension.
//!
//! ## Usage
//!
//! The easiest way to use all extension handlers is with the `create_default_registry()` function:
//!
//! ```
//! use lib3mf::extensions::create_default_registry;
//! use lib3mf::Model;
//!
//! # fn example() -> lib3mf::Result<()> {
//! let model = Model::new();
//! let registry = create_default_registry();
//! registry.validate_all(&model)?;
//! # Ok(())
//! # }
//! ```
//!
//! Alternatively, you can manually register individual handlers:
//!
//! ```
//! use lib3mf::extension::ExtensionRegistry;
//! use lib3mf::extensions::{MaterialExtensionHandler, ProductionExtensionHandler};
//!
//! let mut registry = ExtensionRegistry::new();
//! registry.register(Box::new(MaterialExtensionHandler));
//! registry.register(Box::new(ProductionExtensionHandler));
//! ```

pub mod beam_lattice;
pub mod boolean_ops;
pub mod displacement;
pub mod material;
pub mod production;
pub mod secure_content;
pub mod slice;

pub use beam_lattice::BeamLatticeExtensionHandler;
pub use boolean_ops::BooleanOperationsExtensionHandler;
pub use displacement::DisplacementExtensionHandler;
pub use material::MaterialExtensionHandler;
pub use production::ProductionExtensionHandler;
pub use secure_content::SecureContentExtensionHandler;
pub use slice::SliceExtensionHandler;

use crate::extension::ExtensionRegistry;

/// Create a default extension registry with all standard handlers registered
///
/// This is a convenience function that creates an `ExtensionRegistry` pre-populated
/// with handlers for all standard 3MF extensions:
/// - Material
/// - Production
/// - BeamLattice
/// - Slice
/// - BooleanOperations
/// - Displacement
/// - SecureContent
///
/// # Example
///
/// ```
/// use lib3mf::extensions::create_default_registry;
/// use lib3mf::Model;
///
/// # fn example() -> lib3mf::Result<()> {
/// let model = Model::new();
/// let registry = create_default_registry();
///
/// // Validate all registered extensions
/// registry.validate_all(&model)?;
/// # Ok(())
/// # }
/// ```
///
/// # Returns
///
/// An `ExtensionRegistry` with all standard extension handlers registered
pub fn create_default_registry() -> ExtensionRegistry {
    let mut registry = ExtensionRegistry::new();
    register_all_handlers(&mut registry);
    registry
}

/// Register all standard extension handlers to an existing registry
///
/// This helper function registers all standard 3MF extension handlers to
/// an existing `ExtensionRegistry`. This is useful when you want to start
/// with an existing registry and add all standard handlers to it.
///
/// The following handlers are registered:
/// - Material
/// - Production
/// - BeamLattice
/// - Slice
/// - BooleanOperations
/// - Displacement
/// - SecureContent
///
/// # Arguments
///
/// * `registry` - The extension registry to register handlers to
///
/// # Example
///
/// ```
/// use lib3mf::extension::ExtensionRegistry;
/// use lib3mf::extensions::register_all_handlers;
///
/// let mut registry = ExtensionRegistry::new();
/// register_all_handlers(&mut registry);
///
/// assert_eq!(registry.handlers().len(), 7);
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
