# Security dan privacy

## Ringkasan

Auto File Organizer memproses nama, metadata, dan isi file secara lokal hanya sejauh diperlukan untuk memindahkan/menyalin file. MVP tidak memiliki akun, server, telemetry, analytics, ads, remote config, atau automatic network updater.

## Data lokal

Aplikasi menyimpan data berikut pada application data milik user Windows:

- rule dan source folder;
- settings;
- operation journal dan history;
- diagnostic log dengan retention/size cap;
- recovery copy sementara ketika dibutuhkan oleh safe overwrite/undo.

Database, log, dan recovery file tidak boleh ditulis ke repository atau folder instalasi. Uninstall dan clear-history perlu menjelaskan data mana yang tetap disimpan atau dihapus.

## Network

Fungsi inti tidak membutuhkan network. WebView tidak memuat remote script, font, image, HTML, atau API. Link repository pada halaman About hanya dibuka setelah aksi pengguna di browser default.

Installer standard dan source build dapat membutuhkan network pada fase install/build. Gunakan installer airgap untuk mesin tanpa internet. Lihat [build dan release](./build-and-release.md).

## Akses filesystem

Aplikasi berjalan dengan privilege pengguna saat ini, bukan administrator. Pengguna memilih folder yang dipantau. Backend harus:

- canonicalize path dan memeriksa source ownership rule;
- menolak source/destination cycle;
- tidak mengikuti symlink/reparse point tanpa policy eksplisit;
- tidak menerima arbitrary source/destination pada command execute/undo;
- journal sebelum mutasi dan memakai recovery-backed overwrite;
- menghormati permission OS dan tidak mencoba bypass ACL.

## WebView dan IPC

- Tauri capability dibatasi ke window `main` dan plugin command yang benar-benar dipakai.
- CSP hanya mengizinkan asset lokal dan IPC.
- Shell, generic filesystem, generic SQL, dan remote WebView tidak diaktifkan.
- Semua DTO IPC divalidasi ulang di Rust dengan batas ukuran/jumlah.
- Operation memakai ID persisted, bukan path dari UI.

Capability mengurangi dampak frontend compromise, tetapi tidak memperbaiki command Rust yang terlalu kuat. Review signature dan validasi command adalah release gate keamanan.

## Import/export

Import JSON dibaca backend dengan batas ukuran, schema version, jumlah record, enum, path, dan duplicate ID. Invalid import tidak boleh mengubah konfigurasi aktif. Export hanya menulis ke path hasil dialog pengguna dan tidak menyertakan history/log/recovery kecuali pengguna memilih support bundle terpisah.

## Notification dan log

Privacy mode menyembunyikan full path dari notifikasi OS. Log teknis boleh berisi path hanya jika diperlukan untuk diagnosis lokal, memiliki retention/size cap, dan tidak diekspor otomatis. Support bundle dibuat manual, menampilkan preview isi, serta membutuhkan persetujuan pengguna sebelum dibagikan.

Jangan pernah mencatat isi file, certificate, token, environment secret, atau password.

## Supply chain dan release

- Commit `package-lock.json` dan `src-tauri/Cargo.lock`.
- CI memakai `npm ci`, menolak vulnerability npm berlevel high/critical, dan menjalankan Cargo dengan `--locked`.
- Rust toolchain dipin.
- Dependency update dipisah agar review jelas.
- Public artifact wajib Authenticode-signed dan memiliki SHA-256.
- Unsigned artifact harus tetap draft/internal dan namanya harus menyebut `UNSIGNED`.

## Pelaporan masalah keamanan

Jangan unggah file pribadi, database pengguna, recovery file, full path sensitif, atau credential ke issue publik. Berikan langkah reproduksi dengan temporary test data. Untuk kerentanan yang dapat menyebabkan data loss/arbitrary file operation, gunakan kanal private security advisory repository bila tersedia.
