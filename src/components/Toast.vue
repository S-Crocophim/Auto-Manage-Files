<template>
  <div class="toast-container">
    <TransitionGroup name="toast-slide">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="toast-item"
        :class="toast.type"
      >
        <div class="toast-icon">
          <Icons v-if="toast.type === 'success'" name="check" size="16" />
          <Icons v-else-if="toast.type === 'error'" name="trash" size="16" />
          <Icons v-else name="about" size="16" />
        </div>
        <div class="toast-message">{{ toast.message }}</div>
        <button class="toast-close" @click="$emit('dismiss', toast.id)">✕</button>
      </div>
    </TransitionGroup>
  </div>
</template>

<script setup>
import Icons from './Icons.vue';

defineProps({
  toasts: { type: Array, default: () => [] }
});

defineEmits(['dismiss']);
</script>

<style scoped>
.toast-container {
  position: fixed;
  top: 64px;
  right: 24px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  z-index: 1000;
  pointer-events: none;
}

.toast-item {
  pointer-events: auto;
  min-width: 260px;
  max-width: 380px;
  padding: 12px 16px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 13px;
  font-weight: 600;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.25);
  backdrop-filter: blur(8px);
  border: 1px solid rgba(255, 255, 255, 0.12);
}

.toast-item.success {
  background: rgba(16, 185, 129, 0.95);
  color: #ffffff;
  border-color: rgba(52, 211, 153, 0.4);
}

.toast-item.error {
  background: rgba(239, 68, 68, 0.95);
  color: #ffffff;
  border-color: rgba(248, 113, 113, 0.4);
}

.toast-item.info {
  background: rgba(99, 102, 241, 0.95);
  color: #ffffff;
  border-color: rgba(129, 140, 248, 0.4);
}

.toast-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.2);
  flex-shrink: 0;
}

.toast-message {
  flex: 1;
  line-height: 1.4;
}

.toast-close {
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.8);
  font-size: 12px;
  cursor: pointer;
  padding: 2px 4px;
}

.toast-close:hover {
  color: #ffffff;
}

/* Animations */
.toast-slide-enter-active,
.toast-slide-leave-active {
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.toast-slide-enter-from {
  opacity: 0;
  transform: translateX(40px) scale(0.95);
}

.toast-slide-leave-to {
  opacity: 0;
  transform: translateY(-20px) scale(0.9);
}
</style>
