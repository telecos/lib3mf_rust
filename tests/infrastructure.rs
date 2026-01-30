//! Infrastructure tests
//!
//! Tests for supporting infrastructure including thumbnails, custom extensions,
//! extension registry, mesh operations, and texture paths

mod infrastructure {
    pub mod custom_extensions;
    pub mod extension_registry;
    pub mod extension_support;
    pub mod mesh_operations;
    pub mod post_parse_hooks;
    pub mod texture_paths;
    pub mod thumbnails;
    pub mod writer_registry;
}
