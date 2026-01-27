# Demo: Using the Print Area Feature

This document demonstrates how to use the new menu and print area visualization features.

## Starting the Viewer

```bash
cd tools/viewer
cargo run --release -- ../../test_files/core/box.3mf --ui
```

## Initial Display

When the viewer launches, you'll see:

```
Loading: ../../test_files/core/box.3mf

✓ Model loaded successfully!

═══════════════════════════════════════════════════════════
  Model Information:
  - Objects: 1
  - Triangles: 12
  - Vertices: 8
  - Unit: millimeter

═══════════════════════════════════════════════════════════
  Interactive 3D Viewer Controls
═══════════════════════════════════════════════════════════

  🖱️  Left Mouse + Drag  : Rotate view
  🖱️  Right Mouse + Drag : Pan view
  🖱️  Scroll Wheel       : Zoom in/out
  ⌨️  Arrow Keys         : Pan view
  ⌨️  A Key              : Toggle XYZ axes
  ⌨️  M Key              : Toggle menu          ← NEW!
  ⌨️  P Key              : Toggle print area    ← NEW!
  ⌨️  C Key              : Configure print area ← NEW!
  ⌨️  Ctrl+O             : Open file
  ⌨️  T or B             : Cycle themes
  ⌨️  Ctrl+T             : Browse test suites
  ⌨️  ESC / Close Window : Exit viewer

═══════════════════════════════════════════════════════════
```

The 3D window opens showing:
- Your 3MF model in the center
- XYZ axes (Red=X, Green=Y, Blue=Z)
- Print area wireframe box (light blue, 200x200x200mm)

## Demo 1: Viewing the Menu

**Action:** Press `M` key

**Result:**
```
═══════════════════════════════════════════════════════════
  Menu - Current Settings
═══════════════════════════════════════════════════════════

  Theme:           Dark
  Print Area:      ON
    Width (X):     200.0 mm
    Depth (Y):     200.0 mm
    Height (Z):    200.0 mm
  File:            box.3mf

  Press M to hide menu
  Press C to configure print area
═══════════════════════════════════════════════════════════
```

## Demo 2: Toggling Print Area Visibility

**Action:** Press `P` key

**Result:**
```
Print Area: OFF
```

The wireframe box disappears from the 3D view.

**Action:** Press `P` key again

**Result:**
```
Print Area: ON
```

The wireframe box reappears.

## Demo 3: Configuring for Prusa i3 MK3S (250x210x210mm)

**Action:** Press `C` key

**Result:**
```
═══════════════════════════════════════════════════════════
  Configure Print Area
═══════════════════════════════════════════════════════════

Current settings:
  Width (X):  200.0 mm
  Depth (Y):  200.0 mm
  Height (Z): 200.0 mm

To change settings, use the console:
  - Enter new dimensions when prompted
  - Press Enter to keep current value

═══════════════════════════════════════════════════════════

Enter width (X) in mm [200.0]: 
```

**User types:** `250` and presses Enter

```
Enter depth (Y) in mm [200.0]: 
```

**User types:** `210` and presses Enter

```
Enter height (Z) in mm [200.0]: 
```

**User types:** `210` and presses Enter

```
Enter unit (mm/inch/cm) [mm]: 
```

**User presses:** Enter (to keep mm)

**Result:**
```
✓ Print area updated successfully!
  Width (X):  250.0 mm
  Depth (Y):  210.0 mm
  Height (Z): 210.0 mm
```

The wireframe box in the 3D view updates to show the new dimensions.

## Demo 4: Configuring for Ender 3 (220x220x250mm)

**Action:** Press `C` key

**User enters:**
- Width: `220`
- Depth: `220`
- Height: `250`
- Unit: (press Enter)

**Result:**
```
✓ Print area updated successfully!
  Width (X):  220.0 mm
  Depth (Y):  220.0 mm
  Height (Z): 250.0 mm
```

