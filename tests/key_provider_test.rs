//! Tests for custom key provider functionality
//!
//! These tests validate that custom key providers can be used for decryption

use lib3mf::{
    AccessRight, CEKParams, KEKParams, KeyProvider, ParserConfig, Result, SecureContentInfo,
};
use std::sync::Arc;

/// Mock key provider that uses the same test keys but implements the trait
struct TestKeyProvider;

impl KeyProvider for TestKeyProvider {
    fn decrypt(
        &self,
        cipher_file_data: &[u8],
        cek_params: &CEKParams,
        access_right: &AccessRight,
        secure_content: &SecureContentInfo,
    ) -> Result<Vec<u8>> {
        // Delegate to the test key decryption
        lib3mf::decryption::decrypt_with_test_key(
            cipher_file_data,
            cek_params,
            access_right,
            secure_content,
        )
    }

    fn encrypt(
        &self,
        _plaintext: &[u8],
        _consumer_id: &str,
        _compression: bool,
    ) -> Result<(Vec<u8>, CEKParams, KEKParams, String)> {
        // Not implemented for this test
        unimplemented!("Encryption not implemented in test provider")
    }
}

/// Test that a custom key provider can be configured and used
#[test]
fn test_custom_key_provider_configuration() {
    let provider: Arc<dyn KeyProvider> = Arc::new(TestKeyProvider);
    let config = ParserConfig::new().with_key_provider(provider.clone());

    assert!(config.key_provider().is_some());
}

/// Test that SecureContent files can be decrypted with custom provider
/// This requires Suite 8 test files to be present
#[test]
#[ignore] // Only run when Suite 8 test files are available
fn test_custom_provider_decryption() {
    use std::fs::File;

    let test_file = "test_suites/suite8_secure/positive_test_cases/P_EPX_2102_01_materialExt.3mf";

    // Skip if file doesn't exist
    if !std::path::Path::new(test_file).exists() {
        println!("Skipping test - test file not available: {}", test_file);
        return;
    }

    // Create config with custom key provider
    let provider: Arc<dyn KeyProvider> = Arc::new(TestKeyProvider);
    let config = ParserConfig::with_all_extensions().with_key_provider(provider);

    // Parse the file - this should use the custom provider
    let file = File::open(test_file).expect("Failed to open test file");
    let result = lib3mf::parser::parse_3mf_with_config(file, config);

    assert!(
        result.is_ok(),
        "Failed to parse with custom provider: {:?}",
        result.err()
    );

    let model = result.unwrap();

    // Verify that secure content was processed
    assert!(
        model.secure_content.is_some(),
        "SecureContent info should be present"
    );
}

/// Test that parsing still works with test keys when no custom provider is set
#[test]
#[ignore] // Only run when Suite 8 test files are available
fn test_fallback_to_test_keys() {
    use std::fs::File;

    let test_file = "test_suites/suite8_secure/positive_test_cases/P_EPX_2102_01_materialExt.3mf";

    // Skip if file doesn't exist
    if !std::path::Path::new(test_file).exists() {
        println!("Skipping test - test file not available: {}", test_file);
        return;
    }

    // Parse with default config (no custom provider)
    let config = ParserConfig::with_all_extensions();
    let file = File::open(test_file).expect("Failed to open test file");
    let result = lib3mf::parser::parse_3mf_with_config(file, config);

    assert!(
        result.is_ok(),
        "Failed to parse with test keys: {:?}",
        result.err()
    );

    let model = result.unwrap();

    // Verify that secure content was processed
    assert!(
        model.secure_content.is_some(),
        "SecureContent info should be present"
    );
}

/// Test that custom provider is called when configured
/// This test uses a provider that always fails to verify it's actually being used
#[test]
#[ignore] // Only run when Suite 8 test files are available
fn test_custom_provider_is_called() {
    use std::fs::File;

    struct FailingProvider;

    impl KeyProvider for FailingProvider {
        fn decrypt(
            &self,
            _cipher_file_data: &[u8],
            _cek_params: &CEKParams,
            _access_right: &AccessRight,
            _secure_content: &SecureContentInfo,
        ) -> Result<Vec<u8>> {
            Err(lib3mf::Error::InvalidSecureContent(
                "Custom provider was called".to_string(),
            ))
        }

        fn encrypt(
            &self,
            _plaintext: &[u8],
            _consumer_id: &str,
            _compression: bool,
        ) -> Result<(Vec<u8>, CEKParams, KEKParams, String)> {
            unimplemented!()
        }
    }

    let test_file = "test_suites/suite8_secure/positive_test_cases/P_EPX_2102_01_materialExt.3mf";

    // Skip if file doesn't exist
    if !std::path::Path::new(test_file).exists() {
        println!("Skipping test - test file not available: {}", test_file);
        return;
    }

    let provider: Arc<dyn KeyProvider> = Arc::new(FailingProvider);
    let config = ParserConfig::with_all_extensions().with_key_provider(provider);

    let file = File::open(test_file).expect("Failed to open test file");
    let result = lib3mf::parser::parse_3mf_with_config(file, config);

    // Should fail because custom provider fails
    assert!(
        result.is_err(),
        "Should have failed with custom provider error"
    );

    let error = result.unwrap_err();
    let error_msg = format!("{}", error);

    // Verify it was our custom provider that was called
    assert!(
        error_msg.contains("Custom provider was called"),
        "Error should be from custom provider, got: {}",
        error_msg
    );
}
