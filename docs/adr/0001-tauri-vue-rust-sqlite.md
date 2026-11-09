# ADR-0001: Tauri, Vue, Rust, dan SQLite

- Status: Accepted
- Date: 2026-07-28

## Context

Aplikasi harus berjalan offline di Windows, ringan saat idle, aman untuk operasi file, dapat diuji, dan dapat dipaketkan sebagai installer desktop. Repository sebelumnya sudah memakai Tauri 2, Vue 3, Vite, dan Rust, tetapi business logic, persistence JSON, serta UI masih terlalu saling terikat.

## Decision

Gunakan:

- Tauri 2 sebagai desktop shell dan boundary IPC;
- Vue 3 + TypeScript strict untuk presentation;
- Rust untuk application service, rule engine, watcher, dan seluruh mutasi file;
- SQLite in-process untuk rule, operation journal, history, recovery metadata, settings, dan migration;
- adapter/trait untuk filesystem, clock, persistence, dan OS integration.

Domain/core tidak boleh bergantung pada Tauri. Frontend tidak mendapat generic filesystem atau SQL access.

## Consequences

Positif:

- runtime kecil dibanding desktop shell yang membundel browser penuh;
- operasi file dan persistence dapat dibuat transactional serta diuji;
- aplikasi tidak memerlukan service/database server;
- existing repository dapat dimigrasikan bertahap.

Trade-off:

- tim perlu menguasai TypeScript dan Rust;
- WebView2 adalah runtime eksternal Windows yang harus dipertimbangkan saat packaging;
- typed IPC dan migration menambah pekerjaan foundation tetapi mengurangi risiko data loss.

## Alternatives considered

- Electron: ekosistem luas, tetapi bundle dan idle footprint lebih besar.
- WPF/WinUI: integrasi Windows kuat, tetapi rewrite penuh dan portability tooling lebih terbatas.
- Python/Qt: iterasi cepat, tetapi packaging/runtime dan kontrol concurrency file membutuhkan hardening tambahan.
