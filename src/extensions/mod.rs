//! Extension handlers for 3MF extensions
//!
//! This module contains concrete implementations of the `ExtensionHandler` trait
//! for each supported 3MF extension. These handlers provide validation and
//! processing logic specific to each extension.

mod boolean_ops;

pub use boolean_ops::BooleanOperationsExtensionHandler;
