# PRD: Inventor-style CAD in Rust (working title)

Status: Draft v0.1 (dokumen hidup, diisi bertahap) Owner: Abel

## 1. Ringkasan

Aplikasi CAD 3D parametrik berbasis desktop, ditulis dengan Rust, yang alur kerja dan antarmukanya terinspirasi Autodesk Inventor. Targetnya bukan menyalin Inventor secara persis, tetapi menyediakan inti pemodelan parametrik yang bisa dipakai nyata: sketsa 2D bercontraint, solid 3D, feature tree yang bisa diedit ulang, dan ekspor STEP/STL.

Proyek ini terinspirasi CADCraft dari ArtCraft (reimplementasi AutoCAD clean-room dengan Rust dan egui). Perbedaan utamanya: Inventor adalah pemodelan solid 3D parametrik, bukan drafting 2D, sehingga butuh kernel geometri B-rep dan constraint solver.

## 2. Tujuan dan Non-Tujuan

### Tujuan

- Alur kerja familiar bagi pengguna Inventor: sketch, constraint, feature, assembly.
- Model parametrik: perubahan dimensi di sketsa memperbarui seluruh model.
- Format tukar terbuka (STEP, STL) sebagai prioritas.
- Ringan, cepat, native (tanpa Electron), lintas platform.
- Pengembangan bertahap dengan milestone kecil yang masing-masing bisa didemokan.

### Non-Tujuan (untuk awal)

- Kompatibilitas format `.ipt`, `.iam`, `.idw` (format proprietary).
- Kesamaan fitur 1:1 dengan Inventor.
- Simulasi (FEA), CAM, rendering fotorealistik, iLogic.
- Menyalin kode, ikon, aset, atau teks antarmuka milik Autodesk.

## 3. Prinsip Hukum dan Pengembangan

- Pendekatan clean-room: hanya mengacu pada perilaku publik dan dokumentasi, bukan kode atau aset berhak cipta.
- Nama, logo, dan ikon sendiri; tidak memakai merek Autodesk.
- Pustaka pihak ketiga hanya dengan lisensi yang jelas dan dicatat.

## 4. Pengguna Target

- Hobiis, maker, dan mahasiswa teknik yang butuh CAD parametrik gratis.
- Pengguna yang ingin alternatif open source dengan alur kerja mirip Inventor.
- Pengembang yang ingin kernel CAD berbasis Rust yang bisa diperluas.

## 5. Ruang Lingkup Fitur per Milestone

### M0: Fondasi

- Workspace Rust (crate terpisah untuk core, kernel, UI, CLI).
- Window, UI dasar, dan viewport 3D.
- Kamera orbit, pan, zoom; grid dan sumbu.

### M1: Sketch 2D

- Entitas: garis, lingkaran, arc, persegi, spline sederhana.
- Constraint: coincident, horizontal, vertical, parallel, perpendicular, tangent, equal, fix.
- Dimensi: panjang, jarak, radius, diameter, sudut.
- Constraint solver dan indikator derajat kebebasan.

### M2: Fitur Solid Dasar

- Extrude (add dan cut), revolve.
- Bidang kerja dan sketsa di atas face.
- Boolean: union, subtract, intersect.

### M3: Feature Tree (Parametrik)

- Daftar fitur berurutan, edit ulang, suppress, reorder.
- Regenerasi model saat parameter berubah.
- Parameter dan ekspresi bernama.

### M4: Fitur Lanjutan

- Fillet, chamfer, shell, hole.
- Pattern (linear, circular), mirror.
- Sweep dan loft (kandidat).

### M5: Format Berkas

- Format proyek native (terbuka dan terversi).
- Impor dan ekspor STEP; ekspor STL dan OBJ.
- Undo/redo berbasis riwayat perintah.

### M6: Assembly

- Menempatkan beberapa part dalam satu assembly.
- Joint dan constraint: mate, flush, angle, insert.
- Daftar komponen (BOM) sederhana.

### M7: Drawing

- Tampilan 2D (front, top, side, isometrik) dari model 3D.
- Dimensi dan anotasi; ekspor DXF dan PDF.

## 6. Persyaratan Non-Fungsional

- Platform: Windows, Linux, macOS (web build sebagai opsi kemudian).
- Performa: viewport tetap interaktif untuk model menengah.
- Stabilitas: operasi geometri gagal harus menghasilkan pesan galat jelas, bukan crash.
- Pengujian: unit test untuk solver dan kernel, golden test untuk regenerasi dan ekspor.
- Lisensi proyek: perlu diputuskan (lihat Pertanyaan Terbuka).

