"""
system_tray.py
--------------
System-tray integration using pystray.
Runs in a daemon thread so it does not block the CustomTkinter main loop.

Integrasi system tray menggunakan pystray.
Berjalan di thread daemon agar tidak memblokir loop utama CustomTkinter.
"""

import threading
import logging
from typing import Callable, Optional

from PIL import Image, ImageDraw
import pystray
from pystray import MenuItem as item

logger = logging.getLogger(__name__)


def _create_tray_icon_image(size: int = 64) -> Image.Image:
    """
    Generate a simple monochrome folder icon programmatically.
    Membuat ikon folder monokrom sederhana secara programatis.
    """
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Draw a rounded rectangle as a folder body
    margin = size // 8
    body_top = size // 3
    draw.rounded_rectangle(
        [margin, body_top, size - margin, size - margin],
        radius=size // 10,
        fill="#ffffff",
    )
    # Tab on top-left
    tab_w = size // 2.5
    tab_h = size // 5
    draw.rounded_rectangle(
        [margin, body_top - tab_h + 2, margin + tab_w, body_top + 4],
        radius=size // 14,
        fill="#ffffff",
    )
    # Small inner line
    inner_y = body_top + (size - margin - body_top) // 2
    draw.line(
        [margin + 8, inner_y, size - margin - 8, inner_y],
        fill="#111111", width=2,
    )
    return img


class SystemTray:
    """
    Wraps pystray to provide system-tray functionality.
    Membungkus pystray untuk menyediakan fungsionalitas system tray.
    """

    def __init__(
        self,
        on_open: Callable[[], None],
        on_manual: Callable[[], None],
        on_quit: Callable[[], None],
    ):
        self._on_open = on_open
        self._on_manual = on_manual
        self._on_quit = on_quit
        self._icon: Optional[pystray.Icon] = None
        self._thread: Optional[threading.Thread] = None

    def start(self) -> None:
        """Start the tray icon in a daemon thread / Mulai ikon tray di thread daemon."""
        menu = pystray.Menu(
            item("⚙️  Buka Pengaturan", lambda icon, _: self._on_open()),
            item("🗂️  Rapihkan Sekarang (Manual)", lambda icon, _: self._on_manual()),
            pystray.Menu.SEPARATOR,
            item("❌  Keluar", lambda icon, _: self._on_quit()),
        )

        self._icon = pystray.Icon(
            name="AutoFileOrganizer",
            icon=_create_tray_icon_image(),
            title="Auto File Organizer",
            menu=menu,
        )

        self._thread = threading.Thread(target=self._icon.run, daemon=True)
        self._thread.start()
        logger.info("System tray started.")

    def stop(self) -> None:
        """Stop and remove the tray icon / Hentikan dan hapus ikon tray."""
        if self._icon:
            try:
                self._icon.stop()
            except Exception:
                pass
            self._icon = None
        logger.info("System tray stopped.")

    def update_tooltip(self, text: str) -> None:
        """Update the tray tooltip / Perbarui tooltip tray."""
        if self._icon:
            self._icon.title = text
