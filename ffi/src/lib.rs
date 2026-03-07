//! C/C++ FFI bindings for the lib3mf 3MF file parser.
//!
//! This crate provides a C-compatible API for reading and writing 3MF files.
//! All functions use opaque handles and return error codes for safe interop.
//!
//! # Thread Safety
//!
//! The error state is stored per-thread. Each thread maintains its own last
//! error message.
//!
//! # Memory Management
//!
//! - Handles returned by `lib3mf_model_create` or `lib3mf_model_read_file`
//!   must be freed with `lib3mf_model_destroy`.
//! - String pointers returned by query functions (e.g., `lib3mf_model_get_unit`)
//!   are valid only until the next FFI call on the same thread, or until the
//!   model handle is destroyed.

use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::os::raw::c_char;

use lib3mf::Model;

// ---------------------------------------------------------------------------
// Error codes
// ---------------------------------------------------------------------------

/// Result code returned by all FFI functions.
#[repr(i32)]
pub enum Lib3mfResult {
    /// Operation completed successfully.
    Ok = 0,
    /// A null pointer was passed where a valid pointer was required.
    ErrorNullPointer = 1,
    /// An I/O error occurred (e.g., file not found).
    ErrorIo = 2,
    /// The 3MF file could not be parsed.
    ErrorParse = 3,
    /// An index parameter was out of bounds.
    ErrorIndexOutOfBounds = 4,
    /// An invalid argument was provided.
    ErrorInvalidArg = 5,
    /// An unspecified internal error occurred.
    ErrorInternal = 6,
}

// ---------------------------------------------------------------------------
// Thread-local error state & temporary string storage
// ---------------------------------------------------------------------------

thread_local! {
    static LAST_ERROR: RefCell<CString> = RefCell::new(CString::default());
    static TEMP_STRING: RefCell<CString> = RefCell::new(CString::default());
    static TEMP_STRING2: RefCell<CString> = RefCell::new(CString::default());
}

fn set_last_error(msg: &str) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = CString::new(msg).unwrap_or_default();
    });
}

fn clear_last_error() {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = CString::default();
    });
}

/// Store a string temporarily and return a pointer that is valid until the
/// next call that stores a temporary string on the same thread.
fn store_temp_string(s: &str) -> *const c_char {
    TEMP_STRING.with(|ts| {
        let c = CString::new(s).unwrap_or_default();
        let ptr = c.as_ptr();
        *ts.borrow_mut() = c;
        ptr
    })
}

// ---------------------------------------------------------------------------
// Opaque handle
// ---------------------------------------------------------------------------

/// Opaque handle representing a 3MF model.
///
/// Created by [`lib3mf_model_create`] or [`lib3mf_model_read_file`] and must
/// be freed with [`lib3mf_model_destroy`].
pub struct Lib3mfModel {
    inner: Model,
}

// ---------------------------------------------------------------------------
// C-compatible data types
// ---------------------------------------------------------------------------

