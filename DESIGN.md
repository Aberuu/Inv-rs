# Design Direction: inventor-rs

CAD 3D parametrik desktop native yang efisien, presisi, dan berfokus pada alur kerja insinyur dan maker.

## Dials
- ENERGY: 1 (Calm, utilitarian, focus on geometry and precision)
- RHYTHM: 1 (Predictable, structured CAD layout: Ribbon top, Tree left, Viewport center, Properties right, Status bar bottom)
- MOTION: 1 (Static UI panels, responsive 60 FPS 3D viewport navigation)

## Visual Identity & Character
- **Identity**: Technical, clean-room parametric CAD. Tidak menyalin ikon atau aset berhak cipta Autodesk.
- **Palette**:
  - Background Window/Panels: Neutral Slate Dark (`#181a1f`) / Header (`#21252b`)
  - Viewport Canvas: Deep CAD Navy-Slate Gradient (`#1e2430` ke `#2d3748`)
  - Accents: Engineering Cyan (`#38bdf8`) untuk active tool/selection, Amber/Gold (`#f59e0b`) untuk constraints/warnings
  - Datum Colors: X-Axis Red (`#ef4444`), Y-Axis Green (`#22c55e`), Z-Axis Blue (`#3b82f6`)
  - Sketch Geometry: Fully Constrained Dark Navy/Black (`#0f172a` di light atau `#38bdf8` di dark), Underconstrained Blue (`#60a5fa`)
- **Typography**: Clean system sans-serif (Inter / Segoe UI / SF Pro) dengan Monospace (JetBrains Mono / Consolas) untuk input koordinat, dimensi numerik, dan parameter ekspresi.
- **Components**:
  - Top Ribbon Toolbar dengan tab fungsional (*3D Model*, *Sketch*, *Inspect*, *Tools*).
  - Left Model Browser Tree dengan hirarki collapsible (*Origin*, *Features*, *Sketches*).
  - Central 3D Viewport dengan ViewCube koordinat dan infinite dynamic grid.
  - Bottom Status & Prompt Bar menampilkan pesan panduan alat aktif dan koordinat kursor.
