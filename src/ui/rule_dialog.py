"""
rule_dialog.py
--------------
Modal dialog (CTkToplevel) for adding or editing a file-organization rule.
Uses the monochrome Black & White design language.

Dialog modal (CTkToplevel) untuk menambahkan atau mengedit aturan organisasi file.
Menggunakan bahasa desain monokrom Hitam & Putih.
"""

import os
import uuid
import tkinter as tk
from tkinter import filedialog
import customtkinter as ctk
import logging

logger = logging.getLogger(__name__)

# ------------------------------------------------------------------
# Colour palette (monochrome)
# ------------------------------------------------------------------
_BG_DARK = "#111111"
_BG_CARD = "#1a1a1a"
_BG_INPUT = "#2b2b2b"
_FG_WHITE = "#ffffff"
_FG_GREY = "#aaaaaa"
_FG_LABEL = "#cccccc"
_ACCENT = "#333333"
_HOVER = "#3a3a3a"
_BORDER = "#444444"
_BTN_PRIMARY_BG = "#ffffff"
_BTN_PRIMARY_FG = "#111111"
_BTN_PRIMARY_HOVER = "#dddddd"
_BTN_CANCEL_BG = "#2b2b2b"
_BTN_CANCEL_FG = "#ffffff"
_BTN_CANCEL_HOVER = "#3a3a3a"
_DELETE_RED = "#c0392b"
_DELETE_RED_HOVER = "#a93226"


