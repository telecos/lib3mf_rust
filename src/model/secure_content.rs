//! Secure Content extension types

/// Secure content metadata (read-only awareness)
///
/// This structure provides awareness of secure content elements in a 3MF package
/// without implementing actual cryptographic operations. It parses and exposes the
/// complete keystore structure to enable applications to:
/// - Identify which files are encrypted
/// - Access consumer information (IDs, key IDs, public keys)
/// - Retrieve encryption metadata (algorithms, parameters)
/// - Implement their own decryption using external cryptographic libraries
///
/// **Implementation Status**: The keystore is fully parsed to extract all structural
/// information. This allows applications to access encryption metadata and implement
/// their own decryption logic using external libraries.
///
/// **Security Warning**: This does NOT decrypt content or verify signatures.
/// Applications must implement their own cryptographic operations using established
/// libraries (e.g., ring, RustCrypto, OpenSSL).
#[derive(Debug, Clone, Default)]
pub struct SecureContentInfo {
    /// UUID of the keystore (if present)
    pub keystore_uuid: Option<String>,
    /// Paths to encrypted files in the package (maintained for backward compatibility)
    pub encrypted_files: Vec<String>,
    /// Consumer definitions (authorized parties that can decrypt content)
    pub consumers: Vec<Consumer>,
    /// Resource data groups (sets of encrypted resources with shared CEK)
    pub resource_data_groups: Vec<ResourceDataGroup>,
    /// Consumer IDs (for internal validation)
    pub(crate) consumer_ids: Vec<String>,
    /// Number of consumers (for internal consumer index validation)
    pub(crate) consumer_count: usize,
    /// Encryption algorithms used (for internal validation)
    pub(crate) wrapping_algorithms: Vec<String>,
}

/// Consumer information from SecureContent keystore
///
/// Represents an authorized consumer (party) that can decrypt protected content.
/// Each consumer has a unique identifier and optional key information.
///
/// **Note**: This structure provides metadata only. Applications must implement
/// their own key management and decryption logic.
#[derive(Debug, Clone, PartialEq)]
pub struct Consumer {
    /// Unique consumer identifier (alphanumeric, human-readable)
    pub consumer_id: String,
    /// Optional key identifier for identifying the KEK
    pub key_id: Option<String>,
    /// Optional public key value in PEM format (RSA-2048 public key)
    /// This follows RFC 7468 Section 13: Textual Encoding of Subject Public Key Info
    pub key_value: Option<String>,
}

/// Resource data group from SecureContent keystore
///
/// Groups encrypted resources that share the same Content Encryption Key (CEK).
/// Each group may have multiple access rights (one per consumer) and multiple
/// encrypted resources.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceDataGroup {
    /// UUID identifying the Content Encryption Key
    pub key_uuid: String,
    /// Access rights (one per consumer authorized to decrypt this group)
    pub access_rights: Vec<AccessRight>,
    /// Encrypted resources in this group
    pub resource_data: Vec<ResourceData>,
}

/// Access right from SecureContent keystore
///
/// Links a consumer to encrypted content by providing the wrapped CEK.
/// Each access right contains the CEK encrypted with a specific consumer's KEK.
#[derive(Debug, Clone, PartialEq)]
pub struct AccessRight {
    /// Zero-based index to the consumer element
    pub consumer_index: usize,
    /// Key encryption parameters (wrapping algorithm, MGF, digest)
    pub kek_params: KEKParams,
    /// Base64-encoded wrapped Content Encryption Key
    pub cipher_value: String,
}

/// Key Encryption Key parameters from SecureContent keystore
///
/// Specifies the algorithm and parameters used to wrap (encrypt) the CEK.
#[derive(Debug, Clone, PartialEq)]
pub struct KEKParams {
    /// Wrapping algorithm URI (e.g., rsa-oaep-mgf1p)
    pub wrapping_algorithm: String,
    /// Optional mask generation function URI
    pub mgf_algorithm: Option<String>,
    /// Optional message digest method URI
    pub digest_method: Option<String>,
}

/// Resource data from SecureContent keystore
///
/// Describes a single encrypted resource file in the package, including its
/// path and the parameters needed for decryption.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceData {
    /// Path to the encrypted file in the OPC package
    pub path: String,
    /// Content Encryption Key parameters
    pub cek_params: CEKParams,
}

/// Content Encryption Key parameters from SecureContent keystore
///
/// Specifies the symmetric encryption algorithm and parameters used to
/// encrypt the resource data.
#[derive(Debug, Clone, PartialEq)]
pub struct CEKParams {
    /// Encryption algorithm URI (e.g., AES-256-GCM)
    pub encryption_algorithm: String,
    /// Optional compression algorithm ("none" or "deflate")
    pub compression: String,
    /// Initialization Vector (base64-encoded, typically 96-bit for AES-GCM)
    pub iv: Option<String>,
    /// Authentication Tag (base64-encoded, typically 128-bit for AES-GCM)
    pub tag: Option<String>,
    /// Additional Authenticated Data (base64-encoded, optional)
    pub aad: Option<String>,
}
