# Auto File Organizer

<p align="center">
  <img src="public/icon.png" width="96" height="96" alt="Auto File Organizer logo" />
</p>

Aplikasi desktop Windows offline-first untuk memantau folder, menampilkan preview, lalu mengorganisasi file dengan rule yang dapat diaudit dan di-undo.

> Status: rebuild v2 sedang berlangsung. Development build belum boleh dipakai pada file yang tidak memiliki backup sampai seluruh acceptance criteria dan smoke test rilis selesai.

## Sasaran v2

- Rule berbasis banyak source folder, extension, glob nama, ukuran, priority, dan collision policy.
- Preview/dry-run sebelum manual execution.
- Watcher dengan dedupe, stability probe, retry/backoff, reconciliation, pause/resume, dan health state.
- Safe same-volume serta cross-volume move.
- Recovery-backed overwrite, persistent history, crash recovery, dan undo setelah restart.
- Dashboard, Rules, Activity, Settings, tray, autostart, notifikasi, Bahasa Indonesia/English, serta theme System/Light/Dark.
- SQLite lokal; tanpa akun, telemetry, API server, atau kebutuhan internet saat fungsi inti berjalan.

Detail lengkap terdapat pada [PRD v2](docs/PRD-Auto-Manage-Files-v2.md).

## Prinsip keselamatan

Frontend tidak memindahkan file secara langsung. Rust memvalidasi source/destination, membuat operation journal sebelum mutasi, memverifikasi destination sebelum menghapus source, dan menggunakan operation ID persisted untuk undo.

Saat hasil setelah crash ambigu, aplikasi harus menampilkan `NeedsReview`, bukan melaporkan sukses. Lihat [arsitektur](docs/architecture.md) dan [limitasi](docs/known-limitations.md).

## Teknologi

- Tauri 2 desktop shell.
- Rust untuk domain, persistence, watcher, dan filesystem operations.
- Vue 3 + TypeScript strict + Vite untuk UI.
- SQLite untuk rule, journal, history, recovery metadata, settings, dan migration.
- NSIS current-user installer untuk Windows 10/11 x64.

## Prasyarat developer

- Windows 10/11 x64.
- Node.js 24.14.1.
- Rust 1.96.0 melalui rustup.
- Visual Studio Build Tools: Desktop development with C++ dan Windows SDK.
- WebView2 Runtime untuk development.

## Menjalankan dari source

```powershell
git clone https://github.com/S-Crocophim/Auto-Manage-Files.git
Set-Location Auto-Manage-Files
npm ci
npm run tauri -- dev
```

Jangan memakai folder pribadi sebagai fixture test. Semua integration test filesystem wajib menggunakan temporary directory.

## Quality gate

Jalankan pemeriksaan versi, frontend, Rust, dan Tauri production smoke build melalui satu skrip:

```powershell
.\scripts\verify.ps1
```

Untuk iterasi cepat tanpa desktop build, gunakan `-SkipDesktopBuild`. Gunakan `-Installer` untuk mengganti smoke build dengan NSIS unsigned build. Opsi tersebut tidak dapat digabungkan.

Pull request ke `main` menjalankan gate yang sama pada GitHub Actions Windows.

## Packaging offline

Installer standard:

```powershell
npm run tauri -- build --bundles nsis --ci --no-sign -- --locked
```

Installer air-gapped yang membawa WebView2 offline installer:

```powershell
npm run tauri -- build --bundles nsis --config src-tauri/tauri.offline.conf.json --ci --no-sign -- --locked
```

Runtime offline, installer offline, dan build source offline adalah jaminan berbeda. Fungsi inti ditargetkan tanpa network; flavor airgap dapat dipasang tanpa network; build source tetap membutuhkan dependency npm/crates kecuali dependency divendor terpisah.

## Struktur repository

```text
.github/workflows/       CI dan release tag
src/                     Vue + TypeScript presentation
src-tauri/src/           Rust application, domain, dan adapters
src-tauri/capabilities/  Tauri capability allowlist
src-tauri/migrations/    SQLite migration
scripts/                 pemeriksaan repository/build
docs/                    PRD, arsitektur, ADR, security, release
```

## Dokumentasi

- [PRD v2](docs/PRD-Auto-Manage-Files-v2.md)
- [Arsitektur](docs/architecture.md)
- [Build dan release](docs/build-and-release.md)
- [Security dan privacy](docs/security-and-privacy.md)
- [Limitasi dan release blockers](docs/known-limitations.md)
- [Architecture Decision Records](docs/adr/)
- [Panduan kontribusi](CONTRIBUTING.md)

## Privacy

Data rule, settings, operation journal, history, log, dan recovery disimpan lokal pada profile pengguna. Tidak ada telemetry atau upload otomatis. Support bundle hanya dibuat dan dibagikan melalui aksi eksplisit pengguna.

## Release

Tag harus berbentuk `v<SemVer>` dan sama dengan versi npm, Cargo, serta Tauri. Workflow membangun installer standard dan airgap serta SHA-256.

Selama code signing belum dikonfigurasi, workflow hanya membuat draft artifact yang secara eksplisit bernama `UNSIGNED`. Artifact tersebut untuk internal testing, bukan public stable release.
