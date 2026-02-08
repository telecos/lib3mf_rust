//! Volumetric extension writing for 3MF model files
//!
//! This module provides functionality to write Volumetric extension elements
//! such as `<v:volumetricdata>`, `<v:boundary>`, `<v:voxels>`, `<v:voxel>`,
//! `<v:volumetricpropertygroup>`, and `<v:property>`.

use crate::error::{Error, Result};
use crate::model::*;
use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use std::io::Write as IoWrite;

/// Write a volumetric property group resource (`<v:volumetricpropertygroup>`)
pub(super) fn write_volumetric_property_group<W: IoWrite>(
    writer: &mut Writer<W>,
    group: &VolumetricPropertyGroup,
) -> Result<()> {
    let mut elem = BytesStart::new("v:volumetricpropertygroup");
    elem.push_attribute(("id", group.id.to_string().as_str()));

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write volumetricpropertygroup element: {}", e)))?;

    for prop in &group.properties {
        let mut prop_elem = BytesStart::new("v:property");
        prop_elem.push_attribute(("index", prop.index.to_string().as_str()));
        prop_elem.push_attribute(("value", prop.value.as_str()));

        writer
            .write_event(Event::Empty(prop_elem))
            .map_err(|e| Error::xml_write(format!("Failed to write property element: {}", e)))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("v:volumetricpropertygroup")))
        .map_err(|e| Error::xml_write(format!("Failed to close volumetricpropertygroup element: {}", e)))?;

    Ok(())
}

/// Write a volumetric data resource (`<v:volumetricdata>`)
pub(super) fn write_volumetric_data<W: IoWrite>(
    writer: &mut Writer<W>,
    vol_data: &VolumetricData,
) -> Result<()> {
    let mut elem = BytesStart::new("v:volumetricdata");
    elem.push_attribute(("id", vol_data.id.to_string().as_str()));

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write volumetricdata element: {}", e)))?;

    // Write boundary if present
    if let Some(ref boundary) = vol_data.boundary {
        write_boundary(writer, boundary)?;
    }

    // Write voxel grid if present
    if let Some(ref voxels) = vol_data.voxels {
        write_voxels(writer, voxels)?;
    }

    // Write implicit volume if present
    if let Some(ref implicit) = vol_data.implicit {
        write_implicit(writer, implicit)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("v:volumetricdata")))
        .map_err(|e| Error::xml_write(format!("Failed to close volumetricdata element: {}", e)))?;

    Ok(())
}

