<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { useI18n } from "@/i18n";
import { usePreviewStore } from "@/stores/preview";
import { useUiStore } from "@/stores/ui";
import { formatBytes, policyLabelKey } from "@/utils/rules";

const preview = usePreviewStore();
const ui = useUiStore();
const { t } = useI18n();
const { open, loading, executing, result, batch, selectedIds, selectedCount, selectedSize } = storeToRefs(preview);
const dialog = ref<HTMLElement | null>(null);
const executableItems = computed(() => result.value?.items.filter((item) => item.status.toLowerCase() !== "blocked") ?? []);
const allSelected = computed(() => executableItems.value.length > 0 && executableItems.value.every((item) => selectedIds.value.includes(item.id)));
const progress = computed(() => !batch.value?.totalCount ? 0 : Math.round(((batch.value.completedCount + batch.value.failedCount) / batch.value.totalCount) * 100));
const completed = computed(() => batch.value?.status.toLowerCase() === "completed");

watch(open, async (isOpen) => { if (isOpen) { await nextTick(); dialog.value?.focus(); } });

function fileName(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).at(-1) ?? path;
}

function toggleAll(): void {
  selectedIds.value = allSelected.value ? [] : executableItems.value.map((item) => item.id);
}

async function execute(): Promise<void> {
  try {
    await preview.execute();
    const tone = batch.value?.failedCount ? "warning" : "success";
    ui.notify(t("batchComplete", { done: batch.value?.completedCount ?? 0, failed: batch.value?.failedCount ?? 0 }), tone);
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="open" class="modal-backdrop" @click.self="preview.close">
        <section ref="dialog" class="modal-card preview-modal" role="dialog" aria-modal="true" aria-labelledby="preview-title" tabindex="-1" @keydown.esc="preview.close">
          <header class="modal-header">
            <div>
              <span class="eyebrow">SAFE MODE</span><h2 id="preview-title">
                {{ t("previewTitle") }}
              </h2><p>{{ t("previewSubtitle") }}</p>
            </div>
            <button class="icon-button" type="button" :aria-label="t('close')" :disabled="executing" @click="preview.close">
              ×
            </button>
          </header>

          <div v-if="loading" class="modal-loading" role="status">
            <div class="spinner" aria-hidden="true" /><strong>{{ t("scanning") }}</strong><span>Memeriksa aturan dan konflik tanpa mengubah file.</span>
          </div>

          <template v-else-if="result">
            <div class="preview-summary">
              <div><span>{{ t("totalFiles") }}</span><strong>{{ result.totalCount }}</strong></div>
              <div><span>{{ t("selectedFiles", { count: selectedCount }) }}</span><strong>{{ formatBytes(selectedSize) }}</strong></div>
              <div><span>{{ t("conflict") }}</span><strong>{{ result.items.filter((item) => item.conflict !== "autoRename").length }}</strong></div>
            </div>

            <div v-if="result.warnings.length || result.truncated" class="inline-alert inline-alert--warning" role="status">
              <strong>Catatan preview</strong><span v-if="result.truncated">{{ t("previewTruncated") }}</span><span v-for="warning in result.warnings" :key="warning">{{ warning }}</span>
            </div>

            <div v-if="result.items.length" class="preview-table-wrap">
              <table class="data-table preview-table">
                <thead>
                  <tr>
                    <th class="check-cell">
                      <input type="checkbox" :checked="allSelected" aria-label="Pilih semua file" @change="toggleAll">
                    </th><th>File</th><th>{{ t("destination") }}</th><th>{{ t("conflict") }}</th><th>{{ t("status") }}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="item in result.items" :key="item.id" :class="{ 'row-disabled': item.status.toLowerCase() === 'blocked' }">
                    <td class="check-cell">
                      <input type="checkbox" :checked="selectedIds.includes(item.id)" :disabled="item.status.toLowerCase() === 'blocked' || executing" :aria-label="`Pilih ${fileName(item.sourcePath)}`" @change="preview.toggleItem(item.id)">
                    </td>
                    <td><strong class="file-name">{{ fileName(item.sourcePath) }}</strong><span class="path-text" :title="item.sourcePath">{{ item.sourcePath }}</span><small>{{ formatBytes(item.sizeBytes) }} · {{ item.ruleName }}</small></td>
                    <td><span class="path-text" :title="item.destinationPath">{{ item.destinationPath }}</span></td>
                    <td><span class="tag">{{ t(policyLabelKey(item.conflict)) }}</span></td>
                    <td><span class="status-badge" :class="`status-badge--${item.status.toLowerCase()}`">{{ item.status }}</span></td>
                  </tr>
                </tbody>
              </table>
            </div>
            <div v-else class="empty-panel empty-panel--compact">
              <span class="empty-icon" aria-hidden="true">✓</span><h3>{{ t("previewEmpty") }}</h3>
            </div>

            <div v-if="batch" class="batch-progress" aria-live="polite">
              <div><strong>{{ completed ? t("batchComplete", { done: batch.completedCount, failed: batch.failedCount }) : t("executing") }}</strong><span>{{ progress }}%</span></div>
              <div class="progress-track">
                <span :style="{ width: `${progress}%` }" />
              </div>
            </div>
          </template>

          <footer class="modal-footer">
            <button class="button button--secondary" type="button" :disabled="executing" @click="preview.close">
              {{ completed ? t("close") : t("cancel") }}
            </button>
            <button class="button button--primary" type="button" :disabled="loading || executing || selectedCount === 0 || completed" @click="execute">
              <span v-if="executing" class="button-spinner" aria-hidden="true" />{{ executing ? t("executing") : t("execute") }}
            </button>
          </footer>
        </section>
      </div>
    </Transition>
  </Teleport>
</template>
