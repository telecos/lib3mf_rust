# Screenshot Feature

The lib3mf viewer includes a built-in screenshot feature that allows you to capture the current 3D scene view and save it as a PNG image file.

## Quick Start

1. Launch the interactive viewer with a 3MF file:
   ```bash
   cargo run --release -- path/to/file.3mf --ui
   ```

2. Position the view as desired (rotate, pan, zoom)

3. Press the **P** key to capture a screenshot

4. The screenshot is automatically saved with a timestamped filename

## Features

### Automatic Filename Generation

Screenshots are saved with timestamped filenames to ensure uniqueness and avoid overwriting previous captures:

```
screenshot_2025-01-27_145230.png
screenshot_2025-01-27_145245.png
screenshot_2025-01-27_150103.png
```

Format: `screenshot_YYYY-MM-DD_HHMMSS.png`

### What Gets Captured

The screenshot captures everything visible in the current window:
- The 3D model with current camera angle
- Current zoom level
- Visible color/materials
- XYZ axes (if enabled with 'A' key)
- Current background theme
- Beam lattice geometry (if enabled with 'B' key)

### Save Location

Screenshots are saved in the **current working directory** where you launched the viewer.

## Usage Examples

### Basic Screenshot Workflow

```bash
# 1. Launch viewer
cd tools/viewer
cargo run --release -- ../../test_files/core/sphere.3mf --ui

# 2. In the viewer window:
#    - Rotate the view with left mouse drag
#    - Adjust zoom with scroll wheel
#    - Press 'T' to change background theme if desired
#    - Press 'P' to capture screenshot
#
# 3. Screenshot saved as: screenshot_2025-01-27_145230.png
```

### Multiple Screenshots

You can capture multiple screenshots during a session:

```bash
# Launch viewer
cargo run --release -- ../../test_files/core/box.3mf --ui

# While running:
# 1. Position view from front, press 'P'
#    → screenshot_2025-01-27_100500.png
#
# 2. Rotate to side view, press 'P'
#    → screenshot_2025-01-27_100512.png
#
# 3. Rotate to top view, press 'P'
#    → screenshot_2025-01-27_100525.png
```

### Different Themes

Capture screenshots with different background themes:

```bash
# Launch viewer
cargo run --release -- ../../test_files/material/kinect_scan.3mf --ui

# While running:
# 1. Default dark theme, press 'P'
# 2. Press 'T' to cycle to light theme, press 'P'
# 3. Press 'T' to cycle to blue theme, press 'P'
# 4. Press 'T' to cycle to white theme, press 'P'
# 5. Press 'T' to cycle to black theme, press 'P'
```

### With/Without Axes

```bash
# Launch viewer
cargo run --release -- ../../test_files/core/torus.3mf --ui

# While running:
# 1. With axes visible (default), press 'P'
# 2. Press 'A' to hide axes
# 3. Press 'P' to capture without axes
```

## Keyboard Shortcut

| Key | Action |
|-----|--------|
| **P** | Capture screenshot |

Press 'P' at any time while the viewer window is active to capture a screenshot of the current view.

## Output Format

- **Format**: PNG (Portable Network Graphics)
- **Features**: Lossless compression, supports transparency
- **Resolution**: Matches the current window size
- **Color depth**: Full RGB color

## Console Feedback

When you capture a screenshot, the viewer prints a confirmation message to the console:

```
✓ Screenshot saved: screenshot_2025-01-27_145230.png
```

If there's an error (e.g., disk full, permission denied), an error message is displayed:

```
✗ Error capturing screenshot: Permission denied (os error 13)
```

## Tips

1. **Resize the window** before capturing to control output resolution
2. **Use different themes** ('T' key) for better contrast with your model
3. **Toggle axes** ('A' key) depending on whether you want them in the screenshot
4. **Multiple angles**: Capture several screenshots from different viewpoints
5. **Organize captures**: Consider creating a screenshots directory and running the viewer from there

## Implementation Details

The screenshot feature uses:
- **kiss3d's `snap_image()`**: Captures the current framebuffer
- **image crate**: Saves the captured image as PNG
- **std::time**: Generates unique timestamps for filenames

The implementation is minimal and does not require any external dependencies beyond what's already included in the viewer.

## Comparison with Export Preview

The viewer has two image export features:

| Feature | Screenshot (P key) | Export Preview (--export-preview) |
|---------|-------------------|-----------------------------------|
| **When** | During interactive session | From command line |
| **View** | Current camera position | Preset angles (isometric, top, etc.) |
| **Control** | Full manual control | Automated rendering |
| **Filename** | Auto-timestamped | User-specified |
| **Use Case** | Explore & capture | Automated documentation |

Use **screenshots** when you want to:
- Manually explore and capture specific views
- Capture the exact view you're seeing
- Take multiple shots during a session

Use **export preview** when you want to:
- Generate consistent preview images
- Automate image generation in scripts
- Use specific preset view angles

## Troubleshooting

### Screenshot file not found
- Check the current working directory
- Ensure you have write permissions in the directory

### Black/blank screenshot
- Make sure the window has finished rendering before pressing 'P'
- Try capturing again after moving the view slightly

### Permission denied error
- Check directory write permissions
- Try running from a different directory

## Future Enhancements

Potential future improvements (not currently implemented):
- Custom resolution/scale factor
- Custom filename via prompt
- Configurable save directory
- Copy to clipboard option
- Screenshot history/gallery view

## Related Documentation

- [README.md](README.md) - Main viewer documentation
- [USAGE_EXAMPLES.md](USAGE_EXAMPLES.md) - General usage examples
- [AXIS_VISUALIZATION.md](AXIS_VISUALIZATION.md) - XYZ axes feature
- [BEAM_LATTICE_RENDERING.md](BEAM_LATTICE_RENDERING.md) - Beam lattice visualization
