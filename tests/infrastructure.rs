//! Infrastructure tests
//!
//! Tests for supporting infrastructure including thumbnails, custom extensions,
//! extension registry, mesh operations, and texture paths

mod infrastructure {
    pub mod thumbnails;
    pub mod custom_extensions;
    pub mod extension_registry;
    pub mod extension_support;
    pub mod post_parse_hooks;
    pub mod writer_registry;
    pub mod mesh_operations;
    pub mod texture_paths;
}
