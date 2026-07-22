"""
i18n.py
-------
Provides simple localization (Indonesian / English) for the UI.
"""

_TRANSLATIONS = {
    # ------------------------------------------------------------------
    # Sidebar
    # ------------------------------------------------------------------
    "dashboard_title": {
        "id": "Dashboard",
        "en": "Dashboard"
    },
    "app_desc": {
        "id": "Kelola file otomatis",
        "en": "Auto manage files"
    },
    "nav_dashboard": {
        "id": "🏠  Dashboard",
        "en": "🏠  Dashboard"
    },
    "nav_logs": {
        "id": "📄  Logs",
        "en": "📄  Logs"
    },
    "nav_tutorial": {
        "id": "📚  Tutorial",
        "en": "📚  Tutorial"
    },
    "nav_about": {
        "id": "ℹ️  About",
        "en": "ℹ️  About"
    },
    "nav_add_rule": {
        "id": "➕  Tambah Aturan",
        "en": "➕  Add Rule"
    },
    "nav_manual": {
        "id": "🗂️  Rapihkan Sekarang",
        "en": "🗂️  Organize Now"
    },
    "nav_manual_processing": {
        "id": "⏳  Memproses…",
        "en": "⏳  Processing…"
    },
    "nav_auto_start": {
        "id": "🚀  Auto-Start",
        "en": "🚀  Auto-Start"
    },
    "nav_minimize_tray": {
        "id": "🔽  Minimize to Tray",
        "en": "🔽  Minimize to Tray"
    },
    "nav_minimize_now": {
        "id": "🔽  Minimize ke Tray",
        "en": "🔽  Minimize to Tray"
    },
    "nav_settings": {
        "id": "⚙️  Pengaturan",
        "en": "⚙️  Settings"
    },
    "nav_quit": {
        "id": "❌  Keluar",
        "en": "❌  Quit"
    },
    "watermark_credit": {
        "id": "Dibuat oleh Px0",
        "en": "Made by Px0"
    },
    "nav_export_settings": {
        "id": "📤  Export Settings",
        "en": "📤  Export Settings"
    },
    "nav_import_settings": {
        "id": "📥  Import Settings",
        "en": "📥  Import Settings"
    },

    # ------------------------------------------------------------------
    # Main Dashboard
    # ------------------------------------------------------------------
    "status_active": {
        "id": "● Monitoring Aktif",
        "en": "● Monitoring Active"
    },
    "status_inactive": {
        "id": "○ Monitoring Nonaktif",
        "en": "○ Monitoring Inactive"
    },
    "active_rules_label": {
        "id": "Aturan organisasi file yang aktif",
        "en": "Active file organization rules"
    },
    "no_rules_msg": {
        "id": "Belum ada aturan. Klik ➕ Tambah Aturan untuk memulai.",
        "en": "No rules yet. Click ➕ Add Rule to get started."
    },
    "unnamed_rule": {
        "id": "Tanpa Nama",
        "en": "Unnamed"
    },

    # ------------------------------------------------------------------
    # Logs Tab
    # ------------------------------------------------------------------
    "logs_title": {
        "id": "Catatan Aktivitas",
        "en": "Activity Logs"
    },
    "clear_logs": {
        "id": "🗑️  Bersihkan Log",
        "en": "🗑️  Clear Logs"
    },
    
    # ------------------------------------------------------------------
    # Tutorial & About Tabs
    # ------------------------------------------------------------------
    "tutorial_content": {
        "id": (
            "1. Tambahkan Aturan Baru melalui tombol 'Tambah Aturan'.\n"
            "2. Tentukan nama aturan, ekstensi file, folder pantauan, dan folder tujuan.\n"
            "3. Aktifkan mode Auto-Start agar aplikasi berjalan di background saat PC menyala.\n"
            "4. Gunakan fitur 'Minimize to Tray' agar aplikasi tidak mengganggu taskbar Anda.\n"
            "5. Semua file akan dipindahkan otomatis secara real-time!\n"
            "6. Gunakan 'Rapihkan Sekarang' jika ada file lama yang belum dipindahkan."
        ),
        "en": (
            "1. Add a New Rule via the 'Add Rule' button.\n"
            "2. Set the rule name, file extensions, watch folder, and target folder.\n"
            "3. Enable Auto-Start so the app runs in the background on PC startup.\n"
            "4. Use 'Minimize to Tray' so the app doesn't clutter your taskbar.\n"
            "5. All files will be moved automatically in real-time!\n"
            "6. Use 'Organize Now' if you have old files that need moving."
        )
    },
    "about_content": {
        "id": (
            "Auto File Organizer v1.0.0\n\n"
            "Aplikasi ringan untuk mengatur dan mengelola file-file Anda secara otomatis.\n"
            "Dibuat dengan cinta untuk mempermudah hidup Anda. ❤️\n\n"
            "🔗 GitHub: "
        ),
        "en": (
            "Auto File Organizer v1.0.0\n\n"
            "A lightweight application to organize and manage your files automatically.\n"
            "Made with love to make your life easier. ❤️\n\n"
            "🔗 GitHub: "
        )
    },
    "about_link_text": {
        "id": "https://github.com/S-Crocophim/Auto-Manage-Files",
        "en": "https://github.com/S-Crocophim/Auto-Manage-Files"
    },

    # ------------------------------------------------------------------
    # Settings Area (New)
    # ------------------------------------------------------------------
    "theme_label": {
        "id": "Tema:",
        "en": "Theme:"
    },
    "lang_label": {
        "id": "Bahasa:",
        "en": "Language:"
    },
    "theme_dark": {
        "id": "Gelap",
        "en": "Dark"
    },
    "theme_light": {
        "id": "Terang",
        "en": "Light"
    },

    # ------------------------------------------------------------------
    # Rule Dialog
    # ------------------------------------------------------------------
    "rule_title_add": {
        "id": "➕  Aturan Baru",
        "en": "➕  New Rule"
    },
    "rule_title_edit": {
        "id": "✏️  Edit Aturan",
        "en": "✏️  Edit Rule"
    },
    "rule_name": {
        "id": "Nama Aturan",
        "en": "Rule Name"
    },
    "rule_name_ph": {
        "id": "Contoh: Spreadsheets",
        "en": "Example: Spreadsheets"
    },
    "rule_watch": {
        "id": "Folder Dipantau (bisa lebih dari satu, pisahkan dengan koma)",
        "en": "Watch Folders (can be multiple, separate by comma)"
    },
    "rule_ext": {
        "id": "Ekstensi (pisahkan koma)",
        "en": "Extensions (comma separated)"
    },
    "rule_ext_ph": {
        "id": ".xlsx, .csv, .pdf",
        "en": ".xlsx, .csv, .pdf"
    },
    "rule_dest": {
        "id": "Folder Tujuan",
        "en": "Destination Folder"
    },
    "rule_btn_cancel": {
        "id": "Batal",
        "en": "Cancel"
    },
    "rule_btn_save": {
        "id": "💾  Simpan",
        "en": "💾  Save"
    },
    "rule_warn_empty": {
        "id": "⚠️  Semua field harus diisi!",
        "en": "⚠️  All fields must be filled!"
    },

    # ------------------------------------------------------------------
    # System Tray
    # ------------------------------------------------------------------
    "tray_open": {
        "id": "⚙️  Buka Pengaturan",
        "en": "⚙️  Open Settings"
    },
    "tray_manual": {
        "id": "🗂️  Rapihkan Sekarang (Manual)",
        "en": "🗂️  Organize Now (Manual)"
    },
    "tray_quit": {
        "id": "❌  Keluar",
        "en": "❌  Quit"
    },

    # ------------------------------------------------------------------
    # Notifications
    # ------------------------------------------------------------------
    "notif_success_title": {
        "id": "✨ File Dipindahkan!",
        "en": "✨ File Moved!"
    },
    "notif_done_title": {
        "id": "📂 Organisasi Selesai",
        "en": "📂 Organization Done"
    },
    "notif_done_msg": {
        "id": "Tidak ada file yang perlu dipindahkan.",
        "en": "No files needed to be moved."
    },
    "notif_err_lock_title": {
        "id": "⚠️ Gagal Dipindahkan (Terkunci)",
        "en": "⚠️ Failed to Move (Locked)"
    },
    "notif_err_lock_msg": {
        "id": "Masih digunakan oleh aplikasi lain.",
        "en": "Still in use by another application."
    },
    "notif_err_title": {
        "id": "❌ Gagal Dipindahkan (Error)",
        "en": "❌ Failed to Move (Error)"
    },
    "notif_err_msg": {
        "id": "Error:",
        "en": "Error:"
    }
}

class Translator:
    def __init__(self, lang_code: str = "id"):
        self.lang = lang_code if lang_code in ["id", "en"] else "id"

    def set_language(self, lang_code: str):
        self.lang = lang_code if lang_code in ["id", "en"] else "id"

    def get(self, key: str) -> str:
        """Get the localized string for a given key."""
        entry = _TRANSLATIONS.get(key)
        if not entry:
            return f"[{key}]"
        return entry.get(self.lang, entry["en"])
