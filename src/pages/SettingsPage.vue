<script setup lang="ts">
import { reactive, watch } from "vue";
import { desktopApi } from "@/ipc/commands";
import type { SettingsDto, Theme } from "@/ipc/generated-types";
import { useI18n } from "@/i18n";
import { useActivityStore } from "@/stores/activity";
import { useRulesStore } from "@/stores/rules";
import { useSettingsStore } from "@/stores/settings";
import { useUiStore } from "@/stores/ui";

const settings = useSettingsStore();
const rules = useRulesStore();
const activity = useActivityStore();
const ui = useUiStore();
const { t } = useI18n();
const draft = reactive<SettingsDto>({ ...settings.values });
const configPath = reactive({ value: "" });
const transferring = reactive({ import: false, export: false });

watch(() => settings.values, (values) => Object.assign(draft, values), { deep: true, immediate: true });

async function save(): Promise<void> {
  try {
    await settings.save({ ...draft });
    ui.notify(t("settingsSaved"), "success");
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  }
}

function previewTheme(event: Event): void {
  const theme = (event.target as HTMLSelectElement).value as Theme;
  draft.theme = theme;
  settings.previewTheme(theme);
}

async function chooseConfigFolder(): Promise<void> {
  try {
    const folder = await desktopApi.pickFolder();
    if (folder !== null) configPath.value = `${folder.replace(/[\\/]$/, "")}\\auto-manage-files-config.json`;
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  }
}

async function exportConfig(): Promise<void> {
  if (!configPath.value.trim()) return;
  transferring.export = true;
  try {
    await desktopApi.exportConfig(configPath.value.trim());
    ui.notify(t("exportSuccess"), "success");
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  } finally {
    transferring.export = false;
  }
}

async function importConfig(): Promise<void> {
  if (!configPath.value.trim() || !window.confirm("Impor akan mengganti aturan dan pengaturan saat ini. Lanjutkan?")) return;
  transferring.import = true;
  try {
    const bootstrap = await desktopApi.importConfig(configPath.value.trim());
    rules.hydrate(bootstrap.rules);
    activity.hydrate(bootstrap.recentActivity);
    settings.hydrate(bootstrap.settings);
    ui.notify(t("importSuccess"), "success");
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  } finally {
    transferring.import = false;
  }
}

async function shutdown(): Promise<void> {
  if (!window.confirm("Keluar sepenuhnya dari Auto File Organizer? Monitoring akan dihentikan.")) return;
  try {
    await desktopApi.shutdown();
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  }
}
</script>

