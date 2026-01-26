//! Extension handler implementations
//!
//! This module contains concrete implementations of the `ExtensionHandler` trait
//! for each 3MF extension supported by the library.

mod secure_content;

pub use secure_content::SecureContentExtensionHandler;
