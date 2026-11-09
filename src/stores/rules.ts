import { computed, ref } from "vue";
import { defineStore } from "pinia";
import { desktopApi } from "@/ipc/commands";
import type { RuleDto, UpsertRuleRequest } from "@/ipc/generated-types";

function byPriority(a: RuleDto, b: RuleDto): number {
  return a.priority - b.priority || a.createdAt.localeCompare(b.createdAt);
}

export const useRulesStore = defineStore("rules", () => {
  const items = ref<RuleDto[]>([]);
  const loading = ref(false);
  const mutatingIds = ref<string[]>([]);

  const sorted = computed(() => [...items.value].sort(byPriority));
  const activeCount = computed(() => items.value.filter((rule) => rule.enabled).length);
  const sourceCount = computed(() => {
    const paths = items.value
      .filter((rule) => rule.enabled)
      .flatMap((rule) => rule.sources.map((source) => source.canonicalPath || source.path));
    return new Set(paths).size;
  });

  function hydrate(rules: RuleDto[]): void {
    items.value = [...rules].sort(byPriority);
  }

  async function load(): Promise<void> {
    loading.value = true;
    try {
      hydrate(await desktopApi.listRules());
    } finally {
      loading.value = false;
    }
  }

  async function save(input: UpsertRuleRequest): Promise<RuleDto> {
    const saved = await desktopApi.upsertRule(input);
    const index = items.value.findIndex((rule) => rule.id === saved.id);
    if (index >= 0) items.value.splice(index, 1, saved);
    else items.value.push(saved);
    items.value.sort(byPriority);
    return saved;
  }

  async function remove(id: string): Promise<void> {
    mutatingIds.value.push(id);
    try {
      await desktopApi.deleteRule(id);
      items.value = items.value.filter((rule) => rule.id !== id);
    } finally {
      mutatingIds.value = mutatingIds.value.filter((value) => value !== id);
    }
  }

  async function toggle(id: string, enabled: boolean): Promise<void> {
    mutatingIds.value.push(id);
    try {
      const updated = await desktopApi.setRuleEnabled(id, enabled);
      const index = items.value.findIndex((rule) => rule.id === id);
      if (index >= 0) items.value.splice(index, 1, updated);
    } finally {
      mutatingIds.value = mutatingIds.value.filter((value) => value !== id);
    }
  }

  async function move(id: string, direction: -1 | 1): Promise<void> {
    const ordered = sorted.value;
    const index = ordered.findIndex((rule) => rule.id === id);
    const target = index + direction;
    if (index < 0 || target < 0 || target >= ordered.length) return;
    const current = ordered[index];
    const adjacent = ordered[target];
    if (!current || !adjacent) return;
    ordered[index] = adjacent;
    ordered[target] = current;
    hydrate(await desktopApi.reorderRules(ordered.map((rule) => rule.id)));
  }

  return { items, sorted, activeCount, sourceCount, loading, mutatingIds, hydrate, load, save, remove, toggle, move };
});
