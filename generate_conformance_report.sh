#!/bin/bash
# Generate CONFORMANCE_REPORT.md from 3MF test suite results.
#
# This script runs the conformance tests and generates a detailed report
# with statistics and failure information.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Function to get suite description
get_suite_description() {
    local suite="$1"
    case "$suite" in
        "suite1_core_slice_prod")
            echo "Core + Production + Slice Extensions"
            ;;
        "suite2_core_prod_matl")
            echo "Core + Production + Materials Extensions"
            ;;
        "suite3_core")
            echo "Core Specification (Basic)"
            ;;
        "suite4_core_slice")
            echo "Core + Slice Extension"
            ;;
        "suite5_core_prod")
            echo "Core + Production Extension"
            ;;
        "suite6_core_matl")
            echo "Core + Materials Extension"
            ;;
        "suite7_beam")
            echo "Beam Lattice Extension"
            ;;
        "suite8_secure")
            echo "Secure Content Extension"
            ;;
        "suite9_core_ext")
            echo "Core Extensions"
            ;;
        "suite10_boolean")
            echo "Boolean Operations Extension"
            ;;
        "suite11_Displacement")
            echo "Displacement Extension"
            ;;
        *)
            echo "$suite"
            ;;
    esac
}

echo "============================================================"
echo "3MF Conformance Report Generator"
echo "============================================================"
echo

# Check if test_suites directory exists
if [ ! -d "test_suites" ]; then
    echo "ERROR: test_suites directory not found."
    echo "Please run './run_conformance_tests.sh' first to clone the test suites."
    exit 1
fi

# Create temporary file for test output
TEMP_OUTPUT=$(mktemp)
trap "rm -f $TEMP_OUTPUT" EXIT

echo "Running conformance tests (this may take several minutes)..."
echo

# Run tests and capture output
cargo test --test conformance_tests summary -- --ignored --nocapture > "$TEMP_OUTPUT" 2>&1

echo
echo "Generating report..."

# Extract results and generate markdown
cat > CONFORMANCE_REPORT.md << 'HEADER'
# 3MF Conformance Test Report

HEADER

# Add timestamp
echo "**Generated:** $(date -u '+%Y-%m-%d %H:%M:%S UTC')" >> CONFORMANCE_REPORT.md
echo "" >> CONFORMANCE_REPORT.md

# Add overall summary section
echo "## Overall Summary" >> CONFORMANCE_REPORT.md
echo "" >> CONFORMANCE_REPORT.md

# Extract overall conformance line
grep "Overall conformance:" "$TEMP_OUTPUT" >> CONFORMANCE_REPORT.md || echo "Results processing..." >> CONFORMANCE_REPORT.md
echo "" >> CONFORMANCE_REPORT.md

# Extract total line and format it
TOTAL_LINE=$(grep "^TOTAL" "$TEMP_OUTPUT" || echo "")
if [ -n "$TOTAL_LINE" ]; then
    # TOTAL line format: "TOTAL    Positive:  45/ 50  Negative:  30/ 35"
    # Field 1: TOTAL, Field 2: Positive:, Field 3: 45/, Field 4: 50, Field 5: Negative:, Field 6: 30/, Field 7: 35
    echo "$TOTAL_LINE" | awk '{
        # Remove trailing slash from field 3 and 6
        pos_passed = $3; sub(/\/$/, "", pos_passed)
        pos_total = $4
        neg_passed = $6; sub(/\/$/, "", neg_passed)
        neg_total = $7
        printf "- **Positive Tests:** %s/%s passed\n", pos_passed, pos_total
        printf "- **Negative Tests:** %s/%s passed\n", neg_passed, neg_total
    }' >> CONFORMANCE_REPORT.md || true
fi
echo "" >> CONFORMANCE_REPORT.md

# Add results by suite table
cat >> CONFORMANCE_REPORT.md << 'TABLE_HEADER'
## Results by Test Suite

| Suite | Description | Positive Tests | Negative Tests | Total |
|-------|-------------|----------------|----------------|-------|
TABLE_HEADER

