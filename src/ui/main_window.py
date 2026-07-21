"""
main_window.py
--------------
The primary CustomTkinter dashboard for Auto File Organizer.
Strict monochrome Black & White aesthetic with premium layout.

Dashboard utama CustomTkinter untuk Auto File Organizer.
Estetika monokrom Hitam & Putih yang ketat dengan tata letak premium.
"""

import logging
import threading
from typing import Callable, Optional

import customtkinter as ctk

from src.core import config_manager
from src.core.startup_manager import is_auto_start_enabled, set_auto_start
from src.core.organizer import run_manual_organization_threaded
from src.ui.rule_dialog import RuleDialog

logger = logging.getLogger(__name__)

# ------------------------------------------------------------------
# Colour palette  (monochrome)
# ------------------------------------------------------------------
BG_DARK = "#111111"
BG_SIDEBAR = "#0d0d0d"
BG_CARD = "#1a1a1a"
BG_CARD_HOVER = "#222222"
BG_INPUT = "#2b2b2b"
FG_WHITE = "#ffffff"
FG_GREY = "#aaaaaa"
FG_DIM = "#666666"
FG_LABEL = "#cccccc"
ACCENT = "#333333"
HOVER = "#3a3a3a"
BORDER = "#2e2e2e"
BTN_DARK_BG = "#1e1e1e"
BTN_DARK_FG = "#ffffff"
BTN_DARK_HOVER = "#2e2e2e"
BTN_PRIMARY_BG = "#ffffff"
BTN_PRIMARY_FG = "#111111"
BTN_PRIMARY_HOVER = "#dddddd"
DELETE_RED = "#c0392b"
DELETE_RED_HOVER = "#a93226"
SWITCH_ON = "#ffffff"
SWITCH_OFF = "#555555"
SWITCH_BTN = "#111111"


