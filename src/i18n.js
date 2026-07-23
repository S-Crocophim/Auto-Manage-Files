export const translations = {
  app_name: { id: "Auto File Organizer", en: "Auto File Organizer" },
  app_desc: { id: "Kelola file otomatis", en: "Auto manage files" },
  
  // Navigation
  nav_dashboard: { id: "Dashboard", en: "Dashboard" },
  nav_logs: { id: "Logs & Undo", en: "Logs & Undo" },
  nav_tutorial: { id: "Tutorial", en: "Tutorial" },
  nav_about: { id: "About", en: "About" },
  nav_add_rule: { id: "Tambah Aturan", en: "Add Rule" },
  nav_manual: { id: "Rapihkan Sekarang", en: "Organize Now" },
  nav_manual_processing: { id: "Memproses...", en: "Processing..." },
  nav_auto_start: { id: "Auto-Start", en: "Auto-Start" },
  nav_minimize_tray: { id: "Minimize ke Tray", en: "Minimize to Tray" },
  nav_export_settings: { id: "Export Settings", en: "Export Settings" },
  nav_import_settings: { id: "Import Settings", en: "Import Settings" },
  nav_quit: { id: "Keluar", en: "Quit" },
  watermark_credit: { id: "Dibuat oleh Px0 (Rust + Vue)", en: "Made by Px0 (Rust + Vue)" },

  // Dashboard & KPI Cards
  dashboard_title: { id: "Dashboard Aturan", en: "Rule Dashboard" },
  status_label: { id: "Status Monitoring", en: "Monitoring Status" },
  status_active: { id: "Monitoring Aktif", en: "Monitoring Active" },
  status_inactive: { id: "Monitoring Nonaktif", en: "Monitoring Inactive" },
  active_rules_stat: { id: "Aturan Aktif", en: "Active Rules" },
  total_moved_stat: { id: "File Dirapikan", en: "Files Organized" },
  active_rules_label: { id: "Daftar aturan pemindahan file yang dikonfigurasi", en: "Configured file organization rules" },
  no_rules_msg: { id: "Belum ada aturan. Klik 'Tambah Aturan' untuk memulai.", en: "No rules yet. Click 'Add Rule' to get started." },
  unnamed_rule: { id: "Tanpa Nama", en: "Unnamed" },
  search_ph: { id: "Cari aturan...", en: "Search rules..." },
  rule_list_title: { id: "Daftar Aturan Pemindahan", en: "Organization Rule List" },
  size_max: { id: "Maks", en: "Max" },

  // Logs & History Undo
  logs_title: { id: "Catatan Aktivitas & Histori", en: "Activity Logs & History" },
  logs_subtitle: { id: "Histori pemindahan file & log konsol real-time", en: "Real-time console logs & undoable move history" },
  history_section_title: { id: "Histori Pemindahan File (Bisa Dibatalkan)", en: "File Move History (Undoable)" },
  console_section_title: { id: "Log Konsol Real-time", en: "Real-time Console Log" },
  console_waiting_msg: { id: "Siap. Menunggu aktivitas sistem file...", en: "Ready. Waiting for file system events..." },
  clear_logs: { id: "Bersihkan Log", en: "Clear Logs" },
  btn_undo: { id: "Batalkan (Undo)", en: "Undo Move" },
  no_history_msg: { id: "Belum ada riwayat pemindahan file.", en: "No file move history yet." },

  // Settings
  theme_label: { id: "Tema:", en: "Theme:" },
  lang_label: { id: "Bahasa:", en: "Language:" },
  theme_dark: { id: "Gelap", en: "Dark" },
  theme_light: { id: "Terang", en: "Light" },

  // Rule Dialog
  rule_title_add: { id: "Aturan Baru", en: "New Rule" },
  rule_title_edit: { id: "Edit Aturan", en: "Edit Rule" },
  rule_name: { id: "Nama Aturan", en: "Rule Name" },
  rule_name_ph: { id: "Contoh: Spreadsheets", en: "Example: Spreadsheets" },
  rule_watch: { id: "Folder Dipantau", en: "Watch Folders" },
  rule_watch_ph: { id: "Taruh atau masukkan folder dipantau...", en: "Drop or enter watch folder..." },
  rule_ext: { id: "Ekstensi (pisah koma)", en: "Extensions (comma separated)" },
  rule_ext_ph: { id: ".xlsx, .csv, .pdf", en: ".xlsx, .csv, .pdf" },
  rule_dest: { id: "Folder Tujuan", en: "Destination Folder" },
  rule_dest_ph: { id: "Taruh atau masukkan folder tujuan...", en: "Drop or enter destination folder..." },
  
  // Advanced Filters & Conflict Handling
  conflict_label: { id: "Aksi Jika File Duplikat:", en: "Conflict Strategy:" },
  conflict_rename: { id: "Ganti Nama Otomatis (Auto-rename)", en: "Auto-rename" },
  conflict_skip: { id: "Lewati File (Skip)", en: "Skip" },
  conflict_overwrite: { id: "Timpa File (Overwrite)", en: "Overwrite" },
  pattern_label: { id: "Pola Nama File / Wildcard (Opsional):", en: "Filename Pattern / Wildcard (Optional):" },
  pattern_ph: { id: "Contoh: Invoice_* atau *.pdf", en: "Example: Invoice_* or *.pdf" },
  min_size_label: { id: "Ukuran Min (MB):", en: "Min Size (MB):" },
  max_size_label: { id: "Ukuran Max (MB):", en: "Max Size (MB):" },

  rule_btn_cancel: { id: "Batal", en: "Cancel" },
  rule_btn_save: { id: "Simpan", en: "Save" },
  rule_warn_empty: { id: "Semua field utama harus diisi!", en: "All required fields must be filled!" },

  // Tutorial & About
  tutorial_content: {
    id: [
      "1. Tambahkan Aturan Baru melalui tombol 'Tambah Aturan'.",
      "2. Tentukan nama aturan, ekstensi file, folder pantauan, dan folder tujuan.",
      "3. Anda bisa mengatur filter ukuran file (Min/Max MB) dan pola nama file (seperti Invoice_*).",
      "4. Pilih strategi jika file duplikat: Ganti Nama Otomatis, Lewati, atau Timpa.",
      "5. Tarik & Lepas (Drag & Drop) folder dari Windows Explorer langsung ke input path folder.",
      "6. Gunakan tombol Undo di tab Logs untuk mengembalikan file yang baru dipindahkan.",
      "7. Aktifkan mode Auto-Start agar aplikasi berjalan di background saat PC menyala."
    ],
    en: [
      "1. Add a New Rule via the 'Add Rule' button.",
      "2. Set rule name, file extensions, watch folder, and target folder.",
      "3. You can configure file size bounds (Min/Max MB) and pattern filters (like Invoice_*).",
      "4. Choose conflict strategy: Auto-rename, Skip, or Overwrite.",
      "5. Drag & Drop folders directly from Windows Explorer into folder path inputs.",
      "6. Use the Undo button in the Logs tab to restore recently moved files.",
      "7. Enable Auto-Start so the app runs in the background on startup."
    ]
  },
  about_content: {
    id: "Auto File Organizer v1.0.0 (Rust + Vue)\n\nAplikasi super ringan dan cepat untuk mengatur file secara otomatis.\nPerforma native didukung oleh Rust & Tauri.",
    en: "Auto File Organizer v1.0.0 (Rust + Vue)\n\nA ultra-lightweight and blazingly fast automatic file organizer.\nNative performance powered by Rust & Tauri."
  },
  repo_label: { id: "Repositori:", en: "Repository:" }
};

export function t(key, lang = "id") {
  const item = translations[key];
  if (!item) return `[${key}]`;
  return item[lang] || item["en"];
}
