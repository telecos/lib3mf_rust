//! Data structures representing 3MF models

// Declare all submodules
mod beam_lattice;
mod boolean_ops;
mod core;
mod displacement;
mod material;
mod production;
mod secure_content;
mod slice;

// Re-export all public types from core module
pub use core::{
    Build, BuildItem, Component, CustomElementHandler, CustomElementResult, CustomExtensionContext,
    CustomExtensionInfo, CustomValidationHandler, DisplacementMesh, DisplacementTriangle,
    Extension, Mesh, MetadataEntry, Model, Object, ObjectType, ParserConfig, Resources, Thumbnail,
    Triangle, Vertex,
};

// Re-export all public types from material module
pub use material::{
    BaseMaterial, BaseMaterialGroup, BlendMethod, ColorGroup, Composite, CompositeMaterials,
    Material, Multi, MultiProperties, Tex2Coord, Texture2D, Texture2DGroup,
};

// Re-export all public types from production module
pub use production::ProductionInfo;

// Re-export all public types from slice module
pub use slice::{Slice, SlicePolygon, SliceRef, SliceSegment, SliceStack, Vertex2D};

// Re-export all public types from beam_lattice module
pub use beam_lattice::{Ball, Beam, BeamCapMode, BeamSet};

// Re-export all public types from secure_content module
pub use secure_content::{
    AccessRight, CEKParams, Consumer, KEKParams, ResourceData, ResourceDataGroup, SecureContentInfo,
};

// Re-export all public types from boolean_ops module
pub use boolean_ops::{BooleanOpType, BooleanRef, BooleanShape};

// Re-export all public types from displacement module
pub use displacement::{
    Channel, Disp2DCoords, Disp2DGroup, Displacement2D, FilterMode, NormVector, NormVectorGroup,
    TileStyle,
};
