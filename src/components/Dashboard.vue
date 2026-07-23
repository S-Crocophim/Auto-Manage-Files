<template>
  <div>
    <!-- Top Header & Monitoring Badge at Top-Right -->
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
      <div>
        <h3 style="font-weight: 800; font-size: 18px;">{{ t('dashboard_title', lang) }}</h3>
        <p style="color: var(--fg-dim); font-size: 12px; margin-top: 2px;">
          {{ t('active_rules_label', lang) }}
        </p>
      </div>

      <!-- Monitoring Status Pill Badge on the Right -->
      <div
        style="display: flex; align-items: center; gap: 8px; padding: 6px 14px; border-radius: 20px; font-size: 12px; font-weight: 700; border: 1px solid var(--border);"
        :style="isMonitoringActive ? 'background: rgba(16, 185, 129, 0.12); color: var(--accent-emerald); border-color: rgba(16, 185, 129, 0.3);' : 'background: rgba(239, 68, 68, 0.12); color: var(--delete-red); border-color: rgba(239, 68, 68, 0.3);'"
      >
        <span :class="isMonitoringActive ? 'pulse-dot' : 'static-red-dot'"></span>
        {{ isMonitoringActive ? t('status_active', lang) : t('status_inactive', lang) }}
      </div>
    </div>

    <!-- Stat KPI Overview Cards (2 Columns with Header Icon Badges) -->
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-card-header">
          <div class="stat-label">{{ t('active_rules_stat', lang) }}</div>
          <div class="stat-icon-badge" style="color: var(--accent-indigo);">
            <Icons name="dashboard" size="15" />
          </div>
        </div>
        <div class="stat-value">
          {{ activeRulesCount }} <span style="font-size: 13px; color: var(--fg-dim); font-weight: 600;">/ {{ rules.length }}</span>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-card-header">
          <div class="stat-label">{{ t('total_moved_stat', lang) }}</div>
          <div class="stat-icon-badge" style="color: var(--accent-emerald);">
            <Icons name="check" size="15" />
          </div>
        </div>
        <div class="stat-value" style="color: var(--accent-indigo);">
          {{ historyCount }}
        </div>
      </div>
    </div>

    <!-- Search Bar & List Header -->
    <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 24px; margin-bottom: 12px;">
      <div style="font-size: 12px; font-weight: 700; color: var(--fg-dim); text-transform: uppercase; letter-spacing: 0.04em;">
        {{ t('rule_list_title', lang) }}
      </div>

      <!-- Search Input -->
      <div style="position: relative; width: 220px;">
        <Icons name="search" size="13" style="position: absolute; left: 10px; top: 9px; color: var(--fg-dim);" />
        <input
          type="text"
          v-model="searchQuery"
          :placeholder="t('search_ph', lang)"
          style="padding-left: 28px; height: 32px; font-size: 12px;"
        />
      </div>
    </div>

    <div v-if="filteredRules.length === 0" style="text-align: center; padding: 40px; color: var(--fg-dim); background: var(--bg-card); border-radius: 10px; border: 1px solid var(--border);">
      <Icons name="folder" size="28" style="margin-bottom: 8px; opacity: 0.5;" />
      <div>{{ t('no_rules_msg', lang) }}</div>
    </div>

    <!-- Rule List Visual Cards -->
    <div class="card-list">
      <div v-for="rule in filteredRules" :key="rule.id" class="rule-card">
        <div class="rule-header">
          <div style="display: flex; align-items: center; gap: 8px;">
            <span class="rule-title">{{ rule.name || t('unnamed_rule', lang) }}</span>
            <span
              style="font-size: 10px; font-weight: 700; text-transform: uppercase; padding: 2px 6px; border-radius: 4px; border: 1px solid var(--border);"
              :style="strategyStyle(rule.conflict_action)"
            >
              {{ rule.conflict_action || 'rename' }}
            </span>
          </div>

          <div class="rule-actions">
            <label class="switch">
              <input type="checkbox" :checked="rule.enabled" @change="$emit('toggle-rule', rule.id)" />
              <span class="slider"></span>
            </label>
            <button class="btn btn-secondary" style="padding: 5px 9px;" @click="$emit('edit-rule', rule)">
              <Icons name="edit" size="13" />
            </button>
            <button class="btn btn-danger" style="padding: 5px 9px;" @click="$emit('delete-rule', rule.id)">
              <Icons name="trash" size="13" />
            </button>
          </div>
        </div>

        <!-- Path Flow Diagram -->
        <div class="flow-path">
          <Icons name="folder" size="14" style="color: var(--accent-amber); flex-shrink: 0;" />
          <span class="path-badge" :title="rule.watch_folder">{{ shorten(rule.watch_folder) }}</span>
          <Icons name="arrow-right" size="13" style="color: var(--fg-dim); flex-shrink: 0;" />
          <Icons name="folder" size="14" style="color: var(--accent-emerald); flex-shrink: 0;" />
          <span class="path-badge" :title="rule.destination">{{ shorten(rule.destination) }}</span>
        </div>

        <!-- Extensions Pills & Criteria -->
        <div style="display: flex; justify-content: space-between; align-items: center;">
          <div class="ext-pills">
            <span
              v-for="(ext, idx) in parseExtensions(rule.extensions)"
              :key="idx"
              class="pill-badge"
              :class="extColorClass(ext)"
            >
              {{ ext }}
            </span>
          </div>

          <div v-if="rule.pattern || rule.min_size_mb || rule.max_size_mb" style="font-size: 11px; color: var(--fg-dim);">
            <span v-if="rule.pattern">Pattern: <code>{{ rule.pattern }}</code> &bull; </span>
            <span v-if="rule.min_size_mb || rule.max_size_mb">
              {{ rule.min_size_mb || 0 }}MB - {{ rule.max_size_mb ? rule.max_size_mb + 'MB' : t('size_max', lang) }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';
import { t } from '../i18n.js';
import Icons from './Icons.vue';

const props = defineProps({
  rules: { type: Array, default: () => [] },
  historyCount: { type: Number, default: 0 },
  lang: { type: String, default: 'id' }
});

defineEmits(['toggle-rule', 'edit-rule', 'delete-rule']);

const searchQuery = ref('');

const activeRulesCount = computed(() => props.rules.filter(r => r.enabled).length);
const isMonitoringActive = computed(() => activeRulesCount.value > 0);

const filteredRules = computed(() => {
  if (!searchQuery.value.trim()) return props.rules;
  const q = searchQuery.value.toLowerCase();
  return props.rules.filter(r =>
    r.name.toLowerCase().includes(q) ||
    r.extensions.toLowerCase().includes(q) ||
    r.watch_folder.toLowerCase().includes(q)
  );
});

function parseExtensions(extStr) {
  if (!extStr) return [];
  const commaSplit = extStr.split(',').map(e => e.trim()).filter(Boolean);
  const result = [];
  for (let item of commaSplit) {
    if (!item.startsWith('.')) item = '.' + item;
    // Handle concatenated extensions like .xlsx.xls.csv
    const subExts = item.split('.').filter(Boolean).map(e => '.' + e);
    result.push(...subExts);
  }
  return result;
}

function extColorClass(ext) {
  const clean = ext.toLowerCase();
  if (['.pdf', '.doc', '.docx', '.txt'].includes(clean)) return 'rose';
  if (['.xlsx', '.xls', '.csv', '.json'].includes(clean)) return 'green';
  if (['.jpg', '.jpeg', '.png', '.gif', '.svg', '.jfif', '.webp', '.tif'].includes(clean)) return 'blue';
  return 'amber';
}

function strategyStyle(action) {
  if (action === 'skip') return 'color: var(--accent-amber); background: rgba(245, 158, 11, 0.1);';
  if (action === 'overwrite') return 'color: var(--delete-red); background: rgba(244, 63, 94, 0.1);';
  return 'color: var(--accent-emerald); background: rgba(16, 185, 129, 0.1);';
}

function shorten(path, maxLen = 30) {
  if (!path) return '';
  if (path.length <= maxLen) return path;
  return '…' + path.slice(-(maxLen - 1));
}
</script>
