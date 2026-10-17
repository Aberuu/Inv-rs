# inventor-rs

Clean-room parametric 3D solid modeler in Rust, inspired by Autodesk Inventor workflows.

## Features

- **Parametric Architecture**:
  - `inventor-core`: Document model, feature tree, parameter table with named variables and expressions, undo/redo command stack.
  - `inventor-kernel`: Clean-room B-rep abstraction layer built on top of the pure-Rust `truck` geometric modeling kernel (`truck-modeling`, `truck-polymesh`, `truck-meshalgo`).
  - `inventor-sketch`: 2D geometric sketch entities (Point, Line, Circle, Arc) with 2D geometric constraint solver and degrees-of-freedom tracker.
  - `inventor-render`: CAD camera system (Orbit, Pan, Zoom, Isometric Home F6 view, Top/Front/Right presets), adaptive ground grid, XYZ datum axes, and shaded mesh rendering with crisp wireframe feature edges.
  - `inventor-io`: Standard ISO 10303 STEP AP203/214 export and binary/ASCII STL mesh export.
  - `inventor-app`: Native desktop CAD UI built with `eframe` and `egui`, featuring an Inventor-style ribbon toolbar, model browser tree, 3D viewport with ViewCube, and parameter table dialogs.

## Building & Running

### Requirements
- Rust stable toolchain (`x86_64-pc-windows-gnu` or `x86_64-pc-windows-msvc`)
- MinGW GCC or MSVC C++ toolchain

### Running the Desktop App
```bash
cargo run -p inventor-app
```

### Running the Test Suite
```bash
cargo test
```

## Keyboard Shortcuts
- `F6`: Reset camera to standard Isometric Home view
- `Esc`: Cancel active tool / close modal dialog
- `Ctrl + Z`: Undo command
- `Ctrl + Y`: Redo command
- `Middle Mouse Button Drag`: Orbit camera
- `Shift + Right Click Drag`: Orbit camera
- `Right Click Drag`: Pan camera
- `Scroll Wheel`: Zoom in / out

## License
Dual-licensed under MIT OR Apache-2.0.
