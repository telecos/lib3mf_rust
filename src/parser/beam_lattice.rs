//! Beam lattice extension parsing
//!
//! This module handles parsing of 3MF Beam Lattice extension elements.

use crate::error::{Error, Result};
use crate::model::*;
use quick_xml::Reader;

use super::{parse_attributes, validate_attributes};

/// Parse a beam element
pub(super) fn parse_beam<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<Beam> {
    let attrs = parse_attributes(reader, e)?;

    // Validate only allowed attributes are present
    // Per Beam Lattice Extension spec v1.2.0: v1, v2, r1, r2, cap1, cap2, p1, p2, pid
    validate_attributes(
        &attrs,
        &["v1", "v2", "r1", "r2", "cap1", "cap2", "p1", "p2", "pid"],
        "beam",
    )?;

    let v1 = attrs
        .get("v1")
        .ok_or_else(|| Error::InvalidXml("Beam missing v1 attribute".to_string()))?
        .parse::<usize>()?;

    let v2 = attrs
        .get("v2")
        .ok_or_else(|| Error::InvalidXml("Beam missing v2 attribute".to_string()))?
        .parse::<usize>()?;

    let mut beam = Beam::new(v1, v2);

    if let Some(r1) = attrs.get("r1") {
        let r1_val = r1.parse::<f64>()?;
        // Validate radius is finite and positive
        if !r1_val.is_finite() || r1_val <= 0.0 {
            return Err(Error::InvalidXml(format!(
                "Beam r1 must be positive and finite (got {})",
                r1_val
            )));
        }
        beam.r1 = Some(r1_val);
    }

    if let Some(r2) = attrs.get("r2") {
        let r2_val = r2.parse::<f64>()?;
        // Validate radius is finite and positive
        if !r2_val.is_finite() || r2_val <= 0.0 {
            return Err(Error::InvalidXml(format!(
                "Beam r2 must be positive and finite (got {})",
                r2_val
            )));
        }
        beam.r2 = Some(r2_val);

        // If r2 is specified, r1 must also be specified (per 3MF Beam Lattice spec)
        if beam.r1.is_none() {
            return Err(Error::InvalidXml(
                "Beam attribute r2 is specified but r1 is not. When specifying r2, r1 must also be provided.".to_string()
            ));
        }
    }

    // Parse cap1 attribute (optional, defaults to beamset cap mode)
    if let Some(cap1_str) = attrs.get("cap1") {
        beam.cap1 = Some(cap1_str.parse()?);
    }

    // Parse cap2 attribute (optional, defaults to beamset cap mode)
    if let Some(cap2_str) = attrs.get("cap2") {
        beam.cap2 = Some(cap2_str.parse()?);
    }

    // Parse pid attribute (optional) - material/property group ID
    if let Some(pid_str) = attrs.get("pid") {
        beam.property_id = Some(pid_str.parse::<u32>()?);
    }

    // Parse p1 attribute (optional) - property index at v1
    if let Some(p1_str) = attrs.get("p1") {
        beam.p1 = Some(p1_str.parse::<u32>()?);
    }

    // Parse p2 attribute (optional) - property index at v2
    if let Some(p2_str) = attrs.get("p2") {
        beam.p2 = Some(p2_str.parse::<u32>()?);

        // If p2 is specified, p1 must also be specified (per 3MF spec convention)
        if beam.p1.is_none() {
            return Err(Error::InvalidXml(
                "Beam attribute p2 is specified but p1 is not. When specifying p2, p1 must also be provided.".to_string()
            ));
        }
    }

    Ok(beam)
}

