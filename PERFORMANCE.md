# Performance Characteristics

This document describes the performance characteristics of the lib3mf parser and provides guidance on optimizing performance for large 3MF files.

## Benchmarks

The library includes comprehensive benchmarks using [criterion.rs](https://github.com/bheisler/criterion.rs) that measure parsing performance across different file sizes.

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench -- parse_small
cargo bench -- parse_medium
cargo bench -- parse_large

# Run benchmarks for real files
cargo bench -- parse_real_files
```

### Benchmark Results

Performance scales approximately linearly with the number of vertices and triangles:

- **Small files** (100-1,000 vertices): ~190 µs - 1.3 ms
- **Medium files** (5,000-10,000 vertices): ~3.5 ms - 7 ms
- **Large files** (50,000-100,000 vertices): ~35 ms - 70 ms

These benchmarks are run on the GitHub Actions CI/CD environment. Actual performance will vary based on your hardware.

## Memory Usage

The parser uses streaming XML parsing to minimize memory overhead. Key memory characteristics:

### Memory Allocations

The parser optimizes memory allocations through several techniques:

1. **Pre-allocated buffers**: XML parsing buffer is pre-allocated with 4KB capacity
2. **Attribute HashMaps**: Pre-allocated with reasonable capacity (8 elements) to reduce reallocations
3. **Mesh vectors**: Support pre-allocation via `Mesh::with_capacity()` when counts are known
4. **Edge validation**: Manifold validation pre-allocates HashMap with capacity based on triangle count

### Typical Memory Footprint

For a 3MF file with N vertices and M triangles:

- **Vertex storage**: ~24 bytes per vertex (3 × f64)
- **Triangle storage**: ~56 bytes per triangle (3 × usize indices + optional material references)
- **Validation overhead**: ~16-24 bytes per edge during manifold validation
- **XML parsing**: ~4-8KB for internal buffers

**Example**: A file with 100,000 vertices and 50,000 triangles will use approximately:
- Vertices: 2.4 MB
- Triangles: 2.8 MB
- Validation: ~2.4 MB (temporary, during validation only)
- **Total**: ~5-8 MB during parsing

## Optimization Strategies

### For Application Developers

If you're parsing many 3MF files or very large files, consider these optimizations:

#### 1. Reuse Readers

Avoid creating new file handles repeatedly:

```rust
// Good - reuse file handle
let file = File::open("model.3mf")?;
let model = Model::from_reader(file)?;

// Avoid - multiple opens
for _ in 0..100 {
    let model = Model::from_reader(File::open("model.3mf")?)?;
}
```

#### 2. Use Appropriate Parser Configuration

If you don't need extension validation, use the default parser which accepts all known extensions:

```rust
// Fast - accepts all extensions
let model = Model::from_reader(file)?;

// Slower - validates against specific extensions
let config = ParserConfig::new().with_extension(Extension::Material);
let model = Model::from_reader_with_config(file, config)?;
```

#### 3. Consider Parallel Processing

For batch processing multiple files, parse them in parallel:

```rust
use rayon::prelude::*;

let files: Vec<_> = /* your file paths */;
let models: Vec<_> = files.par_iter()
    .map(|path| Model::from_reader(File::open(path)?))
    .collect();
```

### For Library Contributors

If you're contributing to the library, keep these performance considerations in mind:

#### Hot Paths

The following functions are called most frequently and should be optimized:

1. `parse_vertex()` - Called once per vertex (can be millions of times)
2. `parse_triangle()` - Called once per triangle (can be millions of times)
3. `parse_attributes()` - Called for every XML element
4. `validate_mesh_geometry()` - Validates all triangles and their indices

#### Optimization Techniques Applied

1. **Buffer pre-allocation**: XML reader buffer and HashMaps use `with_capacity()`
2. **Reduced allocations**: Attribute parsing reuses capacity hints
3. **Branch optimization**: Vertex validation combines checks for better branch prediction
4. **Capacity hints**: Mesh structures support pre-allocation when counts are known

#### Profiling

To profile the library's performance:

```bash
# Install flamegraph
cargo install flamegraph

# Run with profiling (requires perf on Linux)
cargo flamegraph --bench parse_benchmark -- --bench

# On macOS, use Instruments or cargo-instruments
cargo install cargo-instruments
cargo instruments -t time --bench parse_benchmark
```

## Streaming and Lazy Parsing

The current implementation uses streaming XML parsing but loads the entire model into memory. For extremely large files (>1GB), consider these future enhancements:

### Potential Improvements

1. **Streaming mesh access**: Iterator-based access to vertices and triangles without loading all into memory
2. **Partial model loading**: Load only specific objects or components on demand
3. **Memory-mapped parsing**: Use memory-mapped files for very large datasets
4. **Incremental validation**: Validate constraints incrementally during parsing

These features are not currently implemented but could be added in future versions if needed.

## Performance Testing

The benchmark suite includes several test scenarios:

- **Small files**: Basic performance regression testing
- **Medium files**: Typical production use cases
- **Large files**: Stress testing for memory and speed
- **Real files**: Validation against actual 3MF files from test suite

### Continuous Integration

Benchmarks are tracked in CI/CD to detect performance regressions. Criterion generates:

- Statistical analysis of performance changes
- HTML reports with graphs and comparisons
- Performance history tracking

## Recommendations

### For Small Files (<10,000 vertices)

Performance is typically not a concern. The parser will complete in under 10ms.

### For Medium Files (10,000-100,000 vertices)

- Expect parsing times of 10-100ms
- Memory usage will be 5-50MB
- No special optimizations needed

### For Large Files (>100,000 vertices)

- Parsing may take 100ms-1s
- Memory usage can reach 50-500MB
- Consider:
  - Profiling your specific use case
  - Implementing progressive loading if needed
  - Using parallel processing for batch operations

### For Very Large Files (>1M vertices)

- Consider chunking or streaming approaches
- Profile memory usage carefully
- May need custom optimizations for your use case
- Consider filing an issue to discuss optimization strategies

## Contributing Performance Improvements

When contributing performance improvements:

1. **Add benchmarks** for new features that might affect performance
2. **Profile before optimizing** to identify actual bottlenecks
3. **Measure the impact** using criterion's statistical analysis
4. **Document trade-offs** if optimization adds complexity
5. **Preserve correctness** - performance should not compromise validation

Performance contributions are welcome! Please include benchmark results in your pull request.
