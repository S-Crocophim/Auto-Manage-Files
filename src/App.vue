<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted } from "vue";
import { storeToRefs } from "pinia";
import { RouterLink, RouterView, useRoute, useRouter } from "vue-router";
import AppToastStack from "@/components/AppToastStack.vue";
import PreviewModal from "@/components/PreviewModal.vue";
import { useI18n } from "@/i18n";
import { useAppStore } from "@/stores/app";
import { usePreviewStore } from "@/stores/preview";
import { useRulesStore } from "@/stores/rules";
import { useSettingsStore } from "@/stores/settings";
import { useUiStore } from "@/stores/ui";

const app = useAppStore();
const preview = usePreviewStore();
const rules = useRulesStore();
const settings = useSettingsStore();
const ui = useUiStore();
const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { ready, loading, error, monitoring } = storeToRefs(app);

const navItems = computed(() => [
  { to: "/dashboard", label: t("navDashboard"), icon: "dashboard" },
  { to: "/rules", label: t("navRules"), icon: "rules" },
  { to: "/activity", label: t("navActivity"), icon: "activity" },
  { to: "/settings", label: t("navSettings"), icon: "settings" },
]);
const showChrome = computed(() => route.meta.chrome !== false);
const pageTitle = computed(() => navItems.value.find((item) => route.path.startsWith(item.to))?.label ?? t("appName"));
const monitoringLabel = computed(() => {
  if (monitoring.value.state === "monitoring") return t("monitoringActive");
  if (["paused", "stopped"].includes(monitoring.value.state)) return t("monitoringPaused");
  if (["attention", "error"].includes(monitoring.value.state)) return t("monitoringAttention");
  return t("monitoringStarting");
});
const monitoringTone = computed(() => {
  if (monitoring.value.state === "monitoring") return "positive";
  if (["attention", "error"].includes(monitoring.value.state)) return "danger";
  return "muted";
});

async function initialize(): Promise<void> {
  try {
    await app.initialize();
    if (!settings.values.onboardingComplete && rules.items.length === 0) await router.replace("/onboarding");
  } catch {
    // A safe error message is rendered by the recovery state.
  }
}

async function organizeNow(): Promise<void> {
  try {
    await preview.create();
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  }
}

async function toggleMonitoring(): Promise<void> {
  try {
    await app.setMonitoring(monitoring.value.state !== "monitoring");
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  }
}

onMounted(() => void initialize());
onBeforeUnmount(() => app.dispose());
</script>

<template>
  <a class="skip-link" href="#main-content">Lewati ke konten utama</a>

  <div v-if="loading && !ready" class="center-state" role="status" aria-live="polite">
    <div class="brand-mark brand-mark--large" aria-hidden="true">
      A
    </div>
    <div class="spinner" aria-hidden="true" />
    <h1>{{ t("appName") }}</h1>
    <p>{{ t("loading") }}</p>
  </div>

  <div v-else-if="error && !ready" class="center-state center-state--error" role="alert">
    <div class="state-icon" aria-hidden="true">
      !
    </div>
    <h1>Aplikasi belum dapat dibuka</h1>
    <p>{{ error }}</p>
    <button class="button button--primary" type="button" @click="initialize">
      {{ t("retry") }}
    </button>
  </div>

  <RouterView v-else-if="!showChrome" />

  <div v-else class="app-shell">
    <aside class="sidebar" aria-label="Navigasi utama">
      <RouterLink class="brand" to="/dashboard" aria-label="Auto File Organizer">
        <span class="brand-mark" aria-hidden="true">A</span>
        <span><strong>Auto File</strong><small>Organizer</small></span>
      </RouterLink>

      <nav class="nav-list">
        <RouterLink v-for="item in navItems" :key="item.to" class="nav-item" :to="item.to" :aria-label="item.label">
          <svg v-if="item.icon === 'dashboard'" viewBox="0 0 24 24" aria-hidden="true"><path d="M4 13h6V4H4v9Zm0 7h6v-5H4v5Zm10 0h6v-9h-6v9Zm0-16v5h6V4h-6Z" /></svg>
          <svg v-else-if="item.icon === 'rules'" viewBox="0 0 24 24" aria-hidden="true"><path d="M5 6h14M5 12h14M5 18h14M8 3v6m8 0v6m-6 0v6" /></svg>
          <svg v-else-if="item.icon === 'activity'" viewBox="0 0 24 24" aria-hidden="true"><path d="M4 12a8 8 0 1 0 3-6.25M4 4v5h5m3-2v5l3 2" /></svg>
          <svg v-else viewBox="0 0 24 24" aria-hidden="true"><path d="M12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7Zm7-3.5 2-1-2-3-2.2.4a8 8 0 0 0-1.4-.8L14.7 4h-3.4l-.7 2.6c-.5.2-1 .5-1.4.8L7 7 5 10l2 2-2 2 2 3 2.2-.4c.4.3.9.6 1.4.8l.7 2.6h3.4l.7-2.6c.5-.2 1-.5 1.4-.8L19 17l2-3-2-2Z" /></svg>
          <span>{{ item.label }}</span>
        </RouterLink>
      </nav>

      <div class="sidebar-foot">
        <div class="privacy-card">
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3 5 6v5c0 4.4 2.8 8.4 7 10 4.2-1.6 7-5.6 7-10V6l-7-3Zm-3 9 2 2 4-5" /></svg>
          <div><strong>100% Offline</strong><span>{{ t("offline") }}</span></div>
        </div>
        <span class="version-label">v2.0.0</span>
      </div>
    </aside>

    <section class="workspace">
      <header class="topbar">
        <div><h1>{{ pageTitle }}</h1></div>
        <div class="topbar-actions">
          <button class="health-pill" :class="`health-pill--${monitoringTone}`" type="button" :aria-label="`${monitoringLabel}. Klik untuk mengubah status.`" @click="toggleMonitoring">
            <span class="health-dot" />{{ monitoringLabel }}
          </button>
          <button class="button button--primary" type="button" @click="organizeNow">
            <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M4 7h16M4 12h12M4 17h16" stroke="currentColor" stroke-width="2" stroke-linecap="round" fill="none" /></svg>
            {{ t("organizeNow") }}
          </button>
        </div>
      </header>
      <main id="main-content" class="page-content" tabindex="-1">
        <RouterView />
      </main>
    </section>
  </div>

  <PreviewModal />
  <AppToastStack />
</template>
