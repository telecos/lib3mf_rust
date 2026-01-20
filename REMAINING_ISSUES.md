# Remaining Issues for lib3mf_rust

This document captures all identified remaining work items, bugs, and enhancements for the lib3mf_rust project. Issues are categorized by type and priority for easy tracking and planning.

---

## Table of Contents

1. [Extension Data Extraction](#extension-data-extraction)
2. [Validation & Conformance](#validation--conformance)
3. [Feature Enhancements](#feature-enhancements)
4. [Testing & Quality](#testing--quality)
5. [Documentation](#documentation)

---

## Extension Data Extraction

These issues relate to fully implementing data extraction for 3MF extensions that are currently only partially supported.

### Issue 1: Production Extension - Extract UUID Attributes
**Priority:** Medium  
**Category:** Extension Support  
**Status:** Not Started

**Description:**
The Production extension is currently recognized and validated, but production-specific data is not extracted. Files parse successfully but UUID attributes and production paths are not captured.

**Current State:**
- ✅ Files with Production extension parse successfully
- ✅ Extension validation works correctly
- ❌ UUID attributes (`p:UUID`) not extracted
- ❌ Production paths not captured
- ❌ Production-specific metadata not accessible

**Expected Outcome:**
- Add data structures for production extension elements
- Extract UUID attributes from objects
- Parse production paths and thumbnails
- Make production data accessible via Model API

**Reference:**
- IMPLEMENTATION_SUMMARY.md, line 90
- EXTENSION_SUPPORT_SUMMARY.md, line 217
- README.md, line 174

**Test Files Available:** Yes (`test_files/box_prod.3mf`)

---

### Issue 2: Slice Extension - Extract Slice Stack Definitions
**Priority:** Medium  
**Category:** Extension Support  
**Status:** Not Started

**Description:**
The Slice extension is currently recognized and validated, but slice-specific data structures (slice stacks, slice references) are not extracted.

**Current State:**
- ✅ Files with Slice extension parse successfully
- ✅ Extension validation works correctly
- ❌ Slice stack definitions (`s:slicestack`) not extracted
- ❌ Slice references not captured
- ❌ Slice data not accessible

**Expected Outcome:**
- Add `SliceStack` data structure
- Add `Slice` data structure with polygon segments
- Extract slice stack definitions from model
- Extract slice references from objects
- Make slice data accessible via Model API

**Reference:**
- IMPLEMENTATION_SUMMARY.md, line 91
- EXTENSION_SUPPORT_SUMMARY.md, line 217
- README.md, line 175

**Test Files Available:** Yes (`test_files/box_sliced.3mf`)

---

### Issue 3: Beam Lattice Extension - Extract Beam Definitions
**Priority:** Medium  
**Category:** Extension Support  
**Status:** Not Started

**Description:**
The Beam Lattice extension is currently recognized and validated, but beam-specific data structures (beam definitions, beam properties) are not extracted.

**Current State:**
- ✅ Files with Beam Lattice extension parse successfully
- ✅ Extension validation works correctly
- ❌ Beam definitions (`b:beamsets`) not extracted
- ❌ Beam properties not captured
- ❌ Lattice structure data not accessible

**Expected Outcome:**
- Add `BeamSet` data structure
- Add `Beam` data structure with radius and properties
- Extract beam definitions from model
- Parse beam properties and clipping modes
- Make beam lattice data accessible via Model API

**Reference:**
- IMPLEMENTATION_SUMMARY.md, line 92
- EXTENSION_SUPPORT_SUMMARY.md, line 217
- README.md, line 176

**Test Files Available:** Yes (`test_files/pyramid.3mf`)

---

### Issue 4: Secure Content Extension - Add Test Coverage
**Priority:** Low  
**Category:** Extension Support  
**Status:** Not Started

**Description:**
The Secure Content extension is recognized for validation purposes but has no test coverage or implementation.

**Current State:**
- ✅ Extension recognized and validated
- ❌ No test files available
- ❌ No implementation for secure content elements
- ❌ Digital signatures not supported
- ❌ Encryption not supported

**Expected Outcome:**
- Obtain test files from 3MF Consortium test suites
- Determine scope of support (read-only validation vs. full signature verification)
- Implement basic parsing if test files are available
- Document security considerations

**Reference:**
- IMPLEMENTATION_SUMMARY.md, line 93
- README.md, line 213

**Test Files Available:** Check suite8 (Secure Content) in conformance tests

---

### Issue 5: Boolean Operations Extension - Add Test Coverage
**Priority:** Low  
**Category:** Extension Support  
**Status:** Not Started

**Description:**
The Boolean Operations extension is recognized for validation purposes but has no test coverage or implementation.

**Current State:**
- ✅ Extension recognized and validated
- ❌ No test files available
- ❌ No implementation for volumetric operations
- ❌ Boolean operation data not extracted

**Expected Outcome:**
- Obtain test files from 3MF Consortium test suites
- Implement parsing for volumetric boolean operations
- Extract boolean operation data structures
- Make boolean operation data accessible via Model API

**Reference:**
- IMPLEMENTATION_SUMMARY.md, line 94
- README.md, line 214

**Test Files Available:** Check suite10 (Boolean Operations) in conformance tests

---

### Issue 6: Displacement Extension - Add Test Coverage
**Priority:** Low  
**Category:** Extension Support  
**Status:** Not Started

**Description:**
The Displacement extension is recognized for validation purposes but has no test coverage or implementation.

**Current State:**
- ✅ Extension recognized and validated
- ❌ No test files available
- ❌ No implementation for displacement maps
- ❌ Surface displacement data not extracted

**Expected Outcome:**
- Obtain test files from 3MF Consortium test suites
- Implement parsing for displacement maps
- Extract displacement data structures
- Make displacement data accessible via Model API

**Reference:**
- README.md, line 215

**Test Files Available:** Check suite11 (Displacement) in conformance tests

---

## Validation & Conformance

These issues relate to improving validation logic to correctly reject invalid 3MF files.

### Issue 7: Improve Negative Test Conformance (1.7% → 100%)
**Priority:** High  
**Category:** Validation  
**Status:** Not Started

**Description:**
Currently only 1.7% (9 out of 543) of invalid 3MF files are correctly rejected. The parser is too permissive and accepts many files that violate the 3MF specification.

**Current State:**
- ✅ 100% positive test compliance (1,698/1,698 valid files parse)
- ⚠️ 1.7% negative test compliance (9/543 invalid files rejected)
- 📊 76.2% overall conformance (1,707/2,241 total tests)

**Expected Outcome:**
- Analyze negative test failures systematically
- Categorize failures by error type (using test codes)
- Implement validation rules for each category
- Achieve >90% negative test compliance
- Document any intentionally permissive behavior

**Reference:**
- README.md, lines 204-206
- TESTING_QUICK_REFERENCE.md, lines 131-133
- examples/categorize_failures.rs
- examples/analyze_negative_tests.rs

**Approach:**
1. Run `cargo run --example categorize_failures` to group failures by code
2. Analyze each code category to understand the spec requirement
3. Implement validator checks for each category
4. Run `cargo run --example analyze_negative_tests` to track progress
5. Focus on high-frequency failure categories first

**Test Files Available:** Yes (543 negative test cases in test_suites)

---

### Issue 8: Validate Base Materials References
**Priority:** Medium  
**Category:** Validation  
**Status:** Not Started

**Description:**
The validator currently only checks color group references but doesn't validate base materials references. This is marked with a TODO in the code.

**Current State:**
- ✅ Color group references are validated
- ❌ Base materials references are not validated
- ❌ `basematerialid` attributes not checked

**Expected Outcome:**
- Add data structure for base materials
- Parse base materials from XML
- Validate that `basematerialid` attributes reference valid base materials
- Ensure `pid` can reference either color groups or base materials

**Reference:**
- src/validator.rs, line 210

**Code Location:** `src/validator.rs:210`

---

### Issue 9: Validate Component References
**Priority:** Medium  
**Category:** Validation  
**Status:** Not Started

**Description:**
3MF supports components (objects that reference other objects to create assemblies). The parser doesn't currently validate these references or extract component data.

**Current State:**
- ❌ Component elements not parsed
- ❌ Component references not validated
- ❌ Circular component references not detected

**Expected Outcome:**
- Add `Component` data structure
- Parse component elements from objects
- Validate component `objectid` references
- Detect circular component references
- Support transformation matrices on components

**Reference:**
- README.md, line 178

**Spec Section:** 3MF Core Specification, Chapter 6 (Components)

---

### Issue 10: Validate Thumbnail References
**Priority:** Low  
**Category:** Validation  
**Status:** Not Started

**Description:**
3MF files can include thumbnail images in the package. The parser doesn't currently validate or extract these.

**Current State:**
- ❌ Thumbnail paths not validated
- ❌ Thumbnail images not accessible

**Expected Outcome:**
- Validate thumbnail paths reference actual files in the package
- Extract thumbnail metadata (path, content type)
- Optionally provide API to read thumbnail data

**Spec Section:** 3MF Core Specification, Chapter 4 (Metadata)

---

### Issue 11: Validate Metadata Requirements
**Priority:** Low  
**Category:** Validation  
**Status:** Not Started

**Description:**
Improve validation of metadata elements according to the spec requirements.

**Current State:**
- ✅ Basic metadata parsing works
- ❌ Required metadata elements not enforced
- ❌ Metadata preservation attributes not validated

**Expected Outcome:**
- Validate required metadata elements
- Check metadata preservation attributes
- Ensure metadata follows spec schema

**Spec Section:** 3MF Core Specification, Chapter 4

---

## Feature Enhancements

These issues relate to new features and capabilities that would enhance the library.

### Issue 12: Support Advanced Material Properties
**Priority:** Low  
**Category:** Feature Enhancement  
**Status:** Not Started

**Description:**
The Materials extension supports textures, composite materials, and multi-properties. Currently only color groups and basic base materials are implemented.

**Current State:**
- ✅ Color groups fully supported
- ✅ Basic base materials parsing
- ❌ Texture2D not supported
- ❌ Composite materials not supported
- ❌ Multi-properties not supported

**Expected Outcome:**
- Add data structures for Texture2D
- Parse texture coordinates and mappings
- Support composite materials
- Support multi-property groups
- Make advanced materials accessible via API

**Reference:**
- README.md, line 177

**Spec:** Materials Extension Specification v1.2.1

---

### Issue 13: Support Custom Extensions
**Priority:** Low  
**Category:** Feature Enhancement  
**Status:** Not Started

**Description:**
Allow users to register and handle custom/proprietary 3MF extensions.

**Current State:**
- ✅ Unknown extensions are silently ignored
- ❌ No API for custom extension handling
- ❌ Custom extension data not accessible

**Expected Outcome:**
- Add API for registering custom extensions
- Provide callback mechanism for custom element parsing
- Allow custom validation rules
- Document custom extension API

**Reference:**
- EXTENSION_SUPPORT_SUMMARY.md, line 218
- README.md, line 179

---

### Issue 14: Add Writing/Serialization Support
**Priority:** Low  
**Category:** Feature Enhancement  
**Status:** Not Started

**Description:**
The library currently only supports reading/parsing 3MF files. Add support for creating and writing 3MF files.

**Current State:**
- ✅ Full reading/parsing support
- ❌ No writing/serialization support
- ❌ Cannot create 3MF files

**Expected Outcome:**
- Implement Model serialization to XML
- Support creating ZIP archives
- Write OPC package structure
- Maintain spec compliance for written files
- Add comprehensive tests for round-trip (read-write-read)

**Scope:**
- Model to XML serialization
- ZIP/OPC package creation
- Content types and relationships
- Extension-aware writing

---

### Issue 15: Improve Performance for Large Files
**Priority:** Low  
**Category:** Performance  
**Status:** Not Started

**Description:**
Optimize parser performance for large 3MF files with many vertices and triangles.

**Current State:**
- Works correctly for all test files
- Performance not measured or optimized

**Expected Outcome:**
- Add benchmarks for large files
- Profile memory usage
- Optimize hot paths in parser and validator
- Consider streaming/lazy parsing for very large files
- Document performance characteristics

**Approach:**
- Use criterion.rs for benchmarking
- Profile with cargo flamegraph
- Optimize allocations and copies

---

## Testing & Quality

These issues relate to improving test coverage and code quality.

### Issue 16: Add Conformance Report Generation
**Priority:** Medium  
**Category:** Testing  
**Status:** Not Started

**Description:**
The README references a CONFORMANCE_REPORT.md file that doesn't exist. This should be auto-generated from test results.

**Current State:**
- ❌ CONFORMANCE_REPORT.md does not exist
- ✅ Conformance tests run successfully
- ❌ Detailed results not persisted

**Expected Outcome:**
- Create script to generate CONFORMANCE_REPORT.md
- Include detailed breakdown by suite
- Show positive/negative test statistics
- List specific test failures
- Run as part of CI and commit report

**Reference:**
- README.md, line 217

---

### Issue 17: Improve Error Messages
**Priority:** Medium  
**Category:** Quality  
**Status:** Not Started

**Description:**
Improve error messages to be more helpful for debugging, especially for validation failures.

**Current State:**
- Error messages exist but could be more descriptive
- Limited context in some errors
- No error codes

**Expected Outcome:**
- Add error codes for categorization
- Include file context (line numbers if possible)
- Provide suggestions for common errors
- Improve error documentation

---

### Issue 18: Add Property-Based Testing
**Priority:** Low  
**Category:** Testing  
**Status:** Not Started

**Description:**
Use property-based testing (QuickCheck/proptest) to generate random valid/invalid 3MF models and find edge cases.

**Expected Outcome:**
- Add proptest dependency
- Create generators for Model structures
- Test invariants (e.g., valid model → parses → validates)
- Find edge cases automatically

---

## Documentation

These issues relate to improving documentation and examples.

### Issue 19: Create Migration Guide from lib3mf (C++)
**Priority:** Low  
**Category:** Documentation  
**Status:** Not Started

**Description:**
Help users migrating from the official C++ lib3mf library to this Rust implementation.

**Expected Outcome:**
- Document API differences
- Provide comparison examples
- Note feature parity status
- Help with common migration patterns

---

### Issue 20: Add More Examples
**Priority:** Low  
**Category:** Documentation  
**Status:** Not Started

**Description:**
Add more comprehensive examples demonstrating various features.

**Current State:**
- ✅ Basic parsing example
- ✅ Extension support example
- ✅ Materials example
- ❌ Limited real-world examples

**Expected Outcome:**
- Example: Converting to other formats (STL, OBJ)
- Example: Validation and error handling
- Example: Working with build items and transformations
- Example: Extracting color information for rendering
- Example: Creating a simple model (when writing support added)

---

## Summary Statistics

### By Priority
- **High:** 1 issue
- **Medium:** 6 issues
- **Low:** 13 issues

### By Category
- **Extension Support:** 6 issues
- **Validation:** 5 issues
- **Feature Enhancement:** 4 issues
- **Testing:** 3 issues
- **Documentation:** 2 issues

### Estimated Effort
- **Small (1-2 days):** Issues 8, 10, 11, 16, 17, 19
- **Medium (3-7 days):** Issues 1, 2, 3, 9, 12, 13, 20
- **Large (1-2 weeks):** Issues 7, 14
- **Research/TBD:** Issues 4, 5, 6, 15, 18

---

## Recommended Prioritization

### Phase 1 - Core Validation (High Priority)
1. **Issue 7:** Improve negative test conformance to 90%+
2. **Issue 8:** Validate base materials references
3. **Issue 9:** Validate component references

### Phase 2 - Extension Support (Medium Priority)
4. **Issue 1:** Extract Production extension data
5. **Issue 2:** Extract Slice extension data
6. **Issue 3:** Extract Beam Lattice extension data

### Phase 3 - Quality & Testing (Medium Priority)
7. **Issue 16:** Generate conformance report
8. **Issue 17:** Improve error messages

### Phase 4 - Advanced Features (Low Priority)
9. **Issue 12:** Advanced material properties
10. **Issue 14:** Writing/serialization support
11. **Issue 15:** Performance optimization

### Phase 5 - Polish (Low Priority)
12. Remaining extension support (Issues 4, 5, 6)
13. Documentation improvements (Issues 19, 20)
14. Additional testing (Issue 18)

---

## Notes

- This document was auto-generated from code analysis, documentation review, and conformance test results
- Issues should be created in GitHub Issues for tracking and discussion
- Each issue includes sufficient context for implementation
- Test files are available for most issues requiring validation
- The 3MF specification documents are available at https://3mf.io/specification/

**Last Updated:** January 20, 2026
