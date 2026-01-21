# Conformance Testing - Quick Reference

## Running Tests

### Local Development

```bash
# Fast: Unit & integration tests (~7s)
cargo test --lib --test integration_test --test test_real_files

# Medium: Summary of all conformance tests (~10min)
cargo test --test conformance_tests summary -- --ignored --nocapture

# Detailed: Specific suite with file-by-file results (~2min)
cargo test --test conformance_parameterized suite3_core_positive -- --nocapture
```

### First Time Setup

```bash
# Clone test suites (1.6GB)
git clone https://github.com/3MFConsortium/test_suites.git

# Or use the setup script
./run_conformance_tests.sh
```

## Generating Conformance Report

Generate CONFORMANCE_REPORT.md with detailed test results:

```bash
# Using bash script (recommended)
./generate_conformance_report.sh

# Or using Python script (alternative)
python3 generate_conformance_report.py
```

**Note:** Report generation takes 10+ minutes as it runs all conformance tests across all suites.

The generated report includes:
- Overall conformance percentage
- Pass/fail statistics by suite
- Positive vs negative test breakdowns
- Detailed results for each test suite

## GitHub Actions

### View Status
- Repository → Actions tab
- Select workflow run
- View logs and download artifacts

### Manual Trigger
1. Actions → Conformance Tests
2. "Run workflow" button
3. Select branch → Run

### CLI Trigger
```bash
gh workflow run conformance.yml
```

## Test Files

| File | Purpose | Speed | When to Use |
|------|---------|-------|-------------|
| `conformance_tests.rs` | Summary of all suites | ~10min | Quick validation |
| `conformance_parameterized.rs` | Detailed per-file | ~2min/suite | Debugging |
| `integration_test.rs` | Basic integration | ~1s | Every commit |
| `test_real_files.rs` | Real file samples | ~7s | Every commit |

## Common Commands

```bash
# Check formatting
cargo fmt -- --check

# Run linter
cargo clippy -- -D warnings

# Build release
cargo build --release

# Run all standard tests
cargo test --lib --test integration_test --test test_real_files

# Run detailed suite tests
cargo test --test conformance_parameterized -- --nocapture

# Run conformance summary
cargo test --test conformance_tests summary -- --ignored --nocapture
```

## CI/CD Workflows

### CI Workflow (Fast)
- **Triggers**: Every push/PR
- **Duration**: ~2-5 minutes
- **Purpose**: Essential checks
- **Tests**: Unit, integration, lint, format

### Conformance Workflow (Comprehensive)
- **Triggers**: Push/PR/Manual
- **Duration**: ~5-15 minutes
- **Purpose**: Full validation
- **Tests**: All 2,241 conformance cases

## Artifacts

Conformance workflow creates:
- **Name**: conformance-report
- **Location**: Actions run → Artifacts
- **Retention**: 30 days
- **Contents**: Detailed test results

## Caching

Test suites (1.6GB) are cached in CI:
- First run: ~2 minutes to download
- Cached runs: ~5 seconds to restore
- Cache key: OS + workflow file hash

## Documentation

- [CONFORMANCE_TESTING.md](docs/CONFORMANCE_TESTING.md) - General guide
- [PARAMETERIZED_TESTING.md](docs/PARAMETERIZED_TESTING.md) - Detailed tests
- [.github/workflows/README.md](.github/workflows/README.md) - CI/CD setup
- [CONFORMANCE_REPORT.md](CONFORMANCE_REPORT.md) - Current results

## Troubleshooting

**"test_suites not found"**
```bash
git clone https://github.com/3MFConsortium/test_suites.git
```

**CI failing on conformance**
- Check Actions → Workflow run → Logs
- Download artifact for detailed report
- Cache may need refreshing

**Tests timing out**
- Normal for conformance tests
- Can take 10-15 minutes for full suite
- Use detailed tests for specific suites

## Current Status

✅ 100% positive tests passing (1,698/1,698)
✅ 33.8% negative tests passing (160/473)
📊 77.4% overall conformance (1,858/2,400)

Last updated: January 21, 2026
