## 12. Status Implementasi dan Keputusan Teknis (v0.2)

Bagian 12 sampai 17 menggantikan bagian sebelumnya jika ada perbedaan. Dokumen ini ditujukan juga sebagai spesifikasi kerja untuk agen pengembang (**Antigravity**).

### Keputusan yang Sudah Diambil
- **Nama proyek dan repositori:** `inventor-rs`
- **Kernel geometri:** `truck` (Rust murni), diakses hanya lewat trait `GeometryKernel`.
- **Fillet, chamfer, dan boolean lanjutan:** Dievaluasi dulu di `truck`. Jika tidak memadai, tambahkan implementasi kedua (misalnya `opencascade-rs`) di balik trait yang sama tanpa mengubah sisi aplikasi.
- **UI:** `eframe` dan `egui`.
- **Constraint solver:** Ditulis sendiri, ditingkatkan dari relaksasi koordinat ke Newton-Raphson atau Levenberg-Marquardt dengan matriks Jacobian (misalnya memakai `nalgebra`).

### Status per Milestone
- **M0 Fondasi:** Hampir selesai (viewport 3D, kamera orbit, grid, ViewCube, ribbon awal, model browser).
- **M1 Sketch 2D:** Sebagian. Entitas dan solver dasar ada. Constraint Tangent, Equal, Angle, dan Diameter belum ada.
- **M2 sampai M4:** Belum tersambung. Solid saat ini dibuat lewat dialog Box dan Cylinder, bukan dari sketsa yang di-extrude.
- **M5 Berkas:** Sebagian (ekspor STL, ekspor STEP, project file JSON, undo/redo).
- **M6 Assembly dan M7 Drawing:** Belum dimulai.
- **Pengujian:** 12 dari 12 unit test lulus (core: 3, kernel: 2, sketch: 2, render: 2, io: 3).

---

## 13. Arsitektur dan Konvensi Kode

### Workspace dan Arah Dependensi
**Crate:** `inventor-core`, `inventor-kernel`, `inventor-sketch`, `inventor-render`, `inventor-io`, `inventor-app`.

**Arah dependensi yang berlaku (tidak boleh dilanggar):**
- `inventor-app` boleh bergantung pada semua crate lain.
- `inventor-render` dan `inventor-io` boleh bergantung pada `inventor-core` dan `inventor-kernel`.
- `inventor-kernel` dan `inventor-sketch` hanya boleh bergantung pada `inventor-core`.
- `inventor-core` **tidak boleh** bergantung pada crate lain di workspace, dan tidak boleh mengetahui UI.

### Konvensi
- Semua perubahan pada dokumen melewati **Command Pattern** dan **HistoryStack** agar undo dan redo selalu bekerja.
- Kernel hanya diakses lewat trait `GeometryKernel`. Tidak ada pemanggilan `truck_*` di luar crate `inventor-kernel`.
- **Satuan internal:** Milimeter untuk panjang dan derajat untuk sudut, dikelola oleh `ParameterTable`.
- Jalur yang bisa dipicu pengguna **tidak boleh panic**. Gunakan `Result` dengan tipe error per crate (disarankan `thiserror`) dan tampilkan pesan galat yang jelas di UI.
- Kode harus lolos `cargo fmt`, `cargo clippy` tanpa peringatan, dan `cargo test`.
- Nama identifier dan komentar kode dalam bahasa Inggris; dokumentasi proyek dalam bahasa Indonesia.

---

## 14. Spesifikasi Ribbon

### Prinsip
- **Ribbon berbasis data:** Struktur `Tab > Panel > Item` didefinisikan di satu tempat (`struct` Rust atau berkas RON/TOML), bukan di-hardcode per tombol.
- Setiap tombol hanya menunjuk ke ID perintah di *command registry*. Registry yang memanggil kernel dan `HistoryStack`.
- **Dukungan item:** Tombol besar, tombol kecil, tombol dengan dropdown, pemisah, dan tooltip.
- Status tombol (aktif, nonaktif, tersorot) dihitung dari konteks (misalnya: ada sketsa aktif atau tidak, ada solid atau tidak).
- Tab bersifat kontekstual per lingkungan (Part dan Assembly).
- Tombol tanpa perintah di baliknya harus nonaktif dengan tooltip *"Belum tersedia"*. Jangan membuat tombol aktif yang tidak melakukan apa-apa.
- Terdapat *quick access toolbar* (New, Open, Save, Undo, Redo, Home) dan kotak pencarian perintah (*command palette*).

