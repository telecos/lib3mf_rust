# Action Plan: Achieve 100% Conformance Test Passing Rate

## Current Status (January 2026)

- ✅ **Positive Tests**: 100% (1,698/1,698) - All valid files parse correctly
- ⚠️ **Negative Tests**: 1.7% (9/543) - Only 9 invalid files rejected
- ❌ **Gap**: 534 invalid files incorrectly accepted
- 📊 **Overall**: 76.2% (1,707/2,241)

**Goal**: 100% passing rate for both positive (maintain) and negative (fix 534 failures)

---

## Phase 1: Analysis & Categorization (CRITICAL FIRST STEP)

### Task 1.1: Run Failure Analysis
**Priority**: CRITICAL  
**Effort**: 1 hour  
**Dependencies**: None

**Actions**:
1. Clone test suites if not present:
   ```bash
   git clone https://github.com/3MFConsortium/test_suites.git
   ```

2. Run failure categorization:
   ```bash
   cargo run --example categorize_failures > failure_analysis.txt
   ```

3. Analyze output to identify:
   - Test code categories (e.g., N_XXX_0205, N_XXX_0304)
   - Number of failures per category
   - Which categories affect most tests

**Deliverable**: `failure_analysis.txt` with categorized failures

**Success Criteria**:
- [ ] All 534 failing tests categorized by code
- [ ] Frequency count per category
- [ ] Top 10 failure categories identified

---

### Task 1.2: Map Test Codes to Spec Violations
**Priority**: CRITICAL  
**Effort**: 4 hours  
**Dependencies**: Task 1.1

**Actions**:
1. For each test code category identified:
   - Review failing test file names
   - Check 3MF specification for violation type
   - Document what validation rule is missing

2. Create mapping document:
   ```markdown
   | Test Code | Violation Type | Spec Section | Current Behavior | Required Fix |
   |-----------|---------------|--------------|------------------|--------------|
   | 0205 | Missing required attribute | 4.1.2 | Accepts | Add attribute validation |
   | 0304 | Invalid vertex reference | 5.2.1 | Accepts | Validate component refs |
   ```

**Deliverable**: `test_code_mapping.md`

**Success Criteria**:
- [ ] All failure codes mapped to spec sections
- [ ] Required validation rule documented for each
- [ ] Categories grouped by implementation area

---

## Phase 2: Quick Wins (High-Impact Validation Rules)

### Task 2.1: Implement Required Attribute Validation
**Priority**: HIGH  
**Effort**: 2-3 days  
**Dependencies**: Task 1.2

**Actions**:
1. Identify all required attributes per 3MF spec
2. Add validation in parser for:
   - `<model>` required attributes
   - `<object>` required attributes (id, type)
   - `<vertex>` required attributes (x, y, z)
   - `<triangle>` required attributes (v1, v2, v3)
   - `<build>` and `<item>` required attributes

3. Return clear errors for missing attributes

**Files to Modify**:
- `src/parser.rs` - Add attribute presence checks
- `src/validator.rs` - Add validation functions
- `src/error.rs` - Add specific error types

**Testing**:
```bash
cargo run --example analyze_negative_tests
# Should show improvement in rejection rate
```

**Success Criteria**:
- [ ] Parser rejects files with missing required attributes
- [ ] Clear error messages reference spec
- [ ] Negative test pass rate improves by 10-20%
- [ ] No regression in positive tests

---

### Task 2.2: Validate Attribute Value Ranges
**Priority**: HIGH  
**Effort**: 2-3 days  
**Dependencies**: Task 1.2

**Actions**:
1. Add range validation for:
   - Object IDs (must be > 0) ✓ Already exists
   - Vertex indices (must be < vertex count) ✓ Already exists
   - Property indices (must be < property count) ✓ Partially exists
   - Color values (0-255 or valid hex)
   - Transformation matrices (valid dimensions)

2. Add type validation:
   - Boolean attributes (0/1, true/false)
   - Enumerated types (unit, objecttype, etc.)

**Files to Modify**:
- `src/parser.rs` - Add value range checks during parsing
- `src/validator.rs` - Add post-parse validation

**Success Criteria**:
- [ ] Out-of-range values rejected
- [ ] Invalid type values rejected
- [ ] Negative test pass rate improves by 10-15%

---

### Task 2.3: Implement Base Materials Validation
**Priority**: HIGH  
**Effort**: 2-3 days  
**Dependencies**: None (has TODO marker)

**Actions**:
1. Add `BaseMaterial` struct:
   ```rust
   pub struct BaseMaterial {
       pub id: usize,
       pub name: String,
       pub displaycolor: String,
   }
   ```

2. Parse `<basematerials>` elements in `src/parser.rs`