## 7. Arsitektur (Rancangan Awal)

- core: model data dokumen, feature tree, parameter, command/undo.
- kernel: lapisan abstraksi geometri B-rep di atas kernel pilihan.
- sketch: entitas sketsa dan constraint solver.
- render: wgpu untuk viewport 3D.
- ui: egui (atau alternatif), panel, browser model, properti.
- io: STEP, STL, format native.
- cli: operasi batch dan pengujian otomatis.

Lapisan abstraksi kernel penting agar kernel bisa diganti tanpa menulis ulang aplikasi.

## 8. Risiko

- Kernel B-rep dan operasi fillet/boolean adalah bagian paling berat dan rawan galat.
- Constraint solver yang tidak stabil membuat sketsa sulit dipakai.
- Masalah penamaan topologi (topological naming) membuat feature tree rapuh saat diedit.
- Cakupan terlalu luas; mitigasi dengan milestone kecil dan non-tujuan yang tegas.
- Risiko hukum bila meniru terlalu dekat antarmuka atau aset Autodesk.

## 9. Metrik Keberhasilan

- M1: sketsa bercontraint penuh dapat dibuat dan diedit stabil.
- M3: mengubah satu dimensi meregenerasi model tanpa galat.
- M5: model ekspor STEP terbuka dengan benar di software CAD lain.
- Model contoh (misalnya braket, roda gigi sederhana) bisa dibuat sepenuhnya di aplikasi.

## 10. Pertanyaan Terbuka

- Nama proyek final.
- Kernel geometri: OpenCascade (via binding) atau truck (Rust murni)?
- Constraint solver: binding ke solver yang ada atau tulis sendiri?
- UI: egui atau alternatif?
- Lisensi: MIT/Apache-2.0, atau copyleft seperti GPL?
- Target platform awal dan perlu tidaknya build web.
- Dikerjakan sendiri atau terbuka untuk kontributor sejak awal?

## 11. Langkah Berikutnya

- Memutuskan pertanyaan terbuka satu per satu.
- Merinci M0 dan M1 menjadi daftar tugas.
- Membuat prototipe kecil untuk menguji kernel pilihan.


## 12. Status Implementasi dan Keputusan Teknis (v0.2)
Bagian 12 sampai 17 menggantikan bagian sebelumnya jika ada perbedaan. Dokumen ini ditujukan juga sebagai spesifikasi kerja untuk agen pengembang (Antigravity).

### Keputusan yang sudah diambil
- Nama proyek dan repositori: inventor-rs.
- Kernel geometri: truck (Rust murni), diakses hanya lewat trait GeometryKernel.
- Fillet, chamfer, dan boolean lanjutan: Evaluasi T8 memastikan pure-Rust truck (0.6.0) belum memiliki algoritma edge filleting / rolling-ball blend dan chamfering B-rep 3D. Arsitektur dual-kernel telah diimplementasikan: `DualKernel` mewadahi `TruckKernel` (default, pure-Rust) dan `OpenCascadeKernel` (backend kedua via FFI) di balik trait `GeometryKernel` yang sama tanpa mengubah kode aplikasi.
- UI: eframe dan egui.
- Constraint solver: ditulis sendiri, ditingkatkan dari relaksasi koordinat ke Newton-Raphson atau Levenberg-Marquardt dengan matriks Jacobian (misalnya memakai nalgebra).

