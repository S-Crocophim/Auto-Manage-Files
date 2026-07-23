<template>
  <div>
    <div style="display: flex; justify-content: space-between; align-items: center;">
      <div>
        <h3 style="font-weight: 800; font-size: 18px;">{{ t('logs_title', lang) }}</h3>
        <p style="color: var(--fg-dim); font-size: 12px; margin-top: 2px;">
          {{ t('logs_subtitle', lang) }}
        </p>
      </div>

      <button class="btn btn-secondary" style="font-size: 11px;" @click="$emit('clear-logs')">
        <Icons name="trash" size="13" />
        {{ t('clear_logs', lang) }}
      </button>
    </div>

    <div class="separator"></div>

    <!-- History list with Undo buttons -->
    <div style="margin-bottom: 24px;">
      <div style="font-size: 11px; font-weight: 700; color: var(--fg-dim); text-transform: uppercase; letter-spacing: 0.04em; margin-bottom: 10px;">
        {{ t('history_section_title', lang) }}
      </div>
      
      <div v-if="history.length === 0" style="color: var(--fg-dim); font-size: 12px; padding: 20px; text-align: center; background: var(--bg-card); border-radius: 8px; border: 1px solid var(--border);">
        {{ t('no_history_msg', lang) }}
      </div>

      <div v-else class="timeline-list" style="max-height: 240px; overflow-y: auto;">
        <div v-for="item in history" :key="item.id" class="timeline-item">
          <div style="display: flex; align-items: center; gap: 10px; overflow: hidden; max-width: 78%;">
            <div style="background: rgba(99, 102, 241, 0.15); color: var(--accent-indigo); padding: 6px; border-radius: 6px;">
              <Icons name="folder" size="14" />
            </div>
            <div style="font-size: 12px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
              <div style="display: flex; align-items: center; gap: 6px;">
                <strong style="color: var(--fg-primary);">{{ item.rule_name }}</strong>
                <span style="font-size: 10px; color: var(--fg-dim); font-family: monospace;">{{ item.timestamp }}</span>
              </div>
              <div style="color: var(--fg-secondary); font-family: monospace; font-size: 11px;">
                {{ shorten(item.src_path) }} &rarr; {{ shorten(item.dest_path) }}
              </div>
            </div>
          </div>

          <button class="btn btn-secondary" style="padding: 5px 10px; font-size: 11px;" @click="$emit('undo-move', item)">
            <Icons name="undo" size="12" />
            {{ t('btn_undo', lang) }}
          </button>
        </div>
      </div>
    </div>

    <!-- Console log -->
    <div style="font-size: 11px; font-weight: 700; color: var(--fg-dim); text-transform: uppercase; letter-spacing: 0.04em; margin-bottom: 8px;">
      {{ t('console_section_title', lang) }}
    </div>
    <div class="log-box" ref="logContainer">
      <div v-if="logs.length === 0" style="color: var(--fg-dim);">{{ t('console_waiting_msg', lang) }}</div>
      <div v-for="(log, idx) in logs" :key="idx">{{ log }}</div>
    </div>
  </div>
</template>

<script setup>
import { ref, onUpdated } from 'vue';
import { t } from '../i18n.js';
import Icons from './Icons.vue';

const props = defineProps({
  logs: { type: Array, default: () => [] },
  history: { type: Array, default: () => [] },
  lang: { type: String, default: 'id' }
});

defineEmits(['clear-logs', 'undo-move']);

const logContainer = ref(null);

onUpdated(() => {
  if (logContainer.value) {
    logContainer.value.scrollTop = logContainer.value.scrollHeight;
  }
});

function shorten(path, maxLen = 32) {
  if (!path) return '';
  if (path.length <= maxLen) return path;
  return '…' + path.slice(-(maxLen - 1));
}
</script>
