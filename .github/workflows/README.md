# GitHub Actions CI/CD Workflows

This directory contains GitHub Actions workflows for automated testing and conformance validation.

## Workflows

### 1. CI Workflow (`.github/workflows/ci.yml`)

**Triggers**: Every push and pull request to `main`/`develop`

**Purpose**: Fast, essential checks

**What it does**:
- ✅ Runs unit tests
- ✅ Runs integration tests  
- ✅ Runs clippy (linter)
- ✅ Checks code formatting
- ✅ Tests on multiple platforms (Linux, Windows, macOS)

**Runtime**: ~2-5 minutes

### 2. Security Audit Workflow (`.github/workflows/security-audit.yml`)

**Triggers**:
- Push/PR to `main`/`develop`
- Daily at 2:00 AM UTC (scheduled)
- Manual trigger via Actions tab

**Purpose**: Scan dependencies for known security vulnerabilities

**What it does**:
- ✅ Installs cargo-audit tool
- ✅ Scans all dependencies against RustSec Advisory Database
- ✅ Caches cargo-audit binary for faster runs
- ✅ Fails build if vulnerabilities found

**Runtime**: ~1-2 minutes (cached), ~3-5 minutes (first run)

**How to run locally**:
```bash
cargo install cargo-audit
cargo audit
```

### 3. Conformance Workflow (`.github/workflows/conformance.yml`)

**Triggers**: 
- Push/PR to `main`/`develop`
- Manual trigger via Actions tab
- Scheduled runs (optional)

**Purpose**: Validate parser against official 3MF test suites

**What it does**:
- ✅ Runs basic tests first (fast validation)
- ✅ Clones 3MF Consortium test suites once (~1.6GB)
- ✅ Caches test suites for faster subsequent runs
- ✅ Distributes test suites as artifact to all parallel jobs
- ✅ Runs 11 conformance suites in parallel (2,241 test cases)
- ✅ Generates conformance report
- ✅ Commits updated CONFORMANCE_REPORT.md (on push to main/develop)
- ✅ Uploads report as artifact

**Runtime**: ~10-15 minutes (first run), ~3-5 minutes (cached)

**Optimization**: Uses a dedicated `setup-test-suites` job to clone test suites once and distribute via artifacts, avoiding 11 redundant git clone operations.

## Test Suite Caching and Distribution

The conformance workflow optimizes test suite distribution using a two-tier approach:

1. **GitHub Actions Cache**: Persists test suites across workflow runs
2. **Artifact Distribution**: Shares test suites across parallel jobs within a single run

```yaml
# Step 1: Single job clones and caches test suites
setup-test-suites:
  steps:
    - name: Cache test suites
      uses: actions/cache@v4
      with:
        path: test_suites
        key: ${{ runner.os }}-test-suites-${{ hashFiles('.github/workflows/conformance.yml') }}
    
    - name: Clone (only if cache miss)
      if: steps.cache-test-suites.outputs.cache-hit != 'true'
      run: git clone --depth 1 https://github.com/3MFConsortium/test_suites.git
    
    - name: Upload artifact for parallel jobs
      uses: actions/upload-artifact@v4
      with:
        name: test-suites
        path: test_suites/

# Step 2: Parallel jobs download artifact (faster than git clone)
conformance:
  needs: setup-test-suites
  strategy:
    matrix:
      suite: [suite1, suite2, ..., suite11]
  steps:
    - name: Download test suites
      uses: actions/download-artifact@v4.1.3
      with:
        name: test-suites
        path: test_suites
```

**Benefits**:
- Test suites cloned only **once** per workflow run (not 11+ times)
- Artifact download (~30 seconds) much faster than git clone (~2-3 minutes)
- Reduces GitHub bandwidth usage
- Improves reliability (fewer network operations)