### Tab dan Panel per Milestone

| Tab | Panel dan Perintah | Kapan Aktif |
| :--- | :--- | :--- |
| **File / Menu Aplikasi** | New, Open, Save, Save As, Export (STEP, STL), Exit | R1 |
| **View** | Home, Front, Top, Right, Perspective atau Orthographic, Grid, ViewCube | R1 (sebagian sudah ada) |
| **Sketch** | **Create:** Line, Circle, Arc, Rectangle, Point<br>**Constrain:** Coincident, Horizontal, Vertical, Parallel, Perpendicular, Tangent, Equal, Fix<br>**Other:** Dimension, Finish Sketch | R2 (bersama M1) |
| **3D Model** | **Sketch:** Create 2D Sketch<br>**Create:** Extrude, Revolve, Box, Cylinder<br>**Modify:** Fillet, Chamfer, Hole, Shell<br>**Pattern:** Rectangular, Circular, Mirror<br>**Work Features:** Plane, Axis, Point | R2 sampai R3 |
| **Tools** | Parameters, Options | R2 |
| **Inspect** | Measure (Distance, Angle) | Setelah M4 |
| **Assemble** | **Component:** Place<br>**Relationships:** Joint, Constrain<br>**Pattern & Manage:** Pattern, Bill of Materials, Parameters<br>**Productivity & Features:** Ground and Root, Work Features | R3 (setelah M6), sebelum itu nonaktif |
| **Annotate** | Dimension, Text, View Gambar | R3 (setelah M7), sebelum itu nonaktif |

> **Catatan Scope:** Tab Design, Factory, Electromechanical, Collaborate, dan Fusion 360 berada di luar ruang lingkup dan tidak dibuat.

### Ikon dan Merek
Tata letak dan nama perintah umum (Extrude, Fillet, Mirror) boleh dipakai. Ikon, warna merek, dan aset Autodesk **tidak boleh disalin**. Gunakan ikon buatan sendiri atau set ikon open-source dengan lisensi yang jelas dan tercatat.

---

## 15. Backlog Tugas Berurutan

> **Aturan Eksekusi:** Kerjakan berurutan. Satu tugas selesai (*acceptance criteria* terpenuhi) sebelum tugas berikutnya dimulai.

1. **T1 Kerangka Ribbon (R0):**
   Tab, panel, tombol besar dan kecil, dropdown, tooltip, status nonaktif, ingat tab terakhir. *Selesai bila ribbon dirender dari definisi data dan menambah tombol cukup dengan menambah satu entri.*
2. **T2 Command Registry dan Perintah yang Sudah Ada (R1):**
   Pindahkan Box, Cylinder, ekspor STEP dan STL, undo dan redo, serta kamera ke registry dan sambungkan ke tombol ribbon. *Selesai bila semua perintah ini bisa dipicu dari ribbon dan quick access toolbar.*
3. **T3 Peningkatan Solver:**
   Newton-Raphson atau Levenberg-Marquardt, deteksi sketsa over-constrained, penghitungan DOF akurat, serta constraint Tangent, Equal, Angle, dan Diameter. *Selesai bila ada test untuk sketsa under-, fully-, dan over-constrained, dan konvergensi stabil pada sketsa dengan constraint saling bergantung.*
4. **T4 Mode Sketsa di UI:**
   Masuk sketsa pada bidang datum atau face, menggambar entitas, menambah constraint dan dimensi, indikator DOF, Finish Sketch. *Selesai bila sketsa persegi berdimensi penuh bisa dibuat dan diedit dari UI.*
5. **T5 Irisan Vertikal Parametrik:**
   Sketsa persegi, profil, fitur Extrude, regenerasi dokumen dari `Document` dan `ParameterTable`. *Selesai bila mengubah satu dimensi di parameter meregenerasi solid tanpa galat dan ada test otomatis untuk itu.*
6. **T6 Feature Tree Interaktif:**
   Edit ulang, suppress, dan reorder fitur; regenerasi ber