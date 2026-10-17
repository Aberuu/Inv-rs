# PRD v0.3: inventor-rs (Paritas Pengalaman Inventor)

Status: v0.3, menggantikan v0.1 dan v0.2 Owner: Abel Dokumen ini juga berfungsi sebagai spesifikasi kerja untuk agen pengembang (Antigravity).

## 1. Ringkasan

inventor-rs adalah aplikasi CAD 3D parametrik desktop native yang ditulis dengan Rust. Targetnya: alur kerja dan perilaku antarmuka yang terasa 1:1 dengan Autodesk Inventor, dengan kemampuan fungsional yang cukup untuk membuat part sederhana sampai menengah (misalnya braket dan roda gigi sederhana).

Proyek ini terinspirasi CADCraft dari ArtCraft (reimplementasi AutoCAD clean-room dengan Rust dan egui). Perbedaan utamanya: Inventor adalah pemodelan solid 3D parametrik, bukan drafting 2D, sehingga butuh kernel B-rep, constraint solver, dan feature tree.

Catatan penting tentang makna "1:1": yang dikejar adalah **pengalaman** (alur kerja, tata letak, perilaku interaksi), bukan paritas fitur penuh dengan perangkat lunak yang dikembangkan puluhan tahun.

## 2. Tujuan dan Non-Tujuan

### Tujuan

- Paritas pengalaman dengan Inventor untuk alur kerja inti: sketsa, constraint, fitur, riwayat fitur.
- Model parametrik: perubahan dimensi memperbarui seluruh model.
- Format tukar terbuka (STEP, STL).
- Aplikasi desktop native yang ringan dan cepat.
- Pengembangan bertahap, setiap tugas punya kriteria selesai yang bisa diuji.

### Non-Tujuan

- Kompatibilitas format `.ipt`, `.iam`, `.idw` (proprietary).
- Paritas fitur penuh dengan Inventor.
- Simulasi (FEA), CAM, rendering fotorealistik, iLogic.
- Menyalin kode, ikon, aset, atau teks antarmuka milik Autodesk.
- Build web.

## 3. Prinsip Hukum dan Pengembangan

- Pendekatan clean-room: tata letak, alur kerja, dan nama perintah umum (Extrude, Fillet, Mirror) boleh ditiru. Ikon, logo, aset, dan kode Autodesk tidak boleh.
- Ikon dibuat sendiri atau memakai set ikon open source dengan lisensi jelas yang dicatat.
- Kemiripan visual yang terlalu persis berisiko hukum. Jika proyek akan dirilis publik atau dikomersialkan, konsultasikan ke ahli hukum kekayaan intelektual.
- Lisensi proyek: MIT OR Apache-2.0. Sebelum merilis biner, baca lisensi OpenCascade (setahu penulis LGPL 2.1 dengan pengecualian tambahan) dan lisensi binding Rust-nya, lalu pastikan cara linking sesuai.

## 4. Pengguna Target

- Hobiis, maker, dan mahasiswa teknik yang butuh CAD parametrik gratis dengan alur kerja familiar.
- Pengguna Inventor yang ingin alternatif open source.
- Pengembang yang ingin kernel dan aplikasi CAD berbasis Rust yang bisa diperluas.

## 5. Keputusan Teknis

- Platform: aplikasi desktop native (egui dan wgpu), tanpa build web.
- Pengembangan: dikerjakan sendiri.
- Kernel geometri: `truck` sebagai default (boolean dasar via `truck-shapeops`), `OpenCascadeKernel` sebagai backend kedua untuk fillet dan chamfer, keduanya di balik trait `GeometryKernel`. Hasil evaluasi ini berasal dari laporan agen dan perlu dibuktikan di U0.
- UI: `eframe` dan `egui`, dengan rendering viewport lewat wgpu dan fallback painter egui untuk lingkungan tanpa GPU.
- Constraint solver: ditulis sendiri (Levenberg-Marquardt dengan Jacobian dan penentuan DOF lewat SVD, menurut laporan agen).
- Lisensi: MIT OR Apache-2.0.

## 6. Status Terkini