3. Validate references in `src/validator.rs`:
   - Check `basematerialid` references valid base material
   - Check `pid` can reference either color group OR base material
   - Remove TODO comment

**Files to Modify**:
- `src/model.rs` - Add BaseMaterial struct
- `src/parser.rs` - Parse base materials
- `src/validator.rs` - Validate references (line 210)
- `src/lib.rs` - Export BaseMaterial

**Success Criteria**:
- [ ] Base materials parsed from XML
- [ ] Invalid basematerialid rejected
- [ ] TODO removed from src/validator.rs:210
- [ ] Tests pass for materials extension files

---

## Phase 3: Component Support (Required for Many Failures)

### Task 3.1: Parse Component Elements
**Priority**: HIGH  
**Effort**: 3-4 days  
**Dependencies**: Task 1.2

**Actions**:
1. Add `Component` struct:
   ```rust
   pub struct Component {
       pub objectid: usize,
       pub transform: Option<[f64; 12]>,
   }
   ```

2. Parse `<component>` elements from `<object>`

3. Store in `Object.components: Vec<Component>`

**Files to Modify**:
- `src/model.rs` - Add Component struct
- `src/parser.rs` - Parse component elements
- `src/lib.rs` - Export Component

**Success Criteria**:
- [ ] Component elements parsed
- [ ] Transform matrices captured
- [ ] Data accessible via Model API

---

### Task 3.2: Validate Component References
**Priority**: HIGH  
**Effort**: 2-3 days  
**Dependencies**: Task 3.1

**Actions**:
1. Validate `objectid` in components references existing object

2. Detect circular references:
   ```rust
   fn detect_circular_components(
       model: &Model,
       object_id: usize,
       visited: &mut HashSet<usize>,
       stack: &mut Vec<usize>
   ) -> Result<()>
   ```

3. Reject self-references (object referencing itself)

**Files to Modify**:
- `src/validator.rs` - Add component validation functions

**Testing**:
- Create test for valid component hierarchy
- Create test for circular reference detection
- Create test for invalid objectid reference

**Success Criteria**:
- [ ] Invalid component objectid rejected
- [ ] Circular references detected and rejected
- [ ] Self-references rejected
- [ ] Negative test pass rate improves significantly

---

## Phase 4: XML Structure Validation

### Task 4.1: Validate Required Elements
**Priority**: MEDIUM  
**Effort**: 2-3 days  
**Dependencies**: Task 1.2

**Actions**:
1. Ensure required elements present:
   - `<model>` must have `<resources>`
   - `<model>` must have `<build>`
   - `<resources>` must have at least one `<object>`
   - `<build>` must have at least one `<item>`

2. Current validation exists (lines 38-51 in validator.rs)

3. Extend to check:
   - Mesh with triangles must have vertices
   - Objects referenced in build must exist

**Files to Modify**:
- `src/validator.rs` - Extend validate_required_structure()

**Success Criteria**:
- [ ] Missing `<resources>` rejected
- [ ] Missing `<build>` rejected
- [ ] Empty resources/build rejected

---

### Task 4.2: Validate XML Schema Compliance
**Priority**: MEDIUM  
**Effort**: 3-4 days  
**Dependencies**: Task 1.2

**Actions**:
1. Stricter element ordering validation
2. Attribute type validation (integers, floats, enums)
3. Namespace validation
4. Element nesting validation

**Files to Modify**:
- `src/parser.rs` - Add schema validation during parsing

**Success Criteria**:
- [ ] Invalid element order rejected
- [ ] Invalid attribute types rejected
- [ ] Malformed XML rejected

---

## Phase 5: OPC Package Validation

### Task 5.1: Validate Relationships
**Priority**: MEDIUM  
**Effort**: 2-3 days  
**Dependencies**: Task 1.2

**Actions**:
1. Validate `_rels/.rels` file structure
2. Check relationship targets exist
3. Validate content types in `[Content_Types].xml`

**Files to Modify**:
- `src/opc.rs` - Add relationship validation

**Success Criteria**:
- [ ] Invalid relationships rejected
- [ ] Missing targets rejected
- [ ] Invalid content types rejected

---

### Task 5.2: Validate Thumbnail References
**Priority**: LOW  
**Effort**: 1-2 days  
**Dependencies**: None

**Actions**:
1. If thumbnail path specified, validate file exists in package
2. Validate thumbnail path format

**Files to Modify**:
- `src/parser.rs` - Add thumbnail validation
- `src/validator.rs` - Check thumbnail file existence

**Success Criteria**:
- [ ] Invalid thumbnail paths rejected
- [ ] Missing thumbnail files rejected

---

## Phase 6: Metadata Validation

### Task 6.1: Validate Metadata Requirements
**Priority**: LOW  
**Effort**: 1-2 days  
**Dependencies**: Task 1.2

