# Automated Fuzzing Bug Reports

This document explains how the automated fuzzing bug reporting system works in lib3mf_rust.

## Overview

The fuzzing CI workflow automatically creates detailed GitHub issues when crashes are discovered during nightly fuzzing runs. This automation:

1. **Saves time** - No manual issue creation needed for fuzzing failures
2. **Provides consistency** - Every crash gets a standardized, detailed report
3. **Accelerates triage** - Pre-analysis helps developers quickly understand the severity and nature of crashes
4. **Prevents duplicates** - Automatically detects if the same crash has already been reported

## How It Works

### Workflow Execution

When the nightly fuzzing workflow runs (at 2 AM UTC daily):

1. **Fuzzing runs** for each target (5 minutes for quick tests, 1 hour for long runs)
2. **Crash detection** checks for artifacts in `fuzz/artifacts/<target>/`
3. **Crash analysis** runs the Python script to analyze the first crash found
4. **Issue creation** automatically creates a GitHub issue with full details
5. **Duplicate checking** prevents creating multiple issues for the same crash

### Analysis Script

The `.github/scripts/analyze_fuzz_crash.py` script performs:

#### 1. Crash Information Extraction
- File name, size, and SHA256 hash
- Used for identifying duplicate crashes

#### 2. Crash Reproduction
- Runs `cargo fuzz run <target> <artifact>` with a 10-second timeout
- Captures stderr output for analysis
- Marks the crash as reproduced if it fails or times out

#### 3. Crash Classification
Automatically identifies crash types:

| Crash Type | Severity | Examples |
|------------|----------|----------|
| Panic: Index Out of Bounds | Medium | Array access with invalid index |
| Panic: Unwrap on None/Err | Medium | Unhandled Result or Option |
| Panic: Integer Overflow | High | Arithmetic overflow in debug mode |
| Stack Overflow | High | Deep recursion or infinite recursion |
| Out of Memory | High | Excessive allocation |
| Timeout/Hang | Medium | Infinite loop or slow algorithm |
| Segmentation Fault | Critical | Memory safety violation (rare in safe Rust) |
| Undefined Behavior | Critical | UB detected (rare in safe Rust) |

#### 4. Stack Trace Extraction
- Parses stderr to extract Rust backtraces
- Includes up to 20 lines of relevant stack frames
- Falls back to the last 10 non-empty lines if no backtrace found

#### 5. Issue Content Generation
Creates a comprehensive issue with:
- Descriptive title with crash type and hash
- Priority badge (🔴 Critical, 🟠 High, 🟡 Medium, 🟢 Low)
- Crash details (type, target, artifact info)
- Stack trace (if available)
- Reproduction steps
- Investigation guidance specific to the crash type
- Suggested labels (bug, fuzzing, P0-P3, security)

### Duplicate Detection

The workflow prevents duplicate issues by:

1. **Querying open issues** with the `fuzzing` label
2. **Extracting crash hash** from the issue title
3. **Comparing hashes** to find matching crashes
4. **Adding comments** to existing issues instead of creating duplicates

Example:
```
[Fuzzing] Panic: Index Out of Bounds in fuzz_parse_3mf (abc12345)
                                                          ^^^^^^^^
                                                      8-char hash
```

If an issue with hash `abc12345` for `fuzz_parse_3mf` already exists:
- ✅ Add comment: "🔄 This crash was reproduced again in fuzzing run: <link>"
- ❌ Skip creating a new issue

### When Issues Are Created

| Trigger | Creates Issues? | Reason |
|---------|----------------|--------|
| Scheduled nightly run | ✅ Yes | Production fuzzing |
| Long continuous fuzzing | ✅ Yes | Extended production fuzzing |
| Manual workflow trigger | ❌ No | Intentional testing |
| Pull request checks | ❌ No | Development testing |

This prevents noise from intentional testing while ensuring production crashes are tracked.

## Issue Structure

### Title Format
```
[Fuzzing] <Crash Type> in <Target> (<8-char hash>)
```

Examples:
- `[Fuzzing] Panic: Index Out of Bounds in fuzz_parse_3mf (abc12345)`
- `[Fuzzing] Stack Overflow in fuzz_parse_with_extensions (def67890)`
- `[Fuzzing] Timeout/Hang in fuzz_xml_parser (12345678)`

### Body Sections

1. **Summary**
   - Priority badge
   - Crash type, target, artifact details
   - Quick overview of the issue

2. **Analysis**
   - AI-generated description of the crash
   - Severity assessment rationale

3. **Stack Trace** (if available)
   - Full backtrace from crash reproduction
   - Code locations in lib3mf