/// Write a `<v:boundary>` element
fn write_boundary<W: IoWrite>(
    writer: &mut Writer<W>,
    boundary: &VolumetricBoundary,
) -> Result<()> {
    let mut elem = BytesStart::new("v:boundary");

    let min_str = format!("{} {} {}", boundary.min.0, boundary.min.1, boundary.min.2);
    let max_str = format!("{} {} {}", boundary.max.0, boundary.max.1, boundary.max.2);

    elem.push_attribute(("min", min_str.as_str()));
    elem.push_attribute(("max", max_str.as_str()));

    writer
        .write_event(Event::Empty(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write boundary element: {}", e)))?;

    Ok(())
}

/// Write a `<v:voxels>` element with child `<v:voxel>` entries
fn write_voxels<W: IoWrite>(writer: &mut Writer<W>, grid: &VoxelGrid) -> Result<()> {
    let mut elem = BytesStart::new("v:voxels");

    let dims_str = format!("{} {} {}", grid.dimensions.0, grid.dimensions.1, grid.dimensions.2);
    elem.push_attribute(("dimensions", dims_str.as_str()));

    if let Some(spacing) = grid.spacing {
        let spacing_str = format!("{} {} {}", spacing.0, spacing.1, spacing.2);
        elem.push_attribute(("spacing", spacing_str.as_str()));
    }

    if let Some(origin) = grid.origin {
        let origin_str = format!("{} {} {}", origin.0, origin.1, origin.2);
        elem.push_attribute(("origin", origin_str.as_str()));
    }

    writer
        .write_event(Event::Start(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write voxels element: {}", e)))?;

    for voxel in &grid.voxels {
        let mut voxel_elem = BytesStart::new("v:voxel");
        voxel_elem.push_attribute(("x", voxel.position.0.to_string().as_str()));
        voxel_elem.push_attribute(("y", voxel.position.1.to_string().as_str()));
        voxel_elem.push_attribute(("z", voxel.position.2.to_string().as_str()));

        if let Some(prop_id) = voxel.property_id {
            voxel_elem.push_attribute(("property", prop_id.to_string().as_str()));
        }

        if let Some(color_id) = voxel.color_id {
            voxel_elem.push_attribute(("color", color_id.to_string().as_str()));
        }

        writer
            .write_event(Event::Empty(voxel_elem))
            .map_err(|e| Error::xml_write(format!("Failed to write voxel element: {}", e)))?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("v:voxels")))
        .map_err(|e| Error::xml_write(format!("Failed to close voxels element: {}", e)))?;

    Ok(())
}

/// Write a `<v:implicit>` element
fn write_implicit<W: IoWrite>(
    writer: &mut Writer<W>,
    implicit: &ImplicitVolume,
) -> Result<()> {
    let mut elem = BytesStart::new("v:implicit");
    elem.push_attribute(("type", implicit.implicit_type.as_str()));

    for (key, value) in &implicit.parameters {
        elem.push_attribute((key.as_str(), value.as_str()));
    }

    writer
        .write_event(Event::Empty(elem))
        .map_err(|e| Error::xml_write(format!("Failed to write implicit element: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_volumetric_property_group() {
        let mut group = VolumetricPropertyGroup::new(1);
        group
            .properties
            .push(VolumetricProperty::new(0, "solid".to_string()));

        let mut buffer = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buffer, b' ', 2);
        write_volumetric_property_group(&mut writer, &group).unwrap();

        let xml = String::from_utf8(buffer).unwrap();
        assert!(xml.contains("v:volumetricpropertygroup"));
        assert!(xml.contains("id=\"1\""));
        assert!(xml.contains("index=\"0\""));
        assert!(xml.contains("value=\"solid\""));
    }

    #[test]
    fn test_write_volumetric_data_with_boundary_and_voxels() {
        let mut vol_data = VolumetricData::new(1);
        vol_data.boundary = Some(VolumetricBoundary::new(
            (0.0, 0.0, 0.0),
            (10.0, 10.0, 10.0),
        ));

        let mut grid = VoxelGrid::new((10, 10, 10));
        grid.spacing = Some((1.0, 1.0, 1.0));
        grid.voxels.push(Voxel::new((5, 5, 5)));
        vol_data.voxels = Some(grid);

        let mut buffer = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buffer, b' ', 2);
        write_volumetric_data(&mut writer, &vol_data).unwrap();

        let xml = String::from_utf8(buffer).unwrap();
        assert!(xml.contains("v:volumetricdata"));
        assert!(xml.contains("v:boundary"));
        assert!(xml.contains("v:voxels"));
        assert!(xml.contains("v:voxel"));
        assert!(xml.contains("x=\"5\""));
    }

    #[test]
    fn test_write_volumetric_data_with_implicit() {
        let mut vol_data = VolumetricData::new(2);
        let mut implicit = ImplicitVolume::new("sdf".to_string());
        implicit
            .parameters
            .push(("radius".to_string(), "5.0".to_string()));
        vol_data.implicit = Some(implicit);

        let mut buffer = Vec::new();
        let mut writer = Writer::new_with_indent(&mut buffer, b' ', 2);
        write_volumetric_data(&mut writer, &vol_data).unwrap();

        let xml = String::from_utf8(buffer).unwrap();
        assert!(xml.contains("v:implicit"));
        assert!(xml.contains("type=\"sdf\""));
        assert!(xml.contains("radius=\"5.0\""));
    }
}