- **U0 Selesai**: Baseline dibuktikan secara otomatis dengan 76 unit dan integration tests lulus (core 8, kernel 11, sketch 12, render 3, io 5, app 37).
  - Bukti (a) fillet kotak ekspor-impor STEP dengan verifikasi 14 faces dan volume lulus (`test_u0_baseline_proof_a_fillet_extrusion_step_roundtrip_and_volume`).
  - Bukti (b) modifikasi parameter dimensi meregenerasi solid lulus (`test_u0_baseline_proof_b_parameter_table_solid_regeneration`).
  - Bukti (c) solver sketsa under-, fully-, dan over-constrained dengan pesan galat jelas lulus (`test_u0_baseline_proof_c_sketch_solver_under_fully_over_constrained_clear_errors`).
  - CI workflow GitHub Actions (`.github/workflows/ci.yml`) dikonfigurasi untuk Windows (`fmt`, `clippy -D warnings`, `test`, `build`).
  - README diperbarui lengkap dengan syarat build, instruksi eksekusi, dan tabel navigasi/pintasan sesuai paritas Inventor.
- Antarmuka saat ini siap memasuki tugas UI per tugas: U1 (Ribbon baru), U2 (Model browser), U3 (Viewport), dst.
- `DESIGN.md` telah ditulis ulang sepenuhnya menjadi spesifikasi paritas pengalaman 1:1.

## 7. Arsitektur dan Konvensi Kode

### Workspace dan arah dependensi

Crate: `inventor-core`, `inventor-kernel`, `inventor-sketch`, `inventor-render`, `inventor-io`, `inventor-app`.

Arah dependensi yang tidak boleh dilanggar:

- `inventor-app` boleh bergantung pada semua crate lain.
- `inventor-render` dan `inventor-io` boleh bergantung pada `inventor-core` dan `inventor-kernel`.
- `inventor-kernel` dan `inventor-sketch` hanya boleh bergantung pada `inventor-core`.
- `inventor-core` tidak bergantung pada crate lain di workspace dan tidak mengetahui UI.

### Konvensi

- Semua perubahan dokumen melewati Command Pattern dan `HistoryStack` agar undo dan redo selalu bekerja.
- Kernel hanya diakses lewat trait `GeometryKernel`. Tidak ada pemanggilan `truck_*` atau OpenCascade di luar crate kernel.
- Satuan internal: milimeter dan derajat, dikelola `ParameterTable`.
- Jalur yang bisa dipicu pengguna tidak boleh `panic`. Gunakan `Result` dengan tipe error per crate dan tampilkan pesan galat yang jelas di UI.
- Ribbon dan pintasan berbasis data (definisi atau tabel konfigurasi), setiap tombol menunjuk ke ID perintah di command registry.
- Kode lolos `cargo fmt`, `cargo clippy` tanpa peringatan, dan `cargo test`.
- Identifier dan komentar kode dalam bahasa Inggris. Dokumentasi proyek dalam bahasa Indonesia.

## 8. Spesifikasi Paritas Pengalaman

### 8.1 Prinsip dan penanda verifikasi

- Clean-room (Bagian 3). UI padat seperti perangkat lunak teknik: tombol besar untuk aksi utama, tombol kecil untuk aksi sekunder.
- Setiap perilaku diberi penanda status. **\[V\]** berarti sudah diverifikasi di Inventor asli oleh Abel. **\[?\]** berarti belum diverifikasi.
- Agen tidak boleh mengubah \[?\] menjadi \[V\] sendiri dan tidak boleh memperlakukan \[?\] sebagai fakta. Semua perilaku \[?\] diimplementasikan lewat tabel konfigurasi yang mudah diubah.

### 8.2 Navigasi kamera

| Masukan | Aksi | Status |
| --- | --- | --- |
| Drag tombol tengah mouse | Pan | \[?\] |
| Shift + drag tombol tengah | Orbit | \[?\] |
| Scroll wheel | Zoom ke posisi kursor | \[?\] |
| Double-click tombol tengah | Zoom All (fit view) | \[?\] (mungkin perilaku AutoCAD, bukan Inventor) |
| F6 | Home view isometrik | \[?\] |
| Esc | Batalkan alat aktif | \[?\] |

### 8.3 Pintasan huruf (semua \[?\])

- Tab 3D Model: E (Extrude), R (Revolve), F (Fillet).
- Tab Sketch: L (Line), C (Circle), A (Arc), P (Point), D (General Dimension).
- Huruf R dipakai untuk Revolve di tab 3D Model dan Rectangle di tab Sketch, sehingga pemetaan bergantung pada konteks aktif.

