#!/usr/bin/env python3
"""
Analyze fuzzing crash artifacts and generate detailed bug reports.

This script examines crash artifacts from cargo-fuzz and generates:
- Crash summary
- Stack trace analysis
- Reproduction steps
- Initial severity assessment
"""

import os
import sys
import json
import hashlib
import subprocess
from pathlib import Path
from typing import Dict, List, Optional, Tuple


def get_crash_info(artifact_path: Path) -> Dict[str, any]:
    """Extract information from a crash artifact."""
    info = {
        'file': artifact_path.name,
        'path': str(artifact_path),
        'size': artifact_path.stat().st_size,
        'hash': hashlib.sha256(artifact_path.read_bytes()).hexdigest()[:16],
    }
    return info


def analyze_crash_type(stderr_output: str) -> Tuple[str, str, str]:
    """
    Analyze crash output to determine crash type and severity.
    
    Returns: (crash_type, severity, description)
    """
    stderr_lower = stderr_output.lower()
    
    # Panic analysis
    if 'panic' in stderr_lower or 'panicked' in stderr_lower:
        crash_type = 'Panic'
        
        # Check for specific panic types
        if 'index out of bounds' in stderr_lower:
            return ('Panic: Index Out of Bounds', 'medium', 
                    'Array/vector access with invalid index - potential DoS')
        elif 'unwrap' in stderr_lower or 'expect' in stderr_lower:
            return ('Panic: Unwrap on None/Err', 'medium',
                    'Unhandled error case - potential DoS')
        elif 'overflow' in stderr_lower or 'underflow' in stderr_lower:
            return ('Panic: Integer Overflow/Underflow', 'high',
                    'Arithmetic overflow - potential security issue')
        elif 'slice' in stderr_lower:
            return ('Panic: Invalid Slice', 'medium',
                    'Slice operation on invalid range - potential DoS')
        else:
            return ('Panic: Unknown', 'medium',
                    'Unexpected panic - requires investigation')
    
    # Stack overflow
    elif 'stack overflow' in stderr_lower:
        return ('Stack Overflow', 'high',
                'Deep recursion or infinite loop - DoS vulnerability')
    
    # Out of memory
    elif 'out of memory' in stderr_lower or 'oom' in stderr_lower:
        return ('Out of Memory', 'high',
                'Excessive memory allocation - DoS vulnerability')
    
    # Timeout
    elif 'timeout' in stderr_lower or 'slow-unit' in stderr_lower:
        return ('Timeout/Hang', 'medium',
                'Excessive CPU usage or infinite loop - DoS risk')
    
    # Undefined behavior (shouldn't happen with safe Rust, but check anyway)
    elif 'undefined behavior' in stderr_lower or 'ub' in stderr_lower:
        return ('Undefined Behavior', 'critical',
                'Potential memory safety issue - CRITICAL')
    
    # Segmentation fault (shouldn't happen in safe Rust)
    elif 'segmentation fault' in stderr_lower or 'sigsegv' in stderr_lower:
        return ('Segmentation Fault', 'critical',
                'Memory safety violation - CRITICAL (check for unsafe code)')
    
    # Generic assertion failure
    elif 'assertion failed' in stderr_lower or 'assert' in stderr_lower:
        return ('Assertion Failure', 'low',
                'Failed assertion - logic error')
    
    else:
        return ('Unknown Crash', 'medium',
                'Unclassified crash - requires manual analysis')


def extract_stack_trace(stderr_output: str) -> Optional[str]:
    """Extract stack trace from crash output."""
    lines = stderr_output.split('\n')
    
    # Look for Rust backtrace
    trace_start = -1
    for i, line in enumerate(lines):
        if 'stack backtrace:' in line.lower() or 'at ' in line:
            trace_start = i
            break
    
    if trace_start >= 0:
        # Extract up to 20 lines of stack trace
        trace_lines = []
        for i in range(trace_start, min(trace_start + 20, len(lines))):
            line = lines[i].strip()
            if line and (line.startswith('at ') or '::' in line or 'lib3mf' in line):
                trace_lines.append(line)
            elif trace_lines and not line:
                # Empty line after trace started - probably end
                break
        
        if trace_lines:
            return '\n'.join(trace_lines)
    
    # If no backtrace found, return last 10 non-empty lines
    relevant_lines = [l for l in lines if l.strip()][-10:]
    return '\n'.join(relevant_lines) if relevant_lines else None