class MainWindow(ctk.CTk):
    """
    The main application window.
    Jendela aplikasi utama.
    """

    def __init__(
        self,
        config_data: dict,
        on_rules_changed: Callable[[list[dict]], None],
        on_manual_trigger: Callable[[], None],
        on_quit: Callable[[], None],
    ):
        super().__init__()

        self._config = config_data
        self._on_rules_changed = on_rules_changed
        self._on_manual_trigger = on_manual_trigger
        self._on_quit_callback = on_quit

        # ---- window setup ----
        self.title("Auto File Organizer")
        self.configure(fg_color=BG_DARK)
        self.geometry("960x640")
        self.minsize(800, 520)

        # Override close button to minimize to tray
        self.protocol("WM_DELETE_WINDOW", self._on_close)

        # ---- appearance ----
        ctk.set_appearance_mode("dark")
        ctk.set_default_color_theme("dark-blue")  # overridden below

        # ---- layout ----
        self._build_sidebar()
        self._build_main_area()

        # ---- initial rule list ----
        self._refresh_rule_cards()

    # ==================================================================
    # SIDEBAR
    # ==================================================================

    def _build_sidebar(self) -> None:
        sidebar = ctk.CTkFrame(self, width=260, corner_radius=0, fg_color=BG_SIDEBAR)
        sidebar.pack(side="left", fill="y")
        sidebar.pack_propagate(False)

        # ---------- Logo / Brand ----------
        brand_frame = ctk.CTkFrame(sidebar, fg_color="transparent")
        brand_frame.pack(fill="x", padx=20, pady=(28, 4))
        ctk.CTkLabel(
            brand_frame, text="📁", font=ctk.CTkFont(size=28),
        ).pack(side="left", padx=(0, 8))
        ctk.CTkLabel(
            brand_frame, text="Auto File\nOrganizer",
            font=ctk.CTkFont(size=18, weight="bold"),
            text_color=FG_WHITE, justify="left",
        ).pack(side="left")

        ctk.CTkLabel(
            sidebar, text="Kelola file otomatis",
            font=ctk.CTkFont(size=12), text_color=FG_DIM, anchor="w",
        ).pack(fill="x", padx=24, pady=(0, 20))

        # ---------- Separator ----------
        ctk.CTkFrame(sidebar, height=1, fg_color=BORDER).pack(fill="x", padx=16, pady=(0, 16))

        # ---------- Navigation buttons ----------
        self._nav_btn(sidebar, "🏠  Dashboard", self._show_dashboard)
        self._nav_btn(sidebar, "➕  Tambah Aturan", self._add_rule)

        # ---------- Manual Organize ----------
        self._manual_btn = ctk.CTkButton(
            sidebar, text="🗂️  Rapihkan Sekarang", height=42, corner_radius=10,
            fg_color=BTN_PRIMARY_BG, hover_color=BTN_PRIMARY_HOVER,
            text_color=BTN_PRIMARY_FG, font=ctk.CTkFont(size=14, weight="bold"),
            command=self._trigger_manual,
        )
        self._manual_btn.pack(fill="x", padx=20, pady=(8, 4))

        # ---------- Spacer ----------
        ctk.CTkFrame(sidebar, fg_color="transparent").pack(fill="both", expand=True)

        # ---------- Auto-Start Toggle ----------
        ctk.CTkFrame(sidebar, height=1, fg_color=BORDER).pack(fill="x", padx=16, pady=(0, 8))

        auto_frame = ctk.CTkFrame(sidebar, fg_color="transparent")
        auto_frame.pack(fill="x", padx=20, pady=(4, 4))
        ctk.CTkLabel(
            auto_frame, text="🚀  Auto-Start", font=ctk.CTkFont(size=13),
            text_color=FG_LABEL,
        ).pack(side="left")
        self._auto_start_var = ctk.BooleanVar(value=is_auto_start_enabled())
        self._auto_start_switch = ctk.CTkSwitch(
            auto_frame, text="", variable=self._auto_start_var,
            command=self._toggle_auto_start,
            onvalue=True, offvalue=False,
            fg_color=SWITCH_OFF, progress_color=SWITCH_ON,
            button_color=SWITCH_BTN, button_hover_color="#222222",
            width=44, height=22,
        )
        self._auto_start_switch.pack(side="right")

        # ---------- Minimize to tray toggle ----------
        tray_frame = ctk.CTkFrame(sidebar, fg_color="transparent")
        tray_frame.pack(fill="x", padx=20, pady=(0, 4))
        ctk.CTkLabel(
            tray_frame, text="🔽  Minimize to Tray", font=ctk.CTkFont(size=13),
            text_color=FG_LABEL,
        ).pack(side="left")
        self._tray_var = ctk.BooleanVar(
            value=self._config.get("settings", {}).get("minimize_to_tray_on_close", True)
        )
        ctk.CTkSwitch(
            tray_frame, text="", variable=self._tray_var,
            command=self._toggle_tray_setting,
            onvalue=True, offvalue=False,
            fg_color=SWITCH_OFF, progress_color=SWITCH_ON,
            button_color=SWITCH_BTN, button_hover_color="#222222",
            width=44, height=22,
        ).pack(side="right")

        # ---------- Quit button ----------
        ctk.CTkButton(
            sidebar, text="❌  Keluar", height=38, corner_radius=10,
            fg_color=DELETE_RED, hover_color=DELETE_RED_HOVER,
            text_color="#ffffff", font=ctk.CTkFont(size=13),
            command=self._on_quit,
        ).pack(fill="x", padx=20, pady=(8, 20))

    def _nav_btn(self, parent, text: str, command) -> None:
        ctk.CTkButton(
            parent, text=text, height=40, corner_radius=10, anchor="w",
            fg_color="transparent", hover_color=HOVER,
            text_color=FG_WHITE, font=ctk.CTkFont(size=14),
            command=command,
        ).pack(fill="x", padx=20, pady=(2, 2))

    # ==================================================================
    # MAIN CONTENT AREA
    # ==================================================================

    def _build_main_area(self) -> None:
        self._main_frame = ctk.CTkFrame(self, fg_color=BG_DARK, corner_radius=0)
        self._main_frame.pack(side="right", fill="both", expand=True)

        # Header
        header = ctk.CTkFrame(self._main_frame, fg_color="transparent")
        header.pack(fill="x", padx=32, pady=(28, 4))
        ctk.CTkLabel(
            header, text="Dashboard", font=ctk.CTkFont(size=26, weight="bold"),
            text_color=FG_WHITE,
        ).pack(side="left")

        # Status badge
        self._status_label = ctk.CTkLabel(
            header, text="● Monitoring Aktif", font=ctk.CTkFont(size=12),
            text_color="#4caf50",
        )
        self._status_label.pack(side="right", padx=(0, 4))

        ctk.CTkLabel(
            self._main_frame, text="Aturan organisasi file yang aktif",
            font=ctk.CTkFont(size=13), text_color=FG_DIM, anchor="w",
        ).pack(fill="x", padx=32, pady=(0, 12))

        ctk.CTkFrame(self._main_frame, height=1, fg_color=BORDER).pack(fill="x", padx=28, pady=(0, 8))

        # Scrollable rule list
        self._scroll = ctk.CTkScrollableFrame(
            self._main_frame, fg_color="transparent",
            scrollbar_button_color=ACCENT, scrollbar_button_hover_color=HOVER,
        )
        self._scroll.pack(fill="both", expand=True, padx=24, pady=(4, 16))

    # ==================================================================
    # RULE CARDS
    # ==================================================================

    def _refresh_rule_cards(self) -> None:
        """Rebuild the scrollable rule list / Bangun ulang daftar aturan yang dapat digulir."""
        for widget in self._scroll.winfo_children():
            widget.destroy()

        rules = self._config.get("rules", [])

        if not rules:
            ctk.CTkLabel(
                self._scroll,
                text="Belum ada aturan. Klik ➕ Tambah Aturan untuk memulai.",
                font=ctk.CTkFont(size=14), text_color=FG_DIM,
            ).pack(pady=40)
            return

        for rule in rules:
            self._create_rule_card(rule)

    def _create_rule_card(self, rule: dict) -> None:
        card = ctk.CTkFrame(
            self._scroll, fg_color=BG_CARD, corner_radius=12, height=100,
            border_width=1, border_color=BORDER,
        )
        card.pack(fill="x", pady=(0, 10), ipady=6)
        card.pack_propagate(False)

        inner = ctk.CTkFrame(card, fg_color="transparent")
        inner.pack(fill="both", expand=True, padx=16, pady=10)

        # ---- Top row: name + switch ----
        top = ctk.CTkFrame(inner, fg_color="transparent")
        top.pack(fill="x")

        ctk.CTkLabel(
            top, text=rule.get("name", "Unnamed"),
            font=ctk.CTkFont(size=16, weight="bold"), text_color=FG_WHITE,
        ).pack(side="left")

        # Enable/disable switch
        enabled_var = ctk.BooleanVar(value=rule.get("enabled", True))
        ctk.CTkSwitch(
            top, text="", variable=enabled_var,
            command=lambda rid=rule["id"], var=enabled_var: self._toggle_rule(rid, var.get()),
            fg_color=SWITCH_OFF, progress_color=SWITCH_ON,
            button_color=SWITCH_BTN, button_hover_color="#222222",
            width=44, height=22,
        ).pack(side="right", padx=(8, 0))

        # Edit & Delete buttons
        ctk.CTkButton(
            top, text="🗑️", width=34, height=30, corner_radius=8,
            fg_color=DELETE_RED, hover_color=DELETE_RED_HOVER, text_color="#ffffff",
            font=ctk.CTkFont(size=13),
            command=lambda rid=rule["id"]: self._delete_rule(rid),
        ).pack(side="right", padx=(4, 0))

        ctk.CTkButton(
            top, text="✏️", width=34, height=30, corner_radius=8,
            fg_color=BTN_DARK_BG, hover_color=BTN_DARK_HOVER, text_color=FG_WHITE,
            font=ctk.CTkFont(size=13),
            command=lambda r=rule: self._edit_rule(r),
        ).pack(side="right", padx=(4, 0))

        # ---- Bottom row: details ----
        bot = ctk.CTkFrame(inner, fg_color="transparent")
        bot.pack(fill="x", pady=(6, 0))

        exts = rule.get("extensions", "")
        watch = rule.get("watch_folder", "")
        dest = rule.get("destination", "")
        detail_text = f"{exts}   •   {self._shorten(watch)} → {self._shorten(dest)}"

        ctk.CTkLabel(
            bot, text=detail_text,
            font=ctk.CTkFont(size=12), text_color=FG_DIM, anchor="w",
        ).pack(side="left", fill="x")

    @staticmethod
    def _shorten(path: str, max_len: int = 35) -> str:
        """Shorten a long path for display / Persingkat jalur panjang untuk tampilan."""
        if len(path) <= max_len:
            return path
        return "…" + path[-(max_len - 1):]

    # ==================================================================
    # ACTIONS
    # ==================================================================

    def _show_dashboard(self) -> None:
        self._refresh_rule_cards()

    def _add_rule(self) -> None:
        dialog = RuleDialog(self)
        if dialog.result:
            self._config = config_manager.add_rule(self._config, dialog.result)
            config_manager.save_config(self._config)
            self._refresh_rule_cards()
            self._on_rules_changed(self._config["rules"])

    def _edit_rule(self, rule: dict) -> None:
        dialog = RuleDialog(self, rule=rule)
        if dialog.result:
            self._config = config_manager.update_rule(self._config, rule["id"], dialog.result)
            config_manager.save_config(self._config)
            self._refresh_rule_cards()
            self._on_rules_changed(self._config["rules"])

    def _delete_rule(self, rule_id: str) -> None:
        self._config = config_manager.delete_rule(self._config, rule_id)
        config_manager.save_config(self._config)
        self._refresh_rule_cards()
        self._on_rules_changed(self._config["rules"])

    def _toggle_rule(self, rule_id: str, enabled: bool) -> None:
        for rule in self._config.get("rules", []):
            if rule["id"] == rule_id:
                rule["enabled"] = enabled
                break
        config_manager.save_config(self._config)
        self._on_rules_changed(self._config["rules"])

    def _toggle_auto_start(self) -> None:
        enabled = self._auto_start_var.get()
        set_auto_start(enabled)
        self._config.setdefault("settings", {})["auto_start"] = enabled
        config_manager.save_config(self._config)

    def _toggle_tray_setting(self) -> None:
        self._config.setdefault("settings", {})["minimize_to_tray_on_close"] = self._tray_var.get()
        config_manager.save_config(self._config)

    def _trigger_manual(self) -> None:
        """Run manual organization / Jalankan organisasi manual."""
        self._manual_btn.configure(text="⏳  Memproses…", state="disabled")

        def _done(total: int):
            # Schedule UI update on the main thread
            self.after(0, lambda: self._manual_btn.configure(
                text="🗂️  Rapihkan Sekarang", state="normal"
            ))

        run_manual_organization_threaded(self._config.get("rules", []), callback=_done)
        self._on_manual_trigger()

    def _on_close(self) -> None:
        """Handle window close: hide to tray or quit / Tangani tutup jendela: sembunyikan ke tray atau keluar."""
        if self._tray_var.get():
            self.withdraw()
        else:
            self._on_quit()

    def _on_quit(self) -> None:
        self._on_quit_callback()

    # ==================================================================
    # Public helpers called from main.py
    # ==================================================================

    def show_window(self) -> None:
        """Bring the window back from tray / Kembalikan jendela dari tray."""
        self.after(0, self._do_show)

    def _do_show(self) -> None:
        self.deiconify()
        self.lift()
        self.focus_force()

    def get_rules(self) -> list[dict]:
        return self._config.get("rules", [])