### 8.4 Tata letak layar

- Ribbon di atas, dengan tombol File (menu aplikasi) di kiri atas dan Quick Access Toolbar (New, Open, Save, Undo, Redo, Home). Menu teks File/Edit/Help dihapus. \[?\] rincian posisi.
- Model browser di kiri.
- Viewport di tengah, dengan ViewCube di kanan atas dan navigation bar di sisi kanan viewport. \[?\]
- Status bar di bawah: prompt alat aktif, koordinat kursor, status DOF sketsa. \[?\]
- Tab dokumen. \[?\] posisi.
- Panel Properties di kanan (dari `DESIGN.md` lama) dikeluarkan dari spesifikasi sampai terbukti ada di Inventor. \[?\]
- Palet warna dan latar viewport: sementara tema gelap netral. Nilai akhir diisi dari screenshot referensi, bukan dikarang.

### 8.5 Ribbon

- Tab 3D Model: panel Sketch (Start 2D Sketch), Create (Extrude, Revolve; Box dan Cylinder di dropdown), Modify (Fillet, Chamfer, Combine, Hole, Shell), Work Features (Plane, Axis, Point), Pattern (Rectangular, Circular, Mirror). \[?\] pengelompokan persis.
- Tab Sketch (aktif otomatis saat masuk mode sketsa): panel Create (Line, Circle, Arc, Rectangle, Point), Modify (Trim, Extend, Offset, Move, Copy), Constrain (Coincident, Horizontal, Vertical, Parallel, Perpendicular, Tangent, Equal, Fix, Concentric), Dimension (General Dimension), Finish Sketch. \[?\]
- Tab lain (Assemble, Annotate, Inspect, Tools, View): tampil sebagai kerangka. Tombol tanpa perintah di baliknya harus nonaktif dengan tooltip "Belum tersedia".
- Tab Design, Factory, Electromechanical, Collaborate, dan Fusion 360 di luar ruang lingkup.

### 8.6 Model browser

- Baris tidak menampilkan tombol inline (Suppress, Edit, panah). Aksi lewat klik ganda (edit fitur atau masuk ke sketsanya) dan klik kanan (Edit Feature, Edit Sketch, Suppress atau Unsuppress, Visibility, Delete, Rename). \[?\]
- Folder Origin berisi bidang YZ, XZ, XY, sumbu X, Y, Z, dan Center Point. Bidang origin tersembunyi secara default dan tampil hanya lewat Visibility. \[?\] urutan dan default.
- Sketsa yang dipakai suatu fitur menjadi anak (child node) fitur itu dan bisa di-expand. Urutan mengikuti kronologi pembuatan. \[?\]

### 8.7 Viewport

- Framing awal membingkai solid dengan margin nyaman tanpa terpotong.
- Shading membedakan sisi atas, depan, dan samping secara jelas. Tepi yang digambar hanya sharp feature edges, tanpa garis diagonal triangulasi.
- ViewCube berbentuk kubus 3D yang berputar mengikuti kamera. Klik sisi mengarahkan kamera tegak lurus, klik sudut ke arah isometrik, dan ada ikon Home. \[?\] detail.
- Grid responsif terhadap zoom. Sumbu origin X merah, Y hijau, Z biru.

### 8.8 Referensi yang diminta dari Abel (screenshot Inventor asli)

1. Model browser dengan beberapa fitur (sketsa, extrude, fillet).
2. Viewport dengan sebuah solid, beserta ViewCube dan navigation bar.
3. Dialog Extrude dan pratinjaunya.
4. Tab Sketch lengkap, dan sketsa dengan constraint.
5. Menu klik kanan di browser dan di viewport.
6. Status bar dan tab dokumen.

## 9. Backlog

### 9.1 Fase 1 (T1 sampai T10), dilaporkan selesai oleh agen

Kerangka ribbon, command registry, solver LM, mode sketsa interaktif, irisan vertikal parametrik, feature tree interaktif, revolve dan boolean, evaluasi kernel (dual-kernel), pengujian lanjutan (STEP round-trip dan mesh watertight), dan migrasi rendering ke wgpu. Status ini dianggap belum terbukti sampai U0 selesai.

### 9.2 Fase 2: Paritas Pengalaman

