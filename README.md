# 🚀 Auto File Organizer

A modern Windows desktop application built with Python (`customtkinter`) to automatically organize files based on their extensions. It features real-time directory monitoring, a sleek dynamic GUI (Dark/Light mode), Multi-Language support (ID/EN), Windows toast notifications, and System Tray integration.

---

## 🇮🇩 Tutorial Instalasi & Penggunaan (Indonesian)

### Prasyarat
Pastikan Anda sudah menginstal **Python 3.x** di PC Anda. 
*(Jika belum, unduh dari [python.org](https://www.python.org/downloads/) dan pastikan mencentang "Add Python to PATH" saat instalasi).*

### 1. Instalasi Dependensi (Library)
Buka Terminal / Command Prompt di folder proyek ini, lalu jalankan perintah berikut untuk menginstal semua library yang dibutuhkan:
```bash
pip install -r requirements.txt
```

### 2. Cara Menjalankan Aplikasi (Mode Script)
Untuk menjalankan aplikasi secara langsung dari kode sumber:
```bash
python src/main.py
```
*(Atau gunakan `py src/main.py` jika perintah `python` tidak dikenali).*

### 3. Cara Membangun (Build) Menjadi File .exe
Aplikasi ini sudah dilengkapi dengan script otomatis untuk mem-*build* versi **Portable** maupun versi **Full Directory** sekaligus.
```bash
python build_exe.py
```
Setelah proses selesai, periksa folder `dist/`:
- `AutoFileOrganizer_Portable.exe` : Versi satu file (Standalone) yang bisa langsung dijalankan.
- `AutoFileOrganizer_Full/` : Folder aplikasi lengkap (digunakan untuk membuat Installer).

### 4. Membuat Full Installer (.exe)
Jika Anda ingin membuat installer profesional yang bisa di-install ke direktori Program Files:
1. Jalankan `python build_exe.py` terlebih dahulu.
2. Unduh dan install [Inno Setup Compiler](https://jrsoftware.org/isinfo.php).
3. Buka file `setup_installer.iss` yang ada di folder utama menggunakan Inno Setup Compiler.
4. Klik tombol **Compile** (ikon panah hijau/Run).
5. File installer `AutoFileOrganizer_Installer.exe` akan muncul di folder `dist/`!

### Fitur-Fitur Utama
- **Top Bar Navigation:** Berpindah menu dengan mudah (Dashboard, Logs, Tutorial, About).
- **Auto-Start & System Tray:** Berjalan senyap di background saat PC menyala.
- **Export/Import Settings:** Simpan konfigurasi *rules* dan setelan Anda ke file `.json` dan muat kembali kapan saja.
- **Dynamic Themes & Languages:** Ganti tema gelap/terang dan bahasa secara instan (Real-time).

---
---

## 🇬🇧 Installation & Usage Tutorial (English)

### Prerequisites
Make sure you have **Python 3.x** installed on your PC. 
*(If not, download it from [python.org](https://www.python.org/downloads/) and ensure you check "Add Python to PATH" during installation).*

### 1. Installing Dependencies
Open a Terminal / Command Prompt in this project folder, and run the following command to install all required libraries:
```bash
pip install -r requirements.txt
```

### 2. How to Run the Application (Script Mode)
To run the application directly from the source code:
```bash
python src/main.py
```
*(Or use `py src/main.py` if the `python` command is not recognized).*

### 3. How to Build Executables (.exe)
This application includes an automated build script to compile both the **Portable** and **Full Directory** versions simultaneously.
```bash
python build_exe.py
```
Once the process is complete, check the `dist/` folder:
- `AutoFileOrganizer_Portable.exe` : A single standalone executable file.
- `AutoFileOrganizer_Full/` : A complete application directory (used for building the Installer).

### 4. Creating a Full Installer (.exe)
If you want to create a professional installer that installs to Program Files:
1. Run `python build_exe.py` first to generate the full directory.
2. Download and install [Inno Setup Compiler](https://jrsoftware.org/isinfo.php).
3. Open the `setup_installer.iss` file located in the root folder using Inno Setup Compiler.
4. Click the **Compile** button (green play icon/Run).
5. The `AutoFileOrganizer_Installer.exe` will be generated in the `dist/` folder!

### Key Features
- **Top Bar Navigation:** Easily switch between menus (Dashboard, Logs, Tutorial, About).
- **Auto-Start & System Tray:** Runs silently in the background on PC startup.
- **Export/Import Settings:** Save your rules and configuration to a `.json` file and load it back anytime.
- **Dynamic Themes & Languages:** Switch between Dark/Light modes and languages instantly.

---
*Made with ❤️ by Px0*
