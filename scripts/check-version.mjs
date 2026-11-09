import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relativePath) =>
  readFile(path.join(root, relativePath), "utf8");

function readTomlSection(source, sectionName) {
  const lines = source.split(/\r?\n/);
  const header = `[${sectionName}]`;
  const start = lines.findIndex((line) => line.trim() === header);
  if (start === -1) {
    return undefined;
  }

  const relativeEnd = lines
    .slice(start + 1)
    .findIndex((line) => /^\s*\[[^\]]+]\s*$/.test(line));
  const end = relativeEnd === -1 ? lines.length : start + 1 + relativeEnd;
  return lines.slice(start + 1, end).join("\n");
}

function fail(message) {
  console.error(message);
  process.exitCode = 1;
}

function parseOptions(args) {
  const options = {
    expectedTag: undefined,
    release: false,
  };

  for (let index = 0; index < args.length; index += 1) {
    const argument = args[index];
    if (argument === "--release") {
      options.release = true;
    } else if (argument === "--tag" && !options.expectedTag) {
      options.expectedTag = args[index + 1];
      index += 1;
      if (!options.expectedTag) {
        throw new Error("Nilai --tag wajib diisi.");
      }
    } else {
      throw new Error(
        "Usage: node scripts/check-version.mjs [--release] [--tag v<SemVer>]",
      );
    }
  }

  return options;
}

async function validateReleaseConfiguration(tauriConfig) {
  const csp = tauriConfig.app?.security?.csp;
  if (typeof csp !== "string" || csp.trim().length === 0) {
    fail("Release membutuhkan CSP production non-empty di tauri.conf.json.");
  }

  const installMode = tauriConfig.bundle?.windows?.nsis?.installMode;
  if (installMode !== "currentUser") {
    fail(
      "Release membutuhkan bundle.windows.nsis.installMode=currentUser.",
    );
  }

  let offlineConfig;
  try {
    offlineConfig = JSON.parse(
      await read("src-tauri/tauri.offline.conf.json"),
    );
  } catch (error) {
    fail(
      `Konfigurasi airgap tidak dapat dibaca: ${
        error instanceof Error ? error.message : String(error)
      }`,
    );
    return;
  }

  const webviewMode =
    offlineConfig.bundle?.windows?.webviewInstallMode?.type;
  if (webviewMode !== "offlineInstaller") {
    fail(
      "tauri.offline.conf.json harus memakai webviewInstallMode.type=offlineInstaller.",
    );
  }
}

const packageJson = JSON.parse(await read("package.json"));
const tauriConfig = JSON.parse(await read("src-tauri/tauri.conf.json"));
const cargoToml = await read("src-tauri/Cargo.toml");
const packageSection = readTomlSection(cargoToml, "package");
const cargoVersion = packageSection?.match(
  /^\s*version\s*=\s*"([^"]+)"\s*(?:#.*)?$/m,
)?.[1];

const versions = new Map([
  ["package.json", packageJson.version],
  ["src-tauri/tauri.conf.json", tauriConfig.version],
  ["src-tauri/Cargo.toml", cargoVersion],
]);
const options = parseOptions(process.argv.slice(2));

const missing = [...versions].filter(
  ([, version]) => typeof version !== "string" || version.length === 0,
);
if (missing.length > 0) {
  for (const [file] of missing) {
    fail(`Version tidak ditemukan atau bukan string di ${file}`);
  }
} else {
  const unique = new Set(versions.values());
  if (unique.size !== 1) {
    fail("Versi aplikasi tidak sinkron:");
    for (const [file, version] of versions) {
      console.error(`- ${file}: ${version}`);
    }
  } else {
    const [version] = unique;
    const semver = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[A-Za-z-][0-9A-Za-z-]*)(?:\.(?:0|[1-9]\d*|\d*[A-Za-z-][0-9A-Za-z-]*))*))?(?:\+([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$/;
    if (!semver.test(version)) {
      fail(`Versi bukan SemVer yang valid: ${version}`);
    }

    if (options.expectedTag && options.expectedTag !== `v${version}`) {
      fail(
        `Tag tidak sinkron: expected v${version}, received ${options.expectedTag}`,
      );
    }

    if (options.release) {
      await validateReleaseConfiguration(tauriConfig);
    }

    if (process.exitCode !== 1) {
      console.log(
        options.expectedTag
          ? `Versi dan tag sinkron: ${version}`
          : `Versi sinkron: ${version}`,
      );
    }
  }
}