**Actions**:
1. Validate required metadata elements per spec
2. Check metadata preservation attributes
3. Validate metadata type values

**Files to Modify**:
- `src/parser.rs` - Parse and validate metadata
- `src/validator.rs` - Add metadata validation

**Success Criteria**:
- [ ] Required metadata present
- [ ] Invalid metadata rejected

---

## Phase 7: Testing & Verification

### Task 7.1: Comprehensive Testing
**Priority**: CRITICAL  
**Effort**: Ongoing  
**Dependencies**: All implementation tasks

**Actions**:
1. After each task, run:
   ```bash
   # Check positive tests still pass
   cargo test
   
   # Check negative test improvement
   cargo run --example analyze_negative_tests
   
   # See specific failures
   cargo run --example categorize_failures
   ```

2. Track progress:
   - Maintain log of test pass rate after each task
   - Document which validation rules fixed which test codes
   - Ensure no regressions in positive tests

**Success Criteria**:
- [ ] 100% positive tests maintained (1,698/1,698)
- [ ] >90% negative tests passing (485+/543)
- [ ] Target: 100% negative tests (543/543)

---

### Task 7.2: Generate Conformance Report
**Priority**: MEDIUM  
**Effort**: 1-2 days  
**Dependencies**: Task 7.1

**Actions**:
1. Create script to run conformance tests and generate report
2. Document pass/fail by suite
3. List any remaining failures with analysis

**Deliverable**: `CONFORMANCE_REPORT.md`

**Success Criteria**:
- [ ] Automated report generation
- [ ] Current statistics documented
- [ ] Remaining failures (if any) analyzed

---

## Phase 8: Error Message Improvements

### Task 8.1: Add Error Codes and Context
**Priority**: LOW  
**Effort**: 2-3 days  
**Dependencies**: None

**Actions**:
1. Add error codes to all validation errors
2. Include file context (element, line if possible)
3. Add helpful hints for common errors

**Files to Modify**:
- `src/error.rs` - Add error code field
- All error creation sites - Include codes

**Success Criteria**:
- [ ] All errors have codes
- [ ] Error messages reference spec sections
- [ ] Hints provided for common errors

---

## Summary & Timeline

### Priority-Ordered Implementation

**Week 1-2: Critical Analysis & Quick Wins**
1. ✅ Task 1.1: Analyze failures (1 hour)
2. ✅ Task 1.2: Map to spec violations (4 hours)
3. ✅ Task 2.1: Required attributes (2-3 days)
4. ✅ Task 2.2: Attribute ranges (2-3 days)
5. ✅ Task 2.3: Base materials (2-3 days)

**Week 3-4: Component Support**
6. ✅ Task 3.1: Parse components (3-4 days)
7. ✅ Task 3.2: Validate components (2-3 days)

**Week 5-6: XML & OPC Validation**
8. ✅ Task 4.1: Required elements (2-3 days)
9. ✅ Task 4.2: XML schema (3-4 days)
10. ✅ Task 5.1: Relationships (2-3 days)

**Week 7-8: Polish & Verification**
11. ✅ Task 5.2: Thumbnails (1-2 days)
12. ✅ Task 6.1: Metadata (1-2 days)
13. ✅ Task 7.2: Report generation (1-2 days)
14. ✅ Task 8.1: Error messages (2-3 days)

**Ongoing**: Task 7.1 - Test after each implementation

### Expected Outcomes by Phase

| Phase | Expected Negative Test Pass Rate |
|-------|-----------------------------------|
| Current | 1.7% (9/543) |
| After Phase 2 | 30-50% (160-270/543) |
| After Phase 3 | 60-75% (325-407/543) |
| After Phase 4 | 80-90% (434-489/543) |
| After Phase 5-6 | 95-100% (516-543/543) |

### Total Effort Estimate

- **Analysis**: 1 day
- **Implementation**: 6-8 weeks
- **Testing**: Ongoing
- **Documentation**: 1 week

**Total**: 8-10 weeks for 100% conformance

### Success Metrics

Final targets:
- ✅ **Positive Tests**: 100% (1,698/1,698) - MAINTAIN
- ✅ **Negative Tests**: 100% (543/543) - FIX 534 FAILURES
- ✅ **Overall**: 100% (2,241/2,241)

---

## Notes

- Start with Phase 1 (Analysis) - CRITICAL to understand actual failure patterns
- Quick wins in Phase 2 will show significant improvement
- Component support (Phase 3) is likely responsible for many failures
- Maintain positive test pass rate throughout - no regressions allowed
- Track progress after each task using analysis examples
- Some test failures may be fixed by earlier tasks than planned
- Adjust priorities based on actual failure analysis results

---

**Document Version**: 1.0  
**Created**: January 23, 2026  
**Status**: Ready for Implementation
