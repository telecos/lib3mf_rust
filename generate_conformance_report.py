#!/usr/bin/env python3
"""
Generate CONFORMANCE_REPORT.md from 3MF test suite results.

This script runs the conformance tests and generates a detailed report
with statistics and failure information.
"""

import subprocess
import sys
import json
import re
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Tuple


def run_conformance_tests() -> str:
    """Run the conformance test summary and return output."""
    print("Running conformance tests...")
    try:
        result = subprocess.run(
            ["cargo", "test", "--test", "conformance_tests", "summary", 
             "--", "--ignored", "--nocapture"],
            capture_output=True,
            text=True,
            timeout=600  # 10 minute timeout
        )
        # Return combined output (stdout + stderr contains test results)
        return result.stdout + result.stderr
    except subprocess.TimeoutExpired:
        print("ERROR: Test execution timed out")
        sys.exit(1)
    except Exception as e:
        print(f"ERROR: Failed to run tests: {e}")
        sys.exit(1)


def parse_test_output(output: str) -> Dict:
    """Parse the test output to extract statistics."""
    
    # Initialize results
    suites = {}
    
    # Pattern to match suite results
    # Example: "suite1_core_slice_prod   Positive:   5/  5  Negative:   3/  3"
    suite_pattern = re.compile(
        r'(\S+)\s+Positive:\s+(\d+)/\s*(\d+)\s+Negative:\s+(\d+)/\s*(\d+)'
    )
    
    # Pattern to match total line
    # Example: "TOTAL                     Positive:  45/ 50  Negative:  30/ 35"
    total_pattern = re.compile(
        r'TOTAL\s+Positive:\s+(\d+)/\s*(\d+)\s+Negative:\s+(\d+)/\s*(\d+)'
    )
    
    # Pattern to match overall conformance
    # Example: "Overall conformance: 92.5% (74/80)"
    conformance_pattern = re.compile(
        r'Overall conformance:\s+([\d.]+)%\s+\((\d+)/(\d+)\)'
    )
    
    total_stats = None
    overall_conformance = None
    
    for line in output.split('\n'):
        # Check for suite results
        suite_match = suite_pattern.search(line)
        if suite_match:
            suite_name = suite_match.group(1)
            pos_passed = int(suite_match.group(2))
            pos_total = int(suite_match.group(3))
            neg_passed = int(suite_match.group(4))
            neg_total = int(suite_match.group(5))
            
            suites[suite_name] = {
                'positive': {'passed': pos_passed, 'total': pos_total},
                'negative': {'passed': neg_passed, 'total': neg_total}
            }
        
        # Check for total
        total_match = total_pattern.search(line)
        if total_match:
            total_stats = {
                'positive': {
                    'passed': int(total_match.group(1)),
                    'total': int(total_match.group(2))
                },
                'negative': {
                    'passed': int(total_match.group(3)),
                    'total': int(total_match.group(4))
                }
            }
        
        # Check for overall conformance
        conf_match = conformance_pattern.search(line)
        if conf_match:
            overall_conformance = {
                'percentage': float(conf_match.group(1)),
                'passed': int(conf_match.group(2)),
                'total': int(conf_match.group(3))
            }
    
    return {
        'suites': suites,
        'total': total_stats,
        'overall': overall_conformance
    }


def get_suite_description(suite_name: str) -> str:
    """Get a human-readable description for each suite."""
    descriptions = {
        'suite1_core_slice_prod': 'Core + Production + Slice Extensions',
        'suite2_core_prod_matl': 'Core + Production + Materials Extensions',
        'suite3_core': 'Core Specification (Basic)',
        'suite4_core_slice': 'Core + Slice Extension',
        'suite5_core_prod': 'Core + Production Extension',
        'suite6_core_matl': 'Core + Materials Extension',
        'suite7_beam': 'Beam Lattice Extension',
        'suite8_secure': 'Secure Content Extension',
        'suite9_core_ext': 'Core Extensions',
        'suite10_boolean': 'Boolean Operations Extension',
        'suite11_Displacement': 'Displacement Extension'
    }
    return descriptions.get(suite_name, suite_name)


