export type Theme = "system" | "light" | "dark";
export type Language = "id" | "en";
export type ConflictPolicy = "autoRename" | "skip" | "safeOverwrite";
export type MonitoringState =
  | "starting"
  | "monitoring"
  | "paused"
  | "attention"
  | "stopped"
  | "error";

export interface RuleSourceDto {
  path: string;
  canonicalPath: string;
  available: boolean;
}

export interface RuleDto {
  id: string;
  name: string;
  enabled: boolean;
  priority: number;
  sources: RuleSourceDto[];
  destination: string;
  extensions: string[];
  nameGlobs: string[];
  minSizeBytes: number | null;
  maxSizeBytes: number | null;
  conflictPolicy: ConflictPolicy;
  recursive: boolean;
  notifications: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface UpsertRuleRequest {
  id?: string;
  name: string;
  enabled: boolean;
  priority?: number;
  sources: string[];
  destination: string;
  extensions: string[];
  nameGlobs: string[];
  minSizeBytes?: number;
  maxSizeBytes?: number;
  conflictPolicy: ConflictPolicy;
  recursive: boolean;
  notifications: boolean;
}

export interface SettingsDto {
  theme: Theme;
  language: Language;
  minimizeToTrayOnClose: boolean;
  autoStart: boolean;
  notificationsEnabled: boolean;
  monitoringEnabled: boolean;
  onboardingComplete: boolean;
  historyRetentionDays: number;
  stabilityWindowMs: number;
  previewBeforeManual: boolean;
}

export interface MonitoringHealth {
  state: MonitoringState;
  watchedSourceCount: number;
  queueDepth: number;
  activeJobs: number;
  lastError: string | null;
}

export interface ActivityDto {
  id: string;
  batchId: string;
  ruleId: string | null;
  ruleName: string | null;
  trigger: "manual" | "realtime" | "undo" | string;
  sourcePath: string;
  destinationPath: string;
  recoveryPath: string | null;
  status: string;
  errorCode: string | null;
  errorMessage: string | null;
  detectedAt: string;
  completedAt: string | null;
  undoneAt: string | null;
  canUndo: boolean;
}

export interface BootstrapDto {
  rules: RuleDto[];
  settings: SettingsDto;
  monitoring: MonitoringHealth;
  recentActivity: ActivityDto[];
}

export interface PreviewItemDto {
  id: string;
  ruleId: string;
  ruleName: string;
  sourcePath: string;
  destinationPath: string;
  sizeBytes: number;
  conflict: ConflictPolicy;
  status: string;
}

export interface PreviewResultDto {
  planId: string;
  items: PreviewItemDto[];
  totalCount: number;
  truncated: boolean;
  warnings: string[];
}

export interface BatchSummaryDto {
  batchId: string;
  status: string;
  totalCount: number;
  completedCount: number;
  failedCount: number;
}

export interface ActivityPageDto {
  items: ActivityDto[];
  nextCursor: string | null;
}

export interface ListActivityRequest {
  cursor?: string;
  limit?: number;
  ruleId?: string;
  status?: string;
  query?: string;
}

export interface AttentionDto {
  code: string;
  message: string;
  ruleId: string | null;
  operationId: string | null;
}

export interface AppErrorDto {
  code: string;
  message?: string;
  safeMessage?: string;
  retryable?: boolean;
}

export interface AppEventMap {
  "amf://activity": ActivityDto;
  "amf://batch-progress": BatchSummaryDto;
  "amf://monitoring-health": MonitoringHealth;
  "amf://attention": AttentionDto;
}