/// A 3D vertex with double-precision coordinates.
#[repr(C)]
pub struct Lib3mfVertex {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// A triangle defined by three vertex indices.
#[repr(C)]
pub struct Lib3mfTriangle {
    pub v1: u32,
    pub v2: u32,
    pub v3: u32,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a lib3mf error into an FFI result code, storing the message.
fn map_err(e: lib3mf::Error) -> Lib3mfResult {
    let msg = format!("{e}");
    set_last_error(&msg);
    match e {
        lib3mf::Error::Io(_) | lib3mf::Error::MissingFile(_) => Lib3mfResult::ErrorIo,
        lib3mf::Error::Zip(_) => Lib3mfResult::ErrorIo,
        lib3mf::Error::Xml(_)
        | lib3mf::Error::XmlAttr(_)
        | lib3mf::Error::InvalidXml(_)
        | lib3mf::Error::InvalidFormat(_)
        | lib3mf::Error::InvalidModel(_)
        | lib3mf::Error::ParseError(_) => Lib3mfResult::ErrorParse,
        _ => Lib3mfResult::ErrorInternal,
    }
}

macro_rules! check_not_null {
    ($ptr:expr) => {
        if $ptr.is_null() {
            set_last_error("null pointer argument");
            return Lib3mfResult::ErrorNullPointer;
        }
    };
}

// ---------------------------------------------------------------------------
// Error query
// ---------------------------------------------------------------------------

/// Returns a pointer to a null-terminated string describing the last error.
///
/// The pointer is valid until the next FFI call on the same thread. If no
/// error has occurred the string is empty.
///
/// # Safety
///
/// The returned pointer must not be used after the next FFI call on the same thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lib3mf_get_last_error() -> *const c_char {
    LAST_ERROR.with(|e| e.borrow().as_ptr())
}

// ---------------------------------------------------------------------------
// Model lifecycle
// ---------------------------------------------------------------------------

/// Creates a new, empty 3MF model.
///
/// On success, `*model_out` is set to a valid handle and `Lib3mfResult::Ok`
/// is returned. The caller must free the handle with [`lib3mf_model_destroy`].
///
/// # Safety
///
/// `model_out` must be a valid, non-null pointer to a `*mut Lib3mfModel`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lib3mf_model_create(model_out: *mut *mut Lib3mfModel) -> Lib3mfResult {
    clear_last_error();
    check_not_null!(model_out);

    let handle = Box::new(Lib3mfModel {
        inner: Model::new(),
    });
    unsafe {
        *model_out = Box::into_raw(handle);
    }
    Lib3mfResult::Ok
}

/// Reads a 3MF file from disk and returns a model handle.
///
/// `path` must be a null-terminated UTF-8 string. On success, `*model_out`
/// is set to a valid handle. The caller must free it with
/// [`lib3mf_model_destroy`].
///
/// # Safety
///
/// `path` must be a valid, non-null pointer to a null-terminated UTF-8 string.
/// `model_out` must be a valid, non-null pointer to a `*mut Lib3mfModel`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lib3mf_model_read_file(
    path: *const c_char,
    model_out: *mut *mut Lib3mfModel,
) -> Lib3mfResult {
    clear_last_error();
    check_not_null!(path);
    check_not_null!(model_out);

    let c_path = unsafe { CStr::from_ptr(path) };
    let path_str = match c_path.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(&format!("invalid UTF-8 path: {e}"));
            return Lib3mfResult::ErrorInvalidArg;
        }
    };

    let file = match File::open(path_str) {
        Ok(f) => f,
        Err(e) => {
            set_last_error(&format!("failed to open file: {e}"));
            return Lib3mfResult::ErrorIo;
        }
    };

    match Model::from_reader(file) {
        Ok(model) => {
            let handle = Box::new(Lib3mfModel { inner: model });
            unsafe {
                *model_out = Box::into_raw(handle);
            }
            Lib3mfResult::Ok
        }
        Err(e) => map_err(e),
    }
}

/// Writes the model to a 3MF file on disk.
///
/// The model handle remains valid after this call.
/// `path` must be a null-terminated UTF-8 string.
///
/// # Safety
///
/// `model` must be a valid handle obtained from [`lib3mf_model_create`] or
/// [`lib3mf_model_read_file`]. `path` must be a valid, non-null pointer to a
/// null-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lib3mf_model_write_file(
    model: *const Lib3mfModel,
    path: *const c_char,
) -> Lib3mfResult {
    clear_last_error();
    check_not_null!(model);
    check_not_null!(path);

    let handle = unsafe { &*model };

    let c_path = unsafe { CStr::from_ptr(path) };
    let path_str = match c_path.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(&format!("invalid UTF-8 path: {e}"));
            return Lib3mfResult::ErrorInvalidArg;
        }
    };

    // Clone so the handle stays valid after writing.
    let cloned = handle.inner.clone();
    match cloned.write_to_file(path_str) {
        Ok(()) => Lib3mfResult::Ok,
        Err(e) => map_err(e),
    }
}