4. **Reproduction Steps**
   - Command to reproduce locally
   - Link to download artifact from workflow run

5. **Initial Investigation**
   - Crash-type-specific guidance
   - Suggested root cause analysis steps
   - Security impact assessment
   - Fix and test recommendations

6. **Labels**
   - Automatic label suggestions
   - Priority mapping (P0-P3)

7. **Artifact Information**
   - How to access the crash artifact
   - Retention period (30 days for regular, 90 for long runs)

### Auto-Applied Labels

Issues are automatically labeled with:

| Label | When Applied |
|-------|-------------|
| `bug` | Always |
| `fuzzing` | Always |
| `P0` | Critical severity |
| `P1` | High severity |
| `P2` | Medium severity |
| `P3` | Low severity |
| `security` | High or critical severity |

## Developer Workflow

When a fuzzing issue is created:

### 1. Initial Triage (5-10 minutes)

- [ ] Review the issue title and priority
- [ ] Read the automated analysis
- [ ] Check if this is a known issue or duplicate
- [ ] Assess if this is a real bug or a fuzzing artifact

### 2. Reproduction (10-15 minutes)

- [ ] Download the crash artifact from GitHub Actions
- [ ] Reproduce locally:
  ```bash
  cargo +nightly fuzz run <target> path/to/artifact
  ```
- [ ] Verify the crash is deterministic
- [ ] Enable full backtraces if needed:
  ```bash
  RUST_BACKTRACE=full cargo +nightly fuzz run <target> path/to/artifact
  ```

### 3. Root Cause Analysis (30-60 minutes)

- [ ] Follow the investigation guidance in the issue
- [ ] Identify the exact line of code causing the crash
- [ ] Understand the input that triggers the crash
- [ ] Determine if this is exploitable (DoS, etc.)

### 4. Fix Implementation (varies)

- [ ] Implement proper error handling
- [ ] Add input validation if needed
- [ ] Use safe arithmetic (checked_*, saturating_*)
- [ ] Add recursion limits for stack overflow
- [ ] Add size limits for OOM issues

### 5. Testing (15-30 minutes)

- [ ] Add regression test with the crash artifact:
  ```rust
  #[test]
  fn test_crash_abc12345() {
      let data = include_bytes!("artifacts/crash-abc12345");
      // Should not panic
      let _ = Model::from_reader(std::io::Cursor::new(data));
  }
  ```
- [ ] Run fuzzer to verify fix:
  ```bash
  cargo +nightly fuzz run <target> -- -max_total_time=300
  ```
- [ ] Run standard test suite:
  ```bash
  cargo test
  ```

### 6. Documentation

- [ ] Update CHANGELOG.md if this is a user-facing bug
- [ ] Document any new input validation rules
- [ ] Update security notes if this was a vulnerability

## Example Issue

Here's what an auto-generated issue looks like:

```markdown
## Fuzzing Crash Report

**Auto-generated by fuzzing CI** - This issue was automatically created when fuzzing discovered a crash.

### Summary

🟠 **HIGH** Priority

**Crash Type:** Stack Overflow  
**Fuzzing Target:** `fuzz_parse_with_extensions`  
**Artifact:** `crash-0a1b2c3d`  
**Artifact Hash:** `0a1b2c3d4e5f6789`  
**Artifact Size:** 1024 bytes

### Analysis

Deep recursion or infinite loop - DoS vulnerability

### Stack Trace

```
thread 'main' panicked at 'stack overflow', src/parser/components.rs:123:5
stack backtrace:
   0: rust_begin_unwind
   1: core::panicking::panic_fmt
   2: lib3mf::parser::parse_component
   3: lib3mf::parser::parse_component
   ...
```

### Reproduction Steps

1. Download the crash artifact from the GitHub Actions run
2. Run the fuzzer with the crash artifact:
   ```bash
   cargo +nightly fuzz run fuzz_parse_with_extensions path/to/crash-artifact
   ```

### Initial Investigation

**Automated Analysis Complete** - The following steps are suggested for manual investigation:

1. **Reproduce Locally:** 
   - Ensure you can reproduce the crash with the artifact
   - Check if the crash is deterministic

2. **Root Cause Analysis:**
   - Look for recursive function calls
   - Check for infinite recursion conditions
   - Consider adding recursion depth limits

3. **Security Impact:**
   - Assess if this is a Denial of Service (DoS) vulnerability
   - Check if arbitrary input can trigger the crash
   - Determine if this affects production use cases

4. **Fix and Test:**
   - Implement a fix with proper error handling
   - Add a regression test with the crash artifact
   - Run extended fuzzing to verify the fix

### Labels

This issue should be labeled with:
- `bug` - This is a defect
- `fuzzing` - Found by fuzzing
- `security` - If this is a DoS or security issue
- `P1` - High priority

### Artifact Information

The crash artifact has been uploaded to the GitHub Actions workflow run. Download it from the "Artifacts" section of the workflow run.

**Workflow Run:** https://github.com/telecos/lib3mf_rust/actions/runs/123456

---
*This issue was automatically generated by the fuzzing CI workflow. For more information, see `.github/workflows/fuzzing.yml`.*
```

