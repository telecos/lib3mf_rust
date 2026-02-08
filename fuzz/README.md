# Fuzzing Infrastructure for lib3mf_rust

This directory contains fuzzing targets for testing the lib3mf_rust library using libFuzzer through cargo-fuzz.

## Overview

Fuzzing is a powerful testing technique that automatically generates random inputs to find bugs, crashes, and security vulnerabilities. The fuzzing infrastructure tests various aspects of the 3MF parsing and processing pipeline.

## Fuzzing Targets

### 1. `fuzz_parse_3mf`
Tests the complete 3MF parsing pipeline with default configuration:
- ZIP/OPC container extraction
- XML parsing
- Model construction
- Core 3MF specification features

**Corpus:** Valid 3MF files from `test_files/` covering core features, materials, and components.

### 2. `fuzz_parse_with_extensions`
Tests 3MF parsing with all extensions enabled:
- Material extension
- Production extension
- Slice extension
- Beam Lattice extension
- Boolean Operations extension
- Displacement extension
- Secure Content extension

**Corpus:** All test files including extension-specific 3MF files.

### 3. `fuzz_xml_parser`
Tests the underlying XML parser (quick-xml) robustness:
- Malformed XML handling
- Edge cases in XML parsing
- XML entity handling

**Corpus:** Empty (structure-aware fuzzing).

### 4. `fuzz_mesh_validation`
Tests mesh operations and validation:
- Volume calculation
- AABB (Axis-Aligned Bounding Box) computation
- Vertex normal calculation
- Mesh slicing

**Corpus:** Empty (structure-aware fuzzing using Arbitrary trait).

## Prerequisites

### Install Rust Nightly

Fuzzing requires Rust nightly:

```bash
rustup install nightly
rustup default nightly
```

### Install cargo-fuzz

```bash
cargo install cargo-fuzz
```

## Running Fuzzers Locally

### Quick Test (5 seconds)

```bash
# Test a specific fuzzer
cargo fuzz run fuzz_parse_3mf -- -max_total_time=5

# Test with corpus
cargo fuzz run fuzz_parse_3mf fuzz/corpus/fuzz_parse_3mf -- -max_total_time=5
```

### Standard Run (5 minutes)

```bash
cargo fuzz run fuzz_parse_3mf -- -max_total_time=300
```

### Extended Run (1 hour)

```bash
cargo fuzz run fuzz_parse_3mf -- -max_total_time=3600
```

### Run All Fuzzers

```bash
for target in fuzz_parse_3mf fuzz_parse_with_extensions fuzz_xml_parser fuzz_mesh_validation; do
    echo "Running $target..."
    cargo fuzz run $target -- -max_total_time=60
done
```

## Understanding Results

### Successful Run
If no crashes are found, you'll see output like:
```
#12345: cov: 2345 ft: 5678 corp: 89 exec/s: 234 ...
```

- `cov`: Total edge coverage
- `ft`: Number of features found
- `corp`: Corpus size (number of interesting inputs)
- `exec/s`: Executions per second

### Crash Found
If a crash is found, artifacts will be saved to:
```
fuzz/artifacts/<target_name>/crash-<hash>
```

To reproduce a crash:
```bash
cargo fuzz run <target_name> fuzz/artifacts/<target_name>/crash-<hash>
```

## CI/CD Integration

Fuzzing runs automatically via GitHub Actions in `.github/workflows/fuzzing.yml`:

### Schedule
- **Nightly runs**: Every day at 2 AM UTC
  - Quick fuzzing (5 minutes per target)
  - Extended fuzzing (1 hour for main parsers)

### Automatic Bug Reporting

**New Feature:** When fuzzing discovers a crash, the CI workflow automatically:

1. **Analyzes the crash** using `.github/scripts/analyze_fuzz_crash.py`:
   - Reproduces the crash to capture stack traces
   - Classifies the crash type (panic, overflow, timeout, etc.)
   - Assesses severity (low, medium, high, critical)
   - Provides initial investigation guidance

2. **Creates a GitHub issue** with:
   - Descriptive title including crash type and hash
   - Priority badge based on severity
   - Stack trace and error details
   - Step-by-step reproduction instructions
   - Initial investigation checklist
   - Automatic labels: `bug`, `fuzzing`, and priority (`P0`-`P3`)
   - Link to the workflow run with the crash artifact

