<template>
  <div id="app" :data-theme="config.settings.theme">
    <!-- Sidebar -->
    <aside class="sidebar">
      <div class="brand">
        <img src="/icon.png" width="30" height="30" alt="Logo" style="border-radius: 6px; object-fit: contain;" />
        <div class="brand-title">Auto File<br />Organizer</div>
      </div>
      <div class="app-desc">{{ t('app_desc', config.settings.language) }}</div>

      <div class="separator"></div>

      <button class="btn btn-secondary" style="margin-bottom: 8px; justify-content: flex-start;" @click="openAddRuleDialog">
        <Icons name="plus" size="14" />
        {{ t('nav_add_rule', config.settings.language) }}
      </button>

      <button class="btn btn-primary" style="justify-content: flex-start;" :disabled="isProcessing" @click="triggerManual">
        <Icons name="play" size="14" />
        {{ isProcessing ? t('nav_manual_processing', config.settings.language) : t('nav_manual', config.settings.language) }}
      </button>

      <div class="sidebar-bottom">
        <div class="separator"></div>

        <div class="setting-row">
          <span>{{ t('theme_label', config.settings.language) }}</span>
          <select v-model="config.settings.theme" @change="saveConfig">
            <option value="dark">{{ t('theme_dark', config.settings.language) }}</option>
            <option value="light">{{ t('theme_light', config.settings.language) }}</option>
          </select>
        </div>

        <div class="setting-row">
          <span>{{ t('lang_label', config.settings.language) }}</span>
          <select v-model="config.settings.language" @change="saveConfig">
            <option value="id">Indonesia</option>
            <option value="en">English</option>
          </select>
        </div>

        <div class="setting-row">
          <span>{{ t('nav_auto_start', config.settings.language) }}</span>
          <label class="switch">
            <input type="checkbox" v-model="config.settings.auto_start" @change="toggleAutoStart" />
            <span class="slider"></span>
          </label>
        </div>

        <div class="setting-row">
          <span>{{ t('nav_minimize_tray', config.settings.language) }}</span>
          <label class="switch">
            <input type="checkbox" v-model="config.settings.minimize_to_tray_on_close" @change="saveConfig" />
            <span class="slider"></span>
          </label>
        </div>

        <div class="separator"></div>

        <div style="font-size: 11px; color: var(--fg-dim); text-align: center;">
          {{ t('watermark_credit', config.settings.language) }}
        </div>

        <button class="btn btn-danger" style="margin-top: 6px; width: 100%; justify-content: center;" @click="quitApp">
          <Icons name="power" size="14" />
          {{ t('nav_quit', config.settings.language) }}
        </button>
      </div>
    </aside>

    <!-- Main Area -->
    <main class="main-area">
      <header class="top-bar">
        <div class="nav-tabs">
          <button class="tab-btn" :class="{ active: currentTab === 'dashboard' }" @click="currentTab = 'dashboard'">
            <Icons name="dashboard" size="14" />
            {{ t('nav_dashboard', config.settings.language) }}
          </button>
          <button class="tab-btn" :class="{ active: currentTab === 'logs' }" @click="currentTab = 'logs'">
            <Icons name="logs" size="14" />
            {{ t('nav_logs', config.settings.language) }}
          </button>
          <button class="tab-btn" :class="{ active: currentTab === 'tutorial' }" @click="currentTab = 'tutorial'">
            <Icons name="tutorial" size="14" />
            {{ t('nav_tutorial', config.settings.language) }}
          </button>
          <button class="tab-btn" :class="{ active: currentTab === 'about' }" @click="currentTab = 'about'">
            <Icons name="about" size="14" />
            {{ t('nav_about', config.settings.language) }}
          </button>
        </div>

        <div style="display: flex; gap: 8px;">
          <button class="btn btn-secondary" style="padding: 5px 10px; font-size: 11px;" @click="exportSettings">
            <Icons name="export" size="13" />
            {{ t('nav_export_settings', config.settings.language) }}
          </button>
          <button class="btn btn-secondary" style="padding: 5px 10px; font-size: 11px;" @click="importSettings">
            <Icons name="import" size="13" />
            {{ t('nav_import_settings', config.settings.language) }}
          </button>
        </div>
      </header>

      <div class="view-container">
        <Dashboard
          v-if="currentTab === 'dashboard'"
          :rules="config.rules"
          :history-count="config.history.length"
          :lang="config.settings.language"
          @toggle-rule="toggleRule"
          @edit-rule="openEditRuleDialog"
          @delete-rule="deleteRule"
        />

        <LogViewer
          v-else-if="currentTab === 'logs'"
          :logs="logs"
          :history="config.history"
          :lang="config.settings.language"
          @clear-logs="logs = []"
          @undo-move="onUndoMove"
        />

        <Tutorial
          v-else-if="currentTab === 'tutorial'"
          :lang="config.settings.language"
        />

        <About
          v-else-if="currentTab === 'about'"
          :lang="config.settings.language"
        />
      </div>
    </main>

    <!-- Rule Dialog Modal -->
    <RuleDialog
      v-if="showRuleDialog"
      :rule="editingRule"
      :lang="config.settings.language"
      @save="onSaveRule"
      @close="showRuleDialog = false"
    />
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open, save } from '@tauri-apps/plugin-dialog';
import { enable, disable } from '@tauri-apps/plugin-autostart';
import { sendNotification } from '@tauri-apps/plugin-notification';