def run_crash_analysis(target: str, artifact_path: Path) -> Dict[str, any]:
    """
    Run the fuzzer with the crash artifact to capture detailed output.
    """
    analysis = {
        'reproduced': False,
        'stderr': '',
        'crash_type': 'Unknown',
        'severity': 'medium',
        'description': '',
        'stack_trace': None,
    }
    
    try:
        # Try to reproduce the crash
        result = subprocess.run(
            ['cargo', 'fuzz', 'run', target, str(artifact_path)],
            cwd=Path(__file__).parent.parent.parent,
            capture_output=True,
            text=True,
            timeout=10,
        )
        
        stderr = result.stderr or ''
        analysis['stderr'] = stderr
        analysis['reproduced'] = result.returncode != 0
        
        if stderr:
            crash_type, severity, description = analyze_crash_type(stderr)
            analysis['crash_type'] = crash_type
            analysis['severity'] = severity
            analysis['description'] = description
            analysis['stack_trace'] = extract_stack_trace(stderr)
        
    except subprocess.TimeoutExpired:
        analysis['crash_type'] = 'Timeout/Hang'
        analysis['severity'] = 'medium'
        analysis['description'] = 'Crash reproduction timed out - possible infinite loop'
        analysis['reproduced'] = True
    except Exception as e:
        print(f"Warning: Could not run crash analysis: {e}", file=sys.stderr)
    
    return analysis


def generate_issue_title(target: str, crash_info: Dict[str, any]) -> str:
    """Generate a descriptive issue title."""
    crash_type = crash_info.get('crash_type', 'Unknown Crash')
    hash_short = crash_info.get('hash', 'unknown')[:8]
    return f"[Fuzzing] {crash_type} in {target} ({hash_short})"


def generate_issue_body(target: str, crash_info: Dict[str, any], analysis: Dict[str, any]) -> str:
    """Generate detailed issue body with crash analysis."""
    
    severity = analysis.get('severity', 'medium')
    crash_type = analysis.get('crash_type', 'Unknown Crash')
    description = analysis.get('description', 'No description available')
    
    # Map severity to priority labels
    priority_map = {
        'critical': '🔴 **CRITICAL**',
        'high': '🟠 **HIGH**',
        'medium': '🟡 **MEDIUM**',
        'low': '🟢 **LOW**',
    }
    priority_badge = priority_map.get(severity, '🟡 **MEDIUM**')
    
    body = f"""## Fuzzing Crash Report

**Auto-generated by fuzzing CI** - This issue was automatically created when fuzzing discovered a crash.

### Summary

{priority_badge} Priority

**Crash Type:** {crash_type}  
**Fuzzing Target:** `{target}`  
**Artifact:** `{crash_info['file']}`  
**Artifact Hash:** `{crash_info['hash']}`  
**Artifact Size:** {crash_info['size']} bytes

### Analysis

{description}

"""
    
    # Add stack trace if available
    if analysis.get('stack_trace'):
        body += f"""### Stack Trace

```
{analysis['stack_trace']}
```

"""
    
    # Add reproduction steps
    body += f"""### Reproduction Steps

1. Download the crash artifact from the GitHub Actions run
2. Run the fuzzer with the crash artifact:
   ```bash
   cargo +nightly fuzz run {target} path/to/crash-artifact
   ```

"""
    
    # Add initial investigation guidance
    body += """### Initial Investigation

**Automated Analysis Complete** - The following steps are suggested for manual investigation:

1. **Reproduce Locally:** 
   - Ensure you can reproduce the crash with the artifact
   - Check if the crash is deterministic

2. **Root Cause Analysis:**
"""
    
    # Add specific guidance based on crash type
    if 'Panic: Index Out of Bounds' in crash_type:
        body += """   - Review array/vector indexing in the affected code path
   - Look for missing bounds checks
   - Consider if input validation is sufficient
"""
    elif 'Panic: Unwrap' in crash_type:
        body += """   - Identify the unwrap() or expect() call that panicked
   - Determine what error condition triggered it
   - Replace with proper error handling (?, Result, or match)
"""
    elif 'Overflow' in crash_type or 'Underflow' in crash_type:
        body += """   - Identify the arithmetic operation that overflowed
   - Use checked arithmetic (checked_add, checked_mul, etc.)
   - Validate input ranges before arithmetic
"""
    elif 'Stack Overflow' in crash_type:
        body += """   - Look for recursive function calls
   - Check for infinite recursion conditions
   - Consider adding recursion depth limits
"""
    elif 'Out of Memory' in crash_type:
        body += """   - Identify large allocations
   - Check for unbounded data structures
   - Add input size validation
"""
    elif 'Timeout' in crash_type:
        body += """   - Profile the code to find hot spots
   - Look for infinite loops or excessive iterations
   - Consider adding complexity limits
"""
    else:
        body += """   - Examine the stack trace to locate the crash
   - Review the code path leading to the failure
   - Check for logic errors or invalid state
"""
    
    body += """
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
"""
    
    # Add severity-specific labels
    if severity == 'critical':
        body += "- `P0` - Critical priority\n"
    elif severity == 'high':
        body += "- `P1` - High priority\n"
    elif severity == 'medium':
        body += "- `P2` - Medium priority\n"
    else:
        body += "- `P3` - Low priority\n"
    
    body += """
### Artifact Information

The crash artifact has been uploaded to the GitHub Actions workflow run. Download it from the "Artifacts" section of the workflow run.

---
*This issue was automatically generated by the fuzzing CI workflow. For more information, see `.github/workflows/fuzzing.yml`.*
"""
    
    return body


