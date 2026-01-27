//! Extension trait system for pluggable 3MF extension architecture
//!
//! This module provides a trait-based system that allows extensions to be implemented
//! in a modular, pluggable way. Each extension can provide its own parsing, validation,
//! and writing logic through the `ExtensionHandler` trait.

use std::sync::Arc;

use crate::error::Result;
use crate::model::{Extension, Model};

/// Handler trait for 3MF extensions
///
/// This trait defines the interface that all extension handlers must implement.
/// It provides hooks for parsing, validation, and writing extension-specific data.
///
/// # Example
///
/// ```ignore
/// struct MyExtensionHandler;
///
/// impl ExtensionHandler for MyExtensionHandler {
///     fn extension_type(&self) -> Extension {
///         Extension::Material
///     }
///
///     fn validate(&self, model: &Model) -> Result<()> {
///         // Perform extension-specific validation
///         Ok(())
///     }
/// }
/// ```
pub trait ExtensionHandler: Send + Sync {
    /// Returns the extension type this handler supports
    fn extension_type(&self) -> Extension;

    /// Returns the namespace URI for this extension
    fn namespace(&self) -> &'static str {
        self.extension_type().namespace()
    }

    /// Returns a human-readable name for this extension
    fn name(&self) -> &'static str {
        self.extension_type().name()
    }

    /// Validate extension-specific data in the model
    ///
    /// This method is called during model validation to check extension-specific
    /// constraints and requirements.
    ///
    /// # Arguments
    ///
    /// * `model` - The 3MF model to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if validation passes
    /// * `Err(...)` if validation fails with a specific error
    fn validate(&self, model: &Model) -> Result<()>;

    /// Check if the extension is present in the model
    ///
    /// This method can be used to determine if the model actually uses this extension.
    /// The default implementation always returns `true`, but extensions can override
    /// this to provide more specific detection.
    ///
    /// # Arguments
    ///
    /// * `model` - The 3MF model to check
    ///
    /// # Returns
    ///
    /// * `true` if the extension is used in the model
    /// * `false` otherwise
    fn is_used_in_model(&self, _model: &Model) -> bool {
        true
    }

    /// Called before writing extension-specific data
    ///
    /// This hook allows extensions to prepare or transform data before serialization.
    /// The default implementation does nothing.
    ///
    /// # Arguments
    ///
    /// * `model` - Mutable reference to the model being written
    fn pre_write(&self, _model: &mut Model) -> Result<()> {
        Ok(())
    }

    /// Called after parsing extension-specific data
    ///
    /// This hook allows extensions to post-process or validate data after parsing.
    /// The default implementation does nothing.
    ///
    /// # Arguments
    ///
    /// * `model` - Mutable reference to the parsed model
    fn post_parse(&self, _model: &mut Model) -> Result<()> {
        Ok(())
    }
}

/// Registry for extension handlers
///
/// This struct manages a collection of extension handlers, allowing them to be
/// registered and retrieved by extension type.
///
/// # Example
///
/// ```ignore
/// use lib3mf::extension::ExtensionRegistry;
/// use std::sync::Arc;
///
/// let mut registry = ExtensionRegistry::new();
/// registry.register(Arc::new(MaterialExtensionHandler));
/// registry.register(Arc::new(ProductionExtensionHandler));
///
/// // Validate all registered extensions
/// registry.validate_all(&model)?;
/// ```
#[derive(Clone)]
pub struct ExtensionRegistry {
    handlers: Vec<Arc<dyn ExtensionHandler>>,
}

impl ExtensionRegistry {
    /// Create a new empty extension registry
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Register an extension handler
    ///
    /// # Arguments
    ///
    /// * `handler` - The extension handler to register
    pub fn register(&mut self, handler: Arc<dyn ExtensionHandler>) {
        self.handlers.push(handler);
    }

