"""
main_window.py
--------------
The primary CustomTkinter dashboard for Auto File Organizer.
Supports Native Smooth Theme Transitions and Dynamic Language Switching.

Dashboard utama CustomTkinter untuk Auto File Organizer.
Mendukung Transisi Tema Halus Bawaan dan Pergantian Bahasa Dinamis.
"""

import logging
import webbrowser
import json
from typing import Callable, Optional

import customtkinter as ctk
from tkinter import filedialog

from src.core import config_manager
from src.core.startup_manager import is_auto_start_enabled, set_auto_start
from src.core.organizer import run_manual_organization_threaded
from src.ui.rule_dialog import RuleDialog
from src.ui.i18n import Translator
from src.ui.theme import _PALETTE

logger = logging.getLogger(__name__)

class WidgetLoggerHandler(logging.Handler):
    """Custom logging handler to route logs to a CTkTextbox."""
    def __init__(self, textbox):
        super().__init__()
        self.textbox = textbox
        self.setFormatter(logging.Formatter('%(asctime)s - %(message)s', '%H:%M:%S'))
        
    def emit(self, record):
        try:
            msg = self.format(record)
            def append():
                try:
                    self.textbox.configure(state="normal")
                    self.textbox.insert("end", msg + "\n")
                    self.textbox.see("end")
                    self.textbox.configure(state="disabled")
                except Exception:
                    pass
            # Schedule safely on main thread
            self.textbox.after(0, append)
        except Exception:
            self.handleError(record)

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

        # ---- Dynamic texts & Animation tasks ----
        self._text_widgets: list[tuple[any, str]] = []
        self._pending_afters: list[str] = []

        # ---- Translator ----
        settings = self._config.get("settings", {})
        self._translator = Translator(settings.get("language", "id"))

        # ---- window setup ----
        self.title("Auto File Organizer")
        self.geometry("960x640")
        self.minsize(800, 520)

        # Override X (close) button
        self.protocol("WM_DELETE_WINDOW", self._on_close)

        # Override minimize (—) button di title bar → masuk ke tray jika setting aktif
        self.bind("<Unmap>", self._on_minimize_event)

        # ---- appearance ----
        theme = settings.get("theme", "dark")
        ctk.set_appearance_mode(theme)
        ctk.set_default_color_theme("dark-blue")
        self.configure(fg_color=_PALETTE["bg_main"])

        # ---- build the UI once ----
        self._build_sidebar()
        self._build_main_area()
        self._refresh_rule_cards()


    # ==================================================================
    # DYNAMIC LANGUAGE
    # ==================================================================

    def _register_text(self, widget, key: str):
        """Daftarkan widget untuk di-update saat bahasa berubah."""
        self._text_widgets.append((widget, key))

    def _update_language_texts(self):
        """Update semua teks terdaftar tanpa merusak UI."""
        for widget, key in self._text_widgets:
            # Cegah error jika widget sudah di-destroy (misal rule cards)
            if widget.winfo_exists():
                widget.configure(text=self._translator.get(key))

        # Update dropdown values
        t = self._translator
        self._theme_dropdown.configure(values=[t.get("theme_dark"), t.get("theme_light")])
        self._lang_dropdown.configure(values=["Indonesia", "English"])

    # ==================================================================
    # SIDEBAR
    # ==================================================================

    def _build_sidebar(self) -> None:
        P = _PALETTE
        t = self._translator

        sidebar = ctk.CTkFrame(self, width=260, corner_radius=0, fg_color=P["bg_sidebar"])
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
            text_color=P["fg_primary"], justify="left",
        ).pack(side="left")

        desc_lbl = ctk.CTkLabel(
            sidebar, text=t.get("app_desc"),
            font=ctk.CTkFont(size=12), text_color=P["fg_dim"], anchor="w",
        )
        desc_lbl.pack(fill="x", padx=24, pady=(0, 20))
        self._register_text(desc_lbl, "app_desc")

        # ---------- Separator ----------
        ctk.CTkFrame(sidebar, height=1, fg_color=P["border"]).pack(fill="x", padx=16, pady=(0, 16))

        # ---------- Navigation buttons ----------
        self._nav_btn(sidebar, "nav_add_rule", self._add_rule)

        # ---------- Manual Organize ----------
        self._manual_btn = ctk.CTkButton(
            sidebar, text=t.get("nav_manual"), height=42, corner_radius=10,
            fg_color=P["btn_primary_bg"], hover_color=P["btn_primary_hover"],
            text_color=P["btn_primary_fg"], font=ctk.CTkFont(size=14, weight="bold"),
            command=self._trigger_manual,
        )
        self._manual_btn.pack(fill="x", padx=20, pady=(8, 4))
        self._register_text(self._manual_btn, "nav_manual")

        # ---------- Bottom Section Wrapper ----------
        # Pack bottom_frame FIRST with side="bottom" so it's immune to being pushed off-screen
        bottom_frame = ctk.CTkFrame(sidebar, fg_color="transparent")
        bottom_frame.pack(side="bottom", fill="x")

        # ---------- Spacer ----------
        # Pack spacer AFTER bottom_frame so it only takes the *remaining* space
        ctk.CTkFrame(sidebar, fg_color="transparent").pack(fill="both", expand=True)

        # ---------- Settings: Theme & Lang ----------
        ctk.CTkFrame(bottom_frame, height=1, fg_color=P["border"]).pack(fill="x", padx=16, pady=(0, 8))

        st_frame = ctk.CTkFrame(bottom_frame, fg_color="transparent")
        st_frame.pack(fill="x", padx=20, pady=(4, 4))
        
        lbl_theme = ctk.CTkLabel(st_frame, text=t.get("theme_label"), font=ctk.CTkFont(size=12), text_color=P["fg_label"])
        lbl_theme.pack(side="left")
        self._register_text(lbl_theme, "theme_label")
        
        current_theme = self._config.get("settings", {}).get("theme", "dark")
        theme_display = t.get("theme_light") if current_theme == "light" else t.get("theme_dark")
        self._theme_var = ctk.StringVar(value=theme_display)
        self._theme_dropdown = ctk.CTkOptionMenu(
            st_frame, variable=self._theme_var,
            values=[t.get("theme_dark"), t.get("theme_light")],
            width=80, height=24, command=self._on_theme_change,
            fg_color=P["bg_input"], button_color=P["switch_btn"], button_hover_color=P["hover"],
            text_color=P["fg_primary"],
        )
        self._theme_dropdown.pack(side="right")

        sl_frame = ctk.CTkFrame(bottom_frame, fg_color="transparent")
        sl_frame.pack(fill="x", padx=20, pady=(0, 4))
        
        lbl_lang = ctk.CTkLabel(sl_frame, text=t.get("lang_label"), font=ctk.CTkFont(size=12), text_color=P["fg_label"])
        lbl_lang.pack(side="left")
        self._register_text(lbl_lang, "lang_label")
        
        current_lang = self._config.get("settings", {}).get("language", "id")
        lang_display = "English" if current_lang == "en" else "Indonesia"
        self._lang_var = ctk.StringVar(value=lang_display)
        self._lang_dropdown = ctk.CTkOptionMenu(
            sl_frame, variable=self._lang_var, values=["Indonesia", "English"],
            width=80, height=24, command=self._on_lang_change,
            fg_color=P["bg_input"], button_color=P["switch_btn"], button_hover_color=P["hover"],
            text_color=P["fg_primary"],
        )
        self._lang_dropdown.pack(side="right")

        # ---------- Auto-Start Toggle ----------
        auto_frame = ctk.CTkFrame(bottom_frame, fg_color="transparent")
        auto_frame.pack(fill="x", padx=20, pady=(4, 4))
        
        lbl_auto = ctk.CTkLabel(
            auto_frame, text=t.get("nav_auto_start"), font=ctk.CTkFont(size=13),
            text_color=P["fg_label"],
        )
        lbl_auto.pack(side="left")
        self._register_text(lbl_auto, "nav_auto_start")
        
        self._auto_start_var = ctk.BooleanVar(value=is_auto_start_enabled())
        self._auto_start_switch = ctk.CTkSwitch(
            auto_frame, text="", variable=self._auto_start_var,
            command=self._toggle_auto_start,
            onvalue=True, offvalue=False,
            fg_color=P["switch_off"], progress_color=P["switch_on"],
            button_color=P["switch_btn"], button_hover_color=P["hover"],
            width=44, height=22,
        )
        self._auto_start_switch.pack(side="right")

        # ---------- Minimize to tray toggle ----------
        tray_frame = ctk.CTkFrame(bottom_frame, fg_color="transparent")
        tray_frame.pack(fill="x", padx=20, pady=(0, 4))
        
        lbl_tray = ctk.CTkLabel(
            tray_frame, text=t.get("nav_minimize_tray"), font=ctk.CTkFont(size=13),
            text_color=P["fg_label"],
        )
        lbl_tray.pack(side="left")
        self._register_text(lbl_tray, "nav_minimize_tray")
        
        self._tray_var = ctk.BooleanVar(
            value=self._config.get("settings", {}).get("minimize_to_tray_on_close", True)
        )
        ctk.CTkSwitch(
            tray_frame, text="", variable=self._tray_var,
            command=self._toggle_tray_setting,
            onvalue=True, offvalue=False,
            fg_color=P["switch_off"], progress_color=P["switch_on"],
            button_color=P["switch_btn"], button_hover_color=P["hover"],
            width=44, height=22,
        ).pack(side="right")

        # ---------- Export / Import Settings ----------
        imp_exp_frame = ctk.CTkFrame(bottom_frame, fg_color="transparent")
        imp_exp_frame.pack(fill="x", padx=20, pady=(4, 4))
        
        btn_export = ctk.CTkButton(
            imp_exp_frame, text=t.get("nav_export_settings"), height=28, corner_radius=8,
            fg_color=P["bg_input"], hover_color=P["hover"],
            text_color=P["fg_primary"], font=ctk.CTkFont(size=11),
            command=self._export_settings,
        )
        btn_export.pack(side="left", fill="x", expand=True, padx=(0, 4))
        self._register_text(btn_export, "nav_export_settings")

        btn_import = ctk.CTkButton(
            imp_exp_frame, text=t.get("nav_import_settings"), height=28, corner_radius=8,
            fg_color=P["bg_input"], hover_color=P["hover"],
            text_color=P["fg_primary"], font=ctk.CTkFont(size=11),
            command=self._import_settings,
        )
        btn_import.pack(side="right", fill="x", expand=True, padx=(4, 0))
        self._register_text(btn_import, "nav_import_settings")

        # ---------- Watermark ----------
        watermark_lbl = ctk.CTkLabel(
            bottom_frame, text=t.get("watermark_credit"),
            font=ctk.CTkFont(size=11, slant="italic"), text_color=P["fg_dim"]
        )
        watermark_lbl.pack(fill="x", padx=20, pady=(12, 4))
        self._register_text(watermark_lbl, "watermark_credit")

        # ---------- Quit button ----------
        quit_btn = ctk.CTkButton(
            bottom_frame, text=t.get("nav_quit"), height=38, corner_radius=10,
            fg_color=P["delete_red"], hover_color=P["delete_red_hover"],
            text_color="#ffffff", font=ctk.CTkFont(size=13),
            command=self._on_close,
        )
        quit_btn.pack(fill="x", padx=20, pady=(4, 20))
        self._register_text(quit_btn, "nav_quit")

    def _nav_btn(self, parent, key: str, command) -> None:
        P = _PALETTE
        btn = ctk.CTkButton(
            parent, text=self._translator.get(key), height=40, corner_radius=10, anchor="w",
            fg_color="transparent", hover_color=P["hover"],
            text_color=P["fg_primary"], font=ctk.CTkFont(size=14),
            command=command,
        )
        btn.pack(fill="x", padx=20, pady=(2, 2))
        self._register_text(btn, key)

    # ==================================================================
    # MAIN CONTENT AREA
    # ==================================================================

    def _build_main_area(self) -> None:
        P = _PALETTE
        
        main_right = ctk.CTkFrame(self, fg_color="transparent", corner_radius=0)
        main_right.pack(side="right", fill="both", expand=True)

        # ---------- TOP BAR ----------
        self._top_bar = ctk.CTkFrame(main_right, height=50, corner_radius=0, fg_color=P["bg_main"])
        self._top_bar.pack(side="top", fill="x")
        
        tab_container = ctk.CTkFrame(self._top_bar, fg_color="transparent")
        tab_container.pack(side="left", padx=10, pady=8)

        def make_tab(key, cmd):
            btn = ctk.CTkButton(
                tab_container, text=self._translator.get(key), height=32, corner_radius=8,
                fg_color="transparent", hover_color=P["hover"],
                text_color=P["fg_primary"], font=ctk.CTkFont(size=13, weight="bold"),
                command=cmd, width=0,
            )
            btn.pack(side="left", padx=4)
            self._register_text(btn, key)

        make_tab("nav_dashboard", self._show_dashboard)
        make_tab("nav_logs", self._show_logs)
        make_tab("nav_tutorial", self._show_tutorial)
        make_tab("nav_about", self._show_about)

        # Container for all views
        self._container = ctk.CTkFrame(main_right, fg_color=P["bg_main"], corner_radius=0)
        self._container.pack(side="top", fill="both", expand=True)
        
        # Dashboard View
        self._dashboard_frame = ctk.CTkFrame(self._container, fg_color="transparent")
        self._dashboard_frame.place(relx=0, rely=0, relwidth=1, relheight=1)
        self._build_dashboard_view()
        
        # Logs View
        self._logs_frame = ctk.CTkFrame(self._container, fg_color="transparent")
        self._logs_frame.place(relx=0, rely=0, relwidth=1, relheight=1)
        self._build_logs_view()
        
        # Tutorial View
        self._tutorial_frame = ctk.CTkFrame(self._container, fg_color="transparent")
        self._tutorial_frame.place(relx=0, rely=0, relwidth=1, relheight=1)
        self._build_tutorial_view()

        # About View
        self._about_frame = ctk.CTkFrame(self._container, fg_color="transparent")
        self._about_frame.place(relx=0, rely=0, relwidth=1, relheight=1)
        self._build_about_view()
        
        # Default view
        self._dashboard_frame.lift()

    def _build_dashboard_view(self) -> None:
        P = _PALETTE
        t = self._translator

        # Header
        header = ctk.CTkFrame(self._dashboard_frame, fg_color="transparent")
        header.pack(fill="x", padx=32, pady=(28, 4))
        
        lbl_head = ctk.CTkLabel(
            header, text=t.get("dashboard_title"), font=ctk.CTkFont(size=26, weight="bold"),
            text_color=P["fg_primary"],
        )
        lbl_head.pack(side="left")
        self._register_text(lbl_head, "dashboard_title")

        # Status badge
        self._status_label = ctk.CTkLabel(
            header, text=t.get("status_active"), font=ctk.CTkFont(size=12),
            text_color="#4caf50",
        )
        self._status_label.pack(side="right", padx=(0, 4))
        self._register_text(self._status_label, "status_active")

        lbl_active = ctk.CTkLabel(
            self._dashboard_frame, text=t.get("active_rules_label"),
            font=ctk.CTkFont(size=13), text_color=P["fg_dim"], anchor="w",
        )
        lbl_active.pack(fill="x", padx=32, pady=(0, 12))
        self._register_text(lbl_active, "active_rules_label")

        ctk.CTkFrame(self._dashboard_frame, height=1, fg_color=P["border"]).pack(fill="x", padx=28, pady=(0, 8))

        # Scrollable rule list
        self._scroll = ctk.CTkScrollableFrame(
            self._dashboard_frame, fg_color="transparent",
            scrollbar_button_color=P["accent"], scrollbar_button_hover_color=P["hover"],
        )
        self._scroll.pack(fill="both", expand=True, padx=24, pady=(4, 16))

    def _build_logs_view(self) -> None:
        P = _PALETTE
        t = self._translator

        # Header
        header = ctk.CTkFrame(self._logs_frame, fg_color="transparent")
        header.pack(fill="x", padx=32, pady=(28, 4))
        
        lbl_head = ctk.CTkLabel(
            header, text=t.get("logs_title"), font=ctk.CTkFont(size=26, weight="bold"),
            text_color=P["fg_primary"],
        )
        lbl_head.pack(side="left")
        self._register_text(lbl_head, "logs_title")

        # Clear button
        btn_clear = ctk.CTkButton(
            header, text=t.get("btn_clear_logs"), width=100, height=32, corner_radius=8,
            fg_color=P["btn_dark_bg"], hover_color=P["btn_dark_hover"], text_color=P["fg_primary"],
            font=ctk.CTkFont(size=12), command=self._clear_logs
        )
        btn_clear.pack(side="right")
        self._register_text(btn_clear, "btn_clear_logs")

        ctk.CTkFrame(self._logs_frame, height=1, fg_color=P["border"]).pack(fill="x", padx=28, pady=(12, 16))

        # Textbox for logs
        self._logs_textbox = ctk.CTkTextbox(
            self._logs_frame, fg_color=P["bg_input"], text_color=P["fg_primary"],
            font=ctk.CTkFont(family="Consolas", size=12), state="disabled", wrap="word"
        )
        self._logs_textbox.pack(fill="both", expand=True, padx=32, pady=(0, 24))

        # Attach custom logger handler
        self._log_handler = WidgetLoggerHandler(self._logs_textbox)
        
        # We attach to the root logger to catch everything
        logging.getLogger().addHandler(self._log_handler)

    def _clear_logs(self):
        self._logs_textbox.configure(state="normal")
        self._logs_textbox.delete("1.0", "end")
        self._logs_textbox.configure(state="disabled")

    # ==================================================================
    # TUTORIAL & ABOUT
    # ==================================================================

    def _build_tutorial_view(self) -> None:
        P = _PALETTE
        t = self._translator

        header = ctk.CTkFrame(self._tutorial_frame, fg_color="transparent")
        header.pack(fill="x", padx=32, pady=(24, 0))

        lbl_head = ctk.CTkLabel(
            header, text=t.get("nav_tutorial"), font=ctk.CTkFont(size=26, weight="bold"),
            text_color=P["fg_primary"]
        )
        lbl_head.pack(side="left")
        self._register_text(lbl_head, "nav_tutorial")

        ctk.CTkFrame(self._tutorial_frame, height=1, fg_color=P["border"]).pack(fill="x", padx=28, pady=(8, 16))

        # Content
        lbl_content = ctk.CTkLabel(
            self._tutorial_frame, text=t.get("tutorial_content"),
            font=ctk.CTkFont(size=14), text_color=P["fg_primary"], justify="left", anchor="nw"
        )
        lbl_content.pack(fill="both", expand=True, padx=32, pady=10)
        self._register_text(lbl_content, "tutorial_content")

    def _build_about_view(self) -> None:
        P = _PALETTE
        t = self._translator

        header = ctk.CTkFrame(self._about_frame, fg_color="transparent")
        header.pack(fill="x", padx=32, pady=(24, 0))

        lbl_head = ctk.CTkLabel(
            header, text=t.get("nav_about"), font=ctk.CTkFont(size=26, weight="bold"),
            text_color=P["fg_primary"]
        )
        lbl_head.pack(side="left")
        self._register_text(lbl_head, "nav_about")

        ctk.CTkFrame(self._about_frame, height=1, fg_color=P["border"]).pack(fill="x", padx=28, pady=(8, 16))

        # Content Wrapper
        content_frame = ctk.CTkFrame(self._about_frame, fg_color="transparent")
        content_frame.pack(fill="both", expand=True, padx=32, pady=10)

        lbl_content = ctk.CTkLabel(
            content_frame, text=t.get("about_content"),
            font=ctk.CTkFont(size=14), text_color=P["fg_primary"], justify="left", anchor="nw"
        )
        lbl_content.pack(anchor="nw")
        self._register_text(lbl_content, "about_content")

        lbl_link = ctk.CTkLabel(
            content_frame, text=t.get("about_link_text"),
            font=ctk.CTkFont(size=14, underline=True), text_color="#1f538d", justify="left", cursor="hand2"
        )
        lbl_link.pack(anchor="nw")
        self._register_text(lbl_link, "about_link_text")
        
        lbl_link.bind("<Button-1>", lambda e: webbrowser.open("https://github.com/S-Crocophim/Auto-Manage-Files"))
        lbl_link.bind("<Enter>", lambda e: lbl_link.configure(text_color="#14375e"))
        lbl_link.bind("<Leave>", lambda e: lbl_link.configure(text_color="#1f538d"))

    # ==================================================================
    # RULE CARDS (CASCADING LOAD)
    # ==================================================================

    def _refresh_rule_cards(self) -> None:
        """Bangun ulang daftar aturan dengan efek Cascading Load."""
        
        # Batalkan semua animasi cascading yang masih menunggu
        for after_id in self._pending_afters:
            try:
                self.after_cancel(after_id)
            except Exception:
                pass
        self._pending_afters.clear()

        # Hancurkan widget lama dengan aman
        for widget in list(self._scroll.winfo_children()):
            try:
                widget.pack_forget()
                for child in widget.winfo_children():
                    child.destroy()
                widget.destroy()
            except Exception:
                pass

        # Bersihkan referensi widget mati dari _text_widgets
        self._text_widgets = [w for w in self._text_widgets if w[0].winfo_exists()]

        rules = self._config.get("rules", [])

        if not rules:
            lbl = ctk.CTkLabel(
                self._scroll,
                text=self._translator.get("no_rules_msg"),
                font=ctk.CTkFont(size=14), text_color=_PALETTE["fg_dim"],
            )
            lbl.pack(pady=40)
            self._register_text(lbl, "no_rules_msg")
            return

        # Efek Cascading Load (muncul bertahap)
        for i, rule in enumerate(rules):
            after_id = self.after(50 * i, self._create_rule_card, rule)
            self._pending_afters.append(after_id)

    def _create_rule_card(self, rule: dict) -> None:
        P = _PALETTE
        
        # Cek jika id tidak ada lagi karena di-delete terlalu cepat
        if not any(r["id"] == rule["id"] for r in self._config.get("rules", [])):
            return

        card = ctk.CTkFrame(
            self._scroll, fg_color=P["bg_card"], corner_radius=12, height=100,
            border_width=1, border_color=P["border"],
        )
        card.pack(fill="x", pady=(0, 10), ipady=6)
        card.pack_propagate(False)

        inner = ctk.CTkFrame(card, fg_color="transparent")
        inner.pack(fill="both", expand=True, padx=16, pady=10)

        # ---- Top row: name + switch ----
        top = ctk.CTkFrame(inner, fg_color="transparent")
        top.pack(fill="x")

        title = rule.get("name", self._translator.get("unnamed_rule"))
        lbl_name = ctk.CTkLabel(
            top, text=title,
            font=ctk.CTkFont(size=16, weight="bold"), text_color=P["fg_primary"],
        )
        lbl_name.pack(side="left")
        if title == self._translator.get("unnamed_rule"):
            self._register_text(lbl_name, "unnamed_rule")

        # Enable/disable switch
        enabled_var = ctk.BooleanVar(value=rule.get("enabled", True))
        ctk.CTkSwitch(
            top, text="", variable=enabled_var,
            command=lambda rid=rule["id"], var=enabled_var: self._toggle_rule(rid, var.get()),
            fg_color=P["switch_off"], progress_color=P["switch_on"],
            button_color=P["switch_btn"], button_hover_color=P["hover"],
            width=44, height=22,
        ).pack(side="right", padx=(8, 0))

        # Edit & Delete buttons
        ctk.CTkButton(
            top, text="🗑️", width=34, height=30, corner_radius=8,
            fg_color=P["delete_red"], hover_color=P["delete_red_hover"], text_color="#ffffff",
            font=ctk.CTkFont(size=13),
            command=lambda rid=rule["id"]: self._delete_rule(rid),
        ).pack(side="right", padx=(4, 0))

        ctk.CTkButton(
            top, text="✏️", width=34, height=30, corner_radius=8,
            fg_color=P["btn_dark_bg"], hover_color=P["btn_dark_hover"], text_color=P["fg_primary"],
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
            font=ctk.CTkFont(size=12), text_color=P["fg_dim"], anchor="w",
        ).pack(side="left", fill="x")

    @staticmethod
    def _shorten(path: str, max_len: int = 35) -> str:
        """Shorten a long path for display."""
        if len(path) <= max_len:
            return path
        return "…" + path[-(max_len - 1):]

    # ==================================================================
    # ACTIONS
    # ==================================================================

    def _show_dashboard(self) -> None:
        self._dashboard_frame.lift()

    def _show_logs(self) -> None:
        self._logs_frame.lift()

    def _show_tutorial(self) -> None:
        self._tutorial_frame.lift()

    def _show_about(self) -> None:
        self._about_frame.lift()

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

    def _export_settings(self) -> None:
        path = filedialog.asksaveasfilename(
            defaultextension=".json",
            filetypes=[("JSON Files", "*.json")],
            initialfile="AutoFileOrganizer_Settings.json",
            title="Export Settings"
        )
        if path:
            try:
                with open(path, "w", encoding="utf-8") as f:
                    json.dump(self._config, f, indent=4, ensure_ascii=False)
                logger.info(f"Settings exported to {path}")
            except Exception as e:
                logger.error(f"Failed to export settings: {e}")

    def _import_settings(self) -> None:
        path = filedialog.askopenfilename(
            filetypes=[("JSON Files", "*.json")],
            title="Import Settings"
        )
        if path:
            try:
                with open(path, "r", encoding="utf-8") as f:
                    new_config = json.load(f)
                
                # Update config
                self._config = new_config
                config_manager.save_config(self._config)
                
                # Apply new settings immediately
                self._on_rules_changed(self._config.get("rules", []))
                
                theme = self._config.get("settings", {}).get("theme", "dark")
                ctk.set_appearance_mode(theme)
                theme_display = self._translator.get("theme_light") if theme == "light" else self._translator.get("theme_dark")
                self._theme_var.set(theme_display)

                lang = self._config.get("settings", {}).get("language", "id")
                self._translator.set_language(lang)
                self._update_language_texts()
                lang_display = "English" if lang == "en" else "Indonesia"
                self._lang_var.set(lang_display)

                self._auto_start_var.set(is_auto_start_enabled())
                self._tray_var.set(self._config.get("settings", {}).get("minimize_to_tray_on_close", True))
                
                logger.info(f"Settings imported from {path}")
            except Exception as e:
                logger.error(f"Failed to import settings: {e}")

    def _on_theme_change(self, display_value: str) -> None:
        t = self._translator
        value = "light" if display_value == t.get("theme_light") else "dark"
        self._config.setdefault("settings", {})["theme"] = value
        config_manager.save_config(self._config)

        # Hanya panggil ini, CustomTkinter akan otomatis me-render efek fade yang smooth!
        ctk.set_appearance_mode(value)
        
        # Update dropdown var display silently
        new_display = t.get("theme_light") if value == "light" else t.get("theme_dark")
        self._theme_var.set(new_display)

    def _on_lang_change(self, display_value: str) -> None:
        value = "en" if display_value == "English" else "id"
        self._translator.set_language(value)
        self._config.setdefault("settings", {})["language"] = value
        config_manager.save_config(self._config)

        # Update text tanpa destroy UI
        self._update_language_texts()

    def _trigger_manual(self) -> None:
        """Jalankan organisasi manual dengan animasi berkedip pada tombol."""
        self._manual_btn.configure(state="disabled")
        
        # Animasi proses (Teks berubah-ubah)
        self._is_processing = True
        self._anim_dots = 0
        
        def animate_button():
            if not self._is_processing:
                return
            base_txt = self._translator.get("nav_manual_processing")
            self._manual_btn.configure(text=base_txt + ("." * (self._anim_dots % 4)))
            self._anim_dots += 1
            self.after(400, animate_button)
            
        animate_button()

        def _done(total: int):
            self._is_processing = False
            # Schedule UI update on the main thread
            self.after(0, lambda: self._manual_btn.configure(
                text=self._translator.get("nav_manual"), state="normal"
            ))

        run_manual_organization_threaded(self._config.get("rules", []), callback=_done)
        self._on_manual_trigger()

    def _on_minimize_event(self, event) -> None:
        """
        Dipanggil saat tombol minimize (—) di title bar diklik.
        Jika 'Minimize to Tray' aktif, sembunyikan ke tray alih-alih ke taskbar.
        """
        # Hanya tangani event dari jendela utama (bukan child widget)
        if event.widget != self:
            return
        if self._tray_var.get():
            # Batalkan minimize bawaan, ganti dengan hide ke tray
            self.after(10, self.withdraw)

    def _on_close(self) -> None:
        if self._tray_var.get():
            self.withdraw()
        else:
            self._on_quit()

    def _on_quit(self) -> None:
        self._on_quit_callback()


    # ==================================================================
    # Public helpers
    # ==================================================================

    def show_window(self) -> None:
        self.after(0, self._do_show)

    def _do_show(self) -> None:
        self.deiconify()
        self.lift()
        self.focus_force()

    def get_rules(self) -> list[dict]:
        return self._config.get("rules", [])
