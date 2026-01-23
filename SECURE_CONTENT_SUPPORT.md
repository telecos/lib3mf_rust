# Secure Content Extension Support

## Overview

This document describes the implementation and security considerations for the 3MF Secure Content Extension in lib3mf_rust.

## Specification Reference

- **Extension Name**: Secure Content
- **Namespace**: `http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07`
- **Version**: 1.0.2
- **Specification**: [3MF Secure Content Specification](https://github.com/3MFConsortium/spec_securecontent/blob/master/3MF%20Secure%20Content.md)

## Current Implementation Status

### What is Supported ✅

1. **Extension Recognition**: The SecureContent extension is recognized and validated
   - The extension enum includes `Extension::SecureContent`
   - Namespace URI is properly mapped
   - Extension can be validated in `requiredextensions` attribute

2. **Validation Framework**: Test infrastructure exists for suite8_secure tests
   - Conformance tests defined in `tests/conformance_tests.rs`
   - Tests pass when test files are available

### What is NOW Supported ✅

1. **Extension Recognition**: The SecureContent extension is recognized and validated
   - The extension enum includes `Extension::SecureContent`
   - Namespace URI is properly mapped
   - Extension can be validated in `requiredextensions` attribute

2. **Complete Keystore Parsing**: Full structural parsing of keystore.xml
   - Extracts keystore UUID
   - Parses consumer definitions (ID, key ID, PEM public keys)
   - Parses resource data groups with encryption metadata
   - Parses access rights linking consumers to encrypted resources
   - Extracts encryption parameters (algorithms, IV, tags, AAD, compression)

3. **Validation Framework**: Comprehensive validation per 3MF spec
   - EPX-2601: Consumer index validation
   - EPX-2602: Consumer existence validation
   - EPX-2603: Algorithm validation (wrapping, MGF, digest)
   - EPX-2604: Consumer ID uniqueness validation
   - EPX-2605: Encrypted file path validation
   - EPX-2607: File existence validation

### What is NOT Supported ❌

1. **No Cryptographic Operations**
   - No signature verification
   - No decryption of encrypted content
   - No certificate/key management
   - No cryptographic primitives implementation
   - No actual key unwrapping or content decryption

2. **No Content Access**
   - Encrypted OPC parts are not decrypted
   - Cannot read encrypted model files
   - Cannot access encrypted resources
   - Applications must implement their own decryption logic

## Scope of Support: Metadata Extraction for External Decryption

This implementation provides **complete metadata extraction** to enable applications
to implement their own decryption using external cryptographic libraries:

### Design Decision: Parse Structure, Delegate Cryptography

**Rationale:**
1. **Complexity**: Full cryptographic support requires:
   - RSA-2048 OAEP key wrapping
   - AES-256 GCM symmetric encryption
   - Public key infrastructure (PKI)
   - Certificate validation
   - Key management ecosystem

2. **Security Concerns**: Implementing cryptography incorrectly is dangerous
   - Risk of timing attacks
   - Risk of side-channel attacks
   - Complex key management
   - Requires security audit and expertise

3. **Use Case**: Most Rust applications need to:
   - Parse and validate 3MF structure
   - Extract metadata without decryption
   - Identify which parts are encrypted
   - Pass encrypted content to specialized security libraries

4. **Ecosystem**: Production security applications typically:
   - Use dedicated HSMs (Hardware Security Modules)
   - Integrate with enterprise PKI systems
   - Use vendor-specific key management
   - Require regulatory compliance (ITAR, GDPR)

### What This Implementation Provides

```rust
use lib3mf::Model;

// Parse a 3MF file with secure content
let model = Model::from_reader(file)?;

// 1. Check if secure content is used
if model.required_extensions.contains(&Extension::SecureContent) {
    println!("This file uses secure content");
}

// 2. Access keystore metadata
if let Some(ref sc) = model.secure_content {
    // Access keystore UUID
    if let Some(ref uuid) = sc.keystore_uuid {
        println!("Keystore UUID: {}", uuid);
    }
    
    // Access consumer information
    for consumer in &sc.consumers {
        println!("Consumer ID: {}", consumer.consumer_id);
        if let Some(ref key_id) = consumer.key_id {
            println!("  Key ID: {}", key_id);
        }
        if let Some(ref key_value) = consumer.key_value {
            println!("  Public Key (PEM): {}", key_value);
        }
    }
    
    // Access encrypted resources and their encryption metadata
    for group in &sc.resource_data_groups {
        println!("Resource Group: {}", group.key_uuid);
        
        // Access rights per consumer
        for access_right in &group.access_rights {
            println!("  Consumer {}: Algorithm {}", 
                access_right.consumer_index,
                access_right.kek_params.wrapping_algorithm);
            // access_right.cipher_value contains the wrapped CEK
        }
        
        // Encrypted files in this group
        for resource in &group.resource_data {
            println!("  Encrypted file: {}", resource.path);
            println!("    Algorithm: {}", resource.cek_params.encryption_algorithm);
            println!("    Compression: {}", resource.cek_params.compression);
            if let Some(ref iv) = resource.cek_params.iv {
                println!("    IV: {}", iv);
            }
            if let Some(ref tag) = resource.cek_params.tag {
                println!("    Tag: {}", tag);
            }
        }
    }
}

// 3. Implement your own decryption using external crypto libraries
// Example: Use the `ring` or `RustCrypto` crates for actual cryptographic operations
```

## Data Structures

### Complete Secure Content Support

The library provides comprehensive data structures that mirror the 3MF SecureContent
extension specification, allowing applications to access all keystore metadata:

```rust
/// Main secure content information container
pub struct SecureContentInfo {
    /// UUID of the keystore
    pub keystore_uuid: Option<String>,
    /// List of encrypted file paths (for quick reference)
    pub encrypted_files: Vec<String>,
    /// Consumer definitions (authorized parties)
    pub consumers: Vec<Consumer>,
    /// Resource data groups (encrypted resources with shared CEK)
    pub resource_data_groups: Vec<ResourceDataGroup>,
}

/// Consumer (authorized party that can decrypt content)
pub struct Consumer {
    /// Unique consumer identifier (alphanumeric, human-readable)
    pub consumer_id: String,
    /// Optional key identifier for the KEK
    pub key_id: Option<String>,
    /// Optional public key in PEM format (RFC 7468)
    pub key_value: Option<String>,
}

/// Resource data group (encrypted resources sharing the same CEK)
pub struct ResourceDataGroup {
    /// UUID identifying the Content Encryption Key
    pub key_uuid: String,
    /// Access rights (one per authorized consumer)
    pub access_rights: Vec<AccessRight>,
    /// Encrypted resources in this group
    pub resource_data: Vec<ResourceData>,
}

/// Access right (links consumer to wrapped CEK)
pub struct AccessRight {
    /// Zero-based index to the consumer
    pub consumer_index: usize,
    /// Key encryption parameters
    pub kek_params: KEKParams,
    /// Base64-encoded wrapped CEK
    pub cipher_value: String,
}

/// Key Encryption Key parameters
pub struct KEKParams {
    /// Wrapping algorithm URI
    pub wrapping_algorithm: String,
    /// Optional mask generation function URI
    pub mgf_algorithm: Option<String>,
    /// Optional message digest method URI
    pub digest_method: Option<String>,
}

/// Encrypted resource metadata
pub struct ResourceData {
    /// Path to encrypted file in package
    pub path: String,
    /// Content encryption parameters
    pub cek_params: CEKParams,
}

/// Content Encryption Key parameters
pub struct CEKParams {
    /// Encryption algorithm URI (e.g., AES-256-GCM)
    pub encryption_algorithm: String,
    /// Compression algorithm ("none" or "deflate")
    pub compression: String,
    /// Initialization Vector (base64, typically 96-bit for AES-GCM)
    pub iv: Option<String>,
    /// Authentication Tag (base64, typically 128-bit for AES-GCM)
    pub tag: Option<String>,
    /// Additional Authenticated Data (base64, optional)
    pub aad: Option<String>,
}
```

These structures provide **all the metadata** needed for an application to implement
decryption using external cryptographic libraries.

## Implementing Decryption (External Libraries Required)

To actually decrypt content, applications must use external cryptographic libraries.
Here's a conceptual workflow:

### Step 1: Extract Keystore Metadata

```rust
use lib3mf::Model;

let model = Model::from_reader(file)?;
let sc = model.secure_content.as_ref().expect("No secure content");

// Identify your consumer
let my_consumer_id = "MyApp#Device#12345";
let consumer_index = sc.consumers.iter()
    .position(|c| c.consumer_id == my_consumer_id)
    .expect("Consumer not authorized");
```

### Step 2: Unwrap the CEK Using Your Private Key

```rust
// Pseudo-code - requires external crypto library (e.g., ring, RustCrypto, OpenSSL)
for group in &sc.resource_data_groups {
    // Find access right for your consumer
    let access_right = group.access_rights.iter()
        .find(|ar| ar.consumer_index == consumer_index)
        .expect("No access right for this consumer");
    
    // Decode the wrapped CEK
    let wrapped_cek = base64::decode(&access_right.cipher_value)?;
    
    // Load your RSA private key (from secure storage)
    let private_key = load_private_key_from_secure_storage()?;
    
    // Unwrap using RSA-OAEP (requires external crypto library)
    let cek = rsa_oaep_unwrap(
        &wrapped_cek,
        &private_key,
        &access_right.kek_params
    )?;
    
    // Now decrypt each resource in this group
    for resource in &group.resource_data {
        decrypt_resource(resource, &cek)?;
    }
}
```

### Step 3: Decrypt Resource Using AES-GCM

```rust
// Pseudo-code - requires external crypto library
fn decrypt_resource(resource: &ResourceData, cek: &[u8]) -> Result<Vec<u8>> {
    // Read encrypted file from package
    let encrypted_data = package.get_file_binary(&resource.path)?;
    
    // Decode parameters
    let iv = base64::decode(resource.cek_params.iv.as_ref().unwrap())?;
    let tag = base64::decode(resource.cek_params.tag.as_ref().unwrap())?;
    let aad = resource.cek_params.aad.as_ref()
        .map(|a| base64::decode(a)).transpose()?;
    
    // Decrypt using AES-256-GCM (requires external crypto library)
    let plaintext = aes_gcm_decrypt(
        &encrypted_data,
        cek,
        &iv,
        &tag,
        aad.as_deref()
    )?;
    
    // Decompress if needed
    if resource.cek_params.compression == "deflate" {
        Ok(decompress(&plaintext)?)
    } else {
        Ok(plaintext)
    }
}
```

### Recommended Cryptographic Libraries

- **[ring](https://crates.io/crates/ring)**: Recommended, well-audited, focused on correctness
- **[RustCrypto](https://github.com/RustCrypto)**: Pure Rust implementations
- **[OpenSSL bindings](https://crates.io/crates/openssl)**: Battle-tested, widely used

**Security Warning**: Implementing cryptography incorrectly is dangerous. Always:
- Use established, well-audited libraries
- Follow library documentation exactly
- Never implement your own cryptographic primitives
- Protect private keys using secure storage (HSM, TPM, key vaults)
- Consider professional security review for production systems

## Security Considerations

### ⚠️ Important Security Warnings

1. **NO DECRYPTION**: This library does NOT decrypt secure content
   - Encrypted files remain encrypted
   - No access to encrypted resources
   - No cryptographic key handling

2. **NO SIGNATURE VERIFICATION**: Digital signatures are NOT verified
   - Cannot validate authenticity
   - Cannot detect tampering
   - Cannot verify signer identity

3. **NO CERTIFICATE VALIDATION**: Certificates/keys are NOT validated
   - No PKI integration
   - No certificate chain validation
   - No revocation checking

4. **METADATA ONLY**: Only structural parsing
   - Can identify encrypted parts
   - Can read unencrypted metadata
   - Cannot access protected content

### Recommended Security Practices

For applications requiring secure content support:

1. **Use Specialized Libraries**: 
   - Integrate with established crypto libraries (ring, RustCrypto)
   - Use HSMs for key management
   - Consider platform-specific security APIs

2. **External Processing**:
   - Decrypt files outside Rust using official tools
   - Process decrypted content separately
   - Use vendor-provided security ecosystems

3. **Compliance Requirements**:
   - ITAR/GDPR compliance requires specialized solutions
   - Medical/aerospace applications need certified implementations
   - Consult security professionals for production use

4. **Key Management**:
   - Never embed keys in source code
   - Use secure key storage (HSM, TPM, key vaults)
   - Implement proper key rotation
   - Follow principle of least privilege

### Attack Surface Analysis

**What This Library Does NOT Protect Against:**

1. **Content Extraction**: If encrypted files are present, they remain encrypted but:
   - File paths are visible
   - File sizes can be observed
   - Metadata leakage is possible

2. **Timing Attacks**: Not applicable (no decryption)

3. **Side-Channel Attacks**: Not applicable (no key operations)

4. **Man-in-the-Middle**: Package transport security is out of scope

**What Users Must Protect:**

1. **Key Material**: Any decryption keys must be protected by the application
2. **Decrypted Content**: Once decrypted, protection is application responsibility
3. **Transport Security**: Use TLS for file transfer
4. **Access Control**: Implement application-level access controls

## Test Files

### Obtaining Test Files

Official 3MF Secure Content test files can be obtained from:
- **3MF Consortium Test Suites**: https://github.com/3MFConsortium/test_suites
- **Suite 8**: Secure Content test cases

To run conformance tests with secure content:

```bash
# Clone test suites repository
git clone https://github.com/3MFConsortium/test_suites
cd test_suites

# Copy suite8_secure to lib3mf_rust/test_suites/
cp -r suite8_secure /path/to/lib3mf_rust/test_suites/

# Run tests
cd /path/to/lib3mf_rust
cargo test suite8_secure
```

### Creating Test Files

For development and testing without official test files:

```xml
<!-- Minimal secure content example -->
<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:sc="http://schemas.microsoft.com/3dmanufacturing/securecontent/2019/07"
       requiredextensions="sc"
       unit="millimeter">
  <!-- Standard 3MF content here -->
</model>
```

## Future Enhancements

### Currently Implemented ✅

1. **Complete Keystore Metadata Extraction**:
   - ✅ Parse complete `<keystore>` structure
   - ✅ Extract consumer IDs, key IDs, and PEM public keys
   - ✅ Identify encryption algorithms (wrapping, MGF, digest)
   - ✅ Track encrypted resource paths
   - ✅ Extract all CEK and KEK parameters (IV, tag, AAD, compression)
   - ✅ Comprehensive EPX validation (EPX-2601 through EPX-2607)

### Potential Future Extensions (Out of Current Scope)

1. **Signature Verification** (requires external library):
   - Integrate with `ring` or `RustCrypto`
   - Verify digital signatures on encrypted content
   - Validate certificate chains
   - Check for tampering

2. **Built-in Decryption Support** (requires external library):
   - Provide optional convenience wrappers around crypto libraries
   - RSA-2048 OAEP key unwrapping
   - AES-256 GCM decryption
   - Automatic content decompression
   - Secure key management integration

3. **PKI Integration**:
   - Certificate validation
   - Trust chain verification
   - Revocation checking (OCSP, CRL)
   - Key distribution protocols

### Why Cryptographic Operations Are Not Built-In

Each of these requires:
- Extensive cryptographic expertise
- Security audit and testing
- Compliance certification
- Ongoing security maintenance
- Risk of implementation flaws

**Recommendation**: For production security needs, use:
- Official 3MF libraries with security support
- Vendor-provided security ecosystems
- Certified cryptographic modules
- Professional security consultation

## References

1. [3MF Secure Content Specification v1.0.3](https://github.com/3MFConsortium/spec_securecontent/blob/master/3MF%20Secure%20Content.md)
2. [3MF Core Specification](https://github.com/3MFConsortium/spec_core)
3. [RFC 7468 - Textual Encodings of PKIX](https://tools.ietf.org/html/rfc7468)
4. [RFC 1951 - DEFLATE Compression](https://tools.ietf.org/html/rfc1951)
5. [NIST SP 800-38D - AES-GCM](https://csrc.nist.gov/publications/detail/sp/800-38d/final)
6. [3MF Consortium Test Suites](https://github.com/3MFConsortium/test_suites)

## Conclusion

This implementation provides **complete keystore metadata extraction** for the Secure Content extension. It:

- ✅ Recognizes and validates the SecureContent extension
- ✅ Parses the complete keystore.xml structure according to the 3MF spec
- ✅ Extracts all consumer information, encryption parameters, and access rights
- ✅ Validates keystore structure per EPX error codes (EPX-2601 through EPX-2607)
- ✅ Provides all metadata needed for applications to implement decryption

Applications can access this metadata to:
- Identify which files are encrypted
- Determine which consumers can decrypt content
- Retrieve encryption algorithm parameters
- Implement decryption using external cryptographic libraries (ring, RustCrypto, OpenSSL)

This "parse structure, delegate cryptography" approach provides maximum flexibility while
maintaining security by leaving actual cryptographic operations to specialized, well-audited
libraries chosen by the application developer.

**For decryption implementation**: Applications must use external cryptographic libraries.
See the "Implementing Decryption" section above for conceptual workflow.

**Security is hard. When in doubt, consult experts.**
