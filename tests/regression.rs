//! Regression tests
//!
//! Tests for specific issues and bug fixes

mod regression {
    pub mod error_type_validation;
    pub mod issue_1605;
    pub mod jpeg_cmyk;
    pub mod oom_decompression_bomb;
    pub mod oom_zip_size_deception;
    pub mod slice_tests;
    pub mod suite1_debug;
    pub mod suite2_fixes;
}