    /// Get a handler for a specific extension type
    ///
    /// # Arguments
    ///
    /// * `extension` - The extension type to find
    ///
    /// # Returns
    ///
    /// * `Some(&dyn ExtensionHandler)` if a handler is registered
    /// * `None` if no handler is found
    pub fn get_handler(&self, extension: Extension) -> Option<&dyn ExtensionHandler> {
        self.handlers
            .iter()
            .find(|h| h.extension_type() == extension)
            .map(|h| h.as_ref())
    }

    /// Validate all registered extensions
    ///
    /// This method calls the `validate` method on all registered handlers.
    /// Validators are called unconditionally to ensure proper validation even when
    /// extensions are declared but not used, or used but not declared.
    ///
    /// # Arguments
    ///
    /// * `model` - The model to validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all validations pass
    /// * `Err(...)` if any validation fails
    pub fn validate_all(&self, model: &Model) -> Result<()> {
        for handler in &self.handlers {
            handler.validate(model)?;
        }
        Ok(())
    }

    /// Call post_parse on all registered extensions
    ///
    /// # Arguments
    ///
    /// * `model` - The model that was just parsed
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all post-parse operations succeed
    /// * `Err(...)` if any operation fails
    pub fn post_parse_all(&self, model: &mut Model) -> Result<()> {
        for handler in &self.handlers {
            if handler.is_used_in_model(model) {
                handler.post_parse(model)?;
            }
        }
        Ok(())
    }

    /// Call pre_write on all registered extensions
    ///
    /// # Arguments
    ///
    /// * `model` - The model about to be written
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all pre-write operations succeed
    /// * `Err(...)` if any operation fails
    pub fn pre_write_all(&self, model: &mut Model) -> Result<()> {
        for handler in &self.handlers {
            if handler.is_used_in_model(model) {
                handler.pre_write(model)?;
            }
        }
        Ok(())
    }

    /// Get all registered handlers
    pub fn handlers(&self) -> &[Arc<dyn ExtensionHandler>] {
        &self.handlers
    }
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestExtensionHandler {
        ext_type: Extension,
        should_fail: bool,
    }

    impl ExtensionHandler for TestExtensionHandler {
        fn extension_type(&self) -> Extension {
            self.ext_type
        }

        fn validate(&self, _model: &Model) -> Result<()> {
            if self.should_fail {
                Err(crate::Error::InvalidModel(format!(
                    "{} validation failed",
                    self.name()
                )))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn test_extension_registry_basic() {
        let mut registry = ExtensionRegistry::new();
        assert!(registry.handlers().is_empty());

        let handler = Arc::new(TestExtensionHandler {
            ext_type: Extension::Material,
            should_fail: false,
        });
        registry.register(handler);

        assert_eq!(registry.handlers().len(), 1);
        assert!(registry.get_handler(Extension::Material).is_some());
        assert!(registry.get_handler(Extension::Production).is_none());
    }

    #[test]
    fn test_extension_handler_properties() {
        let handler = TestExtensionHandler {
            ext_type: Extension::Material,
            should_fail: false,
        };

        assert_eq!(handler.extension_type(), Extension::Material);
        assert_eq!(handler.name(), "Material");
        assert_eq!(
            handler.namespace(),
            "http://schemas.microsoft.com/3dmanufacturing/material/2015/02"
        );
    }

    #[test]
    fn test_validate_all_success() {
        let mut registry = ExtensionRegistry::new();
        registry.register(Arc::new(TestExtensionHandler {
            ext_type: Extension::Material,
            should_fail: false,
        }));
        registry.register(Arc::new(TestExtensionHandler {
            ext_type: Extension::Production,
            should_fail: false,
        }));

        let model = Model::new();
        assert!(registry.validate_all(&model).is_ok());
    }

    #[test]
    fn test_validate_all_failure() {
        let mut registry = ExtensionRegistry::new();
        registry.register(Arc::new(TestExtensionHandler {
            ext_type: Extension::Material,
            should_fail: false,
        }));
        registry.register(Arc::new(TestExtensionHandler {
            ext_type: Extension::Production,
            should_fail: true,
        }));

        let model = Model::new();
        assert!(registry.validate_all(&model).is_err());
    }
}
