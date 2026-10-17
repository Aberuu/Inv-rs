# inventor-rs

Parametric 3D solid modeler in Rust, engineered following a clean-room approach inspired by Autodesk Inventor workflows and user experience parity.

---

## 1. Architecture

The workspace is organized into modular crates with strict unidirectional dependency rules:

- **`inventor-core`**: Core data models with zero external CAD dependencies: `Document`, `FeatureTree`, `ParameterTable` (named variables, expressions, and units), and transactional `HistoryStack` (Undo/Redo command pattern).
- **`inventor-kernel`**: Geometric kernel abstraction layer via the unified `GeometryKernel` trait. Primary backend is pure-Rust `truck` B-rep modeling (`truck-modeling`, `truck-polymesh`, `truck-meshalgo`, `truck-shapeops`), alongside a secondary `OpenCASCADE` (OCCT) dual-kernel adapter for advanced topological blending.
- **`inventor-sketch`**: 2D parametric sketching engine: entities (Point, Line, Circle, Arc), closed-loop profile extraction, and a 2D constraint solver (Levenberg-Marquardt with SVD-based degree-of-freedom tracking).
- **`inventor-render`**: Hardware-accelerated CAD viewport powered by `wgpu 30` with `Depth32Float` z-buffering, custom WGSL shaders (`cad_mesh.wgsl` and `cad_line.wgsl`), CAD camera (Pan, Orbit, Zoom, Isometric Home), adaptive grid, and datum axes.
- **`inventor-io`**: Standard interoperability layer supporting ISO 10303 STEP (AP203/214) export/import with topological B-rep shell reconstruction, and binary/ASCII STL mesh export.
- **`inventor-app`**: Desktop application built with `eframe` and `egui`, featuring an Inventor-style ribbon toolbar, clean model browser with context menus and consumption model, modal dialogs, and native hardware viewport integration.

---

## 2. Prerequisites & Build Requirements

To build and run `inventor-rs` on a clean Windows machine:

1. **Rust Toolchain**:
   - Rust Stable 1.85+ (`x86_64-pc-windows-gnu` or `x86_64-pc-windows-msvc`).
   - Components: `rustfmt`, `clippy`.
   - Install via [rustup.rs](https://rustup.rs/):
     ```bash
     rustup update stable
     rustup component add rustfmt clippy
     ```

2. **C/C++ Compiler**:
   - For `x86_64-pc-windows-gnu`: MinGW GCC (e.g. `w64devkit` or MSYS2 MinGW-w64).
   - For `x86_64-pc-windows-msvc`: Visual Studio 2022 C++ Build Tools.

3. **Graphics / WGPU Prerequisites**:
   - Graphics driver with modern hardware API support (DirectX 12 or Vulkan on Windows).
   - Automatic fallback to software/low-power adapter if discrete GPU is absent.

4. **Geometry Kernel Notes**:
   - Default: Pure-Rust `truck` kernel requires zero external C++ libraries or DLLs.
   - Secondary OpenCASCADE (OCCT) integration: Evaluated behind the `GeometryKernel` trait for edge blending and filleting.

---

## 3. Building & Running

### Build Workspace
```powershell
cargo build --workspace
```

### Run the Desktop CAD Application
```powershell
cargo run -p inventor-app
```

### Run All Workspace Tests
```powershell
cargo test --workspace
```

### Run U0 Baseline Automated Test Proofs
```powershell
cargo test -p inventor-app -- test_u0_baseline
```

---

## 4. Navigation & Shortcuts (Inventor Parity Specification)

Mouse interactions and keyboard shortcuts adhere to the parity specification defined in `PRD v0.3` and `DESIGN.md`.

### Mouse Controls
| Input | Action | Status |
| :--- | :--- | :---: |
| **Middle Mouse Drag** | Pan camera | [?] |
| **Shift + Middle Mouse Drag** | Orbit camera | [?] |
| **Scroll Wheel** | Zoom to cursor position | [?] |
| **Double-click Middle Mouse** | Zoom All (Fit View to bounding box) | [?] |

### Function & Editing Keys
| Shortcut | Action | Status |
| :--- | :--- | :---: |
| **F6** | Standard Isometric Home view | [?] |
| **Esc** | Cancel active tool / Dismiss active modal dialog | [?] |
| **Ctrl + Z** | Undo last feature / command | [V] |
| **Ctrl + Y** | Redo command | [V] |

### Contextual Tool Hotkeys
Hotkeys are context-sensitive based on the active ribbon tab:

- **3D Model Tab**:
  - `E` : Extrude feature dialog [?]
  - `R` : Revolve feature dialog [?]
  - `F` : Fillet feature dialog [?]
- **Sketch Tab** (active during 2D sketch mode):
  - `L` : Line tool [?]
  - `C` : Circle tool [?]
  - `A` : Arc tool [?]
  - `P` : Point tool [?]
  - `D` : General Dimension tool [?]
  - `R` : Rectangle tool [?]

*(Note: `[V]` indicates verified in real Autodesk Inventor; `[?]` indicates awaiting user measurement and configured via data tables).*

---

## 5. Continuous Integration (CI)

Every commit and pull request is automatically tested on Windows via GitHub Actions (`.github/workflows/ci.yml`):
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo build --workspace`

---

## 6. License

Dual-licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