/// Parse beamlattice start element and return initialized beamset
pub(super) fn parse_beamlattice_start<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<BeamSet> {
    let attrs = parse_attributes(reader, e)?;
    let mut beamset = BeamSet::new();

    // Parse radius attribute (default 1.0)
    if let Some(radius_str) = attrs.get("radius") {
        let radius = radius_str.parse::<f64>()?;
        // Validate radius is finite and positive
        if !radius.is_finite() || radius <= 0.0 {
            return Err(Error::InvalidXml(format!(
                "BeamLattice radius must be positive and finite (got {})",
                radius
            )));
        }
        beamset.radius = radius;
    }

    // Parse minlength attribute (default 0.0001)
    if let Some(minlength_str) = attrs.get("minlength") {
        let minlength = minlength_str.parse::<f64>()?;
        // Validate minlength is finite and non-negative
        if !minlength.is_finite() || minlength < 0.0 {
            return Err(Error::InvalidXml(format!(
                "BeamLattice minlength must be non-negative and finite (got {})",
                minlength
            )));
        }
        beamset.min_length = minlength;
    }

    // Parse cap mode attribute (default sphere)
    if let Some(cap_str) = attrs.get("cap") {
        beamset.cap_mode = cap_str.parse()?;
    }

    // Parse clippingmesh ID attribute (optional)
    if let Some(clip_id_str) = attrs.get("clippingmesh") {
        beamset.clipping_mesh_id = Some(clip_id_str.parse::<u32>()?);
    }

    // Parse representationmesh ID attribute (optional)
    if let Some(rep_id_str) = attrs.get("representationmesh") {
        beamset.representation_mesh_id = Some(rep_id_str.parse::<u32>()?);
    }

    // Parse clippingmode attribute (optional)
    if let Some(clip_mode) = attrs.get("clippingmode") {
        beamset.clipping_mode = Some(clip_mode.clone());
    }

    // Parse ballmode attribute (optional) - from balls extension
    // This can be in default namespace or balls namespace (b2:ballmode)
    if let Some(ball_mode) = attrs.get("ballmode").or_else(|| attrs.get("b2:ballmode")) {
        beamset.ball_mode = Some(ball_mode.clone());
    }

    // Parse ballradius attribute (optional) - from balls extension
    // This can be in default namespace or balls namespace (b2:ballradius)
    if let Some(ball_radius_str) = attrs
        .get("ballradius")
        .or_else(|| attrs.get("b2:ballradius"))
    {
        let ball_radius = ball_radius_str.parse::<f64>()?;
        // Validate ball radius is finite and positive
        if !ball_radius.is_finite() || ball_radius <= 0.0 {
            return Err(Error::InvalidXml(format!(
                "BeamLattice ballradius must be positive and finite (got {})",
                ball_radius
            )));
        }
        beamset.ball_radius = Some(ball_radius);
    }

    // Parse pid attribute (optional) - material/property group ID
    if let Some(pid_str) = attrs.get("pid") {
        beamset.property_id = Some(pid_str.parse::<u32>()?);
    }

    // Parse pindex attribute (optional) - property index
    if let Some(pindex_str) = attrs.get("pindex") {
        beamset.property_index = Some(pindex_str.parse::<u32>()?);
    }

    Ok(beamset)
}

