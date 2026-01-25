#!/usr/bin/env python3
"""
Script to migrate expected_failures.json to the new format that supports
multiple suites per test case ID.
"""

import json
import re
from collections import defaultdict

def extract_test_case_id(filename):
    """Extract test case ID from filename.
    
    E.g., "P_XXX_0420_01.3mf" -> "0420_01"
    """
    # Remove .3mf extension
    without_ext = filename.replace('.3mf', '')
    # Split by underscore
    parts = without_ext.split('_')
    
    # Expected format: P/N _ PREFIX _ NNNN _ NN
    # e.g., P_XXX_0420_01 -> parts = ["P", "XXX", "0420", "01"]
    if len(parts) >= 4:
        # Join last two parts for test case ID
        return f"{parts[-2]}_{parts[-1]}"
    return None

def migrate_expected_failures(input_file, output_file):
    """Migrate expected failures to new format."""
    
    with open(input_file, 'r') as f:
        data = json.load(f)
    
    # Group failures by test_case_id, test_type, and similar reasons
    grouped = defaultdict(lambda: {
        'suites': [],
        'test_type': '',
        'reason': '',
        'issue_url': '',
        'date_added': '',
        'expected_error_type': None
    })
    
    # Track which entries couldn't be grouped (keep as-is)
    ungrouped = []
    
    for failure in data['expected_failures']:
        filename = failure.get('file', '')
        suite = failure.get('suite', '')
        test_case_id = extract_test_case_id(filename)
        
        if test_case_id:
            # Normalize reason slightly to help with grouping
            # (e.g., different wording for same issue)
            reason = failure['reason']
            expected_error_type = failure.get('expected_error_type')
            
            # For 0313_01, 0326_03, 0338_01 merge variations of same issue
            normalized_reason = reason
            if test_case_id == "0313_01" and "invalid content type for PNG" in reason:
                normalized_reason = "File contains invalid content type for PNG extension"
            elif test_case_id == "0326_03" and "zero determinant" in reason:
                normalized_reason = "Build item transform matrix with zero determinant"
            elif test_case_id == "0338_01" and "zero determinant" in reason:
                normalized_reason = "Build item transform matrix with zero determinant"
            elif test_case_id == "0418_01" and "Build transform bounds validation" in reason:
                normalized_reason = "Build transform bounds validation"
            
            # Create a key combining test_case_id, test_type, normalized reason, and expected_error_type
            key = (test_case_id, failure['test_type'], normalized_reason, expected_error_type)
            
            entry = grouped[key]
            entry['suites'].append(suite)
            entry['test_type'] = failure['test_type']
            # Use the longest/most detailed reason
            if len(reason) > len(entry['reason']):
                entry['reason'] = reason
            else:
                entry['reason'] = entry['reason'] or reason
            entry['issue_url'] = failure.get('issue_url', '')
            # Use the earliest date
            if entry['date_added']:
                entry['date_added'] = min(entry['date_added'], failure.get('date_added', ''))
            else:
                entry['date_added'] = failure.get('date_added', '')
            if expected_error_type:
                entry['expected_error_type'] = expected_error_type
        else:
            # Couldn't extract test case ID, keep as old format
            ungrouped.append(failure)
    
    # Build new expected_failures list
    new_failures = []
    
    # Add grouped entries (new format)
    for (test_case_id, test_type, normalized_reason, expected_error_type), entry in sorted(grouped.items()):
        new_entry = {
            'test_case_id': test_case_id,
            'suites': sorted(set(entry['suites'])),  # Remove duplicates and sort
            'test_type': entry['test_type'],
            'reason': entry['reason'],
            'issue_url': entry['issue_url'],
            'date_added': entry['date_added']
        }
        if entry['expected_error_type']:
            new_entry['expected_error_type'] = entry['expected_error_type']
        new_failures.append(new_entry)
    
    # Add ungrouped entries (old format)
    for failure in ungrouped:
        new_failures.append(failure)
    
    # Create output data
    output_data = {
        'expected_failures': new_failures
    }
    
    with open(output_file, 'w') as f:
        json.dump(output_data, f, indent=2)
    
    # Print migration summary
    print(f"Migration complete!")
    print(f"  Original entries: {len(data['expected_failures'])}")
    print(f"  Grouped entries (new format): {len(grouped)}")
    print(f"  Ungrouped entries (old format): {len(ungrouped)}")
    print(f"  Total entries in new file: {len(new_failures)}")
    print(f"\nTest cases with multiple suites:")
    for (test_case_id, test_type, normalized_reason, expected_error_type), entry in sorted(grouped.items()):
        if len(set(entry['suites'])) > 1:
            print(f"  {test_case_id} ({test_type}): {sorted(set(entry['suites']))}")

if __name__ == '__main__':
    migrate_expected_failures(
        'tests/expected_failures.json',
        'tests/expected_failures_new.json'
    )