### Status per milestone
- M0 Fondasi: selesai (viewport 3D hardware-accelerated wgpu, kamera orbit, grid, ViewCube, model browser).
- R0 Kerangka Ribbon (T1): selesai. Ribbon berbasis data (`Tab > Panel > Item`), tombol besar dan kecil, dropdown, tooltip, tombol tanpa perintah nonaktif dengan tooltip "Belum tersedia", penyimpanan tab terakhir aktif, quick access toolbar, dan pencarian perintah.
- R1 Command Registry (T2): selesai. Semua perintah (Box, Cylinder, Extrude, Ekspor STEP & STL, Undo & Redo, navigasi Kamera, Parameters, Measure, Sketch) terdaftar di `CommandRegistry` dan dipicu terpusat dari Ribbon, Quick Access Toolbar, Menu Bar, dan shortcut keyboard. Penambahan solid primitive tercatat di `HistoryStack` lewat `AddFeatureCmd`.
- M1 Sketch 2D: selesai (T3 & T4). Entitas 2D lengkap (Point, Line, Circle, Arc), solver numerik Levenberg-Marquardt berbasis Jacobian analitis (`nalgebra`), deteksi over-constrained akurat (SVD rank), penghitungan DOF akurat, dan constraint lengkap (Coincident, Horizontal, Vertical, Parallel, Perpendicular, Distance, FixPoint, Radius, Diameter, Equal, Angle, Tangent). Mode sketsa interaktif UI (T4) selesai: ray-casting plane picking $(u, v)$, auto-snap titik, live rubber-band preview dengan dimensi, pembuatan entitas (Line, Rectangle, Circle, Point), penerapan constraint & dimensi dari ribbon, indikator DOF real-time HUD, pewarnaan dinamis entitas (hijau Autodesk Inventor untuk fully constrained, cyan untuk under-constrained), inspector sketsa di browser dengan kemampuan edit & hapus constraint, Finish Sketch untuk commit profil ke feature tree, dan pengeditan kembali sketsa tersimpan.
- M2 Solid Parametrik & Boolean (T5, T6, T7 selesai): Ekstraksi profil poligon tertutup dari sketsa 2D (`extract_closed_profiles`), operasi fitur Extrude & Revolve parametrik via `GeometryKernel`, operasi Boolean (Union, Subtract, Intersect) berbasis B-rep via `truck-shapeops 0.4.0` dengan penanganan compound multi-body dan isolasi epsilon untuk perpotongan non-manifold, transformasi orientasi bidang datum 3D (`transform_by_basis`), serta regenerasi solid terotomatisasi dari `Document` dan `ParameterTable`. Feature tree interaktif selesai dengan dialog `EditExtrude`, `EditRevolve`, `BooleanCombine`, suppress/unsuppress dengan undo/redo, reordering fitur, deteksi dependensi hulu, dan badge visual error `[⚠ Error]`.
- M3 sampai M4 Solid Dress-Up & Evaluasi Kernel (T8 selesai): Evaluasi edge fillet dan chamfer selesai; pure-Rust `truck` tidak memiliki rolling-ball blending dan vertex corner patch. Arsitektur dual-kernel `DualKernel` (dengan backend `TruckKernel` dan `OpenCascadeKernel`) terintegrasi penuh di `inventor-kernel` dan `inventor-app`. Tombol ribbon `Fillet` (`model.fillet`) dan `Chamfer` (`model.chamfer`) diaktifkan, dialog modal `CreateFillet`, `EditFillet`, `CreateChamfer`, `EditChamfer`, dan `Options` (penggantian backend) tersedia, tree browser menampilkan ikon `⌒` dan `◺`, serta regenerasi solid menampilkan diagnostik `[⚠ Error]` deskriptif pada truck dan operasi blend sukses pada OpenCASCADE.
- M5 Berkas: selesai untuk ekspor STL, ekspor STEP, impor STEP (round-trip topological B-rep shell & CadMesh reconstruction via StepImporter), project file JSON, undo/redo, serta inspeksi analitis mesh tertutup (watertight 2-manifold verification).
- M6 Assembly dan M7 Drawing: belum dimulai.
- Pengujian: 73 dari 73 unit test lulus (core 8, kernel 11, sketch 12, render 3, io 5, app 34).

## 13. Arsitektur dan Konvensi Kode

### Workspace dan arah dependensi
Crate: inventor-core, inventor-kernel, inventor-sketch, inventor-render, inventor-io, inventor-app.
Arah dependensi yang berlaku (tidak boleh dilanggar):
- inventor-app boleh bergantung pada semua crate lain.
- inventor-render dan inventor-io boleh bergantung pada inventor-core dan inventor-kernel.
- inventor-kernel dan inventor-sketch hanya boleh bergantung pada inventor-core.
- inventor-core tidak boleh bergantung pada crate lain di workspace, dan tidak boleh mengetahui UI.

### Konvensi
- Semua perubahan pada dokumen melewati Command Pattern dan HistoryStack agar undo dan redo selalu bekerja.
- Kernel hanya diakses lewat trait GeometryKernel. Tidak ada pemanggilan truck_* di luar crate kernel.
- Satuan internal: milimeter untuk panjang dan derajat untuk sudut, dikelola ParameterTable.
- Jalur yang bisa dipicu pengguna tidak boleh panic. Gunakan Result dengan tipe error per crate (disarankan thiserror) dan tampilkan pesan galat yang jelas di UI.
- Kode harus lolos cargo fmt, cargo clippy tanpa peringatan, dan cargo test.
- Nama identifier dan komentar kode dalam bahasa Inggris; dokumentasi proyek dalam bahasa Indonesia.