import { t } from './i18n.js';
import Icons from './components/Icons.vue';
import Dashboard from './components/Dashboard.vue';
import LogViewer from './components/LogViewer.vue';
import Tutorial from './components/Tutorial.vue';
import About from './components/About.vue';
import RuleDialog from './components/RuleDialog.vue';

const currentTab = ref('dashboard');
const isProcessing = ref(false);
const logs = ref([]);
const showRuleDialog = ref(false);
const editingRule = ref(null);

const config = reactive({
  rules: [],
  settings: {
    auto_start: false,
    minimize_to_tray_on_close: true,
    theme: 'dark',
    language: 'id'
  },
  history: []
});

watch(
  () => config.settings.theme,
  (newTheme) => {
    document.documentElement.setAttribute('data-theme', newTheme);
  },
  { immediate: true }
);

onMounted(async () => {
  await loadConfig();
  
  await listen('log-event', (event) => {
    const time = new Date().toLocaleTimeString();
    const payload = event.payload;
    const formatted = `[${time}] ${payload}`;
    logs.value.push(formatted);

    if (typeof payload === 'string' && payload.includes('Moved:')) {
      const match = payload.match(/\[(.*?)\] Moved: (.*?) -> "(.*?)"/);
      if (match) {
        const ruleName = match[1];
        const filename = match[2];
        const destPath = match[3];
        
        config.history.unshift({
          id: Math.random().toString(36).substring(2, 9),
          rule_name: ruleName,
          src_path: filename,
          dest_path: destPath,
          timestamp: time
        });

        if (config.history.length > 50) config.history.pop();
        saveConfig();
      }
    }
    
    sendNotification({
      title: 'Auto File Organizer',
      body: payload
    }).catch(() => {});
  });
});

async function loadConfig() {
  try {
    const res = await invoke('get_config_cmd');
    config.rules = res.rules || [];
    config.settings = { ...config.settings, ...res.settings };
    config.history = res.history || [];
  } catch (err) {
    console.error("Failed to load config:", err);
  }
}

async function saveConfig() {
  try {
    await invoke('save_config_cmd', { config: { rules: config.rules, settings: config.settings, history: config.history } });
  } catch (err) {
    console.error("Failed to save config:", err);
  }
}

async function onUndoMove(historyItem) {
  try {
    await invoke('undo_file_move_cmd', {
      srcPath: historyItem.src_path,
      destPath: historyItem.dest_path
    });
    config.history = config.history.filter(h => h.id !== historyItem.id);
    await saveConfig();
  } catch (err) {
    alert(`Undo failed: ${err}`);
  }
}

async function toggleRule(ruleId) {
  const target = config.rules.find(r => r.id === ruleId);
  if (target) {
    target.enabled = !target.enabled;
    await saveConfig();
  }
}

function openAddRuleDialog() {
  editingRule.value = null;
  showRuleDialog.value = true;
}

function openEditRuleDialog(rule) {
  editingRule.value = { ...rule };
  showRuleDialog.value = true;
}

async function onSaveRule(savedRule) {
  if (editingRule.value) {
    const idx = config.rules.findIndex(r => r.id === savedRule.id);
    if (idx !== -1) config.rules[idx] = savedRule;
  } else {
    config.rules.push(savedRule);
  }
  showRuleDialog.value = false;
  await saveConfig();
}

async function deleteRule(ruleId) {
  config.rules = config.rules.filter(r => r.id !== ruleId);
  await saveConfig();
}

async function triggerManual() {
  isProcessing.value = true;
  try {
    const moved = await invoke('run_manual_organize_cmd', { rules: config.rules });
    const msg = `Manual organization done. ${moved} file(s) moved.`;
    logs.value.push(`[${new Date().toLocaleTimeString()}] ${msg}`);
  } catch (err) {
    console.error(err);
  } finally {
    isProcessing.value = false;
  }
}

async function toggleAutoStart() {
  try {
    if (config.settings.auto_start) {
      await enable();
    } else {
      await disable();
    }
  } catch (e) {
    console.error("AutoStart plugin error:", e);
  }
  await saveConfig();
}

async function exportSettings() {
  try {
    const filePath = await save({
      defaultPath: 'AutoFileOrganizer_Settings.json',
      filters: [{ name: 'JSON', extensions: ['json'] }]
    });
    if (filePath) {
      const data = JSON.stringify({ rules: config.rules, settings: config.settings }, null, 2);
      const blob = new Blob([data], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'AutoFileOrganizer_Settings.json';
      a.click();
      URL.revokeObjectURL(url);
    }
  } catch (err) {
    console.error(err);
  }
}

async function importSettings() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'JSON', extensions: ['json'] }]
    });
    if (selected) {
      const res = await fetch(`file://${selected}`);
      const data = await res.json();
      if (data.rules && data.settings) {
        config.rules = data.rules;
        config.settings = { ...config.settings, ...data.settings };
        if (data.history) config.history = data.history;
        await saveConfig();
      }
    }
  } catch (err) {
    console.error("Import settings error:", err);
  }
}

function quitApp() {
  window.close();
}
</script>
