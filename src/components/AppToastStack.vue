<script setup lang="ts">
import { storeToRefs } from "pinia";
import { useUiStore } from "@/stores/ui";

const ui = useUiStore();
const { toasts } = storeToRefs(ui);
</script>

<template>
  <Teleport to="body">
    <div class="toast-stack" aria-live="polite" aria-atomic="false">
      <TransitionGroup name="toast">
        <div v-for="toast in toasts" :key="toast.id" class="toast-message" :class="`toast-message--${toast.tone}`" :role="toast.tone === 'error' ? 'alert' : 'status'">
          <span class="toast-symbol" aria-hidden="true">{{ toast.tone === "success" ? "✓" : toast.tone === "error" ? "!" : "i" }}</span>
          <p>{{ toast.message }}</p>
          <button type="button" aria-label="Tutup notifikasi" @click="ui.dismiss(toast.id)">
            ×
          </button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>
