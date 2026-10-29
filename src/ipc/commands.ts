import { invoke } from "@tauri-apps/api/core";
import type {
  ActivityDto,
  ActivityPageDto,
  BatchSummaryDto,
  BootstrapDto,
  ConflictPolicy,
  ListActivityRequest,
  MonitoringHealth,
  PreviewResultDto,
  RuleDto,
  SettingsDto,
  UpsertRuleRequest,
} from "./generated-types";

export class IpcError extends Error {
  readonly code: string;
  readonly retryable: boolean;

  constructor(message: string, code = "IPC_ERROR", retryable = false) {
    super(message);
    this.name = "IpcError";
    this.code = code;
    this.retryable = retryable;
  }
}

function normalizeError(error: unknown): IpcError {
  if (error instanceof IpcError) return error;
  if (typeof error === "string") return new IpcError(error);
  if (error && typeof error === "object") {
    const value = error as Record<string, unknown>;
    const message = value.safeMessage ?? value.message ?? value.error;
    return new IpcError(
      typeof message === "string" ? message : "Operasi desktop gagal.",
      typeof value.code === "string" ? value.code : "IPC_ERROR",
      value.retryable === true,
    );
  }
  return new IpcError("Operasi desktop gagal.");
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw normalizeError(error);
  }
}

export const desktopApi = {
  getBootstrap(): Promise<BootstrapDto> {
    return call("get_bootstrap");
  },

  listRules(): Promise<RuleDto[]> {
    return call("list_rules");
  },

  upsertRule(request: UpsertRuleRequest): Promise<RuleDto> {
    return call("upsert_rule", { request });
  },

  deleteRule(id: string): Promise<void> {
    return call("delete_rule", { request: { id } });
  },

  setRuleEnabled(id: string, enabled: boolean): Promise<RuleDto> {
    return call("set_rule_enabled", { request: { id, enabled } });
  },

  reorderRules(orderedIds: string[]): Promise<RuleDto[]> {
    return call("reorder_rules", { request: { orderedIds } });
  },

  previewRules(ruleIds?: string[], limit = 500): Promise<PreviewResultDto> {
    const request: { ruleIds?: string[]; limit: number } = { limit };
    if (ruleIds?.length) request.ruleIds = ruleIds;
    return call("preview_rules", { request });
  },

  executePlan(planId: string, itemIds?: string[]): Promise<BatchSummaryDto> {
    const request: { planId: string; itemIds?: string[] } = { planId };
    if (itemIds?.length) request.itemIds = itemIds;
    return call("execute_plan", { request });
  },

  listActivity(request: ListActivityRequest = {}): Promise<ActivityPageDto> {
    return call("list_activity", { request });
  },

  executeUndo(
    operationId: string,
    conflictPolicy?: ConflictPolicy,
  ): Promise<ActivityDto> {
    const request: { operationId: string; conflictPolicy?: ConflictPolicy } = {
      operationId,
    };
    if (conflictPolicy) request.conflictPolicy = conflictPolicy;
    return call("execute_undo", { request });
  },

  setMonitoring(enabled: boolean): Promise<MonitoringHealth> {
    return call("set_monitoring", { request: { enabled } });
  },

  updateSettings(settings: Partial<SettingsDto>): Promise<SettingsDto> {
    return call("update_settings", { request: { settings } });
  },

  pickFolder(): Promise<string | null> {
    return call("pick_folder");
  },

  exportConfig(path: string): Promise<void> {
    return call("export_config", { request: { path } });
  },

  importConfig(path: string): Promise<BootstrapDto> {
    return call("import_config", { request: { path } });
  },

  shutdown(): Promise<void> {
    return call("shutdown_app");
  },
};

export type DesktopApi = typeof desktopApi;
