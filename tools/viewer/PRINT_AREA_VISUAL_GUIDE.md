# Visual Guide to Print Area Feature

## Wireframe Box Structure

The print area is visualized as a wireframe box consisting of 12 lines:

```
        7 ────────────── 6
       /|              /|
      / |             / |
     4 ─────────────5  |     ↑ Z axis (height)
     |  |            |  |     |
     |  3 ───────────|─ 2     |
     | /             | /      o──→ Y axis (depth)
     |/              |/      /
     0 ─────────────1      ↙
                          X axis (width)
```

### Corner Points (centered at origin)
- **Bottom Face** (Z = 0, build plate):
  - Point 0: (-width/2, -depth/2, 0) - front left
  - Point 1: (+width/2, -depth/2, 0) - front right
  - Point 2: (+width/2, +depth/2, 0) - back right
  - Point 3: (-width/2, +depth/2, 0) - back left

- **Top Face** (Z = height):
  - Point 4: (-width/2, -depth/2, height) - front left
  - Point 5: (+width/2, -depth/2, height) - front right
  - Point 6: (+width/2, +depth/2, height) - back right
  - Point 7: (-width/2, +depth/2, height) - back left

### 12 Lines
1. **Bottom Face** (4 lines):
   - 0 → 1 (front edge)
   - 1 → 2 (right edge)
   - 2 → 3 (back edge)
   - 3 → 0 (left edge)

2. **Top Face** (4 lines):
   - 4 → 5 (front edge)
   - 5 → 6 (right edge)
   - 6 → 7 (back edge)
   - 7 → 4 (left edge)

3. **Vertical Edges** (4 lines):
   - 0 → 4 (front left)
   - 1 → 5 (front right)
   - 2 → 6 (back right)
   - 3 → 7 (back left)

## Example Viewer Display

```
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

## Menu Display (Press M)

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

## Configuration Dialog (Press C)

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

Enter width (X) in mm [200.0]: 250
Enter depth (Y) in mm [200.0]: 210
Enter height (Z) in mm [200.0]: 210
Enter unit (mm/inch/cm) [mm]: 

✓ Print area updated successfully!
  Width (X):  250.0 mm
  Depth (Y):  210.0 mm
  Height (Z): 210.0 mm
```

## Visual Appearance in 3D View

The print area appears as a light blue/gray wireframe box in the 3D viewport:

```
┌─────────────────────────────────────┐
│                                     │
│            ┌─────────┐              │  ← Top face of print area
│           /         /|              │     (Z = height)
│          /         / |              │
│    Z ↑  /         /  |              │
│      │ └─────────┘   |              │
│      │ |         |   |              │
│      │ |  Model  |   /              │
│      o──→ Y       |  /               │     
│     /   |         | /               │
│    X    └─────────┘ ← Build plate   │
│                       (Z = 0)       │
│                                     │
│  3D Model rendered with materials   │
│  Print area shown as light blue box │
│  XYZ axes: X=Red, Y=Green, Z=Blue   │
│                                     │
└─────────────────────────────────────┘
```

## Color Scheme

- **Print Area Wireframe**: Light blue/gray (RGB: 0.5, 0.7, 0.9)
- **X Axis**: Red (1.0, 0.0, 0.0)
- **Y Axis**: Green (0.0, 1.0, 0.0)
- **Z Axis**: Blue (0.0, 0.0, 1.0)
- **Model**: Colors from materials/color groups or default blue-gray

## Common Printer Configurations

### Prusa i3 MK3S
```
Width:  250 mm
Depth:  210 mm
Height: 210 mm
```

### Creality Ender 3
```
Width:  220 mm
Depth:  220 mm
Height: 250 mm
```

### Creality CR-10
```
Width:  300 mm
Depth:  300 mm
Height: 400 mm
```

### Ultimaker S5
```
Width:  330 mm
Depth:  240 mm
Height: 300 mm
```