**Cache invalidation**: Cache is recreated when:
- The workflow file changes
- Manual cache clear via GitHub UI
- 7 days of inactivity (GitHub's default)

## Running Workflows Manually

### From GitHub UI:
1. Go to your repository
2. Click "Actions" tab
3. Select "Conformance Tests" workflow
4. Click "Run workflow" button
5. Choose branch and click "Run workflow"

### Using GitHub CLI:
```bash
gh workflow run conformance.yml
```

## Workflow Artifacts

The conformance workflow uploads a detailed report:

**Artifact name**: `conformance-report-combined`  
**Location**: Actions run → Artifacts section  
**Retention**: 30 days  
**Contents**: 
- Detailed conformance test results
- conformance-summary.md (test run output)
- CONFORMANCE_REPORT.md (formatted markdown report)
- suite-reports/ (individual suite results)

**Auto-commit**: On pushes to main/develop, the CONFORMANCE_REPORT.md is automatically committed back to the repository with the commit message "Update conformance report [skip ci]".

To download:
```bash
gh run download <run-id> -n conformance-report-combined
```

## Customizing Workflows

### Add More Platforms

Edit `.github/workflows/ci.yml`:

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, windows-latest, macos-latest, macos-14]  # Add more
```

### Change Test Suites Cloned

Edit `.github/workflows/conformance.yml`:

```yaml
- name: Clone 3MF test suites
  run: |
    # Use --depth 1 for faster clone (no history)
    git clone --depth 1 https://github.com/3MFConsortium/test_suites.git
    
    # Or clone specific branch
    git clone --depth 1 -b main https://github.com/3MFConsortium/test_suites.git
```

### Run Detailed Parameterized Tests

The conformance workflow includes an optional job for detailed testing:

```yaml
conformance-detailed:
  # Runs on manual trigger only by default
  if: github.event_name == 'workflow_dispatch'
  
  strategy:
    matrix:
      suite: [suite3_core, suite7_beam, suite10_boolean]
```

To enable on every run, remove the `if` condition.

## Troubleshooting

### Workflow fails with "test_suites not found"

Check cache restore step. If cache miss, ensure clone step runs:

```yaml
- name: Clone 3MF test suites
  if: steps.cache-test-suites.outputs.cache-hit != 'true'  # Only if not cached
  run: git clone --depth 1 https://github.com/3MFConsortium/test_suites.git
```

### Out of disk space

The test suites are large (1.6GB). If running on self-hosted runners, ensure adequate space:

```bash
df -h  # Check available space
```

Consider:
- Using shallow clone (`--depth 1`)
- Cleaning old caches
- Increasing runner disk size

### Tests timeout

Increase timeout in workflow:

```yaml
jobs:
  conformance:
    timeout-minutes: 30  # Default is usually 360
```

### Cache not being used

Check cache key. It should be stable across runs:

```yaml
key: ${{ runner.os }}-test-suites-v1  # Simpler key
```

## Best Practices

### For Development
- CI workflow runs on every commit (fast checks)
- Conformance runs periodically or manually (slower, comprehensive)

### For Release
- Ensure all conformance tests pass before release
- Run detailed parameterized tests for critical suites
- Check conformance report artifact

### For Contributors
- PR checks focus on CI workflow (fast feedback)
- Maintainers can manually trigger conformance for PRs
- Keep test suite cache updated

## Secrets and Variables

No secrets required for current setup. All dependencies are public.

If adding private test suites:
1. Create GitHub secret with access token
2. Reference in workflow: `${{ secrets.TEST_SUITE_TOKEN }}`

## Security

### Dependency Scanning

The security audit workflow runs daily to check for:
- Known vulnerabilities in dependencies (via RustSec Advisory Database)
- Unmaintained crates
- Yanked crates

**When vulnerabilities are found:**
1. The workflow will fail and create a notification
2. Review the advisory details in the workflow logs
3. Update affected dependencies if possible
4. If no fix is available, consider:
   - Finding an alternative dependency
   - Documenting the issue in SECURITY.md
   - Using `--ignore` flag if false positive

**To ignore a specific advisory:**
```yaml
- name: Run security audit
  run: cargo audit --ignore RUSTSEC-2023-0071
```

### Best Practices
- ✅ Keep dependencies up to date
- ✅ Monitor security audit workflow failures
- ✅ Review dependency tree for unnecessary dependencies
- ✅ Use `cargo-deny` for more granular control (optional)

## Monitoring

View workflow status:
- **Dashboard**: Repository → Actions tab
- **Badges**: Add to README.md
  ```markdown
  ![CI](https://github.com/username/repo/workflows/CI/badge.svg)
  ![Security Audit](https://github.com/username/repo/workflows/Security%20Audit/badge.svg)
  ![Conformance](https://github.com/username/repo/workflows/Conformance%20Tests/badge.svg)
  ```

## Future Enhancements

Potential additions:
- Schedule nightly conformance runs
- Matrix testing across Rust versions
- Performance benchmarking
- Code coverage reporting
- Automatic issue creation on conformance regression
- Integration with Dependabot for automated dependency updates

## See Also

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [actions/cache Documentation](https://github.com/actions/cache)
- [CONFORMANCE_TESTING.md](../docs/CONFORMANCE_TESTING.md)
- [PARAMETERIZED_TESTING.md](../docs/PARAMETERIZED_TESTING.md)
