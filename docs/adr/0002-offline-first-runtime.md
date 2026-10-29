# ADR-0002: Runtime offline dan tanpa telemetry

- Status: Accepted
- Date: 2026-07-28

## Context

Pengguna meminta aplikasi desktop yang tetap berfungsi tanpa internet. Istilah offline dapat berarti runtime tanpa network, installer air-gapped, atau build source tanpa network; ketiganya berbeda.

## Decision

MVP menjamin runtime offline:

- tidak ada akun, API server, telemetry, analytics, ads, remote config, atau network updater;
- seluruh script, style, font, icon, dan image dibundel lokal;
- WebView tidak memuat remote HTML/content;
- data aplikasi hanya disimpan lokal pada profile pengguna;
- link repository hanya dibuka secara user-initiated di browser sistem.

Distribusi menyediakan dua NSIS flavor:

- standard untuk ukuran kecil;
- airgap yang menyertakan WebView2 offline installer.

Build source belum dijanjikan air-gapped. Build memerlukan npm registry/crates.io kecuali dependency divendor melalui proses terpisah.

## Consequences

- fungsi inti tetap tersedia ketika firewall memblokir aplikasi;
- tidak ada telemetry untuk diagnosis otomatis; support bundle harus diekspor manual dengan persetujuan pengguna;
- update dilakukan manual melalui installer bertanda tangan/checksum;
- installer airgap jauh lebih besar dan perlu diuji terpisah.

## Verification

Release gate memeriksa CSP dan asset lokal, menjalankan smoke test offline pada milestone hardening, serta menerbitkan SHA-256. Penambahan dependency HTTP/updater atau remote asset memerlukan ADR pengganti dan persetujuan eksplisit.