/// Frees a model handle previously obtained from [`lib3mf_model_create`] or
/// [`lib3mf_model_read_file`].
///
/// Passing a null pointer is a safe no-op.
///
/// # Safety
///
/// `model` must be either null or a valid handle that has not already been
/// destroyed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lib3mf_model_destroy(model: *mut Lib3mfModel) {
    if !model.is_null() {
        unsafe {
            drop(Box::from_raw(model));
        }
    }
}

// ---------------------------------------------------------------------------
// Model queries
// ---------------------------------------------------------------------------

/// Returns a pointer to the model's unit string (e.g. `"millimeter"`).
///
/// The pointer is valid until the next call to this function on the same
/// thread, or until the model is destroyed.
///
/// # Safety
///
/// `model` must be a valid, non-null model handle. `unit_out` must be a
/// valid, non-null pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lib3mf_model_get_unit(
    model: *const Lib3mfModel,
    unit_out: *mut *const c_char,
) -> Lib3mfResult {
    clear_last_error();
    check_not_null!(model);
    check_not_null!(unit_out);

    let handle = unsafe { &*model };
    let ptr = store_temp_string(&handle.inner.unit);
    unsafe {
        *unit_out = ptr;
    }
    Lib3mfResult::Ok
}

/// Returns the number of metadata entries in the model.
///
/// # Safety
///
/// `model` and `count_out` must be valid, non-null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lib3mf_model_get_metadata_count(
    model: *const Lib3mfModel,
    count_out: *mut u32,
) -> Lib3mfResult {
    clear_last_error();
    check_not_null!(model);
    check_not_null!(count_out);

    let handle = unsafe { &*model };
    unsafe {
        *count_out = handle.inner.metadata.len() as u32;
    }
    Lib3mfResult::Ok
}

/// Returns the name and value of a metadata entry by index.
///
/// `name_out` and `value_out` are set to temporary string pointers that are
/// valid until the next FFI call on the same thread.
///
/// # Safety
///
/// `model`, `name_out`, and `value_out` must be valid, non-null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lib3mf_model_get_metadata(
    model: *const Lib3mfModel,
    index: u32,
    name_out: *mut *const c_char,
    value_out: *mut *const c_char,
) -> Lib3mfResult {
    clear_last_error();
    check_not_null!(model);
    check_not_null!(name_out);
    check_not_null!(value_out);

    let handle = unsafe { &*model };
    let idx = index as usize;
    if idx >= handle.inner.metadata.len() {
        set_last_error("metadata index out of bounds");
        return Lib3mfResult::ErrorIndexOutOfBounds;
    }

    let entry = &handle.inner.metadata[idx];

    let name_c = CString::new(entry.name.as_str()).unwrap_or_default();
    let value_c = CString::new(entry.value.as_str()).unwrap_or_default();

    TEMP_STRING.with(|ts| {
        *ts.borrow_mut() = name_c;
    });
    TEMP_STRING2.with(|ts2| {
        *ts2.borrow_mut() = value_c;
    });

    unsafe {
        TEMP_STRING.with(|ts| {
            *name_out = ts.borrow().as_ptr();
        });
        TEMP_STRING2.with(|ts2| {
            *value_out = ts2.borrow().as_ptr();
        });
    }
    Lib3mfResult::Ok
}

// ---------------------------------------------------------------------------
// Object queries
// ---------------------------------------------------------------------------

/// Returns the number of objects in the model.
///
/// # Safety
///
/// `model` and `count_out` must be valid, non-null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lib3mf_model_get_object_count(
    model: *const Lib3mfModel,
    count_out: *mut u32,
) -> Lib3mfResult {
    clear_last_error();
    check_not_null!(model);
    check_not_null!(count_out);

    let handle = unsafe { &*model };
    unsafe {
        *count_out = handle.inner.resources.objects.len() as u32;
    }
    Lib3mfResult::Ok
}

