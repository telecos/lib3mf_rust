# GitHub Issues Creation Guide

This document provides instructions and templates for creating GitHub issues from the REMAINING_ISSUES.md document.

## Overview

The REMAINING_ISSUES.md document contains 20 well-defined issues categorized by:
- Extension Support (6 issues)
- Validation & Conformance (5 issues)
- Feature Enhancement (4 issues)
- Testing & Quality (3 issues)
- Documentation (2 issues)

## How to Create Issues

### Option 1: Manual Creation via GitHub Web UI

1. Go to https://github.com/telecos/lib3mf_rust/issues
2. Click "New Issue"
3. Use the templates below for each issue
4. Add appropriate labels (see Labels section below)
5. Assign to milestone if applicable

### Option 2: Using GitHub CLI

If you have `gh` CLI installed, you can create issues programmatically:

```bash
# Example for Issue 7
gh issue create \
  --title "Improve Negative Test Conformance (1.7% → 100%)" \
  --body-file github_issue_templates/issue_7_negative_test_conformance.md \
  --label "enhancement,validation,high-priority" \
  --repo telecos/lib3mf_rust
```

### Option 3: Using GitHub API

Use the REST API to create issues programmatically. See `create_issues_script.sh` for automation.

## Recommended Labels

Create and use these labels for categorization:

- **Priority Labels:**
  - `priority:high` - Critical issues
  - `priority:medium` - Important but not urgent
  - `priority:low` - Nice to have

- **Category Labels:**
  - `extension-support` - Extension data extraction
  - `validation` - Validation and conformance
  - `feature` - New features
  - `testing` - Test coverage and quality
  - `documentation` - Docs and examples
  - `performance` - Performance improvements

- **Status Labels:**
  - `good first issue` - Good for newcomers
  - `help wanted` - Community help welcome
  - `blocked` - Waiting on dependencies

- **Effort Labels:**
  - `effort:small` - 1-2 days
  - `effort:medium` - 3-7 days
  - `effort:large` - 1-2 weeks
  - `effort:research` - Requires investigation

## Issue Templates

### Template for Extension Support Issues (Issues 1-6)

```markdown
## Description

[Copy description from REMAINING_ISSUES.md]

## Current State

[Copy current state bullets from REMAINING_ISSUES.md]

## Expected Outcome

[Copy expected outcome bullets from REMAINING_ISSUES.md]

## Implementation Notes

**Test Files Available:** [Yes/No, specify files]

**Reference Documentation:**
- [Copy references from REMAINING_ISSUES.md]

**Related Spec:** [Spec section if applicable]

## Acceptance Criteria

- [ ] Data structures added for extension elements
- [ ] Parser extracts extension data
- [ ] Data accessible via Model API
- [ ] Tests added/updated
- [ ] Documentation updated

## Related Issues

[Link to related issues if any]
```

### Template for Validation Issues (Issues 7-11)

```markdown
## Description

[Copy description from REMAINING_ISSUES.md]

## Current State

[Copy current state from REMAINING_ISSUES.md]

## Expected Outcome

[Copy expected outcome from REMAINING_ISSUES.md]

## Implementation Approach

[Copy approach section if available]

## Acceptance Criteria

- [ ] Validation rule implemented
- [ ] Negative tests updated
- [ ] Error messages clear and helpful
- [ ] Tests passing
- [ ] Documentation updated

## Test Files

[Specify test files or test suite]

## Related Issues

[Link to related issues]
```

## Quick Reference: Issue Mappings

| Issue # | Title | Priority | Effort | Labels |
|---------|-------|----------|--------|---------|
| 1 | Production Extension - Extract UUID Attributes | Medium | Medium | extension-support, priority:medium |
| 2 | Slice Extension - Extract Slice Stack Definitions | Medium | Medium | extension-support, priority:medium |
| 3 | Beam Lattice Extension - Extract Beam Definitions | Medium | Medium | extension-support, priority:medium |
| 4 | Secure Content Extension - Add Test Coverage | Low | Research | extension-support, priority:low |
| 5 | Boolean Operations Extension - Add Test Coverage | Low | Research | extension-support, priority:low |
| 6 | Displacement Extension - Add Test Coverage | Low | Research | extension-support, priority:low |
| 7 | Improve Negative Test Conformance (1.7% → 100%) | High | Large | validation, priority:high |
| 8 | Validate Base Materials References | Medium | Small | validation, priority:medium, good first issue |
| 9 | Validate Component References | Medium | Medium | validation, priority:medium |
| 10 | Validate Thumbnail References | Low | Small | validation, priority:low |
| 11 | Validate Metadata Requirements | Low | Small | validation, priority:low |
| 12 | Support Advanced Material Properties | Low | Medium | feature, priority:low |
| 13 | Support Custom Extensions | Low | Medium | feature, priority:low |
| 14 | Add Writing/Serialization Support | Low | Large | feature, priority:low |
| 15 | Improve Performance for Large Files | Low | Research | performance, priority:low |
| 16 | Add Conformance Report Generation | Medium | Small | testing, priority:medium |
| 17 | Improve Error Messages | Medium | Small | quality, priority:medium |
| 18 | Add Property-Based Testing | Low | Research | testing, priority:low |
| 19 | Create Migration Guide from lib3mf (C++) | Low | Small | documentation, priority:low |
| 20 | Add More Examples | Low | Medium | documentation, priority:low |

## Recommended Creation Order

### Batch 1 - High Priority Core Work
1. Issue 7 - Negative test conformance (High priority, foundational)
2. Issue 8 - Base materials validation (Medium, good first issue)
3. Issue 16 - Conformance report (Medium, helps track #7)

### Batch 2 - Extension Support
4. Issue 1 - Production extension
5. Issue 2 - Slice extension
6. Issue 3 - Beam lattice extension

### Batch 3 - Quality Improvements
7. Issue 17 - Error messages
8. Issue 9 - Component validation
9. Issue 10 - Thumbnail validation

### Batch 4 - Future Work
10-20. Remaining issues based on community interest

## Notes

- Each issue in REMAINING_ISSUES.md has complete context for implementation
- Issues reference specific code locations and test files where applicable
- Most issues have clear acceptance criteria
- Consider creating a project board to track progress
- Link issues to milestones for release planning

## Automation Script

A bash script to create all issues at once:

```bash
#!/bin/bash
# This script would parse REMAINING_ISSUES.md and create GitHub issues
# Requires: gh CLI installed and authenticated
# Usage: ./create_issues_script.sh

# Note: This is a template - actual implementation would parse the markdown
# and extract issue details programmatically

echo "Creating issues from REMAINING_ISSUES.md..."
echo "This is a placeholder - implement parsing logic as needed"
echo ""
echo "Consider creating issues manually or using GitHub API for batch creation"
```

---

**Created:** January 20, 2026
**For Repository:** telecos/lib3mf_rust
**Source:** REMAINING_ISSUES.md