## Configuration

### Disabling Auto-Issue Creation

To disable automatic issue creation (e.g., for testing):

1. **Remove the trigger**: Comment out `schedule:` in `.github/workflows/fuzzing.yml`
2. **Add condition**: Add `if: false` to the "Create issue for crash" step

### Customizing Analysis

The crash analysis script can be customized by editing `.github/scripts/analyze_fuzz_crash.py`:

- **Add new crash types**: Update `analyze_crash_type()` function
- **Change severity mapping**: Modify the severity logic
- **Customize issue template**: Edit `generate_issue_body()` function
- **Add more analysis**: Extend `run_crash_analysis()` function

### Testing the Automation

To test issue creation without running full fuzzing:

1. Create a test crash artifact:
   ```bash
   mkdir -p fuzz/artifacts/fuzz_parse_3mf
   echo "test" > fuzz/artifacts/fuzz_parse_3mf/crash-test
   ```

2. Run the analysis script:
   ```bash
   python3 .github/scripts/analyze_fuzz_crash.py fuzz_parse_3mf fuzz/artifacts/fuzz_parse_3mf/crash-test
   ```

3. Check the output in `/tmp/issue_output.json`

## Troubleshooting

### Issue Not Created

**Symptoms**: Fuzzing found a crash but no issue was created

**Possible Causes**:
1. Event is not a scheduled run (manual trigger or PR)
2. Duplicate issue already exists
3. Script failed to run

**Solution**:
- Check workflow logs for "Create issue for crash" step
- Verify the event type in workflow summary
- Check for existing issues with the same crash hash

### Incorrect Crash Analysis

**Symptoms**: Crash type or severity is wrong

**Possible Causes**:
1. New crash pattern not recognized
2. stderr parsing failed

**Solution**:
- Update `analyze_crash_type()` in the script
- Add new patterns to match the crash output
- Test with actual crash artifacts

### Duplicate Issues Created

**Symptoms**: Multiple issues for the same crash

**Possible Causes**:
1. Crash hash collision (very rare)
2. Deduplication logic broken

**Solution**:
- Check the deduplication logic in the workflow
- Manually close duplicate issues
- Update hash algorithm if needed

## Performance Impact

- **Analysis time**: 10-30 seconds per crash
- **Issue creation**: < 1 second
- **Total overhead**: Minimal (only on failures)
- **No impact on fuzzing**: Analysis runs after fuzzing completes

## Security Considerations

### Sensitive Information

The analysis script and issues should NOT contain:
- Full crash artifacts (may contain test data)
- Environment variables
- Secrets or credentials

### What IS Included

- Stack traces (code locations only)
- Crash type and severity
- Hash of the artifact (not the artifact itself)
- Links to private artifact storage (GitHub Actions)

### Artifact Access

- Artifacts are stored in GitHub Actions (requires repo access)
- Retention: 30 days (regular), 90 days (long runs)
- Only authenticated users can download

## Future Enhancements

Potential improvements to the system:

1. **AI-powered analysis** - Use LLM to analyze stack traces and suggest fixes
2. **Historical tracking** - Track crash trends over time
3. **Bisection** - Automatically bisect to find the commit that introduced the crash
4. **Minimization** - Automatically minimize crash inputs
5. **Coverage reports** - Include coverage delta in issue reports
6. **Integration with security scanning** - Cross-reference with CodeQL alerts

## Related Documentation

- [Fuzzing README](../fuzz/README.md) - General fuzzing documentation
- [GitHub Actions Workflow](../.github/workflows/fuzzing.yml) - CI configuration
- [Crash Analysis Script](../.github/scripts/analyze_fuzz_crash.py) - Implementation
- [Contributing Guide](../CONTRIBUTING.md) - How to contribute fixes

## Questions?

If you have questions about the automated fuzzing reports:
1. Check this documentation first
2. Review existing fuzzing issues for examples
3. Ask in GitHub Discussions or create an issue with the `question` label
