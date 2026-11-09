<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { desktopApi } from "@/ipc/commands";
import { useI18n } from "@/i18n";
import { useAppStore } from "@/stores/app";
import { usePreviewStore } from "@/stores/preview";
import { useRulesStore } from "@/stores/rules";
import { useSettingsStore } from "@/stores/settings";
import { useUiStore } from "@/stores/ui";

const router = useRouter();
const app = useAppStore();
const preview = usePreviewStore();
const rules = useRulesStore();
const settings = useSettingsStore();
const ui = useUiStore();
const { t } = useI18n();
const selectedPreset = ref("documents");
const source = ref("");
const destination = ref("");
const saving = ref(false);
const createdRuleId = ref<string | null>(null);
const previewReady = ref(false);

const presets = computed(() => [
  { id: "documents", icon: "▤", name: t("presetDocuments"), extensions: ["pdf", "doc", "docx", "txt", "rtf", "odt"] },
  { id: "images", icon: "▧", name: t("presetImages"), extensions: ["jpg", "jpeg", "png", "gif", "webp", "svg"] },
  { id: "archives", icon: "▣", name: t("presetArchives"), extensions: ["zip", "rar", "7z", "tar", "gz"] },
  { id: "installers", icon: "↓", name: t("presetInstallers"), extensions: ["exe", "msi", "msix", "appx"] },
  { id: "spreadsheets", icon: "▦", name: t("presetSpreadsheets"), extensions: ["xlsx", "xls", "csv", "ods"] },
]);
const canSave = computed(() => Boolean(source.value.trim() && destination.value.trim()));
const configurationLocked = computed(() => createdRuleId.value !== null);

async function pick(target: "source" | "destination"): Promise<void> {
  if (configurationLocked.value) return;
  try {
    const path = await desktopApi.pickFolder();
    if (path !== null) {
      if (target === "source") source.value = path;
      else destination.value = path;
    }
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  }
}

async function preparePreview(): Promise<void> {
  const preset = presets.value.find((item) => item.id === selectedPreset.value);
  if (!preset || !canSave.value) return;
  saving.value = true;
  try {
    if (createdRuleId.value === null) {
      const rule = await rules.save({
        name: `${preset.name} otomatis`,
        enabled: false,
        sources: [source.value.trim()],
        destination: destination.value.trim(),
        extensions: preset.extensions,
        nameGlobs: [],
        conflictPolicy: "autoRename",
        recursive: false,
        notifications: true,
      });
      createdRuleId.value = rule.id;
    }
    await preview.create([createdRuleId.value]);
    previewReady.value = true;
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  } finally {
    saving.value = false;
  }
}

async function activate(): Promise<void> {
  if (!createdRuleId.value || !previewReady.value) return;
  saving.value = true;
  try {
    await rules.toggle(createdRuleId.value, true);
    await settings.save({ onboardingComplete: true, monitoringEnabled: true });
    await app.setMonitoring(true);
    ui.notify(t("setupComplete"), "success");
    await router.replace("/dashboard");
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  } finally {
    saving.value = false;
  }
}

async function editConfiguration(): Promise<void> {
  if (!createdRuleId.value) return;
  saving.value = true;
  try {
    await rules.remove(createdRuleId.value);
    createdRuleId.value = null;
    previewReady.value = false;
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  } finally {
    saving.value = false;
  }
}

async function skip(): Promise<void> {
  saving.value = true;
  try {
    if (createdRuleId.value) await rules.remove(createdRuleId.value);
    await settings.save({ onboardingComplete: true });
    await router.replace("/dashboard");
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <main class="onboarding-shell">
    <header class="onboarding-top">
      <div class="brand">
        <span class="brand-mark" aria-hidden="true">A</span><span><strong>Auto File</strong><small>Organizer</small></span>
      </div>
      <span class="offline-pill">100% Offline</span>
    </header>
    <section class="onboarding-card">
      <div class="onboarding-intro">
        <h1>{{ t("onboardingTitle") }}</h1>
        <p>{{ t("onboardingSubtitle") }}</p>
        <div class="safety-points">
          <span><i>✓</i>Tidak ada file diubah tanpa konfirmasi</span><span><i>✓</i>Konflik nama ditangani aman</span><span><i>✓</i>Aktivasi hanya dilakukan setelah preview</span>
        </div>
      </div>

      <div class="onboarding-form">
        <div class="setup-progress" role="region" aria-label="Langkah konfigurasi">
          <span class="done">Konfigurasi</span><span class="step-divider" aria-hidden="true">/</span>
          <span :class="{ done: previewReady }">Preview</span><span class="step-divider" aria-hidden="true">/</span>
          <span>Aktifkan</span>
        </div>
        <fieldset class="preset-grid" :disabled="configurationLocked">
          <legend><strong>{{ t("preset") }}</strong><span>Pilih jenis file yang ingin dirapikan.</span></legend>
          <label v-for="preset in presets" :key="preset.id" :class="{ selected: selectedPreset === preset.id }"><input v-model="selectedPreset" type="radio" name="preset" :value="preset.id"><span aria-hidden="true">{{ preset.icon }}</span><strong>{{ preset.name }}</strong><small>{{ preset.extensions.slice(0, 4).map((value) => `.${value}`).join(" · ") }}</small></label>
        </fieldset>
        <div class="onboarding-paths">
          <label><span class="field-label">{{ t("sourceFolders") }}</span><div class="path-control"><input v-model="source" class="text-input" :disabled="configurationLocked" placeholder="Contoh: C:\Users\Nama\Downloads"><button class="button button--secondary" type="button" :disabled="configurationLocked" @click="pick('source')">{{ t("browse") }}</button></div></label>
          <span class="onboarding-arrow" aria-hidden="true">→</span>
          <label><span class="field-label">{{ t("destinationFolder") }}</span><div class="path-control"><input v-model="destination" class="text-input" :disabled="configurationLocked" placeholder="Contoh: D:\Arsip\Dokumen"><button class="button button--secondary" type="button" :disabled="configurationLocked" @click="pick('destination')">{{ t("browse") }}</button></div></label>
        </div>
        <div v-if="previewReady" class="inline-alert inline-alert--success">
          <strong>Preview siap ditinjau</strong><span>Monitoring masih nonaktif. Tutup preview, lalu aktifkan jika rencana sudah sesuai.</span>
        </div>
        <div class="onboarding-actions">
          <button v-if="configurationLocked" class="button button--ghost" type="button" :disabled="saving" @click="editConfiguration">
            Ubah konfigurasi
          </button>
          <button v-else class="button button--ghost" type="button" :disabled="saving" @click="skip">
            {{ t("skipSetup") }}
          </button>
          <button v-if="previewReady" class="button button--primary button--large" type="button" :disabled="saving" @click="activate">
            <span v-if="saving" class="button-spinner" aria-hidden="true" />{{ t("activateMonitoring") }}
          </button>
          <button v-else class="button button--primary button--large" type="button" :disabled="!canSave || saving" @click="preparePreview">
            <span v-if="saving" class="button-spinner" aria-hidden="true" />{{ t("preview") }} Aturan
          </button>
        </div>
      </div>
    </section>
    <footer class="onboarding-footer">
      Tidak memerlukan koneksi internet · Data konfigurasi disimpan lokal
    </footer>
  </main>
</template>
