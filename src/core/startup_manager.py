"""
startup_manager.py
------------------
Handles adding/removing the application from Windows auto-start via
the Registry key  HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run.

Menangani penambahan/penghapusan aplikasi dari auto-start Windows melalui
kunci Registry  HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run.
"""

import os
import sys
import logging

logger = logging.getLogger(__name__)

_REG_KEY_PATH = r"Software\Microsoft\Windows\CurrentVersion\Run"
_APP_NAME = "AutoFileOrganizer"


def _get_executable_path() -> str:
    """
    Return the path to the current executable.
    If running as a PyInstaller bundle, returns the .exe path.
    Otherwise returns the Python script invocation command.

    Mengembalikan jalur ke executable saat ini.
    """
    if getattr(sys, "frozen", False):
        # Running as compiled .exe (PyInstaller)
        return sys.executable
    else:
        # Running as a script – use pythonw to avoid console window
        return f'"{sys.executable}" "{os.path.abspath(sys.argv[0])}"'


def is_auto_start_enabled() -> bool:
    """
    Check if the app is registered for auto-start in the Registry.
    Cek apakah aplikasi terdaftar untuk auto-start di Registry.
    """
    try:
        import winreg
        key = winreg.OpenKey(winreg.HKEY_CURRENT_USER, _REG_KEY_PATH, 0, winreg.KEY_READ)
        try:
            winreg.QueryValueEx(key, _APP_NAME)
            return True
        except FileNotFoundError:
            return False
        finally:
            winreg.CloseKey(key)
    except Exception as exc:
        logger.error("Failed to read auto-start registry: %s", exc)
        return False


def enable_auto_start() -> bool:
    """
    Register the application for Windows auto-start.
    Mendaftarkan aplikasi untuk auto-start Windows.
    """
    try:
        import winreg
        exe_path = _get_executable_path()
        key = winreg.OpenKey(
            winreg.HKEY_CURRENT_USER, _REG_KEY_PATH, 0, winreg.KEY_SET_VALUE
        )
        winreg.SetValueEx(key, _APP_NAME, 0, winreg.REG_SZ, exe_path)
        winreg.CloseKey(key)
        logger.info("Auto-start enabled: %s", exe_path)
        return True
    except Exception as exc:
        logger.error("Failed to enable auto-start: %s", exc)
        return False


def disable_auto_start() -> bool:
    """
    Remove the application from Windows auto-start.
    Menghapus aplikasi dari auto-start Windows.
    """
    try:
        import winreg
        key = winreg.OpenKey(
            winreg.HKEY_CURRENT_USER, _REG_KEY_PATH, 0, winreg.KEY_SET_VALUE
        )
        try:
            winreg.DeleteValue(key, _APP_NAME)
        except FileNotFoundError:
            pass  # already removed
        winreg.CloseKey(key)
        logger.info("Auto-start disabled.")
        return True
    except Exception as exc:
        logger.error("Failed to disable auto-start: %s", exc)
        return False


def set_auto_start(enabled: bool) -> bool:
    """Convenience wrapper / Pembungkus praktis."""
    return enable_auto_start() if enabled else disable_auto_start()
