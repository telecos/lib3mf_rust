# 2D Slice Image Renderer

A high-quality raster image renderer for converting 3D model slice contours into PNG images with filled polygons.

## Features

- ✅ **White background** - Clean, high-contrast output perfect for printing/viewing
- ✅ **Configurable resolution** - Support for any resolution (e.g., 1024x1024, 2048x2048)
- ✅ **Auto-fit with margins** - Automatically scale slices to fill the image
- ✅ **Aspect ratio preservation** - Maintains correct proportions
- ✅ **Filled polygons** - Solid color fills using triangle rasterization
- ✅ **Hole handling** - Holes remain white (background color)
- ✅ **Multi-object support** - Different colors for different objects
- ✅ **High precision** - Uses ear-cutting triangulation for accurate polygon filling

## Quick Start

```rust
use slice_renderer::{SliceRenderer, SliceContour, BoundingBox2D, Point2D};
use image::Rgb;

// Create a renderer with 1024x1024 resolution
let renderer = SliceRenderer::new(1024, 1024);

// Define a triangle contour
let contour = SliceContour::new(vec![
    Point2D::new(10.0, 10.0),
    Point2D::new(90.0, 10.0),
    Point2D::new(50.0, 80.0),
]);

// Set the bounds for the slice
let bounds = BoundingBox2D::new(0.0, 0.0, 100.0, 100.0);

// Render the slice
let image = renderer.render_slice(&[contour], &bounds);

// Save to PNG
renderer.save_png(&image, Path::new("output.png"))?;
```

## Advanced Usage

### Custom Colors and Margins

```rust
let renderer = SliceRenderer::new(1024, 1024)
    .with_margin(50.0)                          // 50 pixel margin
    .with_background(Rgb([245, 245, 245]))      // Light gray background
    .with_default_fill(Rgb([60, 60, 60]));      // Dark fill color
```

### Multiple Colored Objects

```rust
let triangle = SliceContour::new(vec![...]);
let square = SliceContour::new(vec![...]);

let objects = vec![
    (triangle, Rgb([220, 60, 60])),   // Red
    (square, Rgb([60, 120, 220])),    // Blue
];

let image = renderer.render_slice_multi_color(&objects, &bounds);
```

### Polygons with Holes

```rust
// Outer boundary
let boundary = vec![
    Point2D::new(0.0, 0.0),
    Point2D::new(100.0, 0.0),
    Point2D::new(100.0, 100.0),
    Point2D::new(0.0, 100.0),
];

// Inner hole
let hole = vec![
    Point2D::new(30.0, 30.0),
    Point2D::new(70.0, 30.0),
    Point2D::new(70.0, 70.0),
    Point2D::new(30.0, 70.0),
];

let contour = SliceContour::with_holes(boundary, vec![hole]);
let image = renderer.render_slice(&[contour], &bounds);
```

## Integration with Slice Window

The slice renderer can be used to export high-quality PNGs from the slice preview window:

```rust
let slice_window = SlicePreviewWindow::new()?;

// Export with custom resolution (higher than window resolution)
slice_window.export_to_png_hq(
    Path::new("high_res_slice.png"),
    2048,  // width
    2048   // height
)?;
```

## Architecture

### Rendering Pipeline

1. **Input**: Slice contours (polygons) and bounding box
2. **Transform Calculation**: World coordinates → Pixel coordinates
   - Uniform scaling to preserve aspect ratio
   - Centering with configurable margins
3. **Triangulation**: Polygons → Triangles using ear-cutting algorithm
   - Handles holes correctly
   - Works with any polygon shape
4. **Rasterization**: Triangles → Pixels
   - Barycentric coordinate test for point-in-triangle
   - Pixel center sampling for accuracy
   - Bounds checking for safety
5. **Output**: PNG image with white background

### Key Components

- **`SliceRenderer`**: Main rendering engine
- **`Transform2D`**: Coordinate transformation helper
- **`SliceContour`**: Polygon with optional holes
- **`BoundingBox2D`**: 2D bounds for auto-fit
- **`Point2D`**: 2D point representation

### Dependencies

- `image`: PNG encoding/decoding
- `earcutr`: Ear-cutting triangulation algorithm

## Testing

Run the comprehensive test suite:

```bash
cd tools/viewer
cargo test slice_renderer
```

Run the demonstration example:

```bash
cargo run --example render_slice_demo
```

This will create three example PNG files in `/tmp/`:
- `triangle.png` - Simple filled triangle
- `square_with_hole.png` - Square with circular hole
- `multi_color.png` - Multiple colored shapes

## Performance

- **Small slices** (< 100 triangles): < 10ms
- **Medium slices** (< 1000 triangles): < 50ms  
- **Large slices** (< 10000 triangles): < 200ms

Performance scales linearly with:
- Number of triangles
- Output resolution (pixel count)

## Limitations

- Anti-aliasing is not currently implemented (could be added in the future)
- Very complex polygons with thousands of holes may be slow to triangulate
- Output is always RGB (no alpha channel support)

## Future Enhancements

- [ ] Multi-sample anti-aliasing (MSAA)
- [ ] Outline mode with configurable thickness
- [ ] Grid overlay rendering
- [ ] Scale bar / ruler
- [ ] Z-height label rendering
- [ ] SVG export for vector output
- [ ] Performance optimizations for very large slices

## License

Same as parent project: MIT OR Apache-2.0
