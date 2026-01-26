//! Key provider trait for SecureContent encryption/decryption
//!
//! This module defines the interface for providing customer keys for
//! decryption during parsing and encryption during writing.

use crate::error::Result;
use crate::model::{AccessRight, CEKParams, KEKParams, SecureContentInfo};

/// Trait for providing encryption/decryption keys for SecureContent
///
/// Implement this trait to provide your own keys for decrypting 3MF files
/// during parsing or encrypting during writing.
///
/// # Example
///
/// ```no_run
/// use lib3mf::{KeyProvider, Result};
/// use lib3mf::{SecureContentInfo, AccessRight, CEKParams, KEKParams};
///
/// struct MyKeyProvider {
///     private_key: Vec<u8>,
/// }
///
/// impl KeyProvider for MyKeyProvider {
///     fn decrypt(
///         &self,
///         cipher_file_data: &[u8],
///         cek_params: &CEKParams,
///         access_right: &AccessRight,
///         secure_content: &SecureContentInfo,
///     ) -> Result<Vec<u8>> {
///         // Implement your decryption logic here
///         // 1. Parse cipher file format
///         // 2. Unwrap CEK using your private key
///         // 3. Decrypt content using AES-GCM
///         // 4. Decompress if needed
///         unimplemented!()
///     }
///
///     fn encrypt(
///         &self,
///         plaintext: &[u8],
///         consumer_id: &str,
///         compression: bool,
///     ) -> Result<(Vec<u8>, CEKParams, KEKParams, String)> {
///         // Implement your encryption logic here
///         // 1. Optionally compress the plaintext
///         // 2. Generate a random CEK
///         // 3. Encrypt using AES-256-GCM
///         // 4. Wrap CEK with consumer's public key
///         // 5. Return encrypted data and parameters
///         unimplemented!()
///     }
/// }
/// ```
pub trait KeyProvider: Send + Sync {
    /// Decrypt encrypted content
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
    fn decrypt(
        &self,
        cipher_file_data: &[u8],
        cek_params: &CEKParams,
        access_right: &AccessRight,
        secure_content: &SecureContentInfo,
    ) -> Result<Vec<u8>>;

    /// Encrypt plaintext content
    ///
    /// # Arguments
    ///
    /// * `plaintext` - The plaintext data to encrypt
    /// * `consumer_id` - The consumer ID for whom to encrypt
    /// * `compression` - Whether to compress the plaintext before encryption
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - Encrypted data (cipher file format)
    /// - CEK parameters (encryption algorithm, IV, tag, AAD, compression)
    /// - KEK parameters (wrapping algorithm, MGF, digest)
    /// - Base64-encoded wrapped CEK (cipher value)
    fn encrypt(
        &self,
        plaintext: &[u8],
        consumer_id: &str,
        compression: bool,
    ) -> Result<(Vec<u8>, CEKParams, KEKParams, String)>;
}