Kerjakan berurutan. Satu tugas satu commit. Setiap tugas selesai bila kriteria terpenuhi, quality gate lulus di CI, dan ada screenshot hasil untuk perubahan yang terlihat.

**U0: Baseline, CI, dan pembuktian.**

- Dorong seluruh pekerjaan T1 sampai T10 ke GitHub.
- GitHub Actions menjalankan `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --workspace`, dan `cargo build --workspace` di setiap push, minimal di Windows.
- Bukti baseline berupa test otomatis: (a) fillet pada sisi kotak hasil extrude lalu ekspor STEP, impor kembali, dan periksa jumlah face dan volume; (b) mengubah satu parameter dimensi meregenerasi solid; (c) solver pada sketsa under-, fully-, dan over-constrained dengan pesan galat jelas.
- README diperbarui: syarat build (termasuk OpenCascade bila ada), cara menjalankan, dan pintasan sesuai spesifikasi.
- Kriteria: CI hijau pada checkout bersih, dan README cukup untuk membangun di mesin bersih.

**U1: Ribbon baru.**

- File button, Quick Access Toolbar, tab 3D Model dan Sketch sesuai 8.5, menu teks lama dihapus. Ribbon dirender dari definisi data.
- Kriteria: test otomatis memastikan setiap tombol aktif menunjuk ke ID perintah yang terdaftar dan tidak ada tombol aktif tanpa perintah. Extrude, Revolve, Fillet, Chamfer, Combine, Box, dan Cylinder berfungsi dari ribbon. Tab Sketch aktif otomatis saat masuk mode sketsa dan kembali ke 3D Model setelah Finish Sketch.

**U2: Model browser.**

- Tanpa tombol inline, klik ganda, menu klik kanan, folder Origin dengan bidang tersembunyi default, sketsa sebagai anak fitur, urutan kronologis.
- Kriteria: test untuk fungsi yang membangun pohon dari `Document` (urutan dan relasi parent-child), test visibilitas default origin, dan setiap item menu klik kanan menjalankan perintah yang benar.

**U3: Viewport.**

- Framing awal dan Zoom All, shading yang membedakan sisi, hanya sharp edges, ViewCube kubus interaktif dengan ikon Home.
- Kriteria: test kamera bahwa seluruh bounding box tampak dengan margin sekitar 10 persen setelah Zoom All. Test render headless bahwa kecerahan sisi atas, depan, dan samping berbeda di atas ambang. Kotak solid hanya menampilkan 12 tepi. Test pemetaan klik ViewCube ke orientasi kamera.

**U4: Navigasi dan pintasan.**

- Semua pintasan dan gerakan mouse dibaca dari satu tabel konfigurasi. Default mengikuti entri \[V\]. Entri \[?\] ditandai di tabel itu.
- Kriteria: test peristiwa masukan menjadi aksi untuk tiap baris tabel, termasuk pemetaan huruf R yang bergantung konteks.

**U5: Dialog Extrude dengan pratinjau.**

- Pemilihan profil, jarak (angka atau ekspresi parameter), arah, dan operasi (join, cut, intersect), dengan pratinjau langsung di kanvas. \[?\] daftar opsi persis.
- Kriteria: pratinjau berubah saat jarak diubah. OK membuat fitur Extrude di browser. Cancel tidak mengubah dokumen maupun tumpukan undo. Klik ganda fitur membuka dialog yang sama untuk edit ulang.

**U6: Peningkatan sketsa.**

- Trim, Extend, Offset, Move, Copy. Inferensi constraint saat menggambar (misalnya garis hampir horizontal otomatis diberi constraint Horizontal dengan penanda visual) dan indikator DOF di status bar.
- Kriteria: test per alat pada kasus baku (misalnya Trim memotong tepat di titik potong). Test inferensi menambah constraint yang benar dan tidak menambah yang salah.

**U7: Navigation bar, status bar, tab dokumen.**

- Navigation bar (Pan, Zoom, Orbit, Zoom All), status bar berfungsi (prompt alat, koordinat kursor, DOF), tab dokumen.
- Kriteria: tiap kontrol memanggil aksi kamera yang sama dengan pintasan di U4. Status bar menampilkan pesan sesuai alat aktif.

**U8: Rapikan dokumentasi.**

- Hapus `DESIGN.md` lama atau ganti dengan Bagian 8, perbarui status di Bagian 6, dan perbarui README.

