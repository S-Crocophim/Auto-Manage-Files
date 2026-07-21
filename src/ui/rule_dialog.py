"""
rule_dialog.py
--------------
Modal dialog (CTkToplevel) for adding or editing a file-organization rule.
Supports Dark / Light themes via parent palette.

Dialog modal (CTkToplevel) untuk menambahkan atau mengedit aturan organisasi file.
Mendukung tema Gelap / Terang melalui palet parent.
"""

import os
import uuid
import tkinter as tk
from tkinter import filedialog
import customtkinter as ctk
import logging

logger = logging.getLogger(__name__)


from src.ui.theme import _PALETTE

class RuleDialog(ctk.CTkToplevel):
    """
    A modal dialog to create or edit a rule.
    Dialog modal untuk membuat atau mengedit aturan.

    After the dialog is closed, check ``self.result`` for the rule dict
    (or None if cancelled).
    """

    def __init__(self, parent, rule: dict | None = None):
        super().__init__(parent)
        self.t = parent._translator
        # Grab the palette from the imported module
        self.P = _PALETTE

        self.result: dict | None = None
        self._editing = rule is not None
        self._rule_id = rule["id"] if rule else uuid.uuid4().hex[:8]

        P = self.P

        # ---- window config ----
        self.title(self.t.get("rule_title_edit") if self._editing else self.t.get("rule_title_add"))
        self.configure(fg_color=P["bg_main"])
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
        P = self.P
        t = self.t
        pad = {"padx": 24, "pady": (8, 0)}

        # ---------- Title ----------
        title_text = t.get("rule_title_edit") if self._editing else t.get("rule_title_add")
        ctk.CTkLabel(
            self, text=title_text, font=ctk.CTkFont(size=20, weight="bold"),
            text_color=P["fg_primary"], anchor="w",
        ).pack(fill="x", padx=24, pady=(24, 16))

        # ---------- Separator ----------
        sep = ctk.CTkFrame(self, height=1, fg_color=P["border"])
        sep.pack(fill="x", padx=24, pady=(0, 12))

        # ---------- Name ----------
        ctk.CTkLabel(
            self, text=t.get("rule_name"), font=ctk.CTkFont(size=13),
            text_color=P["fg_label"], anchor="w",
        ).pack(fill="x", **pad)
        self._name_entry = ctk.CTkEntry(
            self, height=38, corner_radius=8,
            fg_color=P["bg_input"], border_color=P["border"], text_color=P["fg_primary"],
            placeholder_text=t.get("rule_name_ph"),
            placeholder_text_color=P["fg_secondary"],
        )
        self._name_entry.pack(fill="x", padx=24, pady=(4, 0))

        # ---------- Watch Folder ----------
        ctk.CTkLabel(
            self, text=t.get("rule_watch"), font=ctk.CTkFont(size=13),
            text_color=P["fg_label"], anchor="w",
        ).pack(fill="x", **pad)
        wf_frame = ctk.CTkFrame(self, fg_color="transparent")
        wf_frame.pack(fill="x", padx=24, pady=(4, 0))
        self._watch_entry = ctk.CTkEntry(
            wf_frame, height=38, corner_radius=8,
            fg_color=P["bg_input"], border_color=P["border"], text_color=P["fg_primary"],
            placeholder_text_color=P["fg_secondary"],
        )
        self._watch_entry.pack(side="left", fill="x", expand=True, padx=(0, 8))
        ctk.CTkButton(
            wf_frame, text="📂", width=42, height=38, corner_radius=8,
            fg_color=P["accent"], hover_color=P["hover"], text_color=P["fg_primary"],
            command=lambda: self._browse(self._watch_entry, append=True),
        ).pack(side="right")

        # ---------- Extensions ----------
        ctk.CTkLabel(
            self, text=t.get("rule_ext"), font=ctk.CTkFont(size=13),
            text_color=P["fg_label"], anchor="w",
        ).pack(fill="x", **pad)
        self._ext_entry = ctk.CTkEntry(
            self, height=38, corner_radius=8,
            fg_color=P["bg_input"], border_color=P["border"], text_color=P["fg_primary"],
            placeholder_text=t.get("rule_ext_ph"),
            placeholder_text_color=P["fg_secondary"],
        )
        self._ext_entry.pack(fill="x", padx=24, pady=(4, 0))

        # ---------- Destination Folder ----------
        ctk.CTkLabel(
            self, text=t.get("rule_dest"), font=ctk.CTkFont(size=13),
            text_color=P["fg_label"], anchor="w",
        ).pack(fill="x", **pad)
        df_frame = ctk.CTkFrame(self, fg_color="transparent")
        df_frame.pack(fill="x", padx=24, pady=(4, 0))
        self._dest_entry = ctk.CTkEntry(
            df_frame, height=38, corner_radius=8,
            fg_color=P["bg_input"], border_color=P["border"], text_color=P["fg_primary"],
            placeholder_text_color=P["fg_secondary"],
        )
        self._dest_entry.pack(side="left", fill="x", expand=True, padx=(0, 8))
        ctk.CTkButton(
            df_frame, text="📂", width=42, height=38, corner_radius=8,
            fg_color=P["accent"], hover_color=P["hover"], text_color=P["fg_primary"],
            command=lambda: self._browse(self._dest_entry, append=False),
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
            btn_frame, text=t.get("rule_btn_cancel"), width=100, height=40, corner_radius=10,
            fg_color=P["bg_input"], hover_color=P["hover"],
            text_color=P["fg_primary"], font=ctk.CTkFont(size=14),
            command=self._on_cancel,
        ).pack(side="left")

        ctk.CTkButton(
            btn_frame, text=t.get("rule_btn_save"), width=140, height=40, corner_radius=10,
            fg_color=P["btn_primary_bg"], hover_color=P["btn_primary_hover"],
            text_color=P["btn_primary_fg"], font=ctk.CTkFont(size=14, weight="bold"),
            command=self._on_save,
        ).pack(side="right")

    # ------------------------------------------------------------------
    # Helpers
    # ------------------------------------------------------------------

    @staticmethod
    def _browse(entry_widget: ctk.CTkEntry, append: bool = False) -> None:
        """Open a folder browser dialog / Buka dialog penjelajah folder."""
        folder = filedialog.askdirectory()
        if folder:
            if append:
                current = entry_widget.get().strip()
                if current:
                    folder = current + ", " + folder
            entry_widget.delete(0, "end")
            entry_widget.insert(0, folder)

    def _on_save(self) -> None:
        name = self._name_entry.get().strip()
        watch = self._watch_entry.get().strip()
        exts = self._ext_entry.get().strip()
        dest = self._dest_entry.get().strip()

        # Basic validation / Validasi dasar
        if not name or not watch or not exts or not dest:
            # Flash a simple warning
            self.title(self.t.get("rule_warn_empty"))
            self.after(2000, lambda: self.title(
                self.t.get("rule_title_edit") if self._editing else self.t.get("rule_title_add")
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
