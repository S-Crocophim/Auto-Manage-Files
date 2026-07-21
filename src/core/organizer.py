"""
organizer.py
------------
Core file-moving logic used by both Automatic and Manual modes.
Handles collision avoidance (auto-incrementing filenames) and
sends Windows toast notifications via winotify.

Logika inti pemindahan file yang digunakan oleh mode Otomatis dan Manual.
Menangani penghindaran tabrakan (penomoran otomatis nama file) dan
mengirim notifikasi toast Windows melalui winotify.
"""

import os
import sys
import shutil
import time
import logging
import threading

logger = logging.getLogger(__name__)

# ------------------------------------------------------------------
# Notification helper  (winotify)
# ------------------------------------------------------------------
try:
    # pyrefly: ignore [missing-import]
    from winotify import Notification, audio
    from src.ui.i18n import Translator
    from src.core import config_manager
    _WINOTIFY_AVAILABLE = True
except ImportError:
    _WINOTIFY_AVAILABLE = False
    logger.warning("winotify not available – notifications disabled.")


def _send_notification(title_key: str, msg_key: str, msg_arg: str = "") -> None:
    """
    Send a Windows 10/11 toast notification.
    Mengirim notifikasi toast Windows 10/11.
    """
    if not _WINOTIFY_AVAILABLE or sys.platform != "win32":
        return

    # Load current language from config
    try:
        conf = config_manager.load_config()
        lang = conf.get("settings", {}).get("language", "id")
    except Exception:
        lang = "id"
    t = Translator(lang)

    title = t.get(title_key) if title_key else ""
    if msg_key:
        msg = f"{t.get(msg_key)} {msg_arg}".strip() if msg_arg else t.get(msg_key)
    else:
        msg = msg_arg

    try:
        toast = Notification(
            app_id="Auto File Organizer",
            title=title,
            msg=msg,
            duration="short",
        )
        toast.set_audio(audio.Default, loop=False)
        toast.show()
    except Exception as exc:
        logger.error("Notification failed: %s", exc)


# ------------------------------------------------------------------
# Safe file move with collision handling
# Pemindahan file aman dengan penanganan tabrakan
# ------------------------------------------------------------------

def _safe_dest_path(dest_folder: str, filename: str) -> str:
    """
    Return a destination path that does not collide with an existing file.
    If 'report.xlsx' already exists, returns 'report (1).xlsx', etc.

    Mengembalikan jalur tujuan yang tidak bertabrakan dengan file yang ada.
    Jika 'report.xlsx' sudah ada, mengembalikan 'report (1).xlsx', dst.
    """
    base, ext = os.path.splitext(filename)
    candidate = os.path.join(dest_folder, filename)
    counter = 1
    while os.path.exists(candidate):
        candidate = os.path.join(dest_folder, f"{base} ({counter}){ext}")
        counter += 1
    return candidate


def move_file_safely(
    src_path: str,
    dest_folder: str,
    notify: bool = True,
    max_retries: int = 5,
    retry_delay: float = 1.0,
) -> bool:
    """
    Move a single file to *dest_folder* with collision handling and retry
    logic for locked files.

    Memindahkan satu file ke *dest_folder* dengan penanganan tabrakan dan
    logika retry untuk file yang terkunci.

    Returns True on success, False on failure.
    """
    if not os.path.isfile(src_path):
        logger.debug("Source does not exist or is not a file: %s", src_path)
        return False

    filename = os.path.basename(src_path)

    # Create destination folder if it doesn't exist
    # Buat folder tujuan jika belum ada
    os.makedirs(dest_folder, exist_ok=True)

    dest_path = _safe_dest_path(dest_folder, filename)

    # Retry loop for locked files / Loop retry untuk file yang terkunci
    for attempt in range(1, max_retries + 1):
        try:
            shutil.move(src_path, dest_path)
            final_name = os.path.basename(dest_path)
            logger.info("Moved: %s → %s", filename, dest_path)

            if notify:
                _send_notification("notif_success_title", "", f"📁 {filename}\nKe: {dest_folder}")
            return True

        except PermissionError:
            # File might still be locked by another process
            # File mungkin masih terkunci oleh proses lain
            if attempt < max_retries:
                logger.warning(
                    "File locked, retrying (%d/%d): %s",
                    attempt, max_retries, src_path,
                )
                time.sleep(retry_delay)
            else:
                logger.error("Failed to move (locked after %d retries): %s", max_retries, src_path)
                if notify:
                    _send_notification("notif_err_lock_title", "notif_err_lock_msg")
                return False

        except Exception as exc:
            logger.error("Unexpected error moving %s: %s", src_path, exc)
            if notify:
                _send_notification("notif_err_title", "notif_err_msg", str(exc))
            return False

    return False


# ------------------------------------------------------------------
# Manual organisation  (on-demand scan & move)
# Organisasi manual (scan & pindahkan sesuai permintaan)
# ------------------------------------------------------------------

def run_manual_organization(rules: list[dict], notify: bool = True) -> int:
    """
    Iterate over all enabled rules, scan the watch folders, and move
    matching files to their destinations.

    Iterasi semua aturan yang aktif, scan folder yang dipantau, dan
    pindahkan file yang cocok ke tujuannya.

    Returns the total number of files moved.
    Mengembalikan jumlah total file yang dipindahkan.
    """
    total_moved = 0

    for rule in rules:
        if not rule.get("enabled", False):
            continue

        watch_folder = rule.get("watch_folder", "")
        dest_folder = rule.get("destination", "")
        extensions_raw = rule.get("extensions", "")

        if not watch_folder or not dest_folder:
            continue

        # Parse comma-separated extensions into a set
        # Parsing ekstensi yang dipisah koma menjadi set
        extensions = {
            ext.strip().lower()
            for ext in extensions_raw.split(",")
            if ext.strip()
        }

        watch_folders = [w.strip() for w in watch_folder.split(",") if w.strip()]

        for wf in watch_folders:
            if not os.path.isdir(wf):
                logger.warning("Watch folder does not exist: %s", wf)
                continue

            try:
                entries = os.listdir(wf)
            except OSError as exc:
                logger.error("Cannot list %s: %s", wf, exc)
                continue

            for entry in entries:
                full_path = os.path.join(wf, entry)
                if not os.path.isfile(full_path):
                    continue

                _, ext = os.path.splitext(entry)
                if ext.lower() in extensions:
                    if move_file_safely(full_path, dest_folder, notify=notify):
                        total_moved += 1

    logger.info("Manual organization complete – %d file(s) moved.", total_moved)

    if total_moved == 0 and notify:
        _send_notification("notif_done_title", "notif_done_msg")

    return total_moved


def run_manual_organization_threaded(rules: list[dict], callback=None) -> None:
    """
    Run manual organization in a background thread so the UI stays
    responsive.  Calls *callback(total_moved)* on completion.

    Menjalankan organisasi manual di thread latar belakang agar UI tetap
    responsif. Memanggil *callback(total_moved)* saat selesai.
    """
    def _worker():
        total = run_manual_organization(rules)
        if callback:
            callback(total)

    t = threading.Thread(target=_worker, daemon=True)
    t.start()
