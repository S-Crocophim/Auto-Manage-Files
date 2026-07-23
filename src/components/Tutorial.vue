<template>
  <div>
    <h2>{{ t('nav_tutorial', lang) }}</h2>
    <div class="separator"></div>
    <div style="line-height: 1.8; font-size: 14px;">
      <p v-for="(step, idx) in steps" :key="idx" style="margin-bottom: 8px;">
        {{ step }}
      </p>
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue';
import { t } from '../i18n.js';

const props = defineProps({ lang: String });

const steps = computed(() => {
  const content = t('tutorial_content', props.lang);
  return Array.isArray(content) ? content : [content];
});
</script>