## 14. Spesifikasi Ribbon

### Prinsip
- Ribbon berbasis data: struktur Tab > Panel > Item didefinisikan di satu tempat (struct Rust atau berkas RON/TOML), bukan di-hardcode per tombol.
- Setiap tombol hanya menunjuk ke ID perintah di command registry. Registry yang memanggil kernel dan HistoryStack.
- Item mendukung: tombol besar, tombol kecil, tombol dengan dropdown, pemisah, dan tooltip.
- Status tombol (aktif, nonaktif, tersorot) dihitung dari konteks, misalnya ada sketsa aktif atau tidak, ada solid atau tidak.
- Tab bersifat kontekstual per lingkungan (Part dan Assembly).
- Tombol tanpa perintah di baliknya harus nonaktif dengan tooltip "Belum tersedia". Jangan membuat tombol aktif yang tidak melakukan apa-apa.
- Ada quick access toolbar (New, Open, Save, Undo, Redo, Home) dan kotak pencarian perintah (command palette).

### Tab dan panel per milestone
| Tab | Panel dan perintah | Kapan aktif |
|---|---|---|
| File / menu aplikasi | New, Open, Save, Save As, Export (STEP, STL), Exit | R1 |
| View | Home, Front, Top, Right, Perspective atau Orthographic, Grid, ViewCube | R1 (sebagian sudah ada) |
| Sketch | Create (Line, Circle, Arc, Rectangle, Point); Constrain (Coincident, Horizontal, Vertical, Parallel, Perpendicular, Tangent, Equal, Fix); Dimension; Finish Sketch | R2 (bersama M1) |
| 3D Model | Sketch (Create 2D Sketch); Create (Extrude, Revolve, Box, Cylinder); Modify (Fillet, Chamfer, Hole, Shell); Pattern (Rectangular, Circular, Mirror); Work Features (Plane, Axis, Point) | R2 sampai R3 |
| Tools | Parameters, Options | R2 |
| Inspect | Measure (Distance, Angle) | setelah M4 |
| Assemble | Component (Place); Relationships (Joint, Constrain); Pattern; Manage (Bill of Materials, Parameters); Productivity (Ground and Root); Work Features | R3 (setelah M6), sebelum itu nonaktif |
| Annotate | Dimension, Text, view gambar | R3 (setelah M7), sebelum itu nonaktif |

Tab Design, Factory, Electromechanical, Collaborate, dan Fusion 360 di luar ruang lingkup dan tidak dibuat.

### Ikon dan merek
Tata letak dan nama perintah umum (Extrude, Fillet, Mirror) boleh dipakai. Ikon, warna merek, dan aset Autodesk tidak boleh disalin. Gunakan ikon buatan sendiri atau set ikon open source dengan lisensi yang jelas dan tercatat.

