# Build dan release Windows

## Prasyarat lokal

- Windows 10/11 x64.
- Node.js 24.14.1 dan npm yang sesuai lockfile.
- Rust 1.96.0 melalui `rustup`; repository memasang target MSVC, rustfmt, dan Clippy lewat `rust-toolchain.toml`.
- Visual Studio Build Tools dengan workload **Desktop development with C++** dan Windows SDK.
- WebView2 Runtime untuk menjalankan development build.

Periksa tool:

```powershell
node --version
npm --version
rustc --version
cargo --version
```

## Setup dan development

```powershell
npm ci
npm run tauri -- dev
```

Gunakan `npm ci`, bukan `npm install`, pada clean build dan CI agar `package-lock.json` dihormati.

## Quality gate lokal

```powershell
.\scripts\verify.ps1
```

Gunakan `-SkipDesktopBuild` hanya untuk iterasi statis yang cepat. Gunakan `-Installer` untuk mengganti executable smoke build dengan NSIS unsigned build.

Gate dependency menjalankan `npm audit --audit-level=high` terhadap lockfile dan membutuhkan akses registry. Ini adalah pemeriksaan build/CI, bukan kebutuhan network aplikasi saat runtime.

Test filesystem hanya boleh memakai temporary directory. Jangan menunjuk Downloads/Documents pribadi saat menjalankan integration test.

## Installer standard

```powershell
npm run tauri -- build --bundles nsis --ci --no-sign -- --locked
```

Installer berada di `src-tauri/target/release/bundle/nsis/`. Flavor standard berukuran lebih kecil. Bila WebView2 tidak tersedia, proses instalasi dapat memerlukan bootstrapper/network sesuai konfigurasi Tauri.

## Installer air-gapped

```powershell
npm run tauri -- build --bundles nsis --config src-tauri/tauri.offline.conf.json --ci --no-sign -- --locked
```

Flavor airgap membundel WebView2 offline installer. Build-nya memerlukan network untuk mengambil dependency/runtime pada runner bersih, tetapi installer hasilnya dapat dipasang tanpa internet. Ukuran artifact jauh lebih besar.

## CI dan release

`.github/workflows/ci.yml` berjalan pada pull request/push ke `main` dan memeriksa frontend, Rust, version alignment, serta Tauri production smoke build.

`.github/workflows/release.yml` hanya berjalan pada tag `v*.*.*`. Workflow:

1. memastikan tag sama dengan product version;
2. mengulang semua quality gate;
3. membangun NSIS standard dan airgap;
4. membuat SHA-256 per artifact dan `SHA256SUMS.txt`;
5. membuat atau memperbarui draft GitHub Release.

Workflow sengaja memakai `--no-sign` dan nama `UNSIGNED` sampai code signing tersedia. Draft tersebut hanya internal testing.

## Version bump

Sebelum membuat tag, samakan SemVer pada:

- `package.json`;
- `src-tauri/Cargo.toml`;
- `src-tauri/tauri.conf.json` (atau arahkan field version ke package source yang disepakati).

Validasi alignment sebelum commit:

```powershell
node scripts/check-version.mjs
```

Kemudian regenerasi lockfile hanya jika manifest dependency berubah, jalankan quality gate, commit, dan buat annotated tag `v<version>` dari commit di `main`. Workflow release memanggil skrip yang sama dengan `--release --tag <tag>` untuk memvalidasi CSP, current-user NSIS, dan konfigurasi WebView2 airgap sebelum packaging.

## Code signing

Public release harus ditandatangani dengan certificate Windows/Artifact Signing yang disimpan di GitHub environment secrets. Jangan commit PFX, password, client secret, thumbprint pribadi, atau hasil decode certificate. Setelah signing, hitung checksum dari artifact final yang sudah ditandatangani.

## Smoke test rilis

Minimal pada clean Windows 10 22H2 dan Windows 11 x64:

- install/uninstall current-user tanpa admin;
- first run dan database migration;
- create rule, preview, execute, history, dan undo;
- watcher, pause/resume, tray, autostart;
- same-volume dan cross-volume;
- file locked, duplicate name, Unicode/long path, removable drive;
- runtime dengan outbound network diblokir;
- upgrade dari fixture versi sebelumnya;
- verifikasi Authenticode dan SHA-256.
