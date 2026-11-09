<script setup lang="ts">
import { computed, nextTick, reactive, ref, watch } from "vue";
import { desktopApi } from "@/ipc/commands";
import type { ConflictPolicy, RuleDto, UpsertRuleRequest } from "@/ipc/generated-types";
import { useI18n } from "@/i18n";
import { usePreviewStore } from "@/stores/preview";
import { useRulesStore } from "@/stores/rules";
import { useUiStore } from "@/stores/ui";
import { MEBIBYTE, normalizeExtensions, splitList, validateRule, type RuleIssue } from "@/utils/rules";

interface RuleDraft {
  name: string;
  enabled: boolean;
  sources: string[];
  destination: string;
  extensions: string;
  globs: string;
  minSizeMb: string;
  maxSizeMb: string;
  conflictPolicy: ConflictPolicy;
  recursive: boolean;
  notifications: boolean;
}

const props = withDefaults(defineProps<{
  open: boolean;
  rule: RuleDto | null;
  duplicate?: boolean;
}>(), { duplicate: false });
const emit = defineEmits<{
  close: [];
  saved: [rule: RuleDto];
}>();

const rules = useRulesStore();
const preview = usePreviewStore();
const ui = useUiStore();
const { t } = useI18n();
const saving = ref(false);
const nameInput = ref<HTMLInputElement | null>(null);
const issues = ref<RuleIssue[]>([]);
const draft = reactive<RuleDraft>({
  name: "",
  enabled: true,
  sources: [""],
  destination: "",
  extensions: "",
  globs: "",
  minSizeMb: "",
  maxSizeMb: "",
  conflictPolicy: "autoRename",
  recursive: false,
  notifications: true,
});

const title = computed(() => props.rule && !props.duplicate ? t("ruleEditorEdit") : t("ruleEditorNew"));

function reset(): void {
  const rule = props.rule;
  draft.name = rule ? `${rule.name}${props.duplicate ? " (Salinan)" : ""}` : "";
  draft.enabled = rule?.enabled ?? true;
  draft.sources = rule?.sources.map((source) => source.path) ?? [""];
  draft.destination = rule?.destination ?? "";
  draft.extensions = rule?.extensions.join(", ") ?? "";
  draft.globs = rule?.nameGlobs.join(", ") ?? "";
  draft.minSizeMb = rule?.minSizeBytes === null || rule?.minSizeBytes === undefined ? "" : String(rule.minSizeBytes / MEBIBYTE);
  draft.maxSizeMb = rule?.maxSizeBytes === null || rule?.maxSizeBytes === undefined ? "" : String(rule.maxSizeBytes / MEBIBYTE);
  draft.conflictPolicy = rule?.conflictPolicy ?? "autoRename";
  draft.recursive = rule?.recursive ?? false;
  draft.notifications = rule?.notifications ?? true;
  issues.value = [];
}

watch(() => [props.open, props.rule?.id, props.duplicate] as const, async ([isOpen]) => {
  if (!isOpen) return;
  reset();
  await nextTick();
  nameInput.value?.focus();
}, { immediate: true });

function fieldError(field: RuleIssue["field"]): string {
  const issue = issues.value.find((item) => item.field === field);
  return issue ? t(issue.code) : "";
}

function addSource(): void {
  draft.sources.push("");
}

function removeSource(index: number): void {
  if (draft.sources.length === 1) draft.sources[0] = "";
  else draft.sources.splice(index, 1);
}

async function pickSource(index: number): Promise<void> {
  try {
    const path = await desktopApi.pickFolder();
    if (path !== null) draft.sources[index] = path;
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  }
}

async function pickDestination(): Promise<void> {
  try {
    const path = await desktopApi.pickFolder();
    if (path !== null) draft.destination = path;
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  }
}