/// Returns the ID of an object by its index in the object list.
///
/// # Safety
///
/// `model` and `id_out` must be valid, non-null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lib3mf_model_get_object_id(
    model: *const Lib3mfModel,
    index: u32,
    id_out: *mut u32,
) -> Lib3mfResult {
    clear_last_error();
    check_not_null!(model);
    check_not_null!(id_out);

    let handle = unsafe { &*model };
    let idx = index as usize;
    if idx >= handle.inner.resources.objects.len() {
        set_last_error("object index out of bounds");
        return Lib3mfResult::ErrorIndexOutOfBounds;
    }
    unsafe {
        *id_out = handle.inner.resources.objects[idx].id as u32;
    }
    Lib3mfResult::Ok
}

/// Returns the name of an object, or an empty string if unnamed.
///
/// The returned pointer is valid until the next FFI call on the same thread.
///
/// # Safety
///
/// `model` and `name_out` must be valid, non-null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lib3mf_model_get_object_name(
    model: *const Lib3mfModel,
    index: u32,
    name_out: *mut *const c_char,
) -> Lib3mfResult {
    clear_last_error();
    check_not_null!(model);
    check_not_null!(name_out);

    let handle = unsafe { &*model };
    let idx = index as usize;
    if idx >= handle.inner.resources.objects.len() {
        set_last_error("object index out of bounds");
        return Lib3mfResult::ErrorIndexOutOfBounds;
    }

    let name = handle.inner.resources.objects[idx]
        .name
        .as_deref()
        .unwrap_or("");
    let ptr = store_temp_string(name);
    unsafe {
        *name_out = ptr;
    }
    Lib3mfResult::Ok
}

// ---------------------------------------------------------------------------
// Mesh queries
// ---------------------------------------------------------------------------

/// Returns the number of vertices in an object's mesh.
///
/// If the object has no mesh, `*count_out` is set to 0.
///
/// # Safety
///
/// `model` and `count_out` must be valid, non-null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lib3mf_model_get_object_vertex_count(
    model: *const Lib3mfModel,
    object_index: u32,
    count_out: *mut u32,
) -> Lib3mfResult {
    clear_last_error();
    check_not_null!(model);
    check_not_null!(count_out);

    let handle = unsafe { &*model };
    let idx = object_index as usize;
    if idx >= handle.inner.resources.objects.len() {
        set_last_error("object index out of bounds");
        return Lib3mfResult::ErrorIndexOutOfBounds;
    }

    let count = handle.inner.resources.objects[idx]
        .mesh
        .as_ref()
        .map_or(0, |m| m.vertices.len());
    unsafe {
        *count_out = count as u32;
    }
    Lib3mfResult::Ok
}

/// Returns the number of triangles in an object's mesh.
///
/// If the object has no mesh, `*count_out` is set to 0.
///
/// # Safety
///
/// `model` and `count_out` must be valid, non-null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lib3mf_model_get_object_triangle_count(
    model: *const Lib3mfModel,
    object_index: u32,
    count_out: *mut u32,
) -> Lib3mfResult {
    clear_last_error();
    check_not_null!(model);
    check_not_null!(count_out);

    let handle = unsafe { &*model };
    let idx = object_index as usize;
    if idx >= handle.inner.resources.objects.len() {
        set_last_error("object index out of bounds");
        return Lib3mfResult::ErrorIndexOutOfBounds;
    }

    let count = handle.inner.resources.objects[idx]
        .mesh
        .as_ref()
        .map_or(0, |m| m.triangles.len());
    unsafe {
        *count_out = count as u32;
    }
    Lib3mfResult::Ok
}

/// Retrieves a single vertex from an object's mesh.
///
/// # Safety
///
/// `model` and `vertex_out` must be valid, non-null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lib3mf_model_get_object_vertex(
    model: *const Lib3mfModel,
    object_index: u32,
    vertex_index: u32,
    vertex_out: *mut Lib3mfVertex,
) -> Lib3mfResult {
    clear_last_error();
    check_not_null!(model);
    check_not_null!(vertex_out);

    let handle = unsafe { &*model };
    let oidx = object_index as usize;
    if oidx >= handle.inner.resources.objects.len() {
        set_last_error("object index out of bounds");
        return Lib3mfResult::ErrorIndexOutOfBounds;
    }

    let mesh = match &handle.inner.resources.objects[oidx].mesh {
        Some(m) => m,
        None => {
            set_last_error("object has no mesh");
            return Lib3mfResult::ErrorInvalidArg;
        }
    };

    let vidx = vertex_index as usize;
    if vidx >= mesh.vertices.len() {
        set_last_error("vertex index out of bounds");
        return Lib3mfResult::ErrorIndexOutOfBounds;
    }

    let v = &mesh.vertices[vidx];
    unsafe {
        *vertex_out = Lib3mfVertex {
            x: v.x,
            y: v.y,
            z: v.z,
        };
    }
    Lib3mfResult::Ok
}

