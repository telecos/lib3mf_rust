//! 3MF extension tests
//!
//! Tests for all 3MF extensions including Material, Production, Slice,
//! Beam Lattice, Boolean Operations, Displacement, and Secure Content

mod extensions {
    pub mod material {
        pub mod data_structures;
        pub mod integration;
    }

    pub mod production {
        pub mod coordinates;
    }

    pub mod slice {
        pub mod integration;
        pub mod mesh_operations;
    }

    pub mod beam_lattice {
        pub mod integration;
    }

    pub mod boolean_ops {
        pub mod integration;
    }

    pub mod displacement {
        pub mod integration;
        pub mod namespaces;
    }

    pub mod secure_content {
        pub mod handler;
        pub mod integration;
        pub mod key_provider;
    }
}
