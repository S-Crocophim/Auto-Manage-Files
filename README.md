# 📁 Auto File Organizer

<p align="center">
  <img src="public/icon.png" width="100" height="100" alt="Auto File Organizer Logo" />
  <br/>
  <b>Ultra-lightweight, blazingly fast native automatic file organizer built with Tauri v2, Rust & Vue 3.</b>
  <br/>
  <i>Aplikasi pemilah file otomatis super ringan dan cepat ditenagai oleh Rust & Vue 3.</i>
</p>

---

## 🌐 Language Navigation / Navigasi Bahasa
- [🇮🇩 Bahasa Indonesia](#-bahasa-indonesia)
- [🇬🇧 English](#-english)

---

<a name="-bahasa-indonesia"></a>
## 🇮🇩 Bahasa Indonesia

### 📌 Tentang Aplikasi
**Auto File Organizer** adalah aplikasi desktop native untuk Windows yang secara otomatis merapikan file di folder Anda (seperti *Downloads*, *Documents*, atau *Desktop*) secara real-time berdasarkan ekstensi, pola nama file, dan ukuran file.

### ✨ Fitur Utama
- **⚡ Performa Native Super Ringan:** Menggunakan Rust di backend dengan pemakaian RAM sangat minim (~15–30 MB).
- **📂 Pemantauan Real-Time (Folder Watcher):** Memantau folder secara latar belakang tanpa membebankan CPU.
- **🎨 Tampilan Modern & Dwibahasa:** Mode Gelap (Dark) & Terang (Light) dengan pilihan bahasa Indonesia dan Inggris.
- **↩️ Fitur Undo Pemindahan File:** Kembalikan file yang baru saja dipindahkan hanya dengan 1 klik dari tab Logs.
- **🛡️ Penanganan Duplikat Fleksibel:** Pilihan aksi duplikat: *Auto-rename* (ganti nama otomatis), *Skip* (lewati), atau *Overwrite* (timpa).
- **🎯 Filter Lanjutan:** Pengaturan pola nama wildcard (`Invoice_*`) dan batas ukuran file (Min / Max MB).
- **📁 Drag & Drop Folder:** Tarik dan lepas folder dari Explorer langsung ke aplikasi.
- **🔽 System Tray & Auto-Start:** Minimalkan ke System Tray saat ditutup dan opsi berjalan otomatis saat Windows menyala.

---

### 📦 Cara Install & Menggunakan

#### Cara 1: Mengunduh Installer / Portable `.exe` (Untuk Pengguna Langsung)
1. Buka halaman **[Releases](https://github.com/S-Crocophim/Auto-Manage-Files/releases)** di GitHub.
2. Unduh file rilis pilihan Anda:
   - **Installer Setup (`.exe`)**: `Auto File Organizer_1.0.0_x64-setup.exe` (Jalankan dan ikuti wizard instalasi).
   - **Portable Version (`.exe`)**: `AutoFileOrganizer-Portable.exe` (Bisa langsung dijalankan tanpa perlu install).
3. Jalankan aplikasi, buka tab **Dashboard**, dan klik **Tambah Aturan** untuk mulai merapikan file Anda!

---

#### Cara 2: Menjalankan dari Source Code (Untuk Pengembang / Developer)

##### Prasyarat Sistem:
- [Node.js](https://nodejs.org/) (v18+)
- [Rust & Cargo](https://www.rust-lang.org/tools/install)
- [C++ Build Tools for Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

##### Langkah-langkah:
```bash
# 1. Clone repository
git clone https://github.com/S-Crocophim/Auto-Manage-Files.git
cd Auto-Manage-Files

# 2. Install dependensi Node.js
npm install

# 3. Jalankan aplikasi mode Development
npm run tauri dev

# 4. Build installer rilis (.exe)
npx tauri build
```

---
<br/>

<a name="-english"></a>
## 🇬🇧 English

### 📌 About The App
**Auto File Organizer** is a native desktop application for Windows that automatically sorts files in your watched directories (like *Downloads*, *Documents*, or *Desktop*) in real-time based on file extensions, wildcard patterns, and file sizes.

### ✨ Key Features
- **⚡ Blazingly Fast & Lightweight:** Powered by Rust backend with minimal memory footprint (~15–30 MB RAM).
- **📂 Real-Time Folder Monitoring:** Background folder watcher that detects incoming files instantly without CPU overhead.
- **🎨 Modern Bilingual Interface:** Clean dark & light mode with toggleable Indonesian and English language support.
- **↩️ 1-Click Move Undo:** Revert recent file moves with a single click from the Activity Logs view.
- **🛡️ Flexible Conflict Handling:** Choose between *Auto-rename*, *Skip*, or *Overwrite* for duplicate files.
- **🎯 Advanced Filters:** Filter by filename wildcard patterns (`Invoice_*`) and file size bounds (Min / Max MB).
- **📁 Drag & Drop Folder Support:** Drag folders directly from Windows Explorer into path inputs.
- **🔽 System Tray & Auto-Start:** Minimizes to system tray on window close and optional launch on Windows startup.

---

### 📦 Installation & Usage Guide

#### Option 1: Downloading Ready-to-Use `.exe` (For End Users)
1. Go to the **[Releases](https://github.com/S-Crocophim/Auto-Manage-Files/releases)** page on GitHub.
2. Download your preferred binary:
   - **Setup Installer (`.exe`)**: `Auto File Organizer_1.0.0_x64-setup.exe` (Run installer wizard).
   - **Portable Version (`.exe`)**: `AutoFileOrganizer-Portable.exe` (Standalone executable, no setup required).
3. Open the app, navigate to **Dashboard**, and click **Add Rule** to begin organizing files!

---

#### Option 2: Building from Source (For Developers)

##### Prerequisites:
- [Node.js](https://nodejs.org/) (v18+)
- [Rust & Cargo](https://www.rust-lang.org/tools/install)
- [C++ Build Tools for Visual Studio](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

##### Build Steps:
```bash
# 1. Clone the repository
git clone https://github.com/S-Crocophim/Auto-Manage-Files.git
cd Auto-Manage-Files

# 2. Install Node dependencies
npm install

# 3. Run application in development mode
npm run tauri dev

# 4. Build production installer & executable (.exe)
npx tauri build
```

---

## 📜 License & Credits

- **Author / Developer:** Px0 (`S-Crocophim`)
- **Technologies:** [Tauri v2](https://tauri.app/), [Rust](https://www.rust-lang.org/), [Vue 3](https://vuejs.org/), [Vite](https://vitejs.dev/)
- **License:** MIT License