/// Retrieves a single triangle from an object's mesh.
///
/// # Safety
///
/// `model` and `triangle_out` must be valid, non-null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lib3mf_model_get_object_triangle(
    model: *const Lib3mfModel,
    object_index: u32,
    triangle_index: u32,
    triangle_out: *mut Lib3mfTriangle,
) -> Lib3mfResult {
    clear_last_error();
    check_not_null!(model);
    check_not_null!(triangle_out);

    let handle = unsafe { &*model };
    let oidx = object_index as usize;
    if oidx >= handle.inner.resources.objects.len() {
        set_last_error("object index out of bounds");
        return Lib3mfResult::ErrorIndexOutOfBounds;
    }

    let mesh = match &handle.inner.resources.objects[oidx].mesh {
        Some(m) => m,
        None => {
            set_last_error("object has no mesh");
            return Lib3mfResult::ErrorInvalidArg;
        }
    };

    let tidx = triangle_index as usize;
    if tidx >= mesh.triangles.len() {
        set_last_error("triangle index out of bounds");
        return Lib3mfResult::ErrorIndexOutOfBounds;
    }

    let t = &mesh.triangles[tidx];
    unsafe {
        *triangle_out = Lib3mfTriangle {
            v1: t.v1 as u32,
            v2: t.v2 as u32,
            v3: t.v3 as u32,
        };
    }
    Lib3mfResult::Ok
}

// ---------------------------------------------------------------------------
// Build item queries
// ---------------------------------------------------------------------------

/// Returns the number of build items.
///
/// # Safety
///
/// `model` and `count_out` must be valid, non-null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lib3mf_model_get_build_item_count(
    model: *const Lib3mfModel,
    count_out: *mut u32,
) -> Lib3mfResult {
    clear_last_error();
    check_not_null!(model);
    check_not_null!(count_out);

    let handle = unsafe { &*model };
    unsafe {
        *count_out = handle.inner.build.items.len() as u32;
    }
    Lib3mfResult::Ok
}

/// Returns the object ID referenced by a build item.
///
/// # Safety
///
/// `model` and `object_id_out` must be valid, non-null pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lib3mf_model_get_build_item_object_id(
    model: *const Lib3mfModel,
    index: u32,
    object_id_out: *mut u32,
) -> Lib3mfResult {
    clear_last_error();
    check_not_null!(model);
    check_not_null!(object_id_out);

    let handle = unsafe { &*model };
    let idx = index as usize;
    if idx >= handle.inner.build.items.len() {
        set_last_error("build item index out of bounds");
        return Lib3mfResult::ErrorIndexOutOfBounds;
    }

    unsafe {
        *object_id_out = handle.inner.build.items[idx].objectid as u32;
    }
    Lib3mfResult::Ok
}

/// Retrieves the 4×3 affine transformation matrix of a build item.
///
/// If the build item has a transform, `*has_transform_out` is set to `true`
/// and the 12 doubles are written to `transform_out`. Otherwise
/// `*has_transform_out` is set to `false` and `transform_out` is untouched.
///
/// `transform_out` must point to an array of at least 12 `f64` values.
///
/// # Safety
///
/// `model`, `transform_out`, and `has_transform_out` must be valid, non-null
/// pointers. `transform_out` must point to at least 12 contiguous `f64` values.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lib3mf_model_get_build_item_transform(
    model: *const Lib3mfModel,
    index: u32,
    transform_out: *mut f64,
    has_transform_out: *mut bool,
) -> Lib3mfResult {
    clear_last_error();
    check_not_null!(model);
    check_not_null!(transform_out);
    check_not_null!(has_transform_out);

    let handle = unsafe { &*model };
    let idx = index as usize;
    if idx >= handle.inner.build.items.len() {
        set_last_error("build item index out of bounds");
        return Lib3mfResult::ErrorIndexOutOfBounds;
    }

    let item = &handle.inner.build.items[idx];
    match &item.transform {
        Some(t) => unsafe {
            *has_transform_out = true;
            for (i, &val) in t.iter().enumerate() {
                *transform_out.add(i) = val;
            }
        },
        None => unsafe {
            *has_transform_out = false;
        },
    }
    Lib3mfResult::Ok
}

