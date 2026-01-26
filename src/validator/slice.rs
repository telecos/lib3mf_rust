//! Slice extension validation

use crate::error::{Error, Result};
use crate::model::Model;

use super::sorted_ids_from_set;

pub fn validate_slices(model: &Model) -> Result<()> {
    // Validate all slice stacks in resources
    for slice_stack in &model.resources.slice_stacks {
        // N_SPX_1606_01: Validate ztop values are >= zbottom
        // N_SPX_1607_01: Validate ztop values are strictly increasing
        let mut prev_ztop: Option<f64> = None;

        for (slice_idx, slice) in slice_stack.slices.iter().enumerate() {
            // Check ztop >= zbottom
            if slice.ztop < slice_stack.zbottom {
                return Err(Error::InvalidModel(format!(
                    "SliceStack {}: Slice {} has ztop={} which is less than zbottom={}.\n\
                     Per 3MF Slice Extension spec, each slice's ztop must be >= the slicestack's zbottom.",
                    slice_stack.id, slice_idx, slice.ztop, slice_stack.zbottom
                )));
            }

            // Check ztop values are strictly increasing
            if let Some(prev) = prev_ztop {
                if slice.ztop <= prev {
                    return Err(Error::InvalidModel(format!(
                        "SliceStack {}: Slice {} has ztop={} which is not greater than the previous slice's ztop={}.\n\
                         Per 3MF Slice Extension spec, ztop values must be strictly increasing within a slicestack.",
                        slice_stack.id, slice_idx, slice.ztop, prev
                    )));
                }
            }
            prev_ztop = Some(slice.ztop);

            validate_slice(slice_stack.id, slice_idx, slice)?;
        }
    }

    Ok(())
}

/// Validate slice extension requirements
///
/// Per 3MF Slice Extension spec v1.0.2:
/// - SliceRef slicepath must point to /2D/ folder (not /3D/ or other directories)
/// - When an object references a slicestack, transforms must be planar (no Z-axis rotation/shear)
/// - SliceStack must contain either slices OR slicerefs, not both

pub fn validate_slice_extension(model: &Model) -> Result<()> {
    // Check if model uses slice extension
    if model.resources.slice_stacks.is_empty() {
        return Ok(());
    }

    // Validate slicerefs in all slicestacks
    for stack in &model.resources.slice_stacks {
        // Rule: SliceStack must contain either slices OR slicerefs, not both
        if !stack.slices.is_empty() && !stack.slice_refs.is_empty() {
            return Err(Error::InvalidModel(format!(
                "SliceStack {}: Contains both <slice> and <sliceref> elements.\n\
                 Per 3MF Slice Extension spec, a slicestack MUST contain either \
                 <slice> elements or <sliceref> elements, but MUST NOT contain both element types concurrently.",
                stack.id
            )));
        }

        // Note: SliceRef validation happens during loading in parser.rs::load_slice_references()
        // because slice_refs are cleared after loading external files are resolved.
        // Validation performed during loading includes:
        // - SliceRef slicepath must start with "/2D/"
        // - Referenced slicestackid must exist in external file
        // - SliceStack cannot contain both <slice> and <sliceref> elements (mixed elements)
    }

    // Build a set of valid slicestack IDs for reference validation
    let valid_slicestack_ids: std::collections::HashSet<usize> = model
        .resources
        .slice_stacks
        .iter()
        .map(|stack| stack.id)
        .collect();

    // Validate that objects reference existing slicestacks
    for object in &model.resources.objects {
        if let Some(slicestackid) = object.slicestackid {
            if !valid_slicestack_ids.contains(&slicestackid) {
                let available_ids = sorted_ids_from_set(&valid_slicestack_ids);
                return Err(Error::InvalidModel(format!(
                    "Object {}: References non-existent slicestackid {}.\n\
                     Per 3MF Slice Extension spec, the slicestackid attribute must reference \
                     a valid <slicestack> resource defined in the model.\n\
                     Available slicestack IDs: {:?}",
                    object.id, slicestackid, available_ids
                )));
            }
        }
    }

    // Find all objects that reference slicestacks
    let mut objects_with_slices: Vec<&crate::model::Object> = Vec::new();
    for object in &model.resources.objects {
        if object.slicestackid.is_some() {
            objects_with_slices.push(object);
        }
    }

    // If no objects reference slicestacks, we're done
    if objects_with_slices.is_empty() {
        return Ok(());
    }

    // Validate transforms for build items that reference objects with slicestacks
    for item in &model.build.items {
        // Check if this build item references an object with a slicestack
        let object_has_slicestack = objects_with_slices
            .iter()
            .any(|obj| obj.id == item.objectid);

        if !object_has_slicestack {
            continue;
        }

        // If object has slicestack, validate that transform is planar
        if let Some(ref transform) = item.transform {
            validate_planar_transform(
                transform,
                &format!("Build Item referencing object {}", item.objectid),
            )?;
        }
    }

    // Also validate transforms in components that reference objects with slicestacks
    for object in &model.resources.objects {
        for component in &object.components {
            // Check if this component references an object with a slicestack
            let component_has_slicestack = objects_with_slices
                .iter()
                .any(|obj| obj.id == component.objectid);

            if !component_has_slicestack {
                continue;
            }

            // If component references object with slicestack, validate transform
            if let Some(ref transform) = component.transform {
                validate_planar_transform(
                    transform,
                    &format!(
                        "Object {}, Component referencing object {}",
                        object.id, component.objectid
                    ),
                )?;
            }
        }
    }

    Ok(())
}

