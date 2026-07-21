"""
main.py
-------
Application entry point for Auto File Organizer.
Coordinates config loading, the watchdog watcher, the system tray,
and the CustomTkinter main window.

Titik masuk aplikasi untuk Auto File Organizer.
Mengkoordinasikan pemuatan konfigurasi, watchdog watcher, system tray,
dan jendela utama CustomTkinter.
"""

import sys
import os
import logging
import threading

# ------------------------------------------------------------------
# Ensure the project root is on sys.path so that `src.*` imports work
# regardless of how the script is invoked.
# ------------------------------------------------------------------
_PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if _PROJECT_ROOT not in sys.path:
    sys.path.insert(0, _PROJECT_ROOT)

from src.core.config_manager import load_config, save_config
from src.core.watcher import FolderWatcher
from src.core.organizer import run_manual_organization_threaded
from src.ui.main_window import MainWindow
from src.ui.system_tray import SystemTray

# ------------------------------------------------------------------
# Logging setup / Pengaturan logging
# ------------------------------------------------------------------
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
    datefmt="%H:%M:%S",
)
logger = logging.getLogger("AutoFileOrganizer")


class Application:
    """
    Top-level coordinator that owns all components.
    Koordinator tingkat atas yang memiliki semua komponen.
    """

    def __init__(self):
        logger.info("Initializing Auto File Organizer…")

        # ---- Load config ----
        self.config = load_config()

        # ---- Core: folder watcher ----
        self.watcher = FolderWatcher()

        # ---- UI: system tray (runs in daemon thread) ----
        self.tray = SystemTray(
            on_open=self._show_window,
            on_manual=self._run_manual,
            on_quit=self._quit,
        )

        # ---- UI: main window (runs on main thread) ----
        self.window = MainWindow(
            config_data=self.config,
            on_rules_changed=self._on_rules_changed,
            on_manual_trigger=lambda: None,  # manual is handled via button internally
            on_quit=self._quit,
        )

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    def run(self) -> None:
        """Start all components and enter the Tk main loop / Mulai semua komponen dan masuk ke loop utama Tk."""
        # Start file watcher
        self.watcher.start(self.config.get("rules", []))

        # Start system tray
        self.tray.start()

        logger.info("Application started. Entering main loop.")

        # Block on the Tk event loop (must be on the main thread)
        self.window.mainloop()

    # ------------------------------------------------------------------
    # Callbacks
    # ------------------------------------------------------------------

    def _on_rules_changed(self, rules: list[dict]) -> None:
        """Restart the watcher when rules are modified / Restart watcher saat aturan diubah."""
        logger.info("Rules changed – restarting watcher.")
        self.watcher.restart(rules)

    def _show_window(self) -> None:
        """Bring the main window to the foreground (thread-safe) / Tampilkan jendela utama."""
        self.window.show_window()

    def _run_manual(self) -> None:
        """Trigger manual organization from the tray menu / Picu organisasi manual dari menu tray."""
        rules = self.window.get_rules()
        run_manual_organization_threaded(rules)

    def _quit(self) -> None:
        """Cleanly shut down all components / Matikan semua komponen dengan bersih."""
        logger.info("Shutting down…")
        self.watcher.stop()
        self.tray.stop()
        try:
            self.window.destroy()
        except Exception:
            pass
        logger.info("Goodbye!")
        os._exit(0)  # ensure all daemon threads terminate


# ==================================================================
# Entry point
# ==================================================================

def main():
    app = Application()
    app.run()


if __name__ == "__main__":
    main()
