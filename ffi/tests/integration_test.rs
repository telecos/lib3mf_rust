//! Integration tests for the lib3mf FFI layer.
//!
//! These tests exercise the FFI functions against real 3MF files from the
//! repository's test_files/ directory.

use lib3mf_ffi::*;
use std::ffi::{CStr, CString};
use std::ptr;
use tempfile::NamedTempFile;

/// Helper: read the box.3mf test file via FFI and return the model handle.
///
/// # Safety
///
/// The caller must free the returned handle with `lib3mf_model_destroy`.
unsafe fn read_box_model() -> *mut Lib3mfModel {
    let path = CString::new("../test_files/core/box.3mf").unwrap();
    let mut model: *mut Lib3mfModel = ptr::null_mut();
    let result = unsafe { lib3mf_model_read_file(path.as_ptr(), &mut model) };
    assert!(
        matches!(result, Lib3mfResult::Ok),
        "Failed to read box.3mf: {:?}",
        unsafe { CStr::from_ptr(lib3mf_get_last_error()) }
    );
    assert!(!model.is_null());
    model
}

#[test]
fn test_read_box_3mf_objects() {
    unsafe {
        let model = read_box_model();

        // box.3mf should have at least 1 object
        let mut obj_count: u32 = 0;
        let result = lib3mf_model_get_object_count(model, &mut obj_count);
        assert!(matches!(result, Lib3mfResult::Ok));
        assert!(
            obj_count >= 1,
            "Expected at least 1 object, got {obj_count}"
        );

        // The first object should have an ID
        let mut id: u32 = 0;
        let result = lib3mf_model_get_object_id(model, 0, &mut id);
        assert!(matches!(result, Lib3mfResult::Ok));
        assert!(id > 0, "Expected object ID > 0");

        lib3mf_model_destroy(model);
    }
}

#[test]
fn test_read_box_3mf_mesh_data() {
    unsafe {
        let model = read_box_model();

        // A box mesh should have 8 vertices and 12 triangles
        let mut vert_count: u32 = 0;
        let result = lib3mf_model_get_object_vertex_count(model, 0, &mut vert_count);
        assert!(matches!(result, Lib3mfResult::Ok));
        assert_eq!(vert_count, 8, "A box should have 8 vertices");

        let mut tri_count: u32 = 0;
        let result = lib3mf_model_get_object_triangle_count(model, 0, &mut tri_count);
        assert!(matches!(result, Lib3mfResult::Ok));
        assert_eq!(tri_count, 12, "A box should have 12 triangles");

        // Read the first vertex
        let mut vertex = Lib3mfVertex {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let result = lib3mf_model_get_object_vertex(model, 0, 0, &mut vertex);
        assert!(matches!(result, Lib3mfResult::Ok));

        // Read the first triangle
        let mut triangle = Lib3mfTriangle {
            v1: 0,
            v2: 0,
            v3: 0,
        };
        let result = lib3mf_model_get_object_triangle(model, 0, 0, &mut triangle);
        assert!(matches!(result, Lib3mfResult::Ok));
        // Triangle indices should be within bounds
        assert!(triangle.v1 < vert_count);
        assert!(triangle.v2 < vert_count);
        assert!(triangle.v3 < vert_count);

        lib3mf_model_destroy(model);
    }
}

#[test]
fn test_read_box_3mf_build_items() {
    unsafe {
        let model = read_box_model();

        let mut item_count: u32 = 0;
        let result = lib3mf_model_get_build_item_count(model, &mut item_count);
        assert!(matches!(result, Lib3mfResult::Ok));
        assert!(
            item_count >= 1,
            "Expected at least 1 build item, got {item_count}"
        );

        // Get the first build item's object ID
        let mut object_id: u32 = 0;
        let result = lib3mf_model_get_build_item_object_id(model, 0, &mut object_id);
        assert!(matches!(result, Lib3mfResult::Ok));
        assert!(object_id > 0);

        lib3mf_model_destroy(model);
    }
}

#[test]
fn test_read_box_3mf_unit() {
    unsafe {
        let model = read_box_model();

        let mut unit: *const std::os::raw::c_char = ptr::null();
        let result = lib3mf_model_get_unit(model, &mut unit);
        assert!(matches!(result, Lib3mfResult::Ok));
        assert!(!unit.is_null());

        let unit_str = CStr::from_ptr(unit).to_str().unwrap();
        assert_eq!(unit_str, "millimeter");

        lib3mf_model_destroy(model);
    }
}

#[test]
fn test_write_and_read_roundtrip() {
    unsafe {
        let model = read_box_model();

        // Get original vertex count
        let mut orig_vert_count: u32 = 0;
        lib3mf_model_get_object_vertex_count(model, 0, &mut orig_vert_count);

        // Write to a temp file
        let tmp = NamedTempFile::new().unwrap();
        let tmp_path = CString::new(tmp.path().to_str().unwrap()).unwrap();
        let result = lib3mf_model_write_file(model, tmp_path.as_ptr());
        assert!(
            matches!(result, Lib3mfResult::Ok),
            "Write failed: {:?}",
            CStr::from_ptr(lib3mf_get_last_error())
        );

        lib3mf_model_destroy(model);

        // Read back the written file
        let mut model2: *mut Lib3mfModel = ptr::null_mut();
        let result = lib3mf_model_read_file(tmp_path.as_ptr(), &mut model2);
        assert!(
            matches!(result, Lib3mfResult::Ok),
            "Re-read failed: {:?}",
            CStr::from_ptr(lib3mf_get_last_error())
        );

        let mut vert_count2: u32 = 0;
        lib3mf_model_get_object_vertex_count(model2, 0, &mut vert_count2);
        assert_eq!(
            orig_vert_count, vert_count2,
            "Vertex count mismatch after round-trip"
        );

        lib3mf_model_destroy(model2);
    }
}

#[test]
fn test_iterate_all_vertices_and_triangles() {
    unsafe {
        let model = read_box_model();

        let mut vert_count: u32 = 0;
        lib3mf_model_get_object_vertex_count(model, 0, &mut vert_count);

        let mut tri_count: u32 = 0;
        lib3mf_model_get_object_triangle_count(model, 0, &mut tri_count);

        // Read all vertices
        for i in 0..vert_count {
            let mut v = Lib3mfVertex {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            };
            let result = lib3mf_model_get_object_vertex(model, 0, i, &mut v);
            assert!(matches!(result, Lib3mfResult::Ok));
        }

        // Read all triangles
        for i in 0..tri_count {
            let mut t = Lib3mfTriangle {
                v1: 0,
                v2: 0,
                v3: 0,
            };
            let result = lib3mf_model_get_object_triangle(model, 0, i, &mut t);
            assert!(matches!(result, Lib3mfResult::Ok));
            assert!(t.v1 < vert_count);
            assert!(t.v2 < vert_count);
            assert!(t.v3 < vert_count);
        }

        // One past the end should fail
        let mut v = Lib3mfVertex {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let result = lib3mf_model_get_object_vertex(model, 0, vert_count, &mut v);
        assert!(matches!(result, Lib3mfResult::ErrorIndexOutOfBounds));

        lib3mf_model_destroy(model);
    }
}
