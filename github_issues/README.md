# GitHub Issues - Organized by Category

This directory contains ready-to-use GitHub issue templates organized by category based on the current state assessment of lib3mf_rust (January 2026).

## Directory Structure

```
github_issues/
├── extension_support/     - Extension data extraction issues (3)
├── validation/            - Validation and conformance issues (3)
├── features/              - Feature enhancement issues (2)
├── testing/               - Testing and quality issues (2)
└── documentation/         - Documentation issues (1)
```

## Current State Summary

Based on analysis of the codebase as of January 21, 2026:

- **Parser**: ✅ Fully functional read-only implementation
- **Writer**: ❌ **NO SERIALIZATION SUPPORT**
- **Positive Test Conformance**: ✅ 100% (1,698/1,698)
- **Negative Test Conformance**: ⚠️ 1.7% (9/543) - **CRITICAL GAP**
- **Overall Conformance**: 76.2% (1,707/2,241)

## Issues by Category

### Extension Support (3 issues)

Extensions are recognized for validation but data not fully extracted:

1. **01_production_uuid_extraction.md** - Extract Production extension UUID and paths
2. **02_slice_stack_extraction.md** - Extract Slice extension stack data
3. **03_beam_lattice_extraction.md** - Extract Beam Lattice definitions

**Status**: Partial support - files parse but extension data not accessible

### Validation (3 issues)

Critical gaps in validation causing 534 invalid files to be accepted:

1. **01_negative_test_conformance.md** - ⚠️ **CRITICAL**: Improve from 1.7% to 90%+ (534 failing tests)
2. **02_base_materials.md** - Validate base materials (TODO in code)
3. **03_component_validation.md** - Validate component references and circular dependencies

**Status**: Major validation gap - highest priority category

### Features (2 issues)

Major missing features:

1. **01_writer_serialization.md** - ⚠️ **CRITICAL IF NEEDED**: Implement 3MF file writing/serialization
2. **02_advanced_materials.md** - Support textures, composites, multi-properties

**Status**: Read-only library - no writer support

### Testing & Quality (2 issues)

Improve testing and usability:

1. **01_conformance_report_generation.md** - Generate automated CONFORMANCE_REPORT.md
2. **02_error_messages.md** - Add error codes and better context

**Status**: Good foundation, needs polish

### Documentation (1 issue)

1. **01_migration_guide.md** - Create guide for C++ lib3mf users

**Status**: Basic documentation exists

## Priority Assessment

### Critical (If Goal is Full Compliance)
1. **Validation/01** - Negative test conformance (534 failures)
2. **Features/01** - Writer/serialization (if "parser AND writer" is goal)

### High Priority
3. **Validation/02** - Base materials validation (has TODO)
4. **Validation/03** - Component validation
5. **Extension_Support/01-03** - Complete extension data extraction

### Medium Priority
6. **Testing/01** - Conformance report generation
7. **Testing/02** - Error message improvements
8. **Features/02** - Advanced materials

### Low Priority
9. **Documentation/01** - Migration guide

## How to Use These Issues

### Option 1: Copy-Paste to GitHub
1. Open any `.md` file
2. Copy entire contents (including YAML frontmatter)
3. Go to https://github.com/telecos/lib3mf_rust/issues/new
4. Paste and submit

### Option 2: Use GitHub CLI
```bash
cd github_issues
gh issue create --body-file extension_support/01_production_uuid_extraction.md --repo telecos/lib3mf_rust
```

### Option 3: Bulk Creation
Create all issues at once using a script (see create_github_issues.md in project root).

## Issue Template Format

Each issue includes:
- YAML frontmatter (title, labels, assignees)
- Description of the gap
- Current state vs expected outcome
- Implementation notes and code examples
- Acceptance criteria
- Test files and references
- Priority and effort estimate
- Related issues

## Key Findings from Assessment

1. **NO WRITER SUPPORT** - Library is read-only
2. **534 invalid files accepted** - Major validation gap
3. **Extensions partially supported** - Recognized but data not extracted
4. **1 TODO in code** - Base materials validation at src/validator.rs:210
5. **Test infrastructure exists** - categorize_failures.rs, analyze_negative_tests.rs

## Recommended Implementation Order

### Phase 1 - Critical Validation
1. Negative test conformance (Validation/01)
2. Base materials validation (Validation/02)

### Phase 2 - Writer Support (If Needed)
3. Writer/serialization implementation (Features/01)

### Phase 3 - Extension Completeness
4. Production UUID extraction (Extension_Support/01)
5. Slice stack extraction (Extension_Support/02)
6. Beam lattice extraction (Extension_Support/03)

### Phase 4 - Component Support
7. Component validation (Validation/03)
8. Advanced materials (Features/02)

### Phase 5 - Polish
9. Conformance report generation (Testing/01)
10. Error messages (Testing/02)
11. Migration guide (Documentation/01)

## Notes

- All issues based on **current state analysis** (January 2026)
- Issues assume NO additional work has been completed beyond what's visible in code
- Writer support marked CRITICAL if "100% parser AND writer" is actual goal
- Total estimated effort: ~6-8 weeks for all issues
- Can be parallelized with multiple developers

---

**Assessment Date**: January 21, 2026  
**Total Issues**: 11  
**Critical**: 2 | **High**: 5 | **Medium**: 2 | **Low**: 2  
**Repository**: telecos/lib3mf_rust
