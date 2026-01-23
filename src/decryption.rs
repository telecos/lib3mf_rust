//! Decryption support for SecureContent extension
//!
//! This module provides test-only decryption using hardcoded test keys
//! for Suite 8 conformance validation.
//!
//! **Security Note**: This is for test purposes only. The private key is
//! embedded and should NOT be used in production environments.

use crate::error::{Error, Result};
use crate::model::{AccessRight, CEKParams, SecureContentInfo};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use flate2::read::DeflateDecoder;
use rsa::{pkcs1::DecodeRsaPrivateKey, Pkcs1v15Encrypt, RsaPrivateKey};
use std::io::Read;

/// Test consumer ID from Suite 8 test specification
pub const TEST_CONSUMER_ID: &str = "test3mf01";

/// Test key ID from Suite 8 test specification  
#[allow(dead_code)]
const TEST_KEY_ID: &str = "test3mfkek01";

/// RSA private key for test decryption (from Suite 8 Appendix D)
/// **WARNING**: This is a test key only. Never use in production.
const TEST_PRIVATE_KEY_PEM: &str = r#"-----BEGIN RSA PRIVATE KEY-----
MIIEogIBAAKCAQEAubdl5ZV99+wA/1vUZeeM8KQaSQ7dV0W9Vw7PNlXszRdoavwW
4D/e70cajoeJ3TJfarA9zdE3pBVzXsja5VM1axzrPCQn77VvFFTLsMa1lBz3UZck
KK7dAVuoREQCH6042/4UGhvKmVoGq9jt0xMV0CBIgWNgfviE6tuiiezGkoPEJXBb
hg0WXNe6JSxYI3fRkjjPh8fHSla5Jil6L+XrT/n6ehShlLN960tn8suxu1AaXuRv
dimZNxVgK7VQKcYQbfKDfpzEi5Jfd2UKxmuKn/87nrreFYaZCeTjFbadP7FkB8wd
SGGCctsdRfkl/pCBkdLrGsv7Is6jRlW7M0ZoBQIDAQABAoIBAAHH8Pm5K8qXYFES
m+BYTqE2KaxesJ+4Iv81PKZ8P3eeDFnOThfbdPNdfrM0OI2/AGxBAW66XWq86+zS
R0sgt6ft0JG0lQ928XhD8eohlbc0aejF5spfFu5+5we0kUKlgiCV+LJhZtl+pAa8
31cBXVmwHZHkFpZRItEvxwjElQjtp1co+kmCudew4ffpPBPUw7TSuOWuQVjo+d5M
h0xaZzMjjxSornv4LRAm1D4NoCabuCx7jRY2gOgl39nwCWi922vssbEjAUg4+862
Jqe/ted4xIGCk8DP+bwxj3WboLjkM4yp/5AcLGkaovhjupLXru4wDqsWr8wbgwV1
BmzUydcCgYEAvDaO6t58uk0kWVEmlGEueln4AfIUjgjo51qbbb23WsPQTZtlp7N0
/qNNKsWktr0ZPRIdIFcxTprd+gy5LGozQGz41J2lT+9DGsmo3dB2e47r+uKDnNwm
Iegp+4LYFiXGLGDNonn7ESSec4Xj8z8YosVHskr64ptPCOzYzmDCkW8CgYEA/Jqj
wLKOYgBVoUTEZQfMe295VKaKrxtqprYCTHF9J9lysxg2WfIVJByoVnpkmy2EI+Mw
+ubtPrx71Cx413dem/S1aOOIsqJPqdFkc+AERV6ZeT1NWLCgzWoczW/N5ZdneUkW
a0i0B0olAiC9b5zx9HB+p1bm7xEL3zL6OUDPu8sCgYBflkXXOs+Vvn/rbK9vRDva
n765Hj0aNaQze2zcuzFXw4MTJwzlstqESGN0iZQxyq/6uCxatG2yQiziRXv19qm4
2p81PCstAZLPFAPTQ4ApGFj4vfmhvJ0RM1u/BKDB/sU63J8TGWhNOI/Qk/tFGpJk
eFUFU9c/JylomwExLyshuQKBgFd2o+SA7tP4Ea45RVdGEANdYcFxuOtQrujydHFL
im5V2GUyqP8T10YdthvbXSJt7CcQ71CwzMzALpAUpfLVHikZ3gZnYlmX4cWG/yUw
F8p9Kt7T3wgqgEMfzsFDSSOJ/QX9zIlxLwSnI5FNDMqsqQpeOTxv1p5IZLfvyrww
OL1pAoGAM/ZoL7qWenZAzD1Gdzo9HlrxlxBJPnr+ZdYqrJZdo/TwARY8LZu07Vsu
aY1ZAqLlkBARRtypmGj04PGbWWRZ3Pn/M5/FgjGa5M9hVnvLJSBklE7tfKLB4KL5
eMADI7JuelOqfKBxXrp8IlzVlU8Mk0VQRw6hjq1zNKLJtD4EFq4=
-----END RSA PRIVATE KEY-----"#;