// ---------------------------------------------------------------------------
// Tests (Rust-side unit tests for the FFI functions)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    use std::ptr;

    #[test]
    fn test_create_and_destroy() {
        unsafe {
            let mut model: *mut Lib3mfModel = ptr::null_mut();
            let result = lib3mf_model_create(&mut model);
            assert!(matches!(result, Lib3mfResult::Ok));
            assert!(!model.is_null());
            lib3mf_model_destroy(model);
        }
    }

    #[test]
    fn test_null_pointer_returns_error() {
        unsafe {
            let result = lib3mf_model_create(ptr::null_mut());
            assert!(matches!(result, Lib3mfResult::ErrorNullPointer));
        }
    }

    #[test]
    fn test_destroy_null_is_safe() {
        unsafe {
            lib3mf_model_destroy(ptr::null_mut());
        }
    }

    #[test]
    fn test_model_unit() {
        unsafe {
            let mut model: *mut Lib3mfModel = ptr::null_mut();
            lib3mf_model_create(&mut model);

            let mut unit: *const c_char = ptr::null();
            let result = lib3mf_model_get_unit(model, &mut unit);
            assert!(matches!(result, Lib3mfResult::Ok));
            assert!(!unit.is_null());

            let unit_str = CStr::from_ptr(unit).to_str().unwrap();
            assert_eq!(unit_str, "millimeter");

            lib3mf_model_destroy(model);
        }
    }

    #[test]
    fn test_empty_model_counts() {
        unsafe {
            let mut model: *mut Lib3mfModel = ptr::null_mut();
            lib3mf_model_create(&mut model);

            let mut count: u32 = 99;

            lib3mf_model_get_object_count(model, &mut count);
            assert_eq!(count, 0);

            lib3mf_model_get_build_item_count(model, &mut count);
            assert_eq!(count, 0);

            lib3mf_model_get_metadata_count(model, &mut count);
            assert_eq!(count, 0);

            lib3mf_model_destroy(model);
        }
    }

    #[test]
    fn test_object_index_out_of_bounds() {
        unsafe {
            let mut model: *mut Lib3mfModel = ptr::null_mut();
            lib3mf_model_create(&mut model);

            let mut id: u32 = 0;
            let result = lib3mf_model_get_object_id(model, 0, &mut id);
            assert!(matches!(result, Lib3mfResult::ErrorIndexOutOfBounds));

            lib3mf_model_destroy(model);
        }
    }

    #[test]
    fn test_read_file_not_found() {
        unsafe {
            let path = CString::new("/nonexistent/file.3mf").unwrap();
            let mut model: *mut Lib3mfModel = ptr::null_mut();
            let result = lib3mf_model_read_file(path.as_ptr(), &mut model);
            assert!(matches!(result, Lib3mfResult::ErrorIo));
            assert!(model.is_null());

            let err = lib3mf_get_last_error();
            let err_str = CStr::from_ptr(err).to_str().unwrap();
            assert!(!err_str.is_empty());
        }
    }

    #[test]
    fn test_get_last_error_empty_on_success() {
        unsafe {
            let mut model: *mut Lib3mfModel = ptr::null_mut();
            lib3mf_model_create(&mut model);

            let err = lib3mf_get_last_error();
            let err_str = CStr::from_ptr(err).to_str().unwrap();
            assert!(err_str.is_empty());

            lib3mf_model_destroy(model);
        }
    }
}
