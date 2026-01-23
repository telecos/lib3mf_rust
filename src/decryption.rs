//! Decryption support for SecureContent extension
//!
//! This module provides test-only decryption using hardcoded test keys
//! for Suite 8 conformance validation.
//!
//! **Security Note**: This is for test purposes only. The private key is
//! embedded and should NOT be used in production environments.

use crate::error::{Error, Result};
use crate::model::{AccessRight, CEKParams, KEKParams, SecureContentInfo};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use flate2::read::DeflateDecoder;
use rsa::{pkcs1::DecodeRsaPrivateKey, RsaPrivateKey};
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
/// The ciphertext must be in the 3MF cipher file format as described in
/// Appendix D of the SecureContent specification (magic number '%3McF').
///
/// # Arguments
///
/// * `cipher_file_data` - The complete encrypted file data (including cipher file header)
/// * `cek_params` - Content encryption parameters (algorithm, IV, tag, AAD, compression)
/// * `access_right` - Access right containing wrapped CEK and KEK parameters
/// * `secure_content` - Secure content info containing consumer definitions
///
/// # Returns
///
/// Decrypted plaintext data, or error if decryption fails
pub fn decrypt_with_test_key(
    cipher_file_data: &[u8],
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

    // Parse the cipher file format to extract the actual ciphertext
    let ciphertext = parse_cipher_file_format(cipher_file_data)?;

    // Unwrap the CEK using RSA-OAEP (with appropriate digest method)
    let cek = unwrap_cek_with_test_key(&access_right.cipher_value, &access_right.kek_params)?;

    // Decrypt the content using AES-GCM
    let plaintext = decrypt_aes_gcm(&ciphertext, &cek, cek_params)?;

    // Decompress if needed
    if cek_params.compression == "deflate" {
        decompress_deflate(&plaintext)
    } else {
        Ok(plaintext)
    }
}

/// Parse the 3MF cipher file format (Appendix D of SecureContent spec)
///
/// The cipher file format:
/// - Octets 0-4: '%3McF' magic number
/// - Octet 5: Version major (0x00)
/// - Octet 6: Version minor (0x00)
/// - Octet 7: Unused (0x00)
/// - Octets 8-11: Header length (little-endian u32)
/// - Octets 12-(Header length-1): Reserved header data
/// - Octets (Header length)-EOF: Crypto content (actual ciphertext)
fn parse_cipher_file_format(data: &[u8]) -> Result<Vec<u8>> {
    // Check minimum size for header
    if data.len() < 12 {
        return Err(Error::InvalidSecureContent(
            "Cipher file too small (minimum 12 bytes for header)".to_string(),
        ));
    }

    // Check magic number
    if &data[0..5] != b"%3McF" {
        return Err(Error::InvalidSecureContent(format!(
            "Invalid cipher file magic number. Expected '%3McF', got '{}'",
            String::from_utf8_lossy(&data[0..5])
        )));
    }

    // Check version (should be 0.0)
    if data[5] != 0x00 || data[6] != 0x00 {
        return Err(Error::InvalidSecureContent(format!(
            "Unsupported cipher file version {}.{}",
            data[5], data[6]
        )));
    }

    // Parse header length (little-endian u32)
    let header_len = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;

    // Validate header length
    if header_len < 12 {
        return Err(Error::InvalidSecureContent(format!(
            "Invalid header length {} (minimum 12)",
            header_len
        )));
    }

    if header_len > data.len() {
        return Err(Error::InvalidSecureContent(format!(
            "Header length {} exceeds file size {}",
            header_len,
            data.len()
        )));
    }

    // Extract the ciphertext (everything after the header)
    Ok(data[header_len..].to_vec())
}

/// Unwrap (decrypt) the CEK using the test RSA private key
fn unwrap_cek_with_test_key(wrapped_cek_base64: &str, kek_params: &KEKParams) -> Result<Vec<u8>> {
    use rsa::Oaep;
    use sha1::Sha1;
    use sha2::Sha256;

    // Decode base64-encoded wrapped CEK
    let wrapped_cek = BASE64
        .decode(wrapped_cek_base64)
        .map_err(|e| Error::InvalidSecureContent(format!("Invalid base64 CEK: {}", e)))?;

    // Parse the test private key
    let private_key = parse_test_private_key()?;

    // Determine which digest method and MGF algorithm to use
    // Default is SHA-1 if not specified (per PKCS#1 v2.0)
    let digest_is_sha256 = kek_params
        .digest_method
        .as_ref()
        .map(|dm| dm.contains("sha256"))
        .unwrap_or(false);

    let mgf_is_sha256 = kek_params
        .mgf_algorithm
        .as_ref()
        .map(|mgf| mgf.contains("sha256"))
        .unwrap_or(false);

    // Decrypt using RSA-OAEP with appropriate digest method and MGF
    // The OAEP padding has two hash functions:
    // 1. Digest method for the main OAEP hash
    // 2. MGF1 hash for the mask generation function
    // These can be different, so we need to handle all combinations
    let cek = match (digest_is_sha256, mgf_is_sha256) {
        (true, true) => {
            // SHA-256 for both digest and MGF1
            let padding = Oaep::new::<Sha256>();
            private_key.decrypt(padding, &wrapped_cek).map_err(|e| {
                Error::InvalidSecureContent(format!(
                    "RSA-OAEP SHA256/SHA256 decryption failed: {}",
                    e
                ))
            })?
        }
        (true, false) => {
            // SHA-256 for digest, SHA-1 for MGF1
            let padding = Oaep::new_with_mgf_hash::<Sha256, Sha1>();
            private_key.decrypt(padding, &wrapped_cek).map_err(|e| {
                Error::InvalidSecureContent(format!(
                    "RSA-OAEP SHA256/SHA1 decryption failed: {}",
                    e
                ))
            })?
        }
        (false, true) => {
            // SHA-1 for digest, SHA-256 for MGF1
            let padding = Oaep::new_with_mgf_hash::<Sha1, Sha256>();
            private_key.decrypt(padding, &wrapped_cek).map_err(|e| {
                Error::InvalidSecureContent(format!(
                    "RSA-OAEP SHA1/SHA256 decryption failed: {}",
                    e
                ))
            })?
        }
        (false, false) => {
            // SHA-1 for both digest and MGF1 (default)
            let padding = Oaep::new::<Sha1>();
            private_key.decrypt(padding, &wrapped_cek).map_err(|e| {
                Error::InvalidSecureContent(format!("RSA-OAEP SHA1/SHA1 decryption failed: {}", e))
            })?
        }
    };

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
    use aes_gcm::aead::Payload;

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

    // Parse AAD if present
    let aad_bytes = if let Some(ref aad) = params.aad {
        if !aad.is_empty() {
            BASE64
                .decode(aad)
                .map_err(|e| Error::InvalidSecureContent(format!("Invalid AAD: {}", e)))?
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Combine ciphertext and tag for AES-GCM
    let mut combined = ciphertext.to_vec();
    combined.extend_from_slice(&tag_bytes);

    // Create cipher
    let cipher = Aes256Gcm::new_from_slice(cek)
        .map_err(|e| Error::InvalidSecureContent(format!("Invalid key length: {}", e)))?;

    let nonce = Nonce::from_slice(&iv_bytes);

    // Create payload with AAD
    let payload = Payload {
        msg: &combined,
        aad: &aad_bytes,
    };

    // Decrypt
    let plaintext = cipher
        .decrypt(nonce, payload)
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
