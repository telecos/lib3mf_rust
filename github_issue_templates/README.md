# GitHub Issue Templates

This directory contains ready-to-use GitHub issue templates for the 20 issues identified in REMAINING_ISSUES.md.

## High Priority Issues

1. **issue_7_negative_test_conformance.md** - Improve validation to reject invalid files (HIGH PRIORITY)
2. **issue_8_base_materials.md** - Validate base materials references (GOOD FIRST ISSUE)
3. **issue_1_production_extension.md** - Extract Production extension data

## How to Use These Templates

### Option 1: Copy-Paste to GitHub
1. Open the template file
2. Copy the entire contents
3. Go to https://github.com/telecos/lib3mf_rust/issues/new
4. Paste the content
5. Submit the issue

### Option 2: Use GitHub CLI
```bash
gh issue create --body-file github_issue_templates/issue_7_negative_test_conformance.md --repo telecos/lib3mf_rust
```

## Template Format

Each template includes:
- YAML frontmatter with title, labels, and assignees
- Description of the problem
- Current state and expected outcome
- Implementation approach
- Acceptance criteria
- References and related issues

## Next Steps

After creating these high-priority issues, create the remaining 17 issues following the same pattern using the content from REMAINING_ISSUES.md.

See create_github_issues.md for the complete list and creation guide.
