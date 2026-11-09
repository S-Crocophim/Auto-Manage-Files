import { ref } from "vue";
import { defineStore } from "pinia";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { desktopApi, IpcError } from "@/ipc/commands";
import { subscribeToAppEvents } from "@/ipc/events";
import type { AttentionDto, MonitoringHealth } from "@/ipc/generated-types";
import { useActivityStore } from "./activity";
import { usePreviewStore } from "./preview";
import { useRulesStore } from "./rules";
import { useSettingsStore } from "./settings";

const initialMonitoring = (): MonitoringHealth => ({
  state: "starting",
  watchedSourceCount: 0,
  queueDepth: 0,
  activeJobs: 0,
  lastError: null,
});

let unlisteners: UnlistenFn[] = [];

export const useAppStore = defineStore("app", () => {
  const ready = ref(false);
  const loading = ref(false);
  const error = ref("");
  const monitoring = ref<MonitoringHealth>(initialMonitoring());
  const attentions = ref<AttentionDto[]>([]);

  async function initialize(): Promise<void> {
    loading.value = true;
    error.value = "";
    try {
      if (unlisteners.length === 0) {
        unlisteners = await subscribeToAppEvents({
          activity: (payload) => useActivityStore().upsert(payload),
          batchProgress: (payload) => usePreviewStore().applyProgress(payload),
          monitoring: (payload) => { monitoring.value = payload; },
          attention: (payload) => {
            attentions.value = [payload, ...attentions.value.filter((item) => item.code !== payload.code)].slice(0, 8);
          },
        });
      }

      const bootstrap = await desktopApi.getBootstrap();
      monitoring.value = bootstrap.monitoring;
      useRulesStore().hydrate(bootstrap.rules);
      useActivityStore().hydrate(bootstrap.recentActivity);
      useSettingsStore().hydrate(bootstrap.settings);
      ready.value = true;
    } catch (cause) {
      error.value = cause instanceof IpcError ? cause.message : String(cause);
      throw cause;
    } finally {
      loading.value = false;
    }
  }

  async function setMonitoring(enabled: boolean): Promise<void> {
    monitoring.value = await desktopApi.setMonitoring(enabled);
  }

  async function shutdown(): Promise<void> {
    await desktopApi.shutdown();
  }

  function dispose(): void {
    for (const unlisten of unlisteners) unlisten();
    unlisteners = [];
  }

  return { ready, loading, error, monitoring, attentions, initialize, setMonitoring, shutdown, dispose };
});
