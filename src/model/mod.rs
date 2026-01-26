//! Data structures representing 3MF models

// Declare all submodules
mod core;
mod material;
mod production;
mod slice;
mod beam_lattice;
mod secure_content;
mod boolean_ops;
mod displacement;

// Re-export all public types from core module
pub use core::{
    Extension,
    ParserConfig,
    CustomExtensionContext,
    CustomElementResult,
    CustomExtensionInfo,
    CustomElementHandler,
    CustomValidationHandler,
    Vertex,
    Triangle,
    Mesh,
    DisplacementTriangle,
    DisplacementMesh,
    Component,
    Object,
    ObjectType,
    Resources,
    BuildItem,
    Build,
    MetadataEntry,
    Thumbnail,
    Model,
};

// Re-export all public types from material module
pub use material::{
    Material,
    ColorGroup,
    BaseMaterialGroup,
    BaseMaterial,
    Texture2D,
    Tex2Coord,
    Texture2DGroup,
    Composite,
    CompositeMaterials,
    BlendMethod,
    Multi,
    MultiProperties,
};

// Re-export all public types from production module
pub use production::ProductionInfo;

// Re-export all public types from slice module
pub use slice::{
    Vertex2D,
    SliceSegment,
    SlicePolygon,
    Slice,
    SliceRef,
    SliceStack,
};

// Re-export all public types from beam_lattice module
pub use beam_lattice::{
    BeamCapMode,
    Beam,
    Ball,
    BeamSet,
};

// Re-export all public types from secure_content module
pub use secure_content::{
    SecureContentInfo,
    Consumer,
    ResourceDataGroup,
    AccessRight,
    KEKParams,
    ResourceData,
    CEKParams,
};

// Re-export all public types from boolean_ops module
pub use boolean_ops::{
    BooleanOpType,
    BooleanRef,
    BooleanShape,
};

// Re-export all public types from displacement module
pub use displacement::{
    TileStyle,
    FilterMode,
    Channel,
    Displacement2D,
    NormVector,
    NormVectorGroup,
    Disp2DCoords,
    Disp2DGroup,
};
