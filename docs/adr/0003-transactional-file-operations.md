# ADR-0003: Operation journal dan transfer file aman

- Status: Accepted
- Date: 2026-07-28

## Context

`rename`, overwrite, cross-volume copy/delete, crash, file lock, dan duplicate watcher event dapat menyebabkan data hilang atau status palsu bila dijalankan sebagai langkah lepas. Undo juga tidak dapat dipercaya bila original path hanya berasal dari string log/frontend.

## Decision

Semua mutasi file menggunakan persistent operation journal dan state machine. Backend:

1. memvalidasi rule/source/destination canonical;
2. membuat plan dan idempotency key;
3. commit journal sebelum mutasi;
4. memakai atomic rename untuk same-volume bila tersedia;
5. untuk cross-volume, copy ke temporary destination, flush/verify, atomic finalize, lalu hapus source;
6. menyimpan destination lama di recovery storage sebelum overwrite;
7. commit terminal state dan history;
8. menjalankan recovery untuk operation non-terminal saat startup.

Undo menerima operation ID, bukan source/destination bebas, lalu menggunakan record persisted. Hasil ambigu ditandai `NeedsReview` dan tidak dianggap sukses.

## Consequences

- operasi lebih kompleks dan membutuhkan SQLite transaction serta recovery tests;
- disk sementara yang dibutuhkan dapat mendekati ukuran file yang dipindahkan;
- overwrite aman membutuhkan retention dan pembersihan recovery storage;
- reliability, auditability, dan undo setelah restart meningkat signifikan.

## Rejected alternatives

- `remove(destination)` lalu `rename(source)`: dapat menghilangkan file lama sebelum source berhasil.
- `copy(source, destination)` lalu langsung delete source: tidak cukup untuk crash/partial write dan verifikasi.
- history di frontend: tidak authoritative dan hilang/rusak saat WebView gagal.
