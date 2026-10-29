import type {
  ConflictPolicy,
  RuleDto,
  UpsertRuleRequest,
} from "@/ipc/generated-types";

export const MEBIBYTE = 1024 * 1024;

export interface RuleIssue {
  field: "name" | "sources" | "destination" | "extensions" | "size";
  code:
    | "requiredName"
    | "requiredSource"
    | "requiredDestination"
    | "requiredExtension"
    | "invalidSize";
}

export function splitList(value: string): string[] {
  return [...new Set(value.split(",").map((item) => item.trim()).filter(Boolean))];
}

export function normalizeExtensions(value: string): string[] {
  return splitList(value).map((extension) => extension.replace(/^\./, "").toLowerCase());
}

export function createEmptyRule(): UpsertRuleRequest {
  return {
    name: "",
    enabled: true,
    sources: [""],
    destination: "",
    extensions: [],
    nameGlobs: [],
    conflictPolicy: "autoRename",
    recursive: false,
    notifications: true,
  };
}

export function ruleToInput(rule: RuleDto): UpsertRuleRequest {
  const input: UpsertRuleRequest = {
    id: rule.id,
    name: rule.name,
    enabled: rule.enabled,
    priority: rule.priority,
    sources: rule.sources.map((source) => source.path),
    destination: rule.destination,
    extensions: [...rule.extensions],
    nameGlobs: [...rule.nameGlobs],
    conflictPolicy: rule.conflictPolicy,
    recursive: rule.recursive,
    notifications: rule.notifications,
  };
  if (rule.minSizeBytes !== null) input.minSizeBytes = rule.minSizeBytes;
  if (rule.maxSizeBytes !== null) input.maxSizeBytes = rule.maxSizeBytes;
  return input;
}

export function validateRule(rule: UpsertRuleRequest): RuleIssue[] {
  const issues: RuleIssue[] = [];
  if (!rule.name.trim()) issues.push({ field: "name", code: "requiredName" });
  if (!rule.sources.some((source) => source.trim())) {
    issues.push({ field: "sources", code: "requiredSource" });
  }
  if (!rule.destination.trim()) {
    issues.push({ field: "destination", code: "requiredDestination" });
  }
  if (!rule.extensions.length) {
    issues.push({ field: "extensions", code: "requiredExtension" });
  }
  if (
    rule.minSizeBytes !== undefined
    && rule.maxSizeBytes !== undefined
    && rule.minSizeBytes > rule.maxSizeBytes
  ) {
    issues.push({ field: "size", code: "invalidSize" });
  }
  return issues;
}

export function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";
  const units = ["B", "KiB", "MiB", "GiB", "TiB"];
  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const unit = units[index] ?? "B";
  const value = bytes / 1024 ** index;
  return `${value >= 10 || index === 0 ? value.toFixed(0) : value.toFixed(1)} ${unit}`;
}

export function policyLabelKey(policy: ConflictPolicy): "autoRename" | "skip" | "safeOverwrite" {
  return policy;
}
