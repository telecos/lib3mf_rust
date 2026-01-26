//! Tests for the extension registry factory functions
//!
//! This test validates the create_default_registry() and register_all_handlers()
//! functions in the extensions module.

use lib3mf::extension::ExtensionRegistry;
use lib3mf::extensions::{create_default_registry, register_all_handlers};
use lib3mf::{Extension, Model};

#[test]
fn test_create_default_registry() {
    // Create a default registry
    let registry = create_default_registry();

    // Verify all 7 standard handlers are registered
    assert_eq!(
        registry.handlers().len(),
        7,
        "Default registry should have 7 handlers"
    );

    // Verify each expected handler is present
    assert!(
        registry.get_handler(Extension::Material).is_some(),
        "Material handler should be registered"
    );
    assert!(
        registry.get_handler(Extension::Production).is_some(),
        "Production handler should be registered"
    );
    assert!(
        registry.get_handler(Extension::BeamLattice).is_some(),
        "BeamLattice handler should be registered"
    );
    assert!(
        registry.get_handler(Extension::Slice).is_some(),
        "Slice handler should be registered"
    );
    assert!(
        registry.get_handler(Extension::BooleanOperations).is_some(),
        "BooleanOperations handler should be registered"
    );
    assert!(
        registry.get_handler(Extension::Displacement).is_some(),
        "Displacement handler should be registered"
    );
    assert!(
        registry.get_handler(Extension::SecureContent).is_some(),
        "SecureContent handler should be registered"
    );
}

#[test]
fn test_register_all_handlers() {
    // Create an empty registry
    let mut registry = ExtensionRegistry::new();
    assert_eq!(registry.handlers().len(), 0, "Registry should start empty");

    // Register all handlers
    register_all_handlers(&mut registry);

    // Verify all 7 handlers are registered
    assert_eq!(
        registry.handlers().len(),
        7,
        "Registry should have 7 handlers after registration"
    );

    // Verify each handler is present
    assert!(registry.get_handler(Extension::Material).is_some());
    assert!(registry.get_handler(Extension::Production).is_some());
    assert!(registry.get_handler(Extension::BeamLattice).is_some());
    assert!(registry.get_handler(Extension::Slice).is_some());
    assert!(registry.get_handler(Extension::BooleanOperations).is_some());
    assert!(registry.get_handler(Extension::Displacement).is_some());
    assert!(registry.get_handler(Extension::SecureContent).is_some());
}

#[test]
fn test_registry_validate_all_works() {
    // Create a simple model
    let model = Model::new();

    // Create default registry
    let registry = create_default_registry();

    // validate_all should succeed on empty model
    let result = registry.validate_all(&model);
    assert!(
        result.is_ok(),
        "validate_all should succeed on empty model: {:?}",
        result.err()
    );
}

#[test]
fn test_registry_post_parse_all_works() {
    // Create a simple model
    let mut model = Model::new();

    // Create default registry
    let registry = create_default_registry();

    // post_parse_all should succeed on empty model
    let result = registry.post_parse_all(&mut model);
    assert!(
        result.is_ok(),
        "post_parse_all should succeed on empty model: {:?}",
        result.err()
    );
}

#[test]
fn test_registry_pre_write_all_works() {
    // Create a simple model
    let mut model = Model::new();

    // Create default registry
    let registry = create_default_registry();

    // pre_write_all should succeed on empty model
    let result = registry.pre_write_all(&mut model);
    assert!(
        result.is_ok(),
        "pre_write_all should succeed on empty model: {:?}",
        result.err()
    );
}
