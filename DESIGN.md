# Spesifikasi Paritas Pengalaman (Experience Parity): inventor-rs

Dokumen ini mendefinisikan standar visual, tata letak antarmuka, dan perilaku interaksi **1:1 Experience Parity** yang mengacu pada alur kerja **Autodesk Inventor Professional**, dengan tetap mematuhi prinsip **clean-room engineering** (tanpa menyalin aset berhak cipta, ikon proprietary, atau format biner Autodesk).

---

## 1. Prinsip Utama & Filosofi Desain

1. **Autodesk Inventor Workflow Parity**:
   - Setiap interaksi pengguna (navigasi kamera, pembuatan sketsa, ekstrusi, pohon riwayat fitur, hingga hierarki peramban) dirancang agar pengguna Autodesk Inventor merasa langsung familiar tanpa kurva belajar baru.
2. **Clean-Room Implementation**:
   - Tata letak, logika navigasi, dan nomenklatur standar CAD (Extrude, Revolve, Fillet, Coincident, Tangent) diterapkan secara bebas.
   - Tidak menggunakan ikon biner, grafik bitmap, atau kode dari Autodesk. Seluruh ikon menggunakan representasi vektor geometris prosedural atau open-source berlisensi jelas (MIT / Apache-2.0).
3. **Efisiensi & Kepadatan Informasi (Dense Engineering UI)**:
   - Ribbon padat dengan tombol besar (Large Button dengan ikon + label vertikal) untuk aksi primer dan tombol kecil (Small/Dropdown Button) untuk aksi sekunder.
   - Model browser bersih dan terstruktur tanpa tombol inline yang mengotori daftar.

---

## 2. Navigasi Kamera & Pintasan Standar Inventor

Pintasan mouse dan keyboard mengikuti standar baku Autodesk Inventor:

| Masukan Pengguna | Aksi Viewport | Keterangan |
|---|---|---|
| **MMB Drag (Klik Tengah Drag)** | **Pan (Geser Kamera)** | Menggeser bidang pandang secara halus mengikuti gerakan kursor |
| **Shift + MMB Drag** | **Orbit (Putar 3D Bebas)** | Memutar kamera orbit di sekitar titik target fokus |
| **Wheel Scroll** | **Zoom ke Posisi Kursor** | Memperbesar/memperkecil dengan titik zoom berpusat pada posisi mouse |
| **Double-Click MMB** | **Zoom All (Fit View)** | Menyesuaikan framing kamera agar seluruh model solid masuk pas di layar |
| **F6** | **Reset Home View** | Mengembalikan orientasi kamera ke Isometrik Home (Front-Top-Right) |
| **Esc** | **Cancel Active Tool** | Membatalkan alat aktif dan kembali ke mode seleksi kursor default |

---

## 3. Struktur Ribbon & Tata Letak Aplikasi

Menu bar teks lama (*File / Edit / Help*) dihilangkan dan digantikan oleh struktur Ribbon Inventor terpadu:

### 3.1 Tombol Aplikasi & Quick Access Toolbar
- **Tombol Aplikasi "File"**: Berada di sudut kiri atas ribbon dengan aksen warna tegas. Membuka menu aplikasi (New Part, Open, Save, Export STEP/STL, Exit).
- **Quick Access Toolbar (QAT)**: Terletak di samping tombol File atau title bar atas:
  - `New`, `Open`, `Save`, `Undo (Ctrl+Z)`, `Redo (Ctrl+Y)`, `Home View (F6)`.

### 3.2 Tab & Panel Ribbon
Ribbon disusun dalam panel-panel terstruktur dengan label di bagian bawah panel:

#### Tab "3D Model" (Saat Berada di Lingkungan Part Modeling)
1. **Panel "Sketch"**:
   - Tombol Besar: **Start 2D Sketch** (ikon pensil & bidang datum).
2. **Panel "Create"**:
   - Tombol Besar: **Extrude** (`E`), **Revolve** (`R`).
   - Tombol Kecil / Dropdown: **Box Primitive**, **Cylinder Primitive**, **Sweep** *(placeholder)*, **Loft** *(placeholder)*.
3. **Panel "Modify"**:
   - Tombol Sedang/Kecil: **Fillet** (`F`), **Chamfer**, **Combine (Boolean)**, **Hole**, **Shell**.
