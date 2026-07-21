"""
build_exe.py
------------
PyInstaller build script for Auto File Organizer.
Creates a standalone Windows .exe in the `dist/` directory.

Skrip build PyInstaller untuk Auto File Organizer.
Membuat file .exe standalone Windows di direktori `dist/`.

Usage / Penggunaan:
    python build_exe.py
"""

import subprocess
import sys
import os

# ------------------------------------------------------------------
# Configuration / Konfigurasi
# ------------------------------------------------------------------
APP_NAME = "AutoFileOrganizer"
ENTRY_POINT = os.path.join("src", "main.py")
ICON_PATH = os.path.join("src", "assets", "app.ico")  # optional

# Build command / Perintah build
cmd = [
    sys.executable, "-m", "PyInstaller",
    "--noconfirm",
    "--onefile",
    "--windowed",          # no console window
    "--name", APP_NAME,
    "--clean",
]

# Add icon if it exists / Tambahkan ikon jika ada
if os.path.isfile(ICON_PATH):
    cmd += ["--icon", ICON_PATH]

# Add hidden imports that PyInstaller might miss
cmd += [
    "--hidden-import", "pystray._win32",
    "--hidden-import", "winotify",
    "--hidden-import", "customtkinter",
]

# Entry point
cmd.append(ENTRY_POINT)

print("=" * 60)
print(f"  Building {APP_NAME}")
print("=" * 60)
print(f"  Command: {' '.join(cmd)}")
print()

result = subprocess.run(cmd, cwd=os.path.dirname(os.path.abspath(__file__)))

if result.returncode == 0:
    print()
    print("=" * 60)
    print(f"  ✅ Build successful!")
    print(f"  Executable: dist/{APP_NAME}.exe")
    print("=" * 60)
else:
    print()
    print("=" * 60)
    print(f"  ❌ Build failed with exit code {result.returncode}")
    print("=" * 60)

sys.exit(result.returncode)
