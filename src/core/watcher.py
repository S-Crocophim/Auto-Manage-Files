"""
watcher.py
----------
Manages a pool of watchdog Observers that monitor configured folders
in real-time and trigger file moves on ``on_created`` / ``on_moved`` events.

Mengelola kumpulan watchdog Observer yang memantau folder yang dikonfigurasi
secara real-time dan memicu pemindahan file pada event ``on_created`` / ``on_moved``.
"""

import os
import time
import logging
import threading
from typing import Optional

from watchdog.observers import Observer
from watchdog.events import FileSystemEventHandler, FileSystemEvent

from src.core.organizer import move_file_safely

logger = logging.getLogger(__name__)


# ------------------------------------------------------------------
# Event handler – one instance per rule
# ------------------------------------------------------------------

class _RuleHandler(FileSystemEventHandler):
    """
    Handles filesystem events for a single rule.
    Menangani event filesystem untuk satu aturan.
    """

    def __init__(self, rule: dict):
        super().__init__()
        self.rule = rule
        self.extensions: set[str] = {
            ext.strip().lower()
            for ext in rule.get("extensions", "").split(",")
            if ext.strip()
        }
        self.dest_folder: str = rule.get("destination", "")
        # Small delay to let files finish writing
        # Jeda kecil agar file selesai ditulis
        self._settle_seconds = 1.5

    # ---- helpers ----

    def _should_handle(self, path: str) -> bool:
        """Check if the file extension matches this rule / Cek apakah ekstensi file cocok."""
        if not os.path.isfile(path):
            return False
        _, ext = os.path.splitext(path)
        return ext.lower() in self.extensions

    def _handle(self, path: str) -> None:
        """Wait for the file to settle, then move it / Tunggu file selesai, lalu pindahkan."""
        if not self._should_handle(path):
            return

        # Wait a moment for the file to be fully written / Tunggu sebentar agar file sepenuhnya ditulis
        time.sleep(self._settle_seconds)

        # Double-check the file still exists (it may have been moved already)
        if not os.path.isfile(path):
            return

        logger.info("[%s] Detected: %s", self.rule.get("name", "?"), path)
        move_file_safely(path, self.dest_folder)

    # ---- watchdog callbacks ----

    def on_created(self, event: FileSystemEvent) -> None:
        if event.is_directory:
            return
        threading.Thread(target=self._handle, args=(event.src_path,), daemon=True).start()

    def on_moved(self, event: FileSystemEvent) -> None:
        if event.is_directory:
            return
        threading.Thread(target=self._handle, args=(event.dest_path,), daemon=True).start()


# ------------------------------------------------------------------
# FolderWatcher – manages the Observer pool
# ------------------------------------------------------------------

class FolderWatcher:
    """
    Creates / destroys watchdog Observers based on the current set of
    enabled rules.

    Membuat / menghancurkan watchdog Observer berdasarkan kumpulan aturan
    yang aktif saat ini.
    """

    def __init__(self):
        self._observer: Optional[Observer] = None
        self._lock = threading.Lock()
        self._running = False

    # ---- public API ----

    def start(self, rules: list[dict]) -> None:
        """
        Start monitoring all watch folders from *rules* that are enabled.
        Mulai memantau semua folder dari *rules* yang aktif.
        """
        with self._lock:
            self.stop()  # stop previous observer if any

            enabled_rules = [r for r in rules if r.get("enabled", False)]
            if not enabled_rules:
                logger.info("No enabled rules – watcher not started.")
                return

            self._observer = Observer()

            # Group rules by watch_folder to avoid duplicate watches
            # Kelompokkan aturan berdasarkan watch_folder untuk menghindari duplikasi
            folder_handlers: dict[str, list[_RuleHandler]] = {}
            for rule in enabled_rules:
                folder = rule.get("watch_folder", "")
                if not folder or not os.path.isdir(folder):
                    logger.warning("Skipping invalid watch folder: %s", folder)
                    continue
                folder_handlers.setdefault(folder, []).append(_RuleHandler(rule))

            for folder, handlers in folder_handlers.items():
                for handler in handlers:
                    self._observer.schedule(handler, folder, recursive=False)
                logger.info("Watching: %s (%d rule(s))", folder, len(handlers))

            self._observer.daemon = True
            self._observer.start()
            self._running = True
            logger.info("FolderWatcher started with %d rule(s).", len(enabled_rules))

    def stop(self) -> None:
        """
        Stop all observers.
        Menghentikan semua observer.
        """
        if self._observer and self._running:
            self._observer.stop()
            self._observer.join(timeout=5)
            self._observer = None
            self._running = False
            logger.info("FolderWatcher stopped.")

    def restart(self, rules: list[dict]) -> None:
        """
        Restart watchers with a fresh set of rules (e.g. after config change).
        Restart watcher dengan set aturan baru (misal setelah perubahan konfigurasi).
        """
        self.start(rules)

    @property
    def is_running(self) -> bool:
        return self._running