3. **Avoids duplicates** by:
   - Checking for existing open issues with the same crash hash
   - Adding a comment to existing issues if the crash reproduces
   - Only creating new issues for unique crashes

### Manual Trigger
You can manually trigger fuzzing from GitHub Actions with custom duration:
1. Go to Actions tab
2. Select "Fuzzing" workflow
3. Click "Run workflow"
4. Specify fuzzing time in seconds

**Note:** Manual triggers and PR checks do NOT create issues automatically - only scheduled nightly runs create issues. This prevents noise from intentional testing.

### PR Checks
Fuzzing runs on PRs that modify:
- `fuzz/**` - Fuzzing infrastructure
- `.github/workflows/fuzzing.yml` - Fuzzing workflow

## Corpus Management

### Initial Corpus
The corpus is seeded with valid 3MF files from `test_files/`:
- Core: box.3mf, sphere.3mf, cylinder.3mf, torus.3mf, cube_gears.3mf
- Material: kinect_scan.3mf
- Components: assembly.3mf
- Production: box_prod.3mf
- Slices: box_sliced.3mf
- Beam Lattice: pyramid.3mf

### Adding to Corpus
To add new interesting inputs:
```bash
cp my_interesting_file.3mf fuzz/corpus/fuzz_parse_3mf/
```

### Minimizing Corpus
To reduce corpus size while maintaining coverage:
```bash
cargo fuzz cmin fuzz_parse_3mf
```

## Troubleshooting

### Build Errors

If you see errors about sanitizers or unstable features:
- Ensure you're using nightly Rust: `rustup default nightly`
- Update rustup: `rustup update`

### Out of Memory

If fuzzing uses too much memory:
- Reduce corpus size: `cargo fuzz cmin <target>`
- Limit memory: `cargo fuzz run <target> -- -rss_limit_mb=2048`

### Slow Fuzzing

To improve fuzzing speed:
- Build in release mode (default)
- Use parallel fuzzing: `cargo fuzz run <target> -j <cores>`
- Reduce corpus size

## Advanced Options

### Dictionary-based Fuzzing
For XML fuzzing, create a dictionary of known tokens:
```bash
echo "<?xml" > fuzz/dict.txt
echo "<model" >> fuzz/dict.txt
cargo fuzz run fuzz_xml_parser -- -dict=fuzz/dict.txt
```

### Coverage Reports
Generate coverage information:
```bash
cargo fuzz coverage fuzz_parse_3mf
```

### Persistent Mode
For faster fuzzing of small targets:
```bash
cargo fuzz run <target> -- -len_control=0
```

## Security

Fuzzing is a critical security testing tool. All crashes found should be:
1. Investigated for security implications
2. Fixed before release
3. Added to regression tests

Report security issues privately to the maintainers.

### Crash Analysis Script

The `.github/scripts/analyze_fuzz_crash.py` script provides automated crash analysis:

```bash
# Analyze a crash artifact
python3 .github/scripts/analyze_fuzz_crash.py <target> <artifact_path>

# Example
python3 .github/scripts/analyze_fuzz_crash.py fuzz_parse_3mf fuzz/artifacts/fuzz_parse_3mf/crash-abc123
```

**Features:**
- Reproduces the crash to capture full error output
- Classifies crash type (panic, overflow, stack overflow, timeout, etc.)
- Assesses severity (low, medium, high, critical)
- Extracts stack traces automatically
- Provides investigation guidance specific to crash type
- Outputs JSON for CI integration

**Crash Types Detected:**
- Panic (index out of bounds, unwrap, overflow, etc.)
- Stack overflow (deep recursion)
- Out of memory (excessive allocation)
- Timeout/hang (infinite loops)
- Assertion failures
- Critical issues (segfaults, undefined behavior)

The script is automatically used by the CI workflow to generate detailed issue reports.

## Resources

- [cargo-fuzz book](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [libFuzzer documentation](https://llvm.org/docs/LibFuzzer.html)
- [3MF Specification](https://3mf.io/specification/)

## Maintenance

### Regular Tasks
- Review nightly fuzzing results
- Minimize corpus monthly
- Update fuzzers when new features are added
- Archive interesting crash cases

### When Adding New Features
1. Update relevant fuzzing targets
2. Add new corpus examples
3. Run extended fuzzing session (1+ hours)
4. Document new edge cases discovered
