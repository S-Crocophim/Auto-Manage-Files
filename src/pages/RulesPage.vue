<script setup lang="ts">
import { computed, ref } from "vue";
import { storeToRefs } from "pinia";
import RuleEditor from "@/components/RuleEditor.vue";
import { useI18n } from "@/i18n";
import type { RuleDto } from "@/ipc/generated-types";
import { usePreviewStore } from "@/stores/preview";
import { useRulesStore } from "@/stores/rules";
import { useUiStore } from "@/stores/ui";
import { policyLabelKey } from "@/utils/rules";

const rules = useRulesStore();
const preview = usePreviewStore();
const ui = useUiStore();
const { t } = useI18n();
const { sorted, mutatingIds } = storeToRefs(rules);
const query = ref("");
const editorOpen = ref(false);
const editingRule = ref<RuleDto | null>(null);
const duplicateMode = ref(false);

const filtered = computed(() => {
  const needle = query.value.trim().toLowerCase();
  if (!needle) return sorted.value;
  return sorted.value.filter((rule) => [
    rule.name,
    rule.destination,
    ...rule.extensions,
    ...rule.sources.map((source) => source.path),
  ].some((value) => value.toLowerCase().includes(needle)));
});

function openNew(): void {
  editingRule.value = null;
  duplicateMode.value = false;
  editorOpen.value = true;
}

function openEdit(rule: RuleDto, duplicate = false): void {
  editingRule.value = rule;
  duplicateMode.value = duplicate;
  editorOpen.value = true;
}

async function toggle(rule: RuleDto, event: Event): Promise<void> {
  const enabled = (event.target as HTMLInputElement).checked;
  try {
    await rules.toggle(rule.id, enabled);
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  }
}

async function remove(rule: RuleDto): Promise<void> {
  if (!window.confirm(`Hapus aturan “${rule.name}”? Riwayat operasi tidak ikut dihapus.`)) return;
  try {
    await rules.remove(rule.id);
    ui.notify(t("ruleDeleted"), "success");
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  }
}

async function previewAll(): Promise<void> {
  try {
    await preview.create();
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  }
}

async function showPreview(rule: RuleDto): Promise<void> {
  try {
    await preview.create([rule.id]);
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  }
}

async function move(rule: RuleDto, direction: -1 | 1): Promise<void> {
  try {
    await rules.move(rule.id, direction);
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  }
}
</script>

<template>
  <section class="page-stack">
    <div class="page-heading page-heading--actions">
      <div><h2>{{ t("rulesTitle") }}</h2><p>{{ t("rulesSubtitle") }}</p></div>
      <div class="heading-actions">
        <button class="button button--secondary" type="button" :disabled="!sorted.length" @click="previewAll">
          {{ t("previewAll") }}
        </button><button class="button button--primary" type="button" @click="openNew">
          + {{ t("newRule") }}
        </button>
      </div>
    </div>

    <div v-if="sorted.length" class="toolbar panel">
      <label class="search-field"><span aria-hidden="true">⌕</span><input v-model="query" type="search" :placeholder="t('searchRules')" aria-label="Cari aturan"></label>
      <span class="toolbar-count">{{ filtered.length }} dari {{ sorted.length }} aturan</span>
    </div>

    <div v-if="filtered.length" class="rule-list">
      <article v-for="(rule, index) in filtered" :key="rule.id" class="rule-card" :class="{ 'rule-card--disabled': !rule.enabled }">
        <div class="rule-order" :aria-label="`Prioritas ${rule.priority + 1}`">
          {{ rule.priority + 1 }}
        </div>
        <div class="rule-card-main">
          <header class="rule-card-header">
            <div><h3>{{ rule.name }}</h3><span class="status-badge" :class="rule.enabled ? 'status-badge--completed' : 'status-badge--muted'">{{ rule.enabled ? t("enabled") : t("disabled") }}</span></div>
            <label class="switch-control" :aria-label="`${rule.enabled ? t('enabled') : t('disabled')}: ${rule.name}`"><input type="checkbox" role="switch" :checked="rule.enabled" :disabled="mutatingIds.includes(rule.id)" @change="toggle(rule, $event)"><i /></label>
          </header>
          <div class="rule-flow">
            <div><span class="flow-label">{{ t("source") }} · {{ rule.sources.length }}</span><strong v-for="source in rule.sources.slice(0, 2)" :key="source.path" :title="source.path"><i :class="{ 'source-unavailable': !source.available }" />{{ source.path }}</strong><small v-if="rule.sources.length > 2">+{{ rule.sources.length - 2 }} folder lainnya</small></div>
            <span class="flow-arrow" aria-hidden="true">→</span>
            <div><span class="flow-label">{{ t("destination") }}</span><strong :title="rule.destination">⌂ {{ rule.destination }}</strong></div>
          </div>
          <div class="rule-meta">
            <span v-for="extension in rule.extensions.slice(0, 5)" :key="extension" class="tag">{{ extension }}</span><span v-if="rule.extensions.length > 5" class="tag">+{{ rule.extensions.length - 5 }}</span><span class="meta-separator" /><span>{{ t(policyLabelKey(rule.conflictPolicy)) }}</span><span v-if="rule.recursive">· Subfolder</span>
          </div>
        </div>
        <div class="rule-actions">
          <button class="icon-button" type="button" :disabled="index === 0 || query.length > 0" :aria-label="t('moveUp')" @click="move(rule, -1)">
            ↑
          </button>
          <button class="icon-button" type="button" :disabled="index === filtered.length - 1 || query.length > 0" :aria-label="t('moveDown')" @click="move(rule, 1)">
            ↓
          </button>
          <button class="button button--ghost button--small" type="button" @click="showPreview(rule)">
            {{ t("preview") }}
          </button>
          <button class="button button--ghost button--small" type="button" @click="openEdit(rule)">
            {{ t("edit") }}
          </button>
          <button class="icon-button" type="button" :aria-label="`${t('duplicate')} ${rule.name}`" @click="openEdit(rule, true)">
            ⧉
          </button>
          <button class="icon-button icon-button--danger" type="button" :disabled="mutatingIds.includes(rule.id)" :aria-label="`${t('delete')} ${rule.name}`" @click="remove(rule)">
            ×
          </button>
        </div>
      </article>
    </div>

    <div v-else-if="sorted.length" class="empty-panel">
      <span class="empty-icon" aria-hidden="true">⌕</span><h3>Tidak ada aturan yang cocok</h3><p>Coba kata kunci lain atau kosongkan pencarian.</p><button class="button button--secondary" type="button" @click="query = ''">
        Bersihkan pencarian
      </button>
    </div>
    <div v-else class="empty-panel empty-panel--large">
      <div class="empty-illustration" aria-hidden="true">
        <span>⌂</span><i>→</i><span>✓</span>
      </div><h3>{{ t("noRulesTitle") }}</h3><p>{{ t("noRulesBody") }}</p><button class="button button--primary" type="button" @click="openNew">
        + {{ t("createFirstRule") }}
      </button>
    </div>

    <RuleEditor :open="editorOpen" :rule="editingRule" :duplicate="duplicateMode" @close="editorOpen = false" />
  </section>
</template>
