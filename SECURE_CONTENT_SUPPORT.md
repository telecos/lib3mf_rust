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

### What is NOT Supported ❌

1. **No Cryptographic Operations**
   - No signature verification
   - No decryption of encrypted content
   - No certificate/key management
   - No cryptographic primitives implementation

2. **No Key Store Parsing**
   - `<keystore>` elements are not parsed
   - `<consumer>` elements are not extracted
   - `<resourcedatagroup>` elements are not processed
   - Encryption metadata is not captured

3. **No Content Access**
   - Encrypted OPC parts are not decrypted
   - Cannot read encrypted model files
   - Cannot access encrypted resources

## Scope of Support: Read-Only Validation

This implementation provides **read-only validation** of secure content elements:

### Design Decision: Validation Only

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
// Users can:
let model = Model::from_reader(file)?;

// 1. Check if secure content is used
if model.required_extensions.contains(&Extension::SecureContent) {
    println!("This file uses secure content");
}

// 2. Parse file structure (currently limited to non-encrypted parts)
// 3. Validate 3MF package integrity
```

## Data Structures

### Minimal Secure Content Support

For basic awareness and metadata extraction, we define minimal structures:

```rust
/// Secure content metadata (read-only)
#[derive(Debug, Clone)]
pub struct SecureContentInfo {
    /// UUID of the keystore
    pub keystore_uuid: Option<String>,
    /// Paths to encrypted files in the package
    pub encrypted_files: Vec<String>,
}
```

These structures are **intentionally minimal** to avoid giving false confidence that full security features are implemented.

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

### Potential Extensions (Out of Current Scope)

1. **Metadata Extraction**:
   - Parse `<keystore>` structure
   - Extract consumer IDs
   - Identify encryption algorithms used
   - Track encrypted resource paths

2. **Signature Verification** (with external library):
   - Integrate with `ring` or `RustCrypto`
   - Verify digital signatures
   - Validate certificate chains
   - Check for tampering

3. **Decryption Support** (with external library):
   - RSA-2048 OAEP key unwrapping
   - AES-256 GCM decryption
   - Content decompression
   - Secure key management integration

4. **PKI Integration**:
   - Certificate validation
   - Trust chain verification
   - Revocation checking
   - Key distribution protocols

### Why These Are Not Implemented Now

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

This implementation provides **structural validation only** for the Secure Content extension. It correctly recognizes the extension and validates its presence in the `requiredextensions` attribute, but does **NOT** implement cryptographic operations.

For applications requiring actual security features (encryption, signatures, key management), integrate with specialized cryptographic libraries and security ecosystems rather than implementing these features within this general-purpose 3MF parser.

**Security is hard. When in doubt, consult experts.**
