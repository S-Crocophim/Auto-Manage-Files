<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-content" style="width: 520px; max-height: 90vh; overflow-y: auto;">
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
        <h3 style="font-weight: 800; font-size: 16px; display: flex; align-items: center; gap: 8px;">
          <Icons :name="isEditing ? 'edit' : 'plus'" size="16" style="color: var(--accent-indigo);" />
          {{ isEditing ? t('rule_title_edit', lang) : t('rule_title_add', lang) }}
        </h3>
        <button class="btn btn-secondary" style="padding: 4px 8px; font-size: 11px;" @click="$emit('close')">✕</button>
      </div>
      
      <div v-if="errorMsg" style="color: var(--delete-red); font-size: 12px; font-weight: bold; margin-bottom: 12px;">
        {{ errorMsg }}
      </div>

      <!-- Basic Fields -->
      <div class="form-group">
        <label>{{ t('rule_name', lang) }}</label>
        <input type="text" v-model="form.name" :placeholder="t('rule_name_ph', lang)" />
      </div>

      <div class="form-group">
        <label>{{ t('rule_watch', lang) }}</label>
        <div class="input-with-btn" @dragover.prevent @drop.prevent="handleDrop($event, 'watch_folder')">
          <input type="text" v-model="form.watch_folder" :placeholder="t('rule_watch_ph', lang)" />
          <button class="btn btn-secondary" style="cursor: pointer;" @click="selectFolder('watch_folder')">
            <Icons name="folder" size="14" />
          </button>
        </div>
      </div>

      <div class="form-group">
        <label>{{ t('rule_ext', lang) }}</label>
        <input type="text" v-model="form.extensions" :placeholder="t('rule_ext_ph', lang)" />
      </div>

      <div class="form-group">
        <label>{{ t('rule_dest', lang) }}</label>
        <div class="input-with-btn" @dragover.prevent @drop.prevent="handleDrop($event, 'destination')">
          <input type="text" v-model="form.destination" :placeholder="t('rule_dest_ph', lang)" />
          <button class="btn btn-secondary" style="cursor: pointer;" @click="selectFolder('destination')">
            <Icons name="folder" size="14" />
          </button>
        </div>
      </div>

      <div class="separator"></div>

      <!-- Advanced Options -->
      <div class="form-group">
        <label>{{ t('conflict_label', lang) }}</label>
        <select v-model="form.conflict_action" style="width: 100%; padding: 8px;">
          <option value="rename">{{ t('conflict_rename', lang) }}</option>
          <option value="skip">{{ t('conflict_skip', lang) }}</option>
          <option value="overwrite">{{ t('conflict_overwrite', lang) }}</option>
        </select>
      </div>

      <div class="form-group">
        <label>{{ t('pattern_label', lang) }}</label>
        <input type="text" v-model="form.pattern" :placeholder="t('pattern_ph', lang)" />
      </div>

      <div style="display: flex; gap: 12px;">
        <div class="form-group" style="flex: 1;">
          <label>{{ t('min_size_label', lang) }}</label>
          <input type="number" step="0.1" v-model.number="form.min_size_mb" placeholder="0" />
        </div>
        <div class="form-group" style="flex: 1;">
          <label>{{ t('max_size_label', lang) }}</label>
          <input type="number" step="0.1" v-model.number="form.max_size_mb" :placeholder="t('size_max', lang)" />
        </div>
      </div>

      <div style="display: flex; justify-content: flex-end; gap: 8px; margin-top: 16px;">
        <button class="btn btn-secondary" @click="$emit('close')">{{ t('rule_btn_cancel', lang) }}</button>
        <button class="btn btn-primary" @click="save">
          <Icons name="check" size="14" />
          {{ t('rule_btn_save', lang) }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive } from 'vue';
import { t } from '../i18n.js';
import { open } from '@tauri-apps/plugin-dialog';
import Icons from './Icons.vue';

const props = defineProps({
  rule: Object,
  lang: String
});

const emit = defineEmits(['save', 'close']);

const isEditing = ref(!!props.rule);
const errorMsg = ref('');

const form = reactive({
  id: props.rule?.id || '',
  name: props.rule?.name || '',
  watch_folder: props.rule?.watch_folder || '',
  extensions: props.rule?.extensions || '',
  destination: props.rule?.destination || '',
  enabled: props.rule?.enabled ?? true,
  conflict_action: props.rule?.conflict_action || 'rename',
  pattern: props.rule?.pattern || '',
  min_size_mb: props.rule?.min_size_mb ?? null,
  max_size_mb: props.rule?.max_size_mb ?? null,
});

async function selectFolder(field) {
  try {
    const selected = await open({
      directory: true,
      multiple: false
    });
    if (selected) {
      if (field === 'watch_folder' && form.watch_folder) {
        form.watch_folder = `${form.watch_folder}, ${selected}`;
      } else {
        form[field] = selected;
      }
    }
  } catch (err) {
    console.error("Folder selection error:", err);
  }
}

function handleDrop(event, field) {
  const files = event.dataTransfer?.files;
  if (files && files.length > 0) {
    const path = files[0].path;
    if (path) {
      if (field === 'watch_folder' && form.watch_folder) {
        form.watch_folder = `${form.watch_folder}, ${path}`;
      } else {
        form[field] = path;
      }
    }
  }
}

function save() {
  if (!form.name || !form.watch_folder || !form.extensions || !form.destination) {
    errorMsg.value = t('rule_warn_empty', props.lang);
    setTimeout(() => { errorMsg.value = ''; }, 2500);
    return;
  }
  emit('save', {
    ...form,
    min_size_mb: form.min_size_mb ? Number(form.min_size_mb) : null,
    max_size_mb: form.max_size_mb ? Number(form.max_size_mb) : null,
  });
}
</script>