/// Validate a single slice

pub fn validate_slice(
    slice_stack_id: usize,
    slice_idx: usize,
    slice: &crate::model::Slice,
) -> Result<()> {
    // Per 3MF Slice Extension spec and official test suite:
    // Empty slices (no polygons) are allowed - they can represent empty layers
    // or boundaries of the sliced object. However, if a slice has polygons,
    // it must have vertices.

    // If slice is empty (no polygons), it's valid - skip further validation
    if slice.polygons.is_empty() {
        return Ok(());
    }

    // If there are polygons, there must be vertices
    if slice.vertices.is_empty() {
        return Err(Error::InvalidModel(format!(
            "SliceStack {}: Slice {} (ztop={}) has {} polygon(s) but no vertices. \
             Per 3MF Slice Extension spec, slices with polygons must have vertex data. \
             Add vertices to the slice.",
            slice_stack_id,
            slice_idx,
            slice.ztop,
            slice.polygons.len()
        )));
    }

    let num_vertices = slice.vertices.len();

    // Validate polygon vertex indices
    for (poly_idx, polygon) in slice.polygons.iter().enumerate() {
        // Validate startv index
        if polygon.startv >= num_vertices {
            return Err(Error::InvalidModel(format!(
                "SliceStack {}: Slice {} (ztop={}), Polygon {} has invalid startv={} \
                 (slice has {} vertices, valid indices: 0-{}). \
                 Vertex indices must reference valid vertices in the slice.",
                slice_stack_id,
                slice_idx,
                slice.ztop,
                poly_idx,
                polygon.startv,
                num_vertices,
                num_vertices - 1
            )));
        }

        // N_SPX_1609_01: Validate polygon has at least 2 segments (not a single point)
        // A valid polygon needs at least 2 segments to form a shape
        if polygon.segments.len() < 2 {
            return Err(Error::InvalidModel(format!(
                "SliceStack {}: Slice {} (ztop={}), Polygon {} has only {} segment(s).\n\
                 Per 3MF Slice Extension spec, a polygon must have at least 2 segments to form a valid shape.",
                slice_stack_id,
                slice_idx,
                slice.ztop,
                poly_idx,
                polygon.segments.len()
            )));
        }

        // Validate segment v2 indices and check for duplicates
        let mut prev_v2: Option<usize> = None;
        for (seg_idx, segment) in polygon.segments.iter().enumerate() {
            if segment.v2 >= num_vertices {
                return Err(Error::InvalidModel(format!(
                    "SliceStack {}: Slice {} (ztop={}), Polygon {}, Segment {} has invalid v2={} \
                     (slice has {} vertices, valid indices: 0-{}). \
                     Vertex indices must reference valid vertices in the slice.",
                    slice_stack_id,
                    slice_idx,
                    slice.ztop,
                    poly_idx,
                    seg_idx,
                    segment.v2,
                    num_vertices,
                    num_vertices - 1
                )));
            }

            // N_SPX_1608_01: Check for duplicate v2 in consecutive segments
            if let Some(prev) = prev_v2 {
                if segment.v2 == prev {
                    return Err(Error::InvalidModel(format!(
                        "SliceStack {}: Slice {} (ztop={}), Polygon {}, Segments {} and {} have the same v2={}.\n\
                         Per 3MF Slice Extension spec, consecutive segments cannot reference the same vertex.",
                        slice_stack_id,
                        slice_idx,
                        slice.ztop,
                        poly_idx,
                        seg_idx - 1,
                        seg_idx,
                        segment.v2
                    )));
                }
            }
            prev_v2 = Some(segment.v2);
        }

        // N_SPX_1609_02: Validate polygon is closed (last segment v2 == startv)
        if let Some(last_segment) = polygon.segments.last() {
            if last_segment.v2 != polygon.startv {
                return Err(Error::InvalidModel(format!(
                    "SliceStack {}: Slice {} (ztop={}), Polygon {} is not closed.\n\
                     Last segment v2={} does not equal startv={}.\n\
                     Per 3MF Slice Extension spec, polygons must be closed (last segment must connect back to start vertex).",
                    slice_stack_id,
                    slice_idx,
                    slice.ztop,
                    poly_idx,
                    last_segment.v2,
                    polygon.startv
                )));
            }
        }
    }

    Ok(())
}

