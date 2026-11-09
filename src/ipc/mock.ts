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

const STORAGE_KEY = "amf_browser_dev_state_v1";

interface BrowserState {
  rules: RuleDto[];
  settings: SettingsDto;
  monitoring: MonitoringHealth;
  activity: ActivityDto[];
}

const defaultSettings: SettingsDto = {
  theme: "system",
  language: "id",
  minimizeToTrayOnClose: true,
  autoStart: false,
  notificationsEnabled: true,
  monitoringEnabled: true,
  onboardingComplete: false,
  historyRetentionDays: 30,
  stabilityWindowMs: 1500,
  previewBeforeManual: true,
};

const defaultMonitoring: MonitoringHealth = {
  state: "monitoring",
  watchedSourceCount: 0,
  queueDepth: 0,
  activeJobs: 0,
  lastError: null,
};

function loadState(): BrowserState {
  if (typeof window === "undefined" || !window.localStorage) {
    return {
      rules: [],
      settings: { ...defaultSettings },
      monitoring: { ...defaultMonitoring },
      activity: [],
    };
  }

  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw) as Partial<BrowserState>;
      return {
        rules: parsed.rules ?? [],
        settings: { ...defaultSettings, ...(parsed.settings ?? {}) },
        monitoring: { ...defaultMonitoring, ...(parsed.monitoring ?? {}) },
        activity: parsed.activity ?? [],
      };
    }
  } catch {
    // fallback if malformed
  }

  return {
    rules: [],
    settings: { ...defaultSettings },
    monitoring: { ...defaultMonitoring },
    activity: [],
  };
}

function saveState(state: BrowserState): void {
  if (typeof window === "undefined" || !window.localStorage) return;
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch {
    // storage quota or private mode
  }
}

