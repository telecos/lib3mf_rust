//! Beam Lattice extension types

/// Cap mode for beam lattice ends
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BeamCapMode {
    /// Sphere cap (rounded ends)
    #[default]
    Sphere,
    /// Butt cap (flat ends)
    Butt,
    /// Hemisphere cap (half sphere at end)
    Hemisphere,
}

impl std::fmt::Display for BeamCapMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BeamCapMode::Sphere => write!(f, "sphere"),
            BeamCapMode::Butt => write!(f, "butt"),
            BeamCapMode::Hemisphere => write!(f, "hemisphere"),
        }
    }
}

impl std::str::FromStr for BeamCapMode {
    type Err = crate::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sphere" => Ok(BeamCapMode::Sphere),
            "butt" => Ok(BeamCapMode::Butt),
            "hemisphere" => Ok(BeamCapMode::Hemisphere),
            _ => Err(crate::error::Error::InvalidXml(format!(
                "Invalid cap mode '{}'. Must be 'sphere', 'butt', or 'hemisphere'",
                s
            ))),
        }
    }
}

/// A single beam in a beam lattice structure
///
/// Beams connect two vertices with optional varying radii along the beam.
/// Part of the Beam Lattice Extension specification.
#[derive(Debug, Clone, PartialEq)]
pub struct Beam {
    /// Index of first vertex
    pub v1: usize,
    /// Index of second vertex
    pub v2: usize,
    /// Radius at first vertex (optional, defaults to beamset radius)
    pub r1: Option<f64>,
    /// Radius at second vertex (optional, defaults to r1 or beamset radius)
    pub r2: Option<f64>,
    /// Cap mode at first vertex (optional, defaults to beamset cap mode)
    pub cap1: Option<BeamCapMode>,
    /// Cap mode at second vertex (optional, defaults to beamset cap mode)
    pub cap2: Option<BeamCapMode>,
    /// Material/property group ID
    pub property_id: Option<u32>,
    /// Property index at first vertex
    pub p1: Option<u32>,
    /// Property index at second vertex
    pub p2: Option<u32>,
}

impl Beam {
    /// Create a new beam between two vertices
    pub fn new(v1: usize, v2: usize) -> Self {
        Self {
            v1,
            v2,
            r1: None,
            r2: None,
            cap1: None,
            cap2: None,
            property_id: None,
            p1: None,
            p2: None,
        }
    }

    /// Create a new beam with a specific radius at v1
    pub fn with_radius(v1: usize, v2: usize, r1: f64) -> Self {
        Self {
            v1,
            v2,
            r1: Some(r1),
            r2: None,
            cap1: None,
            cap2: None,
            property_id: None,
            p1: None,
            p2: None,
        }
    }

    /// Create a new beam with different radii at both ends
    pub fn with_radii(v1: usize, v2: usize, r1: f64, r2: f64) -> Self {
        Self {
            v1,
            v2,
            r1: Some(r1),
            r2: Some(r2),
            cap1: None,
            cap2: None,
            property_id: None,
            p1: None,
            p2: None,
        }
    }
}

/// A ball element from the Beam Lattice Balls sub-extension
///
/// Balls are spheres placed at beam vertices
#[derive(Debug, Clone)]
pub struct Ball {
    /// Vertex index this ball is centered at
    pub vindex: usize,
    /// Optional radius for this ball
    pub radius: Option<f64>,
    /// Optional property index for this ball
    pub property_index: Option<u32>,
    /// Optional property group ID for this ball
    pub property_id: Option<u32>,
}

impl Ball {
    /// Create a new ball at the given vertex index
    pub fn new(vindex: usize) -> Self {
        Self {
            vindex,
            radius: None,
            property_index: None,
            property_id: None,
        }
    }
}

/// A beam lattice structure containing beams and lattice properties
///
/// Part of the Beam Lattice Extension specification.
/// Defines a lattice structure made of beams connecting vertices.
#[derive(Debug, Clone)]
pub struct BeamSet {
    /// Default radius for beams (when not specified per-beam)
    pub radius: f64,
    /// Minimum length for beams
    pub min_length: f64,
    /// Cap mode for beam ends
    pub cap_mode: BeamCapMode,
    /// List of beams in the lattice
    pub beams: Vec<Beam>,
    /// Optional clipping mesh ID for beam lattice clipping
    pub clipping_mesh_id: Option<u32>,
    /// Optional representation mesh ID for alternative representation
    pub representation_mesh_id: Option<u32>,
    /// Clipping mode (none, inside, outside)
    pub clipping_mode: Option<String>,
    /// Ball mode for beam endpoints (from balls extension)
    pub ball_mode: Option<String>,
    /// Ball radius for beam endpoints (from balls extension)
    pub ball_radius: Option<f64>,
    /// Material/property group ID for beam lattice
    pub property_id: Option<u32>,
    /// Property index within the property group
    pub property_index: Option<u32>,
    /// Beam set references (for metadata grouping) - indices into beams vec
    pub beam_set_refs: Vec<usize>,
    /// Balls (for validation) - from balls sub-extension
    pub balls: Vec<Ball>,
    /// Ball set references (for metadata grouping) - indices into balls vec
    pub ball_set_refs: Vec<usize>,
}

impl BeamSet {
    /// Create a new beam set with default values
    pub fn new() -> Self {
        Self {
            radius: 1.0,
            min_length: 0.0001,
            cap_mode: BeamCapMode::Sphere,
            beams: Vec::new(),
            clipping_mesh_id: None,
            representation_mesh_id: None,
            clipping_mode: None,
            ball_mode: None,
            ball_radius: None,
            property_id: None,
            property_index: None,
            beam_set_refs: Vec::new(),
            balls: Vec::new(),
            ball_set_refs: Vec::new(),
        }
    }

    /// Create a new beam set with a specific radius
    pub fn with_radius(radius: f64) -> Self {
        Self {
            radius,
            min_length: 0.0001,
            cap_mode: BeamCapMode::Sphere,
            beams: Vec::new(),
            clipping_mesh_id: None,
            representation_mesh_id: None,
            clipping_mode: None,
            ball_mode: None,
            ball_radius: None,
            property_id: None,
            property_index: None,
            beam_set_refs: Vec::new(),
            balls: Vec::new(),
            ball_set_refs: Vec::new(),
        }
    }
}

impl Default for BeamSet {
    fn default() -> Self {
        Self::new()
    }
}
