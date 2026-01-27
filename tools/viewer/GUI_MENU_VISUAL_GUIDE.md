# GUI Menu Bar Visual Guide

## Menu Bar Layout

```
┌─────────────────────────────────────────────────────────────┐
│ File   View   Settings   Extensions   Help                  │  ← Menu Bar
├─────────────────────────────────────────────────────────────┤
│                                                              │
│                                                              │
│                    [3D Viewport Area]                        │
│                                                              │
│                                                              │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Menu Dropdowns

### File Menu
```
┌─────────────────────────────┐
│ File                        │
├─────────────────────────────┤
│ Open...              Ctrl+O │
│ Browse Test Suites... Ctrl+T│
│ Export Screenshot...      S │
│ Exit                    ESC │
└─────────────────────────────┘
```

### View Menu
```
┌─────────────────────────────┐
│ View                        │
├─────────────────────────────┤
│ [✓] Show Axes             A │
│ [✓] Show Print Bed        P │
│     Show Grid             G │
│     Reset Camera       Home │
│     Fit to Model          F │
└─────────────────────────────┘
```

### Settings Menu
```
┌─────────────────────────────┐
│ Settings                    │
├─────────────────────────────┤
│     Theme: Light            │
│ [✓] Theme: Dark           T │
│     Print Bed Settings      │
└─────────────────────────────┘
```

### Extensions Menu
```
┌─────────────────────────────┐
│ Extensions                  │
├─────────────────────────────┤
│ [✓] Materials/Colors        │
│ [✓] Beam Lattice          B │
│     Slice Stack           Z │
│     Displacement          D │
│     Boolean Operations    V │
└─────────────────────────────┘
```

### Help Menu
```
┌─────────────────────────────┐
│ Help                        │
├─────────────────────────────┤
│ Keyboard Shortcuts        M │
│ About                       │
└─────────────────────────────┘
```

## Visual Elements

### Menu Bar Colors
- **Background**: Dark gray (semi-transparent)
- **Text**: Light gray (#E5E5E5)
- **Highlight**: Yellow (#FFFF80) when menu is open or item is hovered
- **Disabled**: Gray (#808080)

### Menu States

#### Active Menu
```
File   View   Settings   Extensions   Help
^^^^   (highlighted in yellow)
```

#### Hovered Menu Item
```
┌─────────────────────────────┐
│ File                        │
├─────────────────────────────┤
│ Open...              Ctrl+O │  ← Normal
│ [Browse Test Suites...   ]  │  ← Hovered (highlighted)
│ Export Screenshot...      S │
│ Exit                    ESC │
└─────────────────────────────┘
```

#### Checked Items
```
[✓] Show Axes             A   ← Feature is enabled
[ ] Show Grid             G   ← Feature is disabled (shown as blank)
```

## Interaction Flow

### Opening a Menu
1. **Mouse**: Click on menu label
   ```
   [Click] File
           ↓
   ┌─────────────────────────────┐
   │ Open...              Ctrl+O │
   │ Browse Test Suites... Ctrl+T│
   │ Export Screenshot...      S │
   │ Exit                    ESC │
   └─────────────────────────────┘
   ```

2. **Clicking Menu Item**
   ```
   Click "Open..." 
           ↓
   File dialog appears
           ↓
   Select 3MF file
           ↓
   Model loads
   Menu closes automatically
   ```

### Toggling Features
```
Initial state:          After clicking:
[✓] Show Axes    →     [ ] Show Axes
(axes visible)         (axes hidden)
```

## Positioning

The menu bar is positioned at the very top of the window:
- Height: 25 pixels
- Width: Full window width
- Z-order: Rendered last (on top of 3D viewport)

Menu dropdowns appear directly below their parent menu label:
- Width: 200 pixels
- Height: Variable (based on number of items)
- Item height: 20 pixels each

## Usage Example

```
Step 1: Launch viewer
┌─────────────────────────────────────┐
│ File   View   Settings   Extensions │  ← Menu visible
├─────────────────────────────────────┤
│          [Empty Scene]              │
└─────────────────────────────────────┘

Step 2: Click "File" → "Open..."
File dialog opens...

Step 3: Model loaded
┌─────────────────────────────────────┐
│ File   View   Settings   Extensions │
├─────────────────────────────────────┤
│          [3D Model]                 │
│         🎯 Model center             │
│        /│\                          │
│       X Y Z  ← Axes visible         │
└─────────────────────────────────────┘

Step 4: Click "View" → "Show Axes" to toggle
┌─────────────────────────────────────┐
│ File   View   Settings   Extensions │
│      ┌─────────────────────┐        │
│      │ [ ] Show Axes     A │        │
│      │ [✓] Show Print Bed P│        │
│      └─────────────────────┘        │
│          [3D Model]                 │
│         🎯 Model center             │
│                                     │  ← Axes now hidden
└─────────────────────────────────────┘

Step 5: Press 'M' to hide menu bar
┌─────────────────────────────────────┐
│                                     │  ← Menu hidden
│          [3D Model]                 │
│         🎯 Model center             │
│        More viewport space          │
└─────────────────────────────────────┘
```

## Keyboard Shortcuts Reference

All menu items that have keyboard shortcuts show them on the right side:

```
Open...              Ctrl+O   ← Shortcut shown in gray
Show Axes                 A   ← Single key shortcut
```

You can either:
- Click the menu item with the mouse
- Use the keyboard shortcut directly (menu doesn't need to be open)

## Tips

1. **Quick Access**: Use keyboard shortcuts for frequently used features
2. **Discovery**: Browse menus to discover all available features
3. **State Feedback**: Checkmarks show which features are currently active
4. **Toggle Menu**: Press 'M' to get more viewport space when menu not needed
5. **Hover Help**: Shortcuts are shown on the right side of menu items
