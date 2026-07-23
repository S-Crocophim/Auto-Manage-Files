<template>
  <div>
    <h3 style="font-weight: 800; font-size: 18px; display: flex; align-items: center; gap: 8px;">
      <Icons name="about" size="18" style="color: var(--accent-indigo);" />
      {{ t('nav_about', lang) }}
    </h3>

    <div class="separator"></div>

    <div style="background: var(--bg-card); border: 1px solid var(--border); padding: 24px; border-radius: 10px; margin-top: 16px;">
      <div style="display: flex; align-items: center; gap: 14px; margin-bottom: 16px;">
        <img src="/icon.png" width="48" height="48" alt="Logo" style="border-radius: 10px;" />
        <div>
          <h4 style="font-size: 16px; font-weight: 800;">Auto File Organizer</h4>
          <span style="font-size: 12px; color: var(--fg-dim); font-family: monospace;">v1.0.0 (Tauri 2 + Rust + Vue 3)</span>
        </div>
      </div>

      <p style="font-size: 13px; color: var(--fg-secondary); line-height: 1.6; margin-bottom: 20px; white-space: pre-line;">
        {{ t('about_content', lang) }}
      </p>

      <div style="display: flex; align-items: center; gap: 8px; font-size: 13px;">
        <span style="color: var(--fg-dim);">{{ t('repo_label', lang) }}</span>
        <a
          href="https://github.com/S-Crocophim/Auto-Manage-Files"
          target="_blank"
          @click.prevent="openUrl('https://github.com/S-Crocophim/Auto-Manage-Files')"
          style="color: var(--accent-indigo); text-decoration: underline; cursor: pointer; font-weight: 600;"
        >
          https://github.com/S-Crocophim/Auto-Manage-Files
        </a>
      </div>
    </div>
  </div>
</template>

<script setup>
import { t } from '../i18n.js';
import { openUrl as tauriOpenUrl } from '@tauri-apps/plugin-opener';
import Icons from './Icons.vue';

defineProps({ lang: String });

async function openUrl(url) {
  try {
    await tauriOpenUrl(url);
  } catch {
    window.open(url, '_blank');
  }
}
</script>
