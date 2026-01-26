//! Extension handler implementations for 3MF extensions
//!
//! This module contains concrete implementations of the `ExtensionHandler` trait
//! for each supported 3MF extension. Each handler provides validation, parsing hooks,
//! and extension-specific logic.

pub mod slice;

pub use slice::SliceExtensionHandler;
