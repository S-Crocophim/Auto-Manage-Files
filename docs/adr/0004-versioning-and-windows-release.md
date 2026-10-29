# ADR-0004: Sinkronisasi versi dan release Windows

- Status: Accepted
- Date: 2026-07-28

## Context

Baseline repository memiliki versi berbeda di `package.json`, `Cargo.toml`, dan `tauri.conf.json`. Hal ini dapat menghasilkan nama installer, metadata executable, UI About, dan tag Git yang tidak konsisten.

## Decision

- Satu nomor SemVer digunakan pada npm package, Cargo package, dan Tauri config.
- CI menolak mismatch.
- Tag rilis harus tepat `v<version>`.
- `package-lock.json` dan `src-tauri/Cargo.lock` wajib committed.
- CI memakai `npm ci` serta Cargo `--locked`.
- Rust dipin melalui `rust-toolchain.toml`.
- Release hanya dipicu version tag dan menghasilkan NSIS standard + airgap serta SHA-256.
- Workflow saat ini membuat draft artifact berlabel `UNSIGNED`; artifact tidak boleh dipromosikan sebagai public stable release sebelum code signing resmi aktif.

## Consequences

- bump versi menjadi perubahan terkoordinasi dan mudah diaudit;
- tag salah berhenti sebelum packaging;
- release dapat direproduksi dari commit/tag dan lockfile yang sama;
- code-signing certificate/secrets tetap berada di GitHub environment/secrets, bukan repository.

## Release promotion

Untuk public release, maintainer wajib memverifikasi CI, signature Authenticode, checksum, installer smoke test pada Windows 10/11, migration, serta release notes. Draft unsigned hanya untuk pengujian internal.