## Demo 5: Viewing Updated Menu

**Action:** Press `M` key

**Result:**
```
═══════════════════════════════════════════════════════════
  Menu - Current Settings
═══════════════════════════════════════════════════════════

  Theme:           Dark
  Print Area:      ON
    Width (X):     220.0 mm
    Depth (Y):     220.0 mm
    Height (Z):    250.0 mm
  File:            box.3mf

  Press M to hide menu
  Press C to configure print area
═══════════════════════════════════════════════════════════
```

Notice the dimensions have been updated!

## Demo 6: Unit Validation

**Action:** Press `C` key

```
Enter width (X) in mm [220.0]: 8.66
Enter depth (Y) in mm [220.0]: 8.66
Enter height (Z) in mm [250.0]: 9.84
Enter unit (mm/inch/cm) [mm]: inch
```

**Result:**
```
✓ Print area updated successfully!
  Width (X):  8.66 inch
  Depth (Y):  8.66 inch
  Height (Z): 9.84 inch
```

The unit is validated and normalized. Supported units:
- `mm`, `millimeter`, `millimeters` → normalized to "mm"
- `cm`, `centimeter`, `centimeters` → normalized to "cm"
- `inch`, `inches`, `in` → normalized to "inch"
- `m`, `meter`, `meters` → normalized to "m"

## Demo 7: Invalid Input Handling

**Action:** Press `C` key

```
Enter width (X) in inch [8.66]: abc
```

**Result:** Invalid input is ignored, keeps current value (8.66)

```
Enter depth (Y) in inch [8.66]: -5
```

**Result:** Negative values are rejected, keeps current value (8.66)

```
Enter height (Z) in inch [9.84]: 
```

**Result:** Empty input keeps current value (9.84)

```
Enter unit (mm/inch/cm) [inch]: xyz
Warning: Unknown unit 'xyz', keeping 'inch'
```

**Final Result:**
```
✓ Print area updated successfully!
  Width (X):  8.66 inch
  Depth (Y):  8.66 inch
  Height (Z): 9.84 inch
```

## 3D View Behavior

In the 3D window, you can:

1. **Rotate the view** - Left click and drag to see the model and print area from different angles
2. **Pan the view** - Right click and drag to move around
3. **Zoom** - Scroll wheel to zoom in/out
4. **See the relationship** - The wireframe box helps visualize if your model fits within the print area

## Common Printer Configurations

### Quick Setup Examples

**Prusa i3 MK3S:**
- Width: 250
- Depth: 210
- Height: 210

**Creality Ender 3:**
- Width: 220
- Depth: 220
- Height: 250

**Creality CR-10:**
- Width: 300
- Depth: 300
- Height: 400

**Ultimaker S5:**
- Width: 330
- Depth: 240
- Height: 300

**Anycubic Mega S:**
- Width: 210
- Depth: 210
- Height: 205

## Tips

1. **Start with defaults** - The 200x200x200mm default works for most common printers
2. **Use the menu** - Press M to quickly check your current settings
3. **Toggle visibility** - Press P to hide the print area when taking screenshots
4. **Configure once** - Settings persist during your session, so configure once and load multiple files
5. **Check fit** - Rotate the model to see all angles and ensure it fits within the print area

## Keyboard Shortcuts Quick Reference

| Key | Action |
|-----|--------|
| M | Toggle menu |
| P | Toggle print area visibility |
| C | Configure print area |
| A | Toggle XYZ axes |
| T/B | Cycle themes |
| Ctrl+O | Open file |
| Ctrl+T | Browse test suites |
| ESC | Exit |

## Session Workflow Example

1. Launch viewer with a file
2. Press `C` to configure your printer's build volume
3. Press `M` to verify settings
4. Use mouse to rotate and examine the model
5. Load different files with Ctrl+O (settings persist)
6. Press `P` if you want to hide the print area temporarily
7. Press `M` to check settings anytime
