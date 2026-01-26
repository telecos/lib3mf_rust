//! Extension handler implementations
//!
//! This module contains concrete implementations of the ExtensionHandler trait
//! for each supported 3MF extension.

mod beam_lattice;
pub mod material;

pub use beam_lattice::BeamLatticeExtensionHandler;
pub use material::MaterialExtensionHandler;