def main():
    """Main entry point for crash analysis."""
    if len(sys.argv) < 3:
        print("Usage: analyze_fuzz_crash.py <target> <artifact_path>", file=sys.stderr)
        sys.exit(1)
    
    target = sys.argv[1]
    artifact_path = Path(sys.argv[2])
    
    if not artifact_path.exists():
        print(f"Error: Artifact not found: {artifact_path}", file=sys.stderr)
        sys.exit(1)
    
    print(f"Analyzing crash artifact: {artifact_path}")
    
    # Get basic crash info
    crash_info = get_crash_info(artifact_path)
    
    # Run detailed analysis
    print(f"Running crash reproduction for {target}...")
    analysis = run_crash_analysis(target, artifact_path)
    
    # Generate issue content
    title = generate_issue_title(target, {**crash_info, **analysis})
    body = generate_issue_body(target, crash_info, analysis)
    
    # Output as JSON for GitHub Actions
    output = {
        'title': title,
        'body': body,
        'labels': ['bug', 'fuzzing'],
        'severity': analysis.get('severity', 'medium'),
        'crash_type': analysis.get('crash_type', 'Unknown'),
    }
    
    # Add priority label
    severity_to_priority = {
        'critical': 'P0',
        'high': 'P1',
        'medium': 'P2',
        'low': 'P3',
    }
    priority_label = severity_to_priority.get(analysis.get('severity', 'medium'))
    if priority_label:
        output['labels'].append(priority_label)
    
    # Add security label for high/critical severity
    if analysis.get('severity') in ['high', 'critical']:
        output['labels'].append('security')
    
    print("\n=== ISSUE CONTENT ===")
    print(json.dumps(output, indent=2))
    
    # Also write to file for GitHub Actions to consume
    output_file = Path(os.environ.get('GITHUB_OUTPUT', '/tmp/issue_output.json'))
    if output_file.parent.exists() or output_file == Path('/tmp/issue_output.json'):
        # Write each field as GitHub Actions output
        if str(output_file) != '/tmp/issue_output.json':
            with output_file.open('a') as f:
                f.write(f"issue_title<<EOF\n{output['title']}\nEOF\n")
                f.write(f"issue_body<<EOF\n{output['body']}\nEOF\n")
                f.write(f"issue_labels={','.join(output['labels'])}\n")
                f.write(f"severity={output['severity']}\n")
        
        # Also write full JSON for reference
        json_file = Path('/tmp/issue_output.json')
        json_file.write_text(json.dumps(output, indent=2))
        print(f"\nIssue data written to {json_file}")
    
    return 0


if __name__ == '__main__':
    sys.exit(main())
