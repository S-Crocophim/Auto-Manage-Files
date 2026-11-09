import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AppEventMap } from "./generated-types";

export function isTauriEnvironment(): boolean {
  return typeof window !== "undefined" && Boolean((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__);
}

export async function listenTyped<K extends keyof AppEventMap>(
  eventName: K,
  handler: (payload: AppEventMap[K]) => void,
): Promise<UnlistenFn> {
  if (!isTauriEnvironment()) {
    return () => {};
  }
  return listen<AppEventMap[K]>(eventName, (event) => handler(event.payload));
}

export interface AppEventHandlers {
  activity: (payload: AppEventMap["amf://activity"]) => void;
  batchProgress: (payload: AppEventMap["amf://batch-progress"]) => void;
  monitoring: (payload: AppEventMap["amf://monitoring-health"]) => void;
  attention: (payload: AppEventMap["amf://attention"]) => void;
}

export async function subscribeToAppEvents(
  handlers: AppEventHandlers,
): Promise<UnlistenFn[]> {
  return Promise.all([
    listenTyped("amf://activity", handlers.activity),
    listenTyped("amf://batch-progress", handlers.batchProgress),
    listenTyped("amf://monitoring-health", handlers.monitoring),
    listenTyped("amf://attention", handlers.attention),
  ]);
}