/// Parse ball element
pub(super) fn parse_ball<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart,
) -> Result<Ball> {
    let attrs = parse_attributes(reader, e)?;

    // Parse required vindex attribute
    let vindex = attrs
        .get("vindex")
        .ok_or_else(|| {
            Error::InvalidXml("Ball element missing required vindex attribute".to_string())
        })?
        .parse::<usize>()?;

    let mut ball = Ball::new(vindex);

    // Parse optional radius
    if let Some(r_str) = attrs.get("r") {
        ball.radius = Some(r_str.parse::<f64>()?);
    }

    // Parse optional pid (property group ID)
    if let Some(pid_str) = attrs.get("pid") {
        ball.property_id = Some(pid_str.parse::<u32>()?);
    }

    // Parse optional p (property index)
    if let Some(p_str) = attrs.get("p") {
        ball.property_index = Some(p_str.parse::<u32>()?);
    }

    Ok(ball)
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_model_xml;

    fn mesh_with_beamlattice(beamlattice_attrs: &str, beams_content: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <resources>
    <object id="1">
      <mesh>
        <vertices>
          <vertex x="0" y="0" z="0"/>
          <vertex x="10" y="0" z="0"/>
          <vertex x="0" y="10" z="0"/>
        </vertices>
        <triangles>
          <triangle v1="0" v2="1" v3="2"/>
        </triangles>
        <beamlattice {attrs}>
          <beams>
            {beams}
          </beams>
        </beamlattice>
      </mesh>
    </object>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"#,
            attrs = beamlattice_attrs,
            beams = beams_content,
        )
    }

    #[test]
    fn test_parse_beamlattice_defaults() {
        let xml = mesh_with_beamlattice("radius=\"1.0\" minlength=\"0.01\"", "");
        let model = parse_model_xml(&xml).unwrap();
        let mesh = model.resources.objects[0].mesh.as_ref().unwrap();
        let beamset = mesh.beamset.as_ref().unwrap();
        assert_eq!(beamset.radius, 1.0);
        assert_eq!(beamset.min_length, 0.01);
    }

    #[test]
    fn test_parse_beamlattice_with_cap_mode() {
        let xml = mesh_with_beamlattice("radius=\"1.0\" minlength=\"0.01\" cap=\"hemisphere\"", "");
        assert!(parse_model_xml(&xml).is_ok());
    }

    #[test]
    fn test_parse_beamlattice_with_clipping_mesh() {
        let xml = mesh_with_beamlattice(
            "radius=\"1.0\" minlength=\"0.01\" clippingmesh=\"99\" clippingmode=\"inside\" representationmesh=\"100\"",
            "",
        );
        let model = parse_model_xml(&xml).unwrap();
        let mesh = model.resources.objects[0].mesh.as_ref().unwrap();
        let beamset = mesh.beamset.as_ref().unwrap();
        assert_eq!(beamset.clipping_mesh_id, Some(99));
        assert_eq!(beamset.representation_mesh_id, Some(100));
        assert!(beamset.clipping_mode.is_some());
    }

    #[test]
    fn test_parse_beamlattice_with_property_pid() {
        let xml = mesh_with_beamlattice(
            "radius=\"1.0\" minlength=\"0.01\" pid=\"5\" pindex=\"2\"",
            "",
        );
        let model = parse_model_xml(&xml).unwrap();
        let mesh = model.resources.objects[0].mesh.as_ref().unwrap();
        let beamset = mesh.beamset.as_ref().unwrap();
        assert_eq!(beamset.property_id, Some(5));
        assert_eq!(beamset.property_index, Some(2));
    }

    #[test]
    fn test_parse_beam_with_p1_p2() {
        let xml = mesh_with_beamlattice(
            "radius=\"1.0\" minlength=\"0.01\"",
            r#"<beam v1="0" v2="1" p1="0" p2="1" pid="5"/>"#,
        );
        let model = parse_model_xml(&xml).unwrap();
        let mesh = model.resources.objects[0].mesh.as_ref().unwrap();
        let beamset = mesh.beamset.as_ref().unwrap();
        assert_eq!(beamset.beams[0].p1, Some(0));
        assert_eq!(beamset.beams[0].p2, Some(1));
    }

    #[test]
    fn test_beam_negative_r2_rejected() {
        let xml = mesh_with_beamlattice(
            "radius=\"1.0\" minlength=\"0.01\"",
            r#"<beam v1="0" v2="1" r1="1.0" r2="-0.5"/>"#,
        );
        let result = parse_model_xml(&xml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("r2"));
    }

    #[test]
    fn test_beam_missing_v1_rejected() {
        let xml = mesh_with_beamlattice("radius=\"1.0\" minlength=\"0.01\"", r#"<beam v2="1"/>"#);
        let result = parse_model_xml(&xml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("v1"));
    }

    #[test]
    fn test_beam_missing_v2_rejected() {
        let xml = mesh_with_beamlattice("radius=\"1.0\" minlength=\"0.01\"", r#"<beam v1="0"/>"#);
        let result = parse_model_xml(&xml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("v2"));
    }

    #[test]
    fn test_parse_ball_with_all_attributes() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <resources>
    <object id="1">
      <mesh>
        <vertices>
          <vertex x="0" y="0" z="0"/>
          <vertex x="10" y="0" z="0"/>
          <vertex x="0" y="10" z="0"/>
        </vertices>
        <triangles>
          <triangle v1="0" v2="1" v3="2"/>
        </triangles>
        <beamlattice radius="1.0" minlength="0.01">
          <beams>
            <beam v1="0" v2="1"/>
          </beams>
          <balls>
            <ball vindex="0" r="2.5" pid="3" p="1"/>
          </balls>
        </beamlattice>
      </mesh>
    </object>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"#;
        let model = parse_model_xml(xml).unwrap();
        let mesh = model.resources.objects[0].mesh.as_ref().unwrap();
        let beamset = mesh.beamset.as_ref().unwrap();
        let ball = &beamset.balls[0];
        assert_eq!(ball.vindex, 0);
        assert_eq!(ball.radius, Some(2.5));
        assert_eq!(ball.property_id, Some(3));
        assert_eq!(ball.property_index, Some(1));
    }

    #[test]
    fn test_parse_ball_missing_vindex_rejected() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <resources>
    <object id="1">
      <mesh>
        <vertices>
          <vertex x="0" y="0" z="0"/>
          <vertex x="10" y="0" z="0"/>
          <vertex x="0" y="10" z="0"/>
        </vertices>
        <triangles>
          <triangle v1="0" v2="1" v3="2"/>
        </triangles>
        <beamlattice radius="1.0" minlength="0.01">
          <beams/>
          <balls>
            <ball r="1.0"/>
          </balls>
        </beamlattice>
      </mesh>
    </object>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"#;
        let result = parse_model_xml(xml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("vindex"));
    }

    #[test]
    fn test_beamlattice_ballradius_invalid_rejected() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
  <resources>
    <object id="1">
      <mesh>
        <vertices>
          <vertex x="0" y="0" z="0"/>
          <vertex x="10" y="0" z="0"/>
          <vertex x="0" y="10" z="0"/>
        </vertices>
        <triangles>
          <triangle v1="0" v2="1" v3="2"/>
        </triangles>
        <beamlattice radius="1.0" minlength="0.01" ballradius="-1.0">
          <beams/>
        </beamlattice>
      </mesh>
    </object>
  </resources>
  <build>
    <item objectid="1"/>
  </build>
</model>"#;
        let result = parse_model_xml(xml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("ballradius"));
    }
}