/// Decrypt encrypted content using test keys
///
/// This function attempts to decrypt the provided ciphertext using the
/// test keys from Suite 8 Appendix D. It will only work if the content
/// was encrypted with the corresponding test public key.
///
/// # Arguments
///
/// * `ciphertext` - The encrypted data
/// * `cek_params` - Content encryption parameters (algorithm, IV, tag, AAD, compression)
/// * `access_right` - Access right containing wrapped CEK
/// * `secure_content` - Secure content info containing consumer definitions
///
/// # Returns
///
/// Decrypted plaintext data, or error if decryption fails
pub fn decrypt_with_test_key(
    ciphertext: &[u8],
    cek_params: &CEKParams,
    access_right: &AccessRight,
    secure_content: &SecureContentInfo,
) -> Result<Vec<u8>> {
    // Verify this is the test consumer
    if access_right.consumer_index >= secure_content.consumers.len() {
        return Err(Error::InvalidSecureContent(
            "Invalid consumer index".to_string(),
        ));
    }

    let consumer = &secure_content.consumers[access_right.consumer_index];
    if consumer.consumer_id != TEST_CONSUMER_ID {
        return Err(Error::InvalidSecureContent(format!(
            "Decryption only supported for test consumer '{}', got '{}'",
            TEST_CONSUMER_ID, consumer.consumer_id
        )));
    }

    // Unwrap the CEK using RSA-OAEP
    let cek = unwrap_cek_with_test_key(&access_right.cipher_value)?;

    // Decrypt the content using AES-GCM
    let plaintext = decrypt_aes_gcm(ciphertext, &cek, cek_params)?;

    // Decompress if needed
    if cek_params.compression == "deflate" {
        decompress_deflate(&plaintext)
    } else {
        Ok(plaintext)
    }
}

/// Unwrap (decrypt) the CEK using the test RSA private key
fn unwrap_cek_with_test_key(wrapped_cek_base64: &str) -> Result<Vec<u8>> {
    // Decode base64-encoded wrapped CEK
    let wrapped_cek = BASE64
        .decode(wrapped_cek_base64)
        .map_err(|e| Error::InvalidSecureContent(format!("Invalid base64 CEK: {}", e)))?;

    // Parse the test private key
    let private_key = parse_test_private_key()?;

    // Decrypt using RSA-OAEP (PKCS#1 v1.5 padding)
    let cek = private_key
        .decrypt(Pkcs1v15Encrypt, &wrapped_cek)
        .map_err(|e| Error::InvalidSecureContent(format!("RSA decryption failed: {}", e)))?;

    Ok(cek)
}

/// Parse the test RSA private key from PEM format
fn parse_test_private_key() -> Result<RsaPrivateKey> {
    // Remove PEM headers and decode
    let pem_data = TEST_PRIVATE_KEY_PEM
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect::<String>();

    let der = BASE64
        .decode(&pem_data)
        .map_err(|e| Error::InvalidSecureContent(format!("Invalid PEM key: {}", e)))?;

    RsaPrivateKey::from_pkcs1_der(&der)
        .map_err(|e| Error::InvalidSecureContent(format!("Invalid RSA key: {}", e)))
}

/// Decrypt content using AES-256-GCM
fn decrypt_aes_gcm(ciphertext: &[u8], cek: &[u8], params: &CEKParams) -> Result<Vec<u8>> {
    // Verify algorithm
    if !params.encryption_algorithm.contains("aes256-gcm") {
        return Err(Error::InvalidSecureContent(format!(
            "Unsupported encryption algorithm: {}",
            params.encryption_algorithm
        )));
    }

    // Parse IV, tag, and AAD
    let iv = params
        .iv
        .as_ref()
        .ok_or_else(|| Error::InvalidSecureContent("Missing IV".to_string()))?;
    let tag = params
        .tag
        .as_ref()
        .ok_or_else(|| Error::InvalidSecureContent("Missing tag".to_string()))?;

    let iv_bytes = BASE64
        .decode(iv)
        .map_err(|e| Error::InvalidSecureContent(format!("Invalid IV: {}", e)))?;
    let tag_bytes = BASE64
        .decode(tag)
        .map_err(|e| Error::InvalidSecureContent(format!("Invalid tag: {}", e)))?;

    // Combine ciphertext and tag for AES-GCM
    let mut combined = ciphertext.to_vec();
    combined.extend_from_slice(&tag_bytes);

    // Create cipher
    let cipher = Aes256Gcm::new_from_slice(cek)
        .map_err(|e| Error::InvalidSecureContent(format!("Invalid key length: {}", e)))?;

    let nonce = Nonce::from_slice(&iv_bytes);

    // Decrypt
    let plaintext = cipher
        .decrypt(nonce, combined.as_ref())
        .map_err(|e| Error::InvalidSecureContent(format!("AES-GCM decryption failed: {}", e)))?;

    Ok(plaintext)
}

/// Decompress data using DEFLATE
fn decompress_deflate(compressed: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = DeflateDecoder::new(compressed);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| Error::InvalidSecureContent(format!("Decompression failed: {}", e)))?;
    Ok(decompressed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_private_key() {
        let key = parse_test_private_key();
        assert!(key.is_ok(), "Failed to parse test private key");
    }

    #[test]
    fn test_base64_decode() {
        let data = "SGVsbG8gV29ybGQ=";
        let decoded = BASE64.decode(data).unwrap();
        assert_eq!(decoded, b"Hello World");
    }
}
