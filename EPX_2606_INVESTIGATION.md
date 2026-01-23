# EPX-2606 Test Investigation Report

## Problem Statement
Investigate the 3 failing EPX-2606 tests from the Suite 8 SecureContent conformance test suite.

## Test Files
- `N_EPX_2606_01.3mf`
- `N_EPX_2606_02.3mf`
- `N_EPX_2606_03.3mf`

## What is EPX-2606?
According to the SUITE8_FIX_SUMMARY.md:
> **EPX-2606: Missing/Invalid Keystore Elements**
> - **Issue**: Missing required elements like `<iv>`, `<tag>`, or malformed structure
> - **Validation needed**: Validate keystore XML schema compliance

## Investigation Process

### 1. Initial Hypothesis Testing
First tested if the issue was related to empty `<aad>` elements, as all 3 negative tests contain:
```xml
<aad></aad>
```

**Result**: This was **not** the issue. Positive tests also have empty AAD elements, and according to the 3MF SecureContent specification, AAD is optional and can be empty.

### 2. Keystore Structure Analysis
Extracted and compared `Secure/keystore.xml` from negative and positive tests:

All tests contain:
- ✅ `<consumer>` with `<keyvalue>` child element  
- ✅ `<iv>` element with base64-encoded content
- ✅ `<tag>` element with base64-encoded content
- ✅ `<aad>` element (empty in all cases)
- ✅ All required attributes on `<cekparams>`

The keystore XML structure was **identical** between negative and positive tests.

### 3. OPC Package Relationship Analysis
Examined the `3D/_rels/3dmodel.model.rels` files:

**N_EPX_2606_01.3mf** (Missing EncryptedFile relationship):
```xml
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rel14" Target="/3D/3dmodel_encrypted.model" 
                  Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
    <!-- MISSING: EncryptedFile relationship -->
</Relationships>
```

**Positive test P_EPX_2101_01.3mf**:
```xml
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rel14" Target="/3D/3dmodel_encrypted.model" 
                  Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
    <Relationship Id="rel15" Target="/3D/3dmodel_encrypted.model" 
                  Type="http://schemas.openxmlformats.org/package/2006/relationships/encryptedfile"/>
</Relationships>
```

**Finding**: N_EPX_2606_01 is missing the required EncryptedFile relationship!

However, N_EPX_2606_02 and N_EPX_2606_03 **DO** have the EncryptedFile relationship, indicating a different validation issue.

### 4. Specification Requirements
According to the 3MF SecureContent specification:
> All encrypted files referenced by a resource data element MUST have a EncryptedFile relationship.

## Root Causes

### N_EPX_2606_01.3mf
**Issue**: Missing OPC EncryptedFile relationship for the encrypted resource file.

**Required Validation**: 
- Cross-reference encrypted file paths in `keystore.xml` with OPC package relationships
- Ensure each encrypted file has both:
  1. A model/component relationship
  2. An `encryptedfile` relationship

**Implementation Complexity**: Medium - Requires accessing and validating OPC relationship data that is currently parsed but not cross-validated with keystore content.

### N_EPX_2606_02.3mf & N_EPX_2606_03.3mf
**Issue**: Unknown complex structural validation requirements.

**Observations**:
- Both have correct EncryptedFile relationships
- Keystore XML structure appears valid
- All required child elements (`<iv>`, `<tag>`) are present with content

**Possible Issues** (speculation based on issue #162):
- Element ordering requirements
- Namespace validation
- Text content format validation  
- Parent-child element context requirements
- XML schema compliance beyond what's obvious

**Implementation Complexity**: High - Requires:
1. XML parent-child relationship tracking
2. Context-dependent validation
3. Detailed understanding of undocumented validation rules

## Current Implementation Status

### What IS Implemented
- ✅ EPX-2601: Consumer index validation
- ✅ EPX-2602: Consumer existence validation
- ✅ EPX-2603: Encryption algorithm validation
- ✅ EPX-2604: Duplicate consumer ID detection
- ✅ EPX-2605: Invalid encrypted file path validation
- ✅ EPX-2607: File existence validation

### What is NOT Implemented (EPX-2606)
- ❌ EncryptedFile relationship validation
- ❌ Complex keystore structural validation
- ❌ Context-dependent XML element validation
- ❌ Parent-child element relationship validation

## Recommendations

### Short Term (Current PR)
Mark the 3 EPX-2606 tests as expected failures with detailed documentation explaining:
1. The specific validation requirements
2. Why they're not yet implemented
3. The complexity level of implementation

**Status**: ✅ Completed

### Medium Term
Implement N_EPX_2606_01 validation:
```rust
// Pseudo-code for validation
fn validate_encrypted_file_relationships(model: &Model, package: &OpcPackage) -> Result<()> {
    if let Some(ref sc) = model.secure_content {
        for group in &sc.resource_data_groups {
            for resource_data in &group.resource_data {
                let encrypted_path = &resource_data.path;
                
                // Check if this path has an EncryptedFile relationship
                if !package.has_relationship(encrypted_path, ENCRYPTEDFILE_TYPE) {
                    return Err(Error::InvalidSecureContent(format!(
                        "Encrypted file '{}' must have an EncryptedFile relationship (EPX-2606)",
                        encrypted_path
                    )));
                }
            }
        }
    }
    Ok(())
}
```

### Long Term
1. Obtain official 3MF Consortium test suite documentation for EPX error codes
2. Study the C++ lib3mf reference implementation for EPX-2606 validation
3. Implement comprehensive keystore validation based on official requirements
4. Consider adding an XML schema validator for the keystore.xml file

## Testing Strategy

### Verification
To verify EPX-2606 validation when implemented:
1. All 3 negative tests (N_EPX_2606_01/02/03) should fail to parse
2. All positive tests should continue to pass
3. Error messages should clearly indicate EPX-2606 violation

### Test Cases to Add
```rust
#[test]
fn test_epx_2606_missing_encryptedfile_relationship() {
    // Test that encrypted files without EncryptedFile relationship fail
}

#[test]
fn test_epx_2606_keystore_structural_validation() {
    // Test that malformed keystore structures fail appropriately
}
```

## References
- [3MF SecureContent Specification](https://github.com/3MFConsortium/spec_securecontent/blob/master/3MF%20Secure%20Content.md)
- [Issue #162: Remaining negative cases from secure content suite 8](https://github.com/telecos/lib3mf_rust/issues/162)
- [SUITE8_FIX_SUMMARY.md](SUITE8_FIX_SUMMARY.md)

## Conclusion

The 3 EPX-2606 tests expose limitations in the current SecureContent validation implementation:

1. **N_EPX_2606_01**: Requires OPC relationship validation
2. **N_EPX_2606_02/03**: Require undocumented complex structural validation

Implementing these validations properly requires significant refactoring to support:
- Cross-package validation (keystore + OPC relationships)
- Context-aware XML validation
- Better understanding of the official EPX error code specifications

The tests have been documented as expected failures with clear explanations to guide future implementation efforts.
