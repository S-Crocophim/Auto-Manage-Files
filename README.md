# Auto File Organizer

A modern Windows desktop application built with Python to automatically organize files based on their extensions. It features real-time directory monitoring, a sleek Black & White GUI, Windows toast notifications, and System Tray integration.

---

## 🇮🇩 Tutorial Instalasi & Menjalankan (Indonesian)

### Prasyarat
Pastikan Anda sudah menginstal **Python 3.x** di PC Anda. 
*(Jika belum, unduh dari [python.org](https://www.python.org/downloads/) dan pastikan mencentang "Add Python to PATH" saat instalasi).*

### 1. Instalasi Dependensi (Library)
Buka Terminal / Command Prompt di folder proyek ini, lalu jalankan perintah berikut untuk menginstal semua library yang dibutuhkan:
```bash
pip install -r requirements.txt
```

### 2. Cara Menjalankan Aplikasi (Mode Script)
Untuk menjalankan aplikasi secara langsung dari kode sumber (source code):
```bash
python src/main.py
```
*(Atau gunakan `py src/main.py` jika perintah `python` tidak dikenali).*

### 3. Cara Membangun (Build) Menjadi File .exe (Standalone)
Jika Anda ingin membuat aplikasi ini menjadi program mandiri (`.exe`) agar bisa dijalankan tanpa membuka terminal:
```bash
python build_exe.py
```
Setelah proses selesai, Anda akan menemukan file **`AutoFileOrganizer.exe`** di dalam folder `dist`. Anda bisa memindahkan file `.exe` tersebut ke tempat yang aman dan menjalankannya dari sana.

### Fitur Auto-Start (Berjalan Otomatis Saat PC Menyala)
Jika Anda ingin aplikasi ini langsung berjalan senyap di background saat PC dinyalakan:
1. Buka aplikasi (disarankan menggunakan versi `.exe` yang sudah di-build).
2. Pada menu sidebar sebelah kiri, nyalakan sakelar (toggle) **🚀 Auto-Start**.
3. Selesai! Anda bisa menutup jendela aplikasi, dan aplikasi akan tetap siaga di System Tray (pojok kanan bawah layar).

---
---

## 🇬🇧 Installation & Running Tutorial (English)

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

### 3. How to Build as a Standalone .exe
If you want to compile this application into a standalone Windows executable (`.exe`) so it runs without a console window:
```bash
python build_exe.py
```
Once the process is complete, you will find the **`AutoFileOrganizer.exe`** file inside the `dist` folder. You can move this `.exe` file to a safe location and run it from there.

### Auto-Start Feature (Run Automatically on PC Boot)
If you want the application to automatically start silently in the background when your PC boots up:
1. Open the application (it is recommended to use the built `.exe` version).
2. On the left sidebar menu, toggle on the **🚀 Auto-Start** switch.
3. That's it! You can close the application window, and it will remain active in the System Tray (bottom right corner of your screen).