## 15. Backlog Tugas Berurutan
Kerjakan berurutan. Satu tugas selesai (acceptance criteria terpenuhi) sebelum tugas berikutnya dimulai.
1. **T1 Kerangka ribbon (R0)** [SELESAI]. Tab, panel, tombol besar dan kecil, dropdown, tooltip, status nonaktif ("Belum tersedia"), ingat tab terakhir, quick access toolbar, dan command search. Dirender dari definisi data (`RibbonSchema`), 12 unit test lulus di inventor-app.
2. **T2 Command registry dan perintah yang sudah ada (R1)** [SELESAI]. Pindahkan Box, Cylinder, ekspor STEP dan STL, undo dan redo, serta kamera ke `CommandRegistry` dan sambungkan ke tombol ribbon, quick access toolbar, menu bar, dan shortcut. 18 unit test lulus di inventor-app dan 5 unit test di inventor-core.
3. **T3 Peningkatan solver** [SELESAI]. Solver numerik Levenberg-Marquardt dengan matriks Jacobian analitis (`nalgebra`), penghitungan DOF akurat berbasis dekomposisi nilai singular (SVD rank), deteksi sistem over-constrained (konflik nilai dan redundansi baris), serta tipe constraint baru: `Tangent` (garis-lingkaran dan lingkaran-lingkaran), `Equal` (panjang garis dan radius lingkaran), `Angle` (sudut berarah dalam derajat), dan `Diameter` (diameter lingkaran). Dilengkapi 11 unit test komprehensif yang menguji under-, fully-, dan over-constrained sketches serta konvergensi stabil pada constraint yang saling bergantung.
4. **T4 Mode sketsa di UI** [SELESAI]. Masuk sketsa pada bidang datum (XY, XZ, YZ) atau planar face, ray-casting picking ke koordinat bidang 2D $(u, v)$, auto-snapping ke titik terdekat, live rubber-band preview beserta dimensi interaktif, penempatan entitas (Line, Rectangle, Circle, Point), penerapan constraint geometris dan dimensi dari ribbon & dialog modal, indikator DOF real-time (`Underconstrained`, `FullyConstrained`, `Overconstrained`) dengan status badge dan pewarnaan entitas (hijau untuk fully-constrained, cyan untuk under-constrained, amber untuk terseleksi), inspector sketsa aktif di browser dengan tombol edit & hapus constraint, Finish Sketch untuk commit profil sketsa ke feature tree dokumen, dan pengeditan kembali sketsa tersimpan. 22 unit test lulus di inventor-app.
5. **T5 Irisan vertikal parametrik** [SELESAI]. Ekstraksi profil poligon tertutup dari sketsa 2D (`extract_closed_profiles`), pengikatan parameter `Constraint.parameter_name` / `Constraint.with_parameter`, regenerasi solid dokumen otomatis via `AppState::rebuild_solid` (menyinkronkan `ParameterTable` ke solver Levenberg-Marquardt lalu mengekstrusi profil sketsa via `GeometryKernel::extrude_polygon`), transformasi orientasi bidang datum 3D (`CadMesh::transform_by_basis`), dialog interaktif `CreateExtrude` untuk membuat fitur ekstrusi dari sketsa di UI, serta automated tests komprehensif untuk pengubahan parameter dan ekstrusi sketsa baru. 24 unit test lulus di inventor-app.
6. **T6 Feature tree interaktif** [SELESAI]. Edit ulang parameter fitur via dialog `ActiveModal::EditExtrude` dan mode sketsa; suppress dan unsuppress fitur di tree dengan dukungan undo/redo (`ToggleSuppressFeatureCmd`); reorder fitur modeling naik/turun (`ReorderFeatureCmd`) dengan perlindungan 7 origin datum tetap; validasi dependensi berurutan pada `AppState::rebuild_solid` di mana fitur yang kehilangan dependensi hulu (akibat di-suppress atau dipindah posisinya ke atas sebelum prerequisite) ditandai sebagai `FeatureState::Error` dengan pesan galat deskriptif dan badge visual `[⚠ Error]` pada browser; serta pemulihan otomatis saat dependensi dipulihkan. 27 unit test lulus di inventor-app dan 8 unit test di inventor-core.
7. **T7 Revolve dan boolean** (union, subtract, intersect) lewat GeometryKernel [SELESAI]. Trait `GeometryKernel` diperluas dengan `revolve_polygon`, `boolean_shapes`, dan `build_solid_mesh`. Implementasi pure-Rust B-rep di `TruckKernel` memanfaatkan `truck-shapeops 0.4.0` untuk Union (`or`), Intersect (`and`), dan Subtract (`and` dengan inverted tool `cut.not()`), dilengkapi isolasi crash `std::panic::catch_unwind`, perpanjangan epsilon untuk pemotongan non-manifold, serta dukungan compound multi-body untuk body disjoint/kontak boundary. Di sisi UI: pendaftaran perintah `model.revolve` dan `model.combine`, dialog interaktif `CreateRevolve`, `EditRevolve`, dan `BooleanCombine`, integrasi tree browser dengan ikon `🔄` dan `⧉`, serta regenerasi otomatis solid multi-fitur berurutan di `AppState::rebuild_solid`. 8 unit test di inventor-kernel dan 29 unit test di inventor-app.
8. **T8 Evaluasi kernel untuk fillet dan chamfer** [SELESAI]. Evaluasi mendalam terhadap pure-Rust `truck` (0.6.0) memastikan ketiadaan algoritma edge fillet (rolling-ball blend), chamfer (bevel), dan re-trimming topologi corner vertex pada B-rep 3D. Arsitektur dual-kernel diimplementasikan melalui `DualKernel` dan `OpenCascadeKernel` di balik trait `GeometryKernel` yang identik tanpa merusak isolasi lapisan aplikasi. Tombol ribbon `Fillet` (`model.fillet`) dan `Chamfer` (`model.chamfer`) pada panel `Modify` serta `Options` (`tools.options`) diaktifkan; dialog modal `CreateFillet`, `EditFillet`, `CreateChamfer`, `EditChamfer`, dan `Options` diimplementasikan; browser tree dilengkapi ikon `⌒` dan `◺` beserta tombol edit interaktif; serta regenerasi solid di `AppState::rebuild_solid` menampilkan diagnostik terstruktur `[⚠ Error]` saat backend truck aktif sementara geometri solid dasar tetap utuh di viewport, dan berhasil melakukan operasi dress-up saat backend OpenCASCADE aktif. 10 unit test lulus di inventor-kernel dan 31 unit test di inventor-app.
9. **T9 Pengujian lanjutan** [SELESAI]. Round-trip STEP (ekspor B-rep solid ke format ISO 10303-21, parsing entitas via `truck-stepio`, rekonstruksi topologi shell & solid B-rep via `StepImporter`, dan triangulasi tessellation), pemeriksaan mesh tertutup (inspeksi analitis 2-manifold pada `CadMesh` dengan toleransi 1-mikron, deteksi 0 boundary edges, 0 non-manifold edges, orientasi permukaan konsisten, volume signed via Teorema Divergensi Gauss, dan luas permukaan), serta golden regression test untuk determinisme 5 siklus regenerasi parametrik, verifikasi sensitivitas delta volume, dan round-trip STEP. 11 unit test di inventor-kernel, 5 unit test di inventor-io, dan 33 unit test di inventor-app (total 71 unit test di workspace).
10. **T10 Migrasi rendering ke wgpu** [SELESAI]. Implementasi hardware-accelerated 3D viewport renderer menggunakan `wgpu 30` di `inventor-render` (`WgpuCadRenderer`) dengan custom WGSL shaders (`cad_mesh.wgsl` dan `cad_line.wgsl`), hardware depth testing (`Depth32Float`), polygon offset depth bias (mencegah z-fighting pada feature edges), buffer management (vertex, index, line, camera & light uniforms), offscreen multi-pass rendering, dan integrasi native texture target ke `egui-wgpu` di `inventor-app` (`viewport.rs`). Dilengkapi fallback otomatis ke software painter dan automated tests (3 unit test di `inventor-render`, 34 unit test di `inventor-app`, total 73 unit test lulus di workspace).

