//! lib3mf_rust - Rust implementation for 3MF (3D Manufacturing Format)
//!
//! This library provides functionality for working with 3MF files, which are
//! ZIP archives containing XML data, images (JPEG/PNG), and JSON metadata.

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_dependencies_available() {
        // Test that all required dependencies are available

        // ZIP support
        let _ = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));

        // XML parsing
        let xml = r#"<root><item>test</item></root>"#;
        let _: Result<quick_xml::events::Event, _> = quick_xml::Reader::from_str(xml).read_event();

        // JSON parsing
        use serde::{Deserialize, Serialize};
        #[derive(Serialize, Deserialize)]
        struct Test {
            value: String,
        }
        let test = Test {
            value: "test".to_string(),
        };
        let _json = serde_json::to_string(&test).unwrap();

        // Image support - verify JPEG and PNG codecs are available
        use image::ImageFormat;
        assert!(ImageFormat::Jpeg.can_read());
        assert!(ImageFormat::Jpeg.can_write());
        assert!(ImageFormat::Png.can_read());
        assert!(ImageFormat::Png.can_write());
    }
}
