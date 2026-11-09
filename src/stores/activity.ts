import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { desktopApi } from "@/ipc/commands";
import type { ActivityDto, ListActivityRequest } from "@/ipc/generated-types";

export const useActivityStore = defineStore("activity", () => {
  const items = ref<ActivityDto[]>([]);
  const loading = ref(false);
  const nextCursor = ref<string | null>(null);

  const successfulToday = computed(() => {
    const today = new Date().toDateString();
    return items.value.filter(
      (item) => item.status.toLowerCase() === "completed"
        && new Date(item.completedAt ?? item.detectedAt).toDateString() === today,
    ).length;
  });

  const failedToday = computed(() => {
    const today = new Date().toDateString();
    return items.value.filter(
      (item) => item.status.toLowerCase() === "failed"
        && new Date(item.detectedAt).toDateString() === today,
    ).length;
  });

  function hydrate(activity: ActivityDto[]): void {
    items.value = [...activity];
  }

  function upsert(activity: ActivityDto): void {
    const index = items.value.findIndex((item) => item.id === activity.id);
    if (index >= 0) items.value.splice(index, 1, activity);
    else items.value.unshift(activity);
  }

  async function load(request: ListActivityRequest = {}, append = false): Promise<void> {
    loading.value = true;
    try {
      const response = await desktopApi.listActivity(request);
      items.value = append ? [...items.value, ...response.items] : response.items;
      nextCursor.value = response.nextCursor;
    } finally {
      loading.value = false;
    }
  }

  async function undo(operationId: string): Promise<ActivityDto> {
    const result = await desktopApi.executeUndo(operationId, "autoRename");
    upsert(result);
    return result;
  }

  return {
    items,
    loading,
    nextCursor,
    successfulToday,
    failedToday,
    hydrate,
    upsert,
    load,
    undo,
  };
});