def generate_report(results: Dict) -> str:
    """Generate the markdown report content."""
    
    report = []
    
    # Header
    report.append("# 3MF Conformance Test Report")
    report.append("")
    report.append(f"**Generated:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S UTC')}")
    report.append("")
    
    # Overall Summary
    report.append("## Overall Summary")
    report.append("")
    
    if results['overall']:
        overall = results['overall']
        report.append(f"**Overall Conformance:** {overall['percentage']:.1f}% ({overall['passed']}/{overall['total']} tests passed)")
    
    if results['total']:
        total = results['total']
        pos = total['positive']
        neg = total['negative']
        report.append(f"- **Positive Tests:** {pos['passed']}/{pos['total']} passed ({pos['passed']/pos['total']*100 if pos['total'] > 0 else 0:.1f}%)")
        report.append(f"- **Negative Tests:** {neg['passed']}/{neg['total']} passed ({neg['passed']/neg['total']*100 if neg['total'] > 0 else 0:.1f}%)")
    
    report.append("")
    
    # Results by Suite
    report.append("## Results by Test Suite")
    report.append("")
    report.append("| Suite | Description | Positive Tests | Negative Tests | Total |")
    report.append("|-------|-------------|----------------|----------------|-------|")
    
    for suite_name, stats in sorted(results['suites'].items()):
        description = get_suite_description(suite_name)
        pos = stats['positive']
        neg = stats['negative']
        
        pos_status = f"{pos['passed']}/{pos['total']}"
        neg_status = f"{neg['passed']}/{neg['total']}"
        
        total_passed = pos['passed'] + neg['passed']
        total_tests = pos['total'] + neg['total']
        total_status = f"{total_passed}/{total_tests}"
        
        # Add emoji indicators
        pos_emoji = "✅" if pos['passed'] == pos['total'] else "⚠️" if pos['passed'] > 0 else "❌"
        neg_emoji = "✅" if neg['passed'] == neg['total'] else "⚠️" if neg['passed'] > 0 else "❌"
        total_emoji = "✅" if total_passed == total_tests else "⚠️" if total_passed > 0 else "❌"
        
        report.append(
            f"| {suite_name} | {description} | "
            f"{pos_emoji} {pos_status} | {neg_emoji} {neg_status} | "
            f"{total_emoji} {total_status} |"
        )
    
    report.append("")
    
    # Detailed Suite Breakdown
    report.append("## Detailed Suite Breakdown")
    report.append("")
    
    for suite_name, stats in sorted(results['suites'].items()):
        description = get_suite_description(suite_name)
        report.append(f"### {suite_name}")
        report.append(f"*{description}*")
        report.append("")
        
        pos = stats['positive']
        neg = stats['negative']
        
        # Positive tests
        if pos['total'] > 0:
            pos_rate = (pos['passed'] / pos['total']) * 100
            status = "✅ All passed" if pos['passed'] == pos['total'] else f"⚠️ {pos['total'] - pos['passed']} failed"
            report.append(f"**Positive Tests:** {pos['passed']}/{pos['total']} ({pos_rate:.1f}%) - {status}")
        else:
            report.append("**Positive Tests:** No tests found")
        
        # Negative tests
        if neg['total'] > 0:
            neg_rate = (neg['passed'] / neg['total']) * 100
            status = "✅ All passed" if neg['passed'] == neg['total'] else f"⚠️ {neg['total'] - neg['passed']} failed"
            report.append(f"**Negative Tests:** {neg['passed']}/{neg['total']} ({neg_rate:.1f}%) - {status}")
        else:
            report.append("**Negative Tests:** No tests found")
        
        report.append("")
    
    # Footer
    report.append("---")
    report.append("")
    report.append("## About This Report")
    report.append("")
    report.append("This report is automatically generated by running the conformance test suite against the official 3MF Consortium test cases from [3MFConsortium/test_suites](https://github.com/3MFConsortium/test_suites).")
    report.append("")
    report.append("**Test Methodology:**")
    report.append("- **Positive tests** validate that valid 3MF files parse successfully")
    report.append("- **Negative tests** validate that invalid 3MF files are properly rejected")
    report.append("")
    report.append("**How to Regenerate:**")
    report.append("```bash")
    report.append("python3 generate_conformance_report.py")
    report.append("```")
    report.append("")
    
    return '\n'.join(report)


def main():
    """Main entry point."""
    print("=" * 60)
    print("3MF Conformance Report Generator")
    print("=" * 60)
    print()
    
    # Check if test_suites directory exists
    if not Path("test_suites").exists():
        print("ERROR: test_suites directory not found.")
        print("Please run './run_conformance_tests.sh' first to clone the test suites.")
        sys.exit(1)
    
    # Run tests
    output = run_conformance_tests()
    print()
    
    # Parse results
    print("Parsing test results...")
    results = parse_test_output(output)
    
    if not results['suites']:
        print("ERROR: Failed to parse test results. Output:")
        print(output)
        sys.exit(1)
    
    print(f"Found results for {len(results['suites'])} test suites")
    print()
    
    # Generate report
    print("Generating report...")
    report_content = generate_report(results)
    
    # Write to file
    output_file = Path("CONFORMANCE_REPORT.md")
    output_file.write_text(report_content)
    
    print(f"✅ Report generated: {output_file}")
    print()
    
    # Print summary
    if results['overall']:
        overall = results['overall']
        print("=" * 60)
        print(f"Overall Conformance: {overall['percentage']:.1f}% ({overall['passed']}/{overall['total']})")
        print("=" * 60)


if __name__ == "__main__":
    main()
