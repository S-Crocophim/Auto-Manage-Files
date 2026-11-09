import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { desktopApi } from "@/ipc/commands";
import type { BatchSummaryDto, PreviewResultDto } from "@/ipc/generated-types";

export const usePreviewStore = defineStore("preview", () => {
  const open = ref(false);
  const loading = ref(false);
  const executing = ref(false);
  const error = ref<string | null>(null);
  const result = ref<PreviewResultDto>();
  const batch = ref<BatchSummaryDto>();
  const selectedIds = ref<string[]>([]);

  const selectedCount = computed(() => selectedIds.value.length);
  const selectedSize = computed(() =>
    (result.value?.items ?? [])
      .filter((item) => selectedIds.value.includes(item.id))
      .reduce((total, item) => total + item.sizeBytes, 0),
  );

  async function create(ruleIds?: string[]): Promise<void> {
    open.value = true;
    loading.value = true;
    executing.value = false;
    error.value = null;
    result.value = undefined;
    batch.value = undefined;
    try {
      result.value = await desktopApi.previewRules(ruleIds);
      selectedIds.value = result.value.items
        .filter((item) => item.status.toLowerCase() !== "blocked")
        .map((item) => item.id);
    } catch (cause) {
      error.value = String(cause);
      throw cause;
    } finally {
      loading.value = false;
    }
  }

  async function execute(): Promise<void> {
    if (!result.value || selectedIds.value.length === 0) return;
    executing.value = true;
    try {
      batch.value = await desktopApi.executePlan(result.value.planId, selectedIds.value);
    } finally {
      executing.value = false;
    }
  }

  function applyProgress(progress: BatchSummaryDto): void {
    if (!batch.value || batch.value.batchId === progress.batchId) batch.value = progress;
  }

  function close(): void {
    if (executing.value) return;
    open.value = false;
  }

  function toggleItem(id: string): void {
    selectedIds.value = selectedIds.value.includes(id)
      ? selectedIds.value.filter((value) => value !== id)
      : [...selectedIds.value, id];
  }

  return { open, loading, executing, error, result, batch, selectedIds, selectedCount, selectedSize, create, execute, applyProgress, close, toggleItem };
});
