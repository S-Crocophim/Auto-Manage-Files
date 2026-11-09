<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { storeToRefs } from "pinia";
import { useI18n } from "@/i18n";
import type { ActivityDto, ListActivityRequest } from "@/ipc/generated-types";
import { useActivityStore } from "@/stores/activity";
import { useRulesStore } from "@/stores/rules";
import { useUiStore } from "@/stores/ui";

const activity = useActivityStore();
const rules = useRulesStore();
const ui = useUiStore();
const { t, language } = useI18n();
const { items, loading, nextCursor } = storeToRefs(activity);
const { sorted } = storeToRefs(rules);
const filters = reactive({ query: "", status: "", ruleId: "" });
const loadError = ref<string | null>(null);

function request(cursor?: string): ListActivityRequest {
  const value: ListActivityRequest = { limit: 50 };
  if (cursor) value.cursor = cursor;
  if (filters.query.trim()) value.query = filters.query.trim();
  if (filters.status) value.status = filters.status;
  if (filters.ruleId) value.ruleId = filters.ruleId;
  return value;
}

async function load(append = false): Promise<void> {
  loadError.value = null;
  try {
    await activity.load(request(append ? nextCursor.value ?? undefined : undefined), append);
  } catch (cause) {

    loadError.value = String(cause);
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  }
}

async function undo(item: ActivityDto): Promise<void> {
  if (!window.confirm(t("undoConfirm"))) return;
  try {
    await activity.undo(item.id);
    ui.notify(t("undoSuccess"), "success");
  } catch (cause) {

    loadError.value = String(cause);
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  }
}

function fileName(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).at(-1) ?? path;
}

function formatDate(value: string | null): string {
  if (!value) return "-";
  return new Intl.DateTimeFormat(language.value === "en" ? "en-US" : "id-ID", { dateStyle: "medium", timeStyle: "medium" }).format(new Date(value));
}

function statusTone(status: string): string {
  const value = status.toLowerCase();
  if (["completed", "undone", "success"].includes(value)) return "completed";
  if (["failed", "error"].includes(value)) return "failed";
  return "pending";
}

onMounted(() => void load());
</script>

<template>
  <section class="page-stack">
    <div class="page-heading">
      <div><h2>{{ t("activityTitle") }}</h2><p>{{ t("activitySubtitle") }}</p></div>
    </div>

    <form class="toolbar toolbar--filters panel" @submit.prevent="load()">
      <label class="search-field"><span aria-hidden="true">⌕</span><input v-model="filters.query" type="search" :placeholder="t('searchActivity')" aria-label="Cari aktivitas"></label>
      <select v-model="filters.status" class="select-input" aria-label="Filter status">
        <option value="">
          {{ t("allStatuses") }}
        </option><option value="completed">
          Completed
        </option><option value="failed">
          Failed
        </option><option value="pending">
          Pending
        </option><option value="undone">
          Undone
        </option>
      </select>
      <select v-model="filters.ruleId" class="select-input" aria-label="Filter aturan">
        <option value="">
          {{ t("allRules") }}
        </option><option v-for="rule in sorted" :key="rule.id" :value="rule.id">
          {{ rule.name }}
        </option>
      </select>
      <button class="button button--secondary" type="submit" :disabled="loading">
        {{ t("refresh") }}
      </button>
    </form>

    <section class="panel table-panel">
      <div v-if="loading && !items.length" class="table-loading" role="status">
        <div class="spinner" aria-hidden="true" /><span>{{ t("loading") }}</span>
      </div>
      <div v-else-if="items.length" class="data-table-wrap">
        <table class="data-table activity-table">
          <thead><tr><th>{{ t("status") }}</th><th>File & operasi</th><th>Aturan</th><th>{{ t("trigger") }}</th><th>{{ t("detectedAt") }}</th><th><span class="sr-only">Aksi</span></th></tr></thead>
          <tbody>
            <tr v-for="item in items" :key="item.id">
              <td><span class="status-badge" :class="`status-badge--${statusTone(item.status)}`"><i />{{ item.status }}</span></td>
              <td class="activity-file-cell">
                <strong>{{ fileName(item.sourcePath) }}</strong><span class="path-text" :title="item.sourcePath">{{ item.sourcePath }}</span><span class="path-move" :title="item.destinationPath">→ {{ item.destinationPath }}</span><small v-if="item.errorMessage" class="error-text">{{ item.errorMessage }}</small>
              </td>
              <td>{{ item.ruleName ?? "-" }}</td>
              <td><span class="tag tag--plain">{{ item.trigger }}</span></td>
              <td><time :datetime="item.completedAt ?? item.detectedAt">{{ formatDate(item.completedAt ?? item.detectedAt) }}</time></td>
              <td class="table-actions">
                <button v-if="item.canUndo" class="button button--ghost button--small" type="button" @click="undo(item)">
                  ↶ {{ t("undo") }}
                </button><span v-else-if="item.undoneAt" class="muted-text">Sudah di-undo</span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <div v-else-if="loadError" class="empty-panel empty-panel--error" role="alert">
        <span class="empty-icon" aria-hidden="true">!</span><h3>Gagal memuat riwayat aktivitas</h3><p>{{ loadError }}</p><button class="button button--secondary" type="button" @click="load()">
          {{ t("retry") }}
        </button>
      </div>
      <div v-else class="empty-panel">
        <span class="empty-icon" aria-hidden="true">◷</span><h3>{{ t("noMatchingActivity") }}</h3><p>Ubah filter atau jalankan organisasi untuk membuat riwayat baru.</p>
      </div>
      <footer v-if="nextCursor" class="table-footer">
        <button class="button button--secondary" type="button" :disabled="loading" @click="load(true)">
          {{ loading ? t("loading") : t("loadMore") }}
        </button>
      </footer>
    </section>
  </section>
</template>