class RuleDialog(ctk.CTkToplevel):
    """
    A modal dialog to create or edit a rule.
    Dialog modal untuk membuat atau mengedit aturan.

    After the dialog is closed, check ``self.result`` for the rule dict
    (or None if cancelled).
    """

    def __init__(self, parent, rule: dict | None = None):
        super().__init__(parent)

        self.result: dict | None = None
        self._editing = rule is not None
        self._rule_id = rule["id"] if rule else uuid.uuid4().hex[:8]

        # ---- window config ----
        self.title("Edit Aturan" if self._editing else "Tambah Aturan Baru")
        self.configure(fg_color=_BG_DARK)
        self.geometry("520x520")
        self.resizable(False, False)
        self.transient(parent)
        self.grab_set()

        # Center on parent
        self.update_idletasks()
        px = parent.winfo_x() + (parent.winfo_width() // 2) - 260
        py = parent.winfo_y() + (parent.winfo_height() // 2) - 260
        self.geometry(f"+{px}+{py}")

        # ---- build form ----
        self._build_ui(rule)

        # Prevent interaction with parent until this is closed
        self.protocol("WM_DELETE_WINDOW", self._on_cancel)
        self.wait_window()

    # ------------------------------------------------------------------
    # UI construction
    # ------------------------------------------------------------------

    def _build_ui(self, rule: dict | None) -> None:
        pad = {"padx": 24, "pady": (8, 0)}

        # ---------- Title ----------
        title_text = "✏️  Edit Aturan" if self._editing else "➕  Aturan Baru"
        ctk.CTkLabel(
            self, text=title_text, font=ctk.CTkFont(size=20, weight="bold"),
            text_color=_FG_WHITE, anchor="w",
        ).pack(fill="x", padx=24, pady=(24, 16))

        # ---------- Separator ----------
        sep = ctk.CTkFrame(self, height=1, fg_color=_BORDER)
        sep.pack(fill="x", padx=24, pady=(0, 12))

        # ---------- Name ----------
        ctk.CTkLabel(
            self, text="Nama Aturan", font=ctk.CTkFont(size=13),
            text_color=_FG_LABEL, anchor="w",
        ).pack(fill="x", **pad)
        self._name_entry = ctk.CTkEntry(
            self, height=38, corner_radius=8,
            fg_color=_BG_INPUT, border_color=_BORDER, text_color=_FG_WHITE,
            placeholder_text="Contoh: Spreadsheets",
            placeholder_text_color=_FG_GREY,
        )
        self._name_entry.pack(fill="x", padx=24, pady=(4, 0))

        # ---------- Watch Folder ----------
        ctk.CTkLabel(
            self, text="Folder Dipantau", font=ctk.CTkFont(size=13),
            text_color=_FG_LABEL, anchor="w",
        ).pack(fill="x", **pad)
        wf_frame = ctk.CTkFrame(self, fg_color="transparent")
        wf_frame.pack(fill="x", padx=24, pady=(4, 0))
        self._watch_entry = ctk.CTkEntry(
            wf_frame, height=38, corner_radius=8,
            fg_color=_BG_INPUT, border_color=_BORDER, text_color=_FG_WHITE,
            placeholder_text_color=_FG_GREY,
        )
        self._watch_entry.pack(side="left", fill="x", expand=True, padx=(0, 8))
        ctk.CTkButton(
            wf_frame, text="📂", width=42, height=38, corner_radius=8,
            fg_color=_ACCENT, hover_color=_HOVER, text_color=_FG_WHITE,
            command=lambda: self._browse(self._watch_entry),
        ).pack(side="right")

        # ---------- Extensions ----------
        ctk.CTkLabel(
            self, text="Ekstensi (pisahkan koma)", font=ctk.CTkFont(size=13),
            text_color=_FG_LABEL, anchor="w",
        ).pack(fill="x", **pad)
        self._ext_entry = ctk.CTkEntry(
            self, height=38, corner_radius=8,
            fg_color=_BG_INPUT, border_color=_BORDER, text_color=_FG_WHITE,
            placeholder_text=".xlsx, .csv, .pdf",
            placeholder_text_color=_FG_GREY,
        )
        self._ext_entry.pack(fill="x", padx=24, pady=(4, 0))

        # ---------- Destination Folder ----------
        ctk.CTkLabel(
            self, text="Folder Tujuan", font=ctk.CTkFont(size=13),
            text_color=_FG_LABEL, anchor="w",
        ).pack(fill="x", **pad)
        df_frame = ctk.CTkFrame(self, fg_color="transparent")
        df_frame.pack(fill="x", padx=24, pady=(4, 0))
        self._dest_entry = ctk.CTkEntry(
            df_frame, height=38, corner_radius=8,
            fg_color=_BG_INPUT, border_color=_BORDER, text_color=_FG_WHITE,
            placeholder_text_color=_FG_GREY,
        )
        self._dest_entry.pack(side="left", fill="x", expand=True, padx=(0, 8))
        ctk.CTkButton(
            df_frame, text="📂", width=42, height=38, corner_radius=8,
            fg_color=_ACCENT, hover_color=_HOVER, text_color=_FG_WHITE,
            command=lambda: self._browse(self._dest_entry),
        ).pack(side="right")

        # ---------- Pre-fill if editing ----------
        if rule:
            self._name_entry.insert(0, rule.get("name", ""))
            self._watch_entry.insert(0, rule.get("watch_folder", ""))
            self._ext_entry.insert(0, rule.get("extensions", ""))
            self._dest_entry.insert(0, rule.get("destination", ""))

        # ---------- Buttons ----------
        btn_frame = ctk.CTkFrame(self, fg_color="transparent")
        btn_frame.pack(fill="x", padx=24, pady=(24, 24))

        ctk.CTkButton(
            btn_frame, text="Batal", width=100, height=40, corner_radius=10,
            fg_color=_BTN_CANCEL_BG, hover_color=_BTN_CANCEL_HOVER,
            text_color=_BTN_CANCEL_FG, font=ctk.CTkFont(size=14),
            command=self._on_cancel,
        ).pack(side="left")

        ctk.CTkButton(
            btn_frame, text="💾  Simpan", width=140, height=40, corner_radius=10,
            fg_color=_BTN_PRIMARY_BG, hover_color=_BTN_PRIMARY_HOVER,
            text_color=_BTN_PRIMARY_FG, font=ctk.CTkFont(size=14, weight="bold"),
            command=self._on_save,
        ).pack(side="right")

    # ------------------------------------------------------------------
    # Helpers
    # ------------------------------------------------------------------

    @staticmethod
    def _browse(entry_widget: ctk.CTkEntry) -> None:
        """Open a folder browser dialog / Buka dialog penjelajah folder."""
        folder = filedialog.askdirectory()
        if folder:
            entry_widget.delete(0, "end")
            entry_widget.insert(0, folder)

    def _on_save(self) -> None:
        name = self._name_entry.get().strip()
        watch = self._watch_entry.get().strip()
        exts = self._ext_entry.get().strip()
        dest = self._dest_entry.get().strip()

        # Basic validation / Validasi dasar
        if not name or not watch or not exts or not dest:
            # Flash a simple warning – keep it lightweight
            self.title("⚠️  Semua field harus diisi!")
            self.after(2000, lambda: self.title(
                "Edit Aturan" if self._editing else "Tambah Aturan Baru"
            ))
            return

        self.result = {
            "id": self._rule_id,
            "name": name,
            "watch_folder": watch,
            "extensions": exts,
            "destination": dest,
            "enabled": True,
        }
        self.grab_release()
        self.destroy()

    def _on_cancel(self) -> None:
        self.result = None
        self.grab_release()
        self.destroy()