4. **Panel "Work Features"**:
   - Tombol Kecil: **Plane**, **Axis**, **Point**.
5. **Panel "Pattern"**:
   - Tombol Kecil: **Rectangular**, **Circular**, **Mirror**.

#### Tab "Sketch" (Otomatis Aktif Saat Masuk Mode Sketsa 2D)
1. **Panel "Create"**: Line (`L`), Circle (`C`), Arc (`A`), Rectangle (`R`), Point (`P`).
2. **Panel "Constrain"**: Coincident, Horizontal, Vertical, Parallel, Perpendicular, Tangent, Equal, Fix, Concentric.
3. **Panel "Dimension"**: Tombol Besar General Dimension (`D`).
4. **Panel "Exit"**: Tombol Besar **Finish Sketch** (ikon centang hijau khas penyelesaian sketsa).

---

## 4. Perilaku Model Browser (Pohon Fitur Part)

Model browser di panel kiri harus berpenampilan bersih dan berfungsi sesuai paradigma konsumsi fitur Inventor:

1. **Struktur Bersih (Tanpa Tombol Inline Statis)**:
   - Baris browser tidak menampilkan tombol permanen seperti `[Suppress]`, `[Edit]`, `[▲]`, `[▼]`.
   - Aksi dilakukan melalui:
     - **Klik Ganda (Double Click)**: Membuka dialog edit fitur atau masuk ke mode sketsa fitur terkait.
     - **Klik Kanan (Context Menu)**: Menampilkan menu pop-up: *Edit Feature*, *Edit Sketch*, *Suppress / Unsuppress*, *Visibility*, *Delete*, *Rename*.
2. **Folder "Origin"**:
   - Berada di posisi paling atas pohon fitur dokumen:
     - `YZ Plane`, `XZ Plane`, `XY Plane` (default: **Hidden / Invisible**).
     - `X Axis`, `Y Axis`, `Z Axis`.
     - `Center Point`.
   - Bidang datum origin hanya ditampilkan jika pengguna secara eksplisit mengaktifkan *Visibility* lewat klik kanan.
3. **Model Konsumsi Fitur (Hierarchical Consumption)**:
   - Sketsa yang digunakan oleh suatu fitur (misalnya `Sketch1` untuk `Extrude1`) secara otomatis **dikonsumsi** dan menjadi anak (*child node*) yang dapat di-*expand/collapse* di bawah `Extrude1`.
   - Urutan fitur di browser mengikuti urutan kronologis pembuatan (*history tree*), bukan terbalik.

---

## 5. Viewport 3D, ViewCube, & Rendering

1. **Framing & Shading Solid**:
   - Kamera default membingkai solid dengan margin nyaman (tidak terpotong di tepi layar).
   - Shading permukaan 3D menggunakan pencahayaan directional yang memberikan diferensiasi kontras jelas antara sisi atas, depan, dan samping kotak solid.
   - Garis wireframe permukaan planar hanya menampilkan garis batas tepi (*sharp feature edges*), tanpa menampilkan garis diagonal segitiga triangulasi internal.
2. **Interactive ViewCube (Pojok Kanan Atas Viewport)**:
   - Kubus 3D interaktif yang berputar sinkron dengan orientasi kamera.
   - Sisi kubus bertuliskan **TOP**, **FRONT**, **RIGHT**, **BACK**, **BOTTOM**, **LEFT**.
   - Mengklik salah satu sisi mengarahkan kamera tegak lurus ke sisi tersebut secara halus.
   - Hotspot sudut kubus mengarahkan kamera ke sudut isometrik terkait.
   - Ikon rumah (**Home Icon**) kecil di atas ViewCube untuk mereset kamera ke Home View (`F6`).
3. **Datum Grid & Origin Axes**:
   - Grid lantai koordinat CAD responsif terhadap zoom level.
   - Sumbu origin X (Merah), Y (Hijau), Z (Biru) ditampilkan di tengah origin dengan indikator huruf jelas.

---

## 6. Protokol Manajemen Repositori & Commit

- Setiap tugas dari backlog dikerjakan satu per satu secara terisolasi.
- **Satu Tugas = Satu Commit Git** yang deskriptif dan dapat ditinjau mandiri setelah seluruh quality gates (`cargo fmt`, `cargo clippy`, `cargo test`) lulus 100%.
