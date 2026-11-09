# Limitasi dan release blockers

Dokumen ini membedakan batas produk yang disengaja dari fitur yang belum terverifikasi. Development build bukan pengganti backup dan tidak boleh dipakai pada file yang tidak dapat diganti sampai seluruh acceptance criteria PRD lulus.

## Baseline yang didukung

- Windows 10 22H2 dan Windows 11 x64.
- NTFS sebagai filesystem utama.
- Instalasi current-user tanpa administrator.
- Action MVP utama adalah move; fitur tambahan mengikuti prioritas PRD.

Platform 32-bit, ARM64, macOS, Linux, Windows Server, dan filesystem/network provider lain belum menjadi support target MVP.

## Belum dijamin

Sebelum public stable release, skenario berikut harus dianggap belum dijamin kecuali ada test evidence pada PR terkait:

- crash/power-loss pada setiap state operasi;
- disk penuh dan ruang destination tidak cukup;
- network share, cloud placeholder, symlink/junction, dan reparse point;
- FAT/exFAT/removable drive serta eject di tengah transfer;
- file sangat besar, sparse file, alternate data stream, ACL, dan extended attribute;
- Unicode edge case dan Windows long path;
- event watcher overflow/loss serta reconciliation;
- upgrade database dari seluruh versi lama;
- endurance untuk 100 rule, 100 source, dan 100.000 entry.

Jika state filesystem ambigu, aplikasi harus memilih `NeedsReview`, bukan menebak sukses.

## Offline

- Runtime inti ditargetkan offline.
- Installer standard dapat memerlukan WebView2/bootstrapper jika runtime belum ada.
- Installer airgap jauh lebih besar karena membawa WebView2 offline installer.
- Build source pada mesin bersih masih membutuhkan dependency npm/crates dan belum dijamin air-gapped.
- Link About dapat membuka browser setelah aksi pengguna; tidak dipakai oleh fungsi inti.

## Release dan signing

Workflow release saat ini menghasilkan draft artifact bernama `UNSIGNED`. SmartScreen dapat memperingatkan pengguna. Artifact ini hanya untuk internal testing dan tidak boleh dipromosikan sebagai public stable build sebelum code signing serta Windows 10/11 smoke test selesai.

## Recovery dan undo

Undo bergantung pada operation journal dan recovery retention. Undo tidak dapat dijanjikan setelah pengguna menghapus recovery data, retention berakhir, destination diubah aplikasi lain, media dilepas, atau permission berubah. UI harus menjelaskan alasan ketika undo tidak tersedia.

## Batas keamanan

Aplikasi hanya memiliki permission akun pengguna; ia tidak dapat memindahkan file yang ditolak ACL/lock OS. Aplikasi tidak boleh meminta admin hanya untuk mengatasi permission error. Import, arbitrary path IPC, dan overwrite merupakan area berisiko tinggi dan tetap release blockers sampai test terkait lulus.
