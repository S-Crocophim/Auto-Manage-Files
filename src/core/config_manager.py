"""
config_manager.py
-----------------
Manages loading and saving application configuration (rules & settings)
to a JSON file. Initializes default presets on first run.

Mengelola pemuatan dan penyimpanan konfigurasi aplikasi (aturan & pengaturan)
ke file JSON. Menginisialisasi preset default pada saat pertama kali dijalankan.
"""

import json
import os
import uuid
import logging

logger = logging.getLogger(__name__)

# ------------------------------------------------------------------
# Default path for the config file, stored next to the executable/script
# Jalur default untuk file konfigurasi, disimpan di samping executable/script
# ------------------------------------------------------------------
_APP_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DEFAULT_CONFIG_PATH = os.path.join(_APP_DIR, "config.json")

# ------------------------------------------------------------------
# Default download folder (used by preset rules)
# Folder unduhan default (digunakan oleh aturan preset)
# ------------------------------------------------------------------
_DOWNLOADS = os.path.join(os.path.expanduser("~"), "Downloads")
_DOCUMENTS = os.path.join(os.path.expanduser("~"), "Documents")
_PICTURES = os.path.join(os.path.expanduser("~"), "Pictures")


def _generate_id() -> str:
    """Generate a short unique ID for each rule / Buat ID unik pendek untuk setiap aturan."""
    return uuid.uuid4().hex[:8]


def _default_rules() -> list[dict]:
    """
    Return the three default preset rules.
    Mengembalikan tiga aturan preset default.
    """
    return [
        {
            "id": _generate_id(),
            "name": "Spreadsheets",
            "watch_folder": _DOWNLOADS,
            "extensions": ".xlsx, .xls, .csv",
            "destination": os.path.join(_DOCUMENTS, "Excel Files"),
            "enabled": True,
        },
        {
            "id": _generate_id(),
            "name": "Images",
            "watch_folder": _DOWNLOADS,
            "extensions": ".jpg, .png, .gif",
            "destination": os.path.join(_PICTURES, "Downloaded Images"),
            "enabled": True,
        },
        {
            "id": _generate_id(),
            "name": "Documents",
            "watch_folder": _DOWNLOADS,
            "extensions": ".pdf, .docx, .txt",
            "destination": os.path.join(_DOCUMENTS, "Downloaded Docs"),
            "enabled": True,
        },
    ]


def _default_settings() -> dict:
    """Return the default application settings / Mengembalikan pengaturan default aplikasi."""
    return {
        "auto_start": False,
        "minimize_to_tray_on_close": True,
    }


# ====================================================================
# Public API
# ====================================================================

def load_config(path: str = DEFAULT_CONFIG_PATH) -> dict:
    """
    Load config from *path*. If the file does not exist, create it
    with default rules and settings.

    Memuat konfigurasi dari *path*. Jika file tidak ada, buat file
    dengan aturan dan pengaturan default.
    """
    if os.path.exists(path):
        try:
            with open(path, "r", encoding="utf-8") as f:
                data = json.load(f)
            logger.info("Config loaded from %s", path)
            # Ensure required keys exist (backward compatibility)
            if "rules" not in data:
                data["rules"] = _default_rules()
            if "settings" not in data:
                data["settings"] = _default_settings()
            return data
        except (json.JSONDecodeError, IOError) as exc:
            logger.warning("Failed to read config (%s), regenerating defaults.", exc)

    # First run or corrupted file – create defaults
    data = {
        "rules": _default_rules(),
        "settings": _default_settings(),
    }
    save_config(data, path)
    logger.info("Default config created at %s", path)
    return data


def save_config(data: dict, path: str = DEFAULT_CONFIG_PATH) -> None:
    """
    Persist *data* to *path* as formatted JSON.
    Menyimpan *data* ke *path* sebagai JSON yang diformat.
    """
    try:
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=4, ensure_ascii=False)
        logger.info("Config saved to %s", path)
    except IOError as exc:
        logger.error("Failed to save config: %s", exc)


def add_rule(data: dict, rule: dict) -> dict:
    """
    Add a new rule to the config data.
    Menambahkan aturan baru ke data konfigurasi.
    """
    if "id" not in rule or not rule["id"]:
        rule["id"] = _generate_id()
    data["rules"].append(rule)
    return data


def update_rule(data: dict, rule_id: str, updated: dict) -> dict:
    """
    Update an existing rule identified by *rule_id*.
    Memperbarui aturan yang ada berdasarkan *rule_id*.
    """
    for i, r in enumerate(data["rules"]):
        if r["id"] == rule_id:
            updated["id"] = rule_id  # preserve ID
            data["rules"][i] = updated
            break
    return data


def delete_rule(data: dict, rule_id: str) -> dict:
    """
    Delete a rule by its ID.
    Menghapus aturan berdasarkan ID-nya.
    """
    data["rules"] = [r for r in data["rules"] if r["id"] != rule_id]
    return data