<template>
  <section class="page-stack settings-page">
    <div class="page-heading page-heading--actions">
      <div><h2>{{ t("settingsTitle") }}</h2><p>{{ t("settingsSubtitle") }}</p></div><button class="button button--primary" type="button" :disabled="settings.saving" @click="save">
        <span v-if="settings.saving" class="button-spinner" aria-hidden="true" />{{ t("save") }}
      </button>
    </div>

    <div class="settings-layout">
      <div class="settings-main">
        <section class="panel settings-section">
          <header><span class="settings-icon settings-icon--violet" aria-hidden="true">◐</span><div><h3>{{ t("appearance") }}</h3><p>Sesuaikan tampilan tanpa memengaruhi operasi file.</p></div></header>
          <div class="settings-fields">
            <label><span class="field-label">{{ t("theme") }}</span><select :value="draft.theme" class="select-input" @change="previewTheme"><option value="system">{{ t("themeSystem") }}</option><option value="light">{{ t("themeLight") }}</option><option value="dark">{{ t("themeDark") }}</option></select></label>
            <label><span class="field-label">{{ t("language") }}</span><select v-model="draft.language" class="select-input"><option value="id">Bahasa Indonesia</option><option value="en">English</option></select></label>
          </div>
        </section>

        <section class="panel settings-section">
          <header><span class="settings-icon settings-icon--blue" aria-hidden="true">⚙</span><div><h3>{{ t("behavior") }}</h3><p>Kontrol startup, tray, dan notifikasi Windows.</p></div></header>
          <div class="switch-list">
            <label class="switch-row"><span><strong>{{ t("startAtLogin") }}</strong><small>Buka organizer otomatis setelah Anda login.</small></span><input v-model="draft.autoStart" type="checkbox" role="switch"><i /></label>
            <label class="switch-row"><span><strong>{{ t("minimizeToTray") }}</strong><small>Monitoring tetap berjalan ketika jendela ditutup.</small></span><input v-model="draft.minimizeToTrayOnClose" type="checkbox" role="switch"><i /></label>
            <label class="switch-row"><span><strong>{{ t("notifications") }}</strong><small>Hanya tampilkan hasil dan masalah penting.</small></span><input v-model="draft.notificationsEnabled" type="checkbox" role="switch"><i /></label>
            <label class="switch-row"><span><strong>{{ t("monitoringByDefault") }}</strong><small>Mulai watcher saat aplikasi dibuka.</small></span><input v-model="draft.monitoringEnabled" type="checkbox" role="switch"><i /></label>
            <label class="switch-row"><span><strong>Preview sebelum operasi manual</strong><small>Wajibkan konfirmasi daftar file sebelum dipindahkan.</small></span><input v-model="draft.previewBeforeManual" type="checkbox" role="switch"><i /></label>
          </div>
        </section>

        <section class="panel settings-section">
          <header><span class="settings-icon settings-icon--green" aria-hidden="true">◷</span><div><h3>Riwayat & stabilitas</h3><p>Atur retensi audit dan waktu tunggu file selesai ditulis.</p></div></header>
          <div class="settings-fields">
            <label><span class="field-label">Simpan riwayat (hari)</span><input v-model.number="draft.historyRetentionDays" class="text-input" type="number" min="1" max="3650"><small class="field-help">Rekomendasi: 30 hari.</small></label>
            <label><span class="field-label">Jendela stabilitas (ms)</span><input v-model.number="draft.stabilityWindowMs" class="text-input" type="number" min="250" max="30000" step="250"><small class="field-help">Rekomendasi: 1500 ms.</small></label>
          </div>
        </section>

        <section class="panel settings-section">
          <header><span class="settings-icon settings-icon--amber" aria-hidden="true">⇄</span><div><h3>{{ t("dataTransfer") }}</h3><p>Backup portabel berisi aturan dan pengaturan, tanpa file pribadi.</p></div></header>
          <label class="field-label" for="config-path">{{ t("configPath") }}</label>
          <div class="path-control">
            <input id="config-path" v-model="configPath.value" class="text-input" placeholder="D:\Backup\auto-manage-files-config.json"><button class="button button--ghost button--small" type="button" @click="chooseConfigFolder">
              {{ t("browse") }}
            </button>
          </div>
          <span class="field-help">{{ t("configPathHelp") }}</span>
          <div class="button-row">
            <button class="button button--secondary" type="button" :disabled="!configPath.value || transferring.export" @click="exportConfig">
              {{ t("exportConfig") }}
            </button><button class="button button--secondary" type="button" :disabled="!configPath.value || transferring.import" @click="importConfig">
              {{ t("importConfig") }}
            </button>
          </div>
        </section>
      </div>

      <aside class="settings-aside">
        <section class="panel about-card">
          <div class="brand-mark brand-mark--large" aria-hidden="true">
            A
          </div><h3>{{ t("appName") }}</h3><span>Version 2.0.0</span><p>{{ t("aboutBody") }}</p><dl><div><dt>Runtime</dt><dd>Tauri 2 + Rust</dd></div><div><dt>Antarmuka</dt><dd>Vue 3 + TypeScript</dd></div><div><dt>Penyimpanan</dt><dd>SQLite lokal</dd></div></dl>
        </section>
        <section class="panel privacy-summary">
          <span aria-hidden="true">◆</span><h3>Privasi bawaan</h3><p>Tidak ada akun, cloud, telemetry, atau transfer jaringan dalam runtime aplikasi.</p>
        </section>
        <button class="button button--danger button--full" type="button" @click="shutdown">
          {{ t("shutdown") }}
        </button>
      </aside>
    </div>
  </section>
</template>