### 9.3 Fase 3 (setelah Fase 2)

- Fitur lanjutan: pattern, sweep, loft, hole, shell yang matang, dan tab Inspect (pengukuran).
- Assembly: menempatkan beberapa part, joint dan constraint, daftar komponen (BOM).
- Drawing: tampilan 2D dari model 3D, dimensi dan anotasi, ekspor DXF dan PDF.
- Tiap item dirinci dengan kriteria uji sebelum dikerjakan.

## 10. Aturan untuk Agen Pengembang

- Jangan menambah fitur di luar tugas yang sedang aktif.
- Satu tugas, satu commit yang bisa ditinjau sendiri.
- Jangan menandai tugas selesai sebelum `cargo fmt`, `cargo clippy`, dan `cargo test` lolos di CI dan setiap kriteria diperiksa satu per satu. Laporan teks saja tidak cukup: sertakan hasil uji dan screenshot untuk perubahan yang terlihat.
- Patuhi arah dependensi crate di Bagian 7. Jika tugas tampak menuntut pelanggarannya, berhenti dan catat di Pertanyaan Terbuka.
- Patuhi prinsip clean-room. Jangan menyalin kode, ikon, teks, atau aset Autodesk, dan jangan mengklaim kompatibilitas format `.ipt`, `.iam`, atau `.idw`.
- Jangan mengubah penanda \[?\] menjadi \[V\]. Saat ragu pada perilaku Inventor, catat pertanyaannya dan pakai konfigurasi yang mudah diganti.
- Saat ragu pada keputusan arsitektur, jangan menebak. Tulis pilihan beserta trade-off di Pertanyaan Terbuka.
- Perbarui status di Bagian 6 setiap tugas selesai.

## 11. Persyaratan Non-Fungsional

- Platform utama: Windows. Linux dan macOS sebagai target kemudian.
- Viewport tetap interaktif untuk model menengah.
- Operasi geometri yang gagal menghasilkan pesan galat jelas, bukan crash.
- Pengujian: unit test untuk solver dan kernel, golden test untuk regenerasi dan ekspor, serta CI yang menjalankan semua quality gate.
- Build harus bisa direproduksi di mesin bersih mengikuti README.

## 12. Risiko

- Kernel B-rep dan operasi fillet, chamfer, boolean adalah bagian paling berat dan rawan galat.
- Topological naming: referensi ke face dan edge bisa putus saat fitur lama diedit, membuat feature tree rapuh.
- Solver constraint yang tidak stabil pada sketsa nyata.
- Lisensi OpenCascade dan binding Rust-nya terhadap rencana rilis biner (LGPL).
- Perpindahan bentuk antar kernel: apakah satu model selalu dikerjakan satu kernel atau berpindah di tengah feature tree, dan bagaimana presisi serta topologi dijaga.
- Dua jalur render (wgpu dan fallback painter egui): perilaku bisa berbeda, dan test bisa diam-diam memakai fallback.
- Laporan agen yang tidak terverifikasi: mitigasi lewat CI dan bukti nyata di tiap tugas.
- Risiko hukum bila meniru antarmuka atau aset Autodesk terlalu dekat.
- Cakupan terlalu luas: mitigasi dengan fase, non-tujuan yang tegas, dan kriteria uji.

## 13. Metrik Keberhasilan

- U0: CI hijau dan tiga bukti baseline lulus.
- Seluruh Fase 2: alur kerja "buat sketsa, beri constraint dan dimensi, extrude, edit ulang dimensi" bisa dijalankan dari ribbon, browser, dan dialog tanpa langkah di luar UI, dan tampilannya sesuai referensi Inventor yang sudah diverifikasi.
- Model contoh (braket dan roda gigi sederhana) dapat dibuat sepenuhnya di aplikasi dan diekspor ke STEP yang terbuka dengan benar di CAD lain.

## 14. Pertanyaan Terbuka

- Nilai akhir perilaku bertanda \[?\] di Bagian 8 (menunggu verifikasi dan screenshot dari Abel).
- Cara bentuk berpindah antar kernel (Bagian 12).
- Syarat lisensi OpenCascade dan binding Rust-nya untuk rilis biner.
- Set ikon berlisensi jelas yang akan dipakai.
- Palet warna dan latar viewport final (menunggu screenshot referensi).