function buildRequest(): UpsertRuleRequest {
  const request: UpsertRuleRequest = {
    name: draft.name.trim(),
    enabled: draft.enabled,
    sources: draft.sources.map((source) => source.trim()).filter(Boolean),
    destination: draft.destination.trim(),
    extensions: normalizeExtensions(draft.extensions),
    nameGlobs: splitList(draft.globs),
    conflictPolicy: draft.conflictPolicy,
    recursive: draft.recursive,
    notifications: draft.notifications,
  };
  if (props.rule && !props.duplicate) {
    request.id = props.rule.id;
    request.priority = props.rule.priority;
  }
  const minimum = Number(draft.minSizeMb);
  const maximum = Number(draft.maxSizeMb);
  if (draft.minSizeMb.trim() && Number.isFinite(minimum) && minimum >= 0) request.minSizeBytes = Math.round(minimum * MEBIBYTE);
  if (draft.maxSizeMb.trim() && Number.isFinite(maximum) && maximum >= 0) request.maxSizeBytes = Math.round(maximum * MEBIBYTE);
  return request;
}

async function submit(openPreview: boolean): Promise<void> {
  const request = buildRequest();
  issues.value = validateRule(request);
  if (issues.value.length) return;
  saving.value = true;
  try {
    const saved = await rules.save(request);
    ui.notify(t("ruleSaved"), "success");
    emit("saved", saved);
    emit("close");
    if (openPreview) await preview.create([saved.id]);
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="drawer">
      <div v-if="open" class="drawer-backdrop" @click.self="!saving && emit('close')">
        <section class="drawer-panel" role="dialog" aria-modal="true" aria-labelledby="rule-editor-title" @keydown.esc="!saving && emit('close')">
          <header class="drawer-header">
            <div>
              <h2 id="rule-editor-title">
                {{ title }}
              </h2><p>Filter dijalankan berurutan berdasarkan prioritas.</p>
            </div>
            <button class="icon-button" type="button" :aria-label="t('close')" :disabled="saving" @click="emit('close')">
              ×
            </button>
          </header>

          <form id="rule-form" class="drawer-body" @submit.prevent="submit(false)">
            <section class="form-section">
              <div class="section-heading">
                <span>1</span><div><h3>Identitas aturan</h3><p>Beri nama yang mudah dikenali.</p></div>
              </div>
              <label class="field-label" for="rule-name">{{ t("ruleName") }}</label>
              <input id="rule-name" ref="nameInput" v-model="draft.name" class="text-input" :class="{ 'input-error': fieldError('name') }" :placeholder="t('ruleNamePlaceholder')" autocomplete="off">
              <span v-if="fieldError('name')" class="field-error">{{ fieldError("name") }}</span>
              <label class="switch-row switch-row--subtle"><span><strong>{{ t("enabled") }}</strong><small>Aturan aktif langsung dipantau setelah disimpan.</small></span><input v-model="draft.enabled" type="checkbox" role="switch"><i /></label>
            </section>

            <section class="form-section">
              <div class="section-heading">
                <span>2</span><div><h3>{{ t("sourceFolders") }}</h3><p>Anda dapat memantau beberapa folder dengan aturan yang sama.</p></div>
              </div>
              <div v-for="(_, index) in draft.sources" :key="index" class="path-control">
                <span class="path-control-icon" aria-hidden="true">↳</span>
                <input v-model="draft.sources[index]" class="text-input" :aria-label="`${t('source')} ${index + 1}`" placeholder="C:\Users\Nama\Downloads">
                <button class="button button--ghost button--small" type="button" @click="pickSource(index)">
                  {{ t("browse") }}
                </button>
                <button class="icon-button icon-button--danger" type="button" :aria-label="`${t('remove')} ${index + 1}`" @click="removeSource(index)">
                  ×
                </button>
              </div>
              <span v-if="fieldError('sources')" class="field-error">{{ fieldError("sources") }}</span>
              <button class="text-button" type="button" @click="addSource">
                + {{ t("addSource") }}
              </button>

              <label class="field-label" for="rule-destination">{{ t("destinationFolder") }}</label>
              <div class="path-control">
                <span class="path-control-icon" aria-hidden="true">⌂</span>
                <input id="rule-destination" v-model="draft.destination" class="text-input" :class="{ 'input-error': fieldError('destination') }" placeholder="D:\Arsip\Dokumen">
                <button class="button button--ghost button--small" type="button" @click="pickDestination">
                  {{ t("browse") }}
                </button>
              </div>
              <span v-if="fieldError('destination')" class="field-error">{{ fieldError("destination") }}</span>
            </section>

            <section class="form-section">
              <div class="section-heading">
                <span>3</span><div><h3>{{ t("filters") }}</h3><p>Ekstensi wajib; pola nama dan ukuran bersifat opsional.</p></div>
              </div>
              <label class="field-label" for="rule-extensions">{{ t("fileExtensions") }}</label>
              <input id="rule-extensions" v-model="draft.extensions" class="text-input" :class="{ 'input-error': fieldError('extensions') }" placeholder="pdf, docx, txt">
              <span class="field-help">{{ t("extensionHelp") }}</span>
              <span v-if="fieldError('extensions')" class="field-error">{{ fieldError("extensions") }}</span>

              <label class="field-label" for="rule-globs">{{ t("namePatterns") }}</label>
              <input id="rule-globs" v-model="draft.globs" class="text-input" placeholder="invoice-*.pdf, laporan-202?.xlsx">
              <span class="field-help">{{ t("patternHelp") }}</span>

              <div class="field-grid">
                <label><span class="field-label">{{ t("minSize") }}</span><input v-model="draft.minSizeMb" class="text-input" min="0" step="0.1" type="number" placeholder="0"></label>
                <label><span class="field-label">{{ t("maxSize") }}</span><input v-model="draft.maxSizeMb" class="text-input" min="0" step="0.1" type="number" placeholder="Tanpa batas"></label>
              </div>
              <span v-if="fieldError('size')" class="field-error">{{ fieldError("size") }}</span>
              <label class="switch-row switch-row--subtle"><span><strong>{{ t("recursive") }}</strong><small>Pindai semua folder di dalam sumber.</small></span><input v-model="draft.recursive" type="checkbox" role="switch"><i /></label>
            </section>

            <section class="form-section">
              <div class="section-heading">
                <span>4</span><div><h3>Keamanan operasi</h3><p>Tentukan perilaku saat file tujuan sudah ada.</p></div>
              </div>
              <fieldset class="radio-cards">
                <legend class="field-label">
                  {{ t("conflictPolicy") }}
                </legend>
                <label><input v-model="draft.conflictPolicy" type="radio" value="autoRename"><span><strong>{{ t("autoRename") }}</strong><small>Tambahkan nomor unik; pilihan paling aman.</small></span></label>
                <label><input v-model="draft.conflictPolicy" type="radio" value="skip"><span><strong>{{ t("skip") }}</strong><small>Biarkan file tetap di folder sumber.</small></span></label>
                <label><input v-model="draft.conflictPolicy" type="radio" value="safeOverwrite"><span><strong>{{ t("safeOverwrite") }}</strong><small>Simpan file lama di recovery sebelum menimpa.</small></span></label>
              </fieldset>
              <label class="switch-row switch-row--subtle"><span><strong>{{ t("ruleNotifications") }}</strong><small>Tampilkan hasil penting setelah aturan berjalan.</small></span><input v-model="draft.notifications" type="checkbox" role="switch"><i /></label>
            </section>
          </form>

          <footer class="drawer-footer">
            <button class="button button--secondary" type="button" :disabled="saving" @click="emit('close')">
              {{ t("cancel") }}
            </button>
            <button class="button button--secondary" type="submit" form="rule-form" :disabled="saving">
              {{ t("save") }}
            </button>
            <button class="button button--primary" type="button" :disabled="saving" @click="submit(true)">
              <span v-if="saving" class="button-spinner" aria-hidden="true" />{{ t("saveAndPreview") }}
            </button>
          </footer>
        </section>
      </div>
    </Transition>
  </Teleport>
</template>