## 16. Aturan untuk Agen Pengembang
- Jangan menambah fitur di luar tugas yang sedang aktif.
- Satu tugas, satu perubahan (commit atau PR) yang bisa ditinjau sendiri.
- Jangan menandai tugas selesai sebelum cargo fmt, cargo clippy, dan cargo test lolos, dan acceptance criteria diperiksa satu per satu.
- Patuhi arah dependensi crate di Bagian 13. Jika sebuah tugas tampak menuntut pelanggarannya, berhenti dan catat di Pertanyaan Terbuka.
- Prinsip clean-room berlaku: jangan menyalin kode, ikon, teks, atau aset Autodesk, dan jangan mengklaim kompatibilitas format .ipt, .iam, atau .idw.
- Saat ragu pada keputusan arsitektur, jangan menebak. Tulis pilihannya beserta trade-off di Pertanyaan Terbuka.
- Perbarui status milestone di Bagian 12 setiap tugas selesai.

## 17. Pertanyaan Terbuka (Sisa)
- Lisensi proyek (MIT atau Apache-2.0, atau copyleft seperti GPL).
- Target platform awal dan perlu tidaknya build web.
- Dikerjakan sendiri atau terbuka untuk kontributor sejak awal.
- Hasil evaluasi truck untuk fillet, chamfer, dan boolean (T8): [TERJAWAB / SELESAI]. Truck memadai untuk B-rep boolean dasar (union, subtract, intersect via `truck-shapeops 0.4.0`), tetapi tidak memiliki algoritma fillet dan chamfer 3D. Solusi dual-kernel (`TruckKernel` sebagai backend pure-Rust default + `OpenCascadeKernel` sebagai backend kedua) di balik trait `GeometryKernel` yang terpadu telah diterapkan dan diverifikasi.
- Pemilihan set ikon yang berlisensi jelas.
