# Performance Characteristics

This document describes the performance characteristics of the lib3mf parser and provides guidance on optimizing performance for large 3MF files.

## Parser Modes

The library provides two parsing modes to handle different use cases:

### Standard Parser (Default)

The standard parser loads the entire model into memory:

```rust
use lib3mf::Model;
use std::fs::File;

let file = File::open("model.3mf")?;
let model = Model::from_reader(file)?;
```

**Best for:**
- Small to medium files (<100K vertices)
- Random access to model data
- Applications that need the full model structure

### Streaming Parser (Memory Efficient)

The streaming parser processes objects one at a time without loading the entire model:

```rust
use lib3mf::streaming::StreamingParser;
use std::fs::File;

let file = File::open("large_model.3mf")?;
let mut parser = StreamingParser::new(file)?;

// Process objects one at a time
for object in parser.objects() {
    let object = object?;
    // Process object...
    // Previous objects can be dropped, freeing memory
}
```

**Best for:**
- Very large files (>100K vertices)
- Sequential processing workflows
- Memory-constrained environments
- Conversion or validation tools

**Memory savings:**
- Standard parser: Loads all objects at once
- Streaming parser: Loads one object at a time
- For a file with N objects, streaming uses ~1/N the memory

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

The library provides a streaming parser for processing very large files without loading everything into memory.

### When to Use Streaming

Use the streaming parser when:
- Files have >100K vertices or triangles
- Memory is constrained (<1GB available)
- Processing objects sequentially (conversion, validation, analysis)
- You don't need random access to all objects

Use the standard parser when:
- Files are small to medium (<100K vertices)
- You need random access to model data
- Memory is not a constraint
- Building data structures that reference the full model

### Streaming Example

```rust
use lib3mf::streaming::StreamingParser;
use std::fs::File;

let file = File::open("huge_model.3mf")?;
let mut parser = StreamingParser::new(file)?;

let mut total_vertices = 0;
let mut total_triangles = 0;

// Process each object one at a time
for result in parser.objects() {
    let object = result?;
    
    if let Some(ref mesh) = object.mesh {
        total_vertices += mesh.vertices.len();
        total_triangles += mesh.triangles.len();
        
        // Object is dropped here, freeing its memory
        // before the next object is loaded
    }
}

println!("Total: {} vertices, {} triangles", total_vertices, total_triangles);
```

### Memory Comparison

For a 3MF file with 10 objects, each with 100K vertices:

**Standard Parser:**
- Peak memory: ~240 MB (all objects in memory)
- Constant memory throughout processing

**Streaming Parser:**
- Peak memory: ~24 MB (one object at a time)
- 10x memory reduction
- Memory freed as objects are processed

### Streaming Limitations

The current streaming implementation:
- ✅ Iterates through objects one at a time
- ✅ Reduces memory footprint significantly
- ❌ Does not stream within a single object (all vertices/triangles of an object are loaded)
- ❌ Cannot access objects in random order
- ❌ Requires sequential processing

### Future Enhancements

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