# Process each suite line
grep -E "^suite[0-9]+_" "$TEMP_OUTPUT" | while read -r line; do
    suite=$(echo "$line" | awk '{print $1}')
    pos=$(echo "$line" | awk '{print $3}')
    # Negative value is split across fields 5 and 6 due to space (e.g., "20/ 80")
    neg=$(echo "$line" | awk '{print $5 $6}')  # Concatenate to get "20/80"
    
    # Get description using function
    desc=$(get_suite_description "$suite")
    
    # Parse pass/total for positive and negative
    pos_passed=$(echo "$pos" | cut -d'/' -f1 | tr -d ' ')
    pos_total=$(echo "$pos" | cut -d'/' -f2 | tr -d ' ')
    neg_passed=$(echo "$neg" | cut -d'/' -f1 | tr -d ' ')
    neg_total=$(echo "$neg" | cut -d'/' -f2 | tr -d ' ')
    
    # Calculate totals
    total_passed=$((pos_passed + neg_passed))
    total_tests=$((pos_total + neg_total))
    
    # Add emojis
    if [ "$pos_passed" = "$pos_total" ]; then
        pos_emoji="✅"
    elif [ "$pos_passed" -gt 0 ]; then
        pos_emoji="⚠️"
    else
        pos_emoji="❌"
    fi
    
    if [ "$neg_passed" = "$neg_total" ]; then
        neg_emoji="✅"
    elif [ "$neg_passed" -gt 0 ]; then
        neg_emoji="⚠️"
    else
        neg_emoji="❌"
    fi
    
    if [ "$total_passed" = "$total_tests" ]; then
        total_emoji="✅"
    elif [ "$total_passed" -gt 0 ]; then
        total_emoji="⚠️"
    else
        total_emoji="❌"
    fi
    
    echo "| $suite | $desc | $pos_emoji $pos_passed/$pos_total | $neg_emoji $neg_passed/$neg_total | $total_emoji $total_passed/$total_tests |" >> CONFORMANCE_REPORT.md
done

# Add detailed breakdown
cat >> CONFORMANCE_REPORT.md << 'DETAILED_HEADER'

## Detailed Suite Breakdown

DETAILED_HEADER

# Process detailed results for each suite
grep -E "^suite[0-9]+_" "$TEMP_OUTPUT" | while read -r line; do
    suite=$(echo "$line" | awk '{print $1}')
    pos=$(echo "$line" | awk '{print $3}')
    # Negative value is split across fields 5 and 6 due to space (e.g., "20/ 80")
    neg=$(echo "$line" | awk '{print $5 $6}')  # Concatenate to get "20/80"
    
    # Get description using function
    desc=$(get_suite_description "$suite")
    
    echo "### $suite" >> CONFORMANCE_REPORT.md
    echo "*$desc*" >> CONFORMANCE_REPORT.md
    echo "" >> CONFORMANCE_REPORT.md
    
    # Parse pass/total for positive and negative
    pos_passed=$(echo "$pos" | cut -d'/' -f1 | tr -d ' ')
    pos_total=$(echo "$pos" | cut -d'/' -f2 | tr -d ' ')
    neg_passed=$(echo "$neg" | cut -d'/' -f1 | tr -d ' ')
    neg_total=$(echo "$neg" | cut -d'/' -f2 | tr -d ' ')
    
    # Positive tests
    if [ "$pos_total" -gt 0 ]; then
        pos_rate=$(awk "BEGIN {printf \"%.1f\", ($pos_passed/$pos_total)*100}")
        if [ "$pos_passed" = "$pos_total" ]; then
            status="✅ All passed"
        else
            failed=$((pos_total - pos_passed))
            status="⚠️ $failed failed"
        fi
        echo "**Positive Tests:** $pos_passed/$pos_total ($pos_rate%) - $status" >> CONFORMANCE_REPORT.md
    else
        echo "**Positive Tests:** No tests found" >> CONFORMANCE_REPORT.md
    fi
    
    # Negative tests
    if [ "$neg_total" -gt 0 ]; then
        neg_rate=$(awk "BEGIN {printf \"%.1f\", ($neg_passed/$neg_total)*100}")
        if [ "$neg_passed" = "$neg_total" ]; then
            status="✅ All passed"
        else
            failed=$((neg_total - neg_passed))
            status="⚠️ $failed failed"
        fi
        echo "**Negative Tests:** $neg_passed/$neg_total ($neg_rate%) - $status" >> CONFORMANCE_REPORT.md
    else
        echo "**Negative Tests:** No tests found" >> CONFORMANCE_REPORT.md
    fi
    
    echo "" >> CONFORMANCE_REPORT.md
done

# Add footer
cat >> CONFORMANCE_REPORT.md << 'FOOTER'
---

## About This Report

This report is automatically generated by running the conformance test suite against the official 3MF Consortium test cases from [3MFConsortium/test_suites](https://github.com/3MFConsortium/test_suites).

**Test Methodology:**
- **Positive tests** validate that valid 3MF files parse successfully
- **Negative tests** validate that invalid 3MF files are properly rejected

**How to Regenerate:**
```bash
./generate_conformance_report.sh
```

FOOTER

echo "✅ Report generated: CONFORMANCE_REPORT.md"
echo

# Print summary from temp file
echo "============================================================"
grep "Overall conformance:" "$TEMP_OUTPUT" || echo "Processing complete"
echo "============================================================"
