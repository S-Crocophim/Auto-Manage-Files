# Contributing

Terima kasih sudah membantu rebuild Auto File Organizer. Prioritas tertinggi proyek ini adalah mencegah silent data loss.

## Workflow

1. Buat branch pendek dari `main`, misalnya `feat/rule-preview` atau `fix/cross-volume-recovery`.
2. Batasi satu PR pada satu tujuan yang dapat direview.
3. Tambahkan test untuk behavior baru atau bug yang diperbaiki.
4. Jalankan semua quality gate lokal.
5. Buka PR dengan scope, risiko data, screenshot UI bila relevan, test evidence, dan migration/recovery notes.

Jangan commit langsung ke `main` dan jangan mencampur dependency update besar dengan perubahan behavior.

## Setup

Lihat [panduan build](docs/build-and-release.md). Gunakan versi toolchain yang dipin repository dan install dependency frontend dengan `npm ci`.

## Gate wajib

```powershell
.\scripts\verify.ps1
```

`-SkipDesktopBuild` boleh dipakai saat iterasi, tetapi gate lengkap wajib dijalankan sebelum PR dinyatakan siap review. `-Installer` menjalankan NSIS unsigned build sebagai pengganti executable smoke build.

## Aturan keselamatan test

- Gunakan temporary directory yang dibuat test runner.
- Jangan pernah menunjuk Downloads, Documents, Desktop, repository root, home directory, atau drive root.
- Uji failure sebelum source deletion: permission denied, destination collision, disk full simulation, interrupted copy, dan verification failure.
- Integration test harus membersihkan hanya path temporary yang dibuat test tersebut.
- Jangan mengubah atau menghapus file pengguna sebagai bagian test manual tanpa backup dan persetujuan eksplisit.

## Boundary kode

- Business rule, path planning, collision policy, dan state transition berada di Rust domain/core.
- Vue component tidak menjalankan mutasi filesystem dan tidak menyimpan authoritative history.
- IPC command yang mengubah state menerima ID/DTO tervalidasi; hindari arbitrary source/destination paths.
- Semua path di-canonicalize backend dan diperiksa terhadap source milik rule.
- Command baru, plugin baru, atau permission baru harus disertai alasan serta update capability/CSP bila diperlukan.
- Jangan menambahkan shell plugin, generic filesystem/SQL access, remote WebView content, telemetry, atau updater tanpa ADR pengganti.

## Database dan migration

- Migration bersifat append-only setelah dirilis.
- Perubahan schema membutuhkan fixture upgrade test dan rollback/recovery note.
- Operation journal harus ditulis sebelum mutasi file.
- Jangan membentuk history dengan parsing log string.
- Retention tidak boleh menghapus recovery file yang masih dibutuhkan operation yang dapat di-undo.

## TypeScript dan Rust

- TypeScript strict; hindari `any`, unchecked cast, dan business logic di template.
- Error dari IPC diperlakukan `unknown` lalu dinormalisasi.
- Rust tidak boleh mengabaikan `Result` penting dengan `.ok()` pada jalur data safety.
- Hindari `unwrap`/`expect` pada input, lock, I/O, dan recovery path; propagasikan stable error.
- Clippy warning adalah error pada CI.

## Versi dan dependency

- Jaga versi `package.json`, `src-tauri/Cargo.toml`, dan Tauri config tetap sama.
- Commit `package-lock.json` dan `src-tauri/Cargo.lock`.
- Jangan mengedit lockfile manual.
- Dependency update dibuat pada PR terpisah dan tetap menjalankan offline/CSP review.
- Tag rilis hanya dibuat maintainer setelah commit berada di `main` dan gate lulus.

## Security dan privacy

Jangan commit secret, PFX, token, database pengguna, log pengguna, support bundle, installer, `node_modules`, `dist`, atau `target`. Gunakan synthetic path/data pada issue dan test. Kerentanan arbitrary file operation atau data loss harus dilaporkan lewat private security advisory bila tersedia.

## Checklist PR

- [ ] Scope dan user impact dijelaskan.
- [ ] Test baru/diubah dan semua gate lulus.
- [ ] Tidak ada mutasi file di frontend.
- [ ] Failure, recovery, concurrency, dan undo dipertimbangkan.
- [ ] Capability/CSP tetap minimum.
- [ ] Migration serta version impact dicatat.
- [ ] UI dapat digunakan keyboard dan screenshot disertakan bila relevan.
- [ ] Tidak ada runtime network dependency baru.
