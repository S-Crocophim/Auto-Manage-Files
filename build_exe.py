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

# Build Portable
print("Building Portable...")
cmd_portable = [
    sys.executable, "-m", "PyInstaller",
    "--noconfirm", "--onefile", "--windowed",
    "--name", "AutoFileOrganizer_Portable",
    "--clean"
]
if os.path.isfile(ICON_PATH):
    cmd_portable += ["--icon", ICON_PATH]
cmd_portable += ["--hidden-import", "pystray._win32", "--hidden-import", "winotify", "--hidden-import", "customtkinter", ENTRY_POINT]
subprocess.run(cmd_portable, cwd=os.path.dirname(os.path.abspath(__file__)))

# Build Full Directory
print("Building Full Directory...")
cmd_full = [
    sys.executable, "-m", "PyInstaller",
    "--noconfirm", "--onedir", "--windowed",
    "--name", "AutoFileOrganizer_Full",
    "--clean"
]
if os.path.isfile(ICON_PATH):
    cmd_full += ["--icon", ICON_PATH]
cmd_full += ["--hidden-import", "pystray._win32", "--hidden-import", "winotify", "--hidden-import", "customtkinter", ENTRY_POINT]
result = subprocess.run(cmd_full, cwd=os.path.dirname(os.path.abspath(__file__)))

if result.returncode == 0:
    print("\n" + "=" * 60)
    print("  ✅ All Builds Successful!")
    print("  Portable: dist/AutoFileOrganizer_Portable.exe")
    print("  Full Dir: dist/AutoFileOrganizer_Full/")
    print("=" * 60)
else:
    print("\n" + "=" * 60)
    print(f"  ❌ Build failed with exit code {result.returncode}")
    print("=" * 60)

sys.exit(result.returncode)