export const browserMockApi = {
  async getBootstrap(): Promise<BootstrapDto> {
    const state = loadState();
    state.monitoring.watchedSourceCount = new Set(
      state.rules.filter((r) => r.enabled).flatMap((r) => r.sources.map((s) => s.path)),
    ).size;
    return {
      rules: state.rules,
      settings: state.settings,
      monitoring: state.monitoring,
      recentActivity: state.activity.slice(0, 10),
    };
  },

  async listRules(): Promise<RuleDto[]> {
    return loadState().rules;
  },

  async upsertRule(request: UpsertRuleRequest): Promise<RuleDto> {
    const state = loadState();
    const now = new Date().toISOString();
    const id = request.id ?? `rule-${Date.now()}`;
    const priority = request.priority ?? state.rules.length;

    const rule: RuleDto = {
      id,
      name: request.name,
      enabled: request.enabled,
      priority,
      sources: request.sources.map((path) => ({
        path,
        canonicalPath: path,
        available: true,
      })),
      destination: request.destination,
      extensions: request.extensions,
      nameGlobs: request.nameGlobs,
      minSizeBytes: request.minSizeBytes ?? null,
      maxSizeBytes: request.maxSizeBytes ?? null,
      conflictPolicy: request.conflictPolicy,
      recursive: request.recursive,
      notifications: request.notifications,
      createdAt: now,
      updatedAt: now,
    };

    const existingIndex = state.rules.findIndex((r) => r.id === id);
    if (existingIndex >= 0) {
      state.rules[existingIndex] = { ...state.rules[existingIndex], ...rule, updatedAt: now };
    } else {
      state.rules.push(rule);
    }

    saveState(state);
    return rule;
  },

  async deleteRule(id: string): Promise<void> {
    const state = loadState();
    state.rules = state.rules.filter((r) => r.id !== id);
    saveState(state);
  },

  async setRuleEnabled(id: string, enabled: boolean): Promise<RuleDto> {
    const state = loadState();
    const rule = state.rules.find((r) => r.id === id);
    if (!rule) throw new Error(`Rule ${id} not found`);
    rule.enabled = enabled;
    rule.updatedAt = new Date().toISOString();
    saveState(state);
    return rule;
  },

  async reorderRules(orderedIds: string[]): Promise<RuleDto[]> {
    const state = loadState();
    const ruleMap = new Map(state.rules.map((r) => [r.id, r]));
    state.rules = orderedIds
      .map((id, index) => {
        const rule = ruleMap.get(id);
        if (rule) {
          rule.priority = index;
          return rule;
        }
        return null;
      })
      .filter((r): r is RuleDto => r !== null);
    saveState(state);
    return state.rules;
  },

  async previewRules(ruleIds?: string[], limit = 500): Promise<PreviewResultDto> {
    const state = loadState();
    const targetRules = ruleIds?.length
      ? state.rules.filter((r) => ruleIds.includes(r.id))
      : state.rules.filter((r) => r.enabled);

    const items = targetRules.slice(0, limit).map((rule, index) => ({
      id: `item-${index + 1}`,
      ruleId: rule.id,
      ruleName: rule.name,
      sourcePath: `${rule.sources[0]?.path ?? "C:\\Downloads"}\\file_${index + 1}.${rule.extensions[0] ?? "pdf"}`,
      destinationPath: `${rule.destination}\\file_${index + 1}.${rule.extensions[0] ?? "pdf"}`,
      sizeBytes: 1024 * 1024 * (index + 2),
      conflict: rule.conflictPolicy,
      status: "Planned",
    }));

    return {
      planId: `plan-${Date.now()}`,
      totalCount: items.length,
      truncated: false,
      warnings: [],
      items,
    };
  },

  async executePlan(planId: string, itemIds?: string[]): Promise<BatchSummaryDto> {
    const state = loadState();
    const count = itemIds?.length ?? 2;
    const now = new Date().toISOString();

    for (let i = 0; i < count; i++) {
      const item: ActivityDto = {
        id: `act-${Date.now()}-${i}`,
        batchId: planId,
        ruleId: state.rules[0]?.id ?? null,
        ruleName: state.rules[0]?.name ?? "Aturan manual",
        trigger: "manual",
        sourcePath: `C:\\Downloads\\dokumen_${i + 1}.pdf`,
        destinationPath: `D:\\Arsip\\dokumen_${i + 1}.pdf`,
        recoveryPath: null,
        status: "completed",
        errorCode: null,
        errorMessage: null,
        detectedAt: now,
        completedAt: now,
        undoneAt: null,
        canUndo: true,
      };
      state.activity.unshift(item);
    }

    saveState(state);
    return {
      batchId: planId,
      totalCount: count,
      completedCount: count,
      failedCount: 0,
      status: "completed",
    };
  },

  async listActivity(request: ListActivityRequest = {}): Promise<ActivityPageDto> {
    const state = loadState();
    let filtered = [...state.activity];

    if (request.query) {
      const q = request.query.toLowerCase();
      filtered = filtered.filter(
        (a) => a.sourcePath.toLowerCase().includes(q) || a.destinationPath.toLowerCase().includes(q),
      );
    }
    if (request.status) {
      filtered = filtered.filter((a) => a.status.toLowerCase() === request.status?.toLowerCase());
    }
    if (request.ruleId) {
      filtered = filtered.filter((a) => a.ruleId === request.ruleId);
    }

    const limit = request.limit ?? 50;
    return {
      items: filtered.slice(0, limit),
      nextCursor: null,
    };
  },

  async executeUndo(operationId: string, conflictPolicy?: ConflictPolicy): Promise<ActivityDto> {
    void conflictPolicy;
    const state = loadState();
    const act = state.activity.find((a) => a.id === operationId);
    if (!act) throw new Error("Aktivitas tidak ditemukan");
    act.undoneAt = new Date().toISOString();
    act.status = "undone";
    act.canUndo = false;
    saveState(state);
    return act;
  },

  async setMonitoring(enabled: boolean): Promise<MonitoringHealth> {
    const state = loadState();
    state.monitoring.state = enabled ? "monitoring" : "paused";
    saveState(state);
    return state.monitoring;
  },

  async updateSettings(settings: Partial<SettingsDto>): Promise<SettingsDto> {
    const state = loadState();
    state.settings = { ...state.settings, ...settings };
    saveState(state);
    return state.settings;
  },

  async pickFolder(): Promise<string | null> {
    if (typeof window !== "undefined") {
      const result = window.prompt("Pilih folder (Mode Browser):", "C:\\Users\\User\\Downloads");
      return result ? result.trim() : null;
    }
    return "C:\\Users\\User\\Downloads";
  },

  async exportConfig(path: string): Promise<void> {
    void path;
  },

  async importConfig(path: string): Promise<BootstrapDto> {
    void path;
    return this.getBootstrap();
  },

  async shutdown(): Promise<void> {
    if (typeof window !== "undefined") {
      window.alert("Aplikasi ditutup (Mode Browser)");
    }
  },
};