/// Validate that a transform is planar (no Z-axis rotation or shear)
///
/// Per 3MF Slice Extension spec:
/// When an object references slice model data, the 3D transform matrices in <build><item>
/// and <component> elements are limited to those that do not impact the slicing orientation
/// (planar transformations). Therefore, any transform applied (directly or indirectly) to an
/// object that references a <slicestack> MUST have m02, m12, m20, and m21 equal to zero and
/// m22 equal to one.
///
/// Transform matrix layout (3x3 rotation + translation, stored in row-major order as 12 elements):
/// ```text
/// Matrix representation:
/// [m00, m01, m02, tx,
///  m10, m11, m12, ty,
///  m20, m21, m22, tz]
///
/// Array indices:
/// [0:m00, 1:m01, 2:m02, 3:tx,
///  4:m10, 5:m11, 6:m12, 7:ty,
///  8:m20, 9:m21, 10:m22, 11:tz]
/// ```
///
/// For planar transforms:
/// - m02 (index 2), m12 (index 5), m20 (index 6), m21 (index 7) must be exactly 0.0
/// - m22 (index 8) must be exactly 1.0

pub fn validate_planar_transform(transform: &[f64; 12], context: &str) -> Result<()> {
    // Check m02 (index 2)
    if transform[2] != 0.0 {
        return Err(Error::InvalidModel(format!(
            "{}: Transform is not planar. Matrix element m02 = {} (must be 0.0).\n\
             Per 3MF Slice Extension spec, when an object references a slicestack, \
             transforms must be planar (no Z-axis rotation or shear). Elements m02, m12, m20, m21 \
             must be 0.0 and m22 must be 1.0.",
            context, transform[2]
        )));
    }

    // Check m12 (index 5)
    if transform[5] != 0.0 {
        return Err(Error::InvalidModel(format!(
            "{}: Transform is not planar. Matrix element m12 = {} (must be 0.0).\n\
             Per 3MF Slice Extension spec, when an object references a slicestack, \
             transforms must be planar (no Z-axis rotation or shear). Elements m02, m12, m20, m21 \
             must be 0.0 and m22 must be 1.0.",
            context, transform[5]
        )));
    }

    // Check m20 (index 6)
    if transform[6] != 0.0 {
        return Err(Error::InvalidModel(format!(
            "{}: Transform is not planar. Matrix element m20 = {} (must be 0.0).\n\
             Per 3MF Slice Extension spec, when an object references a slicestack, \
             transforms must be planar (no Z-axis rotation or shear). Elements m02, m12, m20, m21 \
             must be 0.0 and m22 must be 1.0.",
            context, transform[6]
        )));
    }

    // Check m21 (index 7)
    if transform[7] != 0.0 {
        return Err(Error::InvalidModel(format!(
            "{}: Transform is not planar. Matrix element m21 = {} (must be 0.0).\n\
             Per 3MF Slice Extension spec, when an object references a slicestack, \
             transforms must be planar (no Z-axis rotation or shear). Elements m02, m12, m20, m21 \
             must be 0.0 and m22 must be 1.0.",
            context, transform[7]
        )));
    }

    // Check m22 (index 8)
    if transform[8] != 1.0 {
        return Err(Error::InvalidModel(format!(
            "{}: Transform is not planar. Matrix element m22 = {} (must be 1.0).\n\
             Per 3MF Slice Extension spec, when an object references a slicestack, \
             transforms must be planar (no Z-axis rotation or shear). Elements m02, m12, m20, m21 \
             must be 0.0 and m22 must be 1.0.",
            context, transform[8]
        )));
    }

    Ok(())
}
