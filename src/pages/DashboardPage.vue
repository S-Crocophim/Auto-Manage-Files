<script setup lang="ts">
import { computed } from "vue";
import { storeToRefs } from "pinia";
import { RouterLink } from "vue-router";
import { useI18n } from "@/i18n";
import type { ActivityDto } from "@/ipc/generated-types";
import { useActivityStore } from "@/stores/activity";
import { useAppStore } from "@/stores/app";
import { usePreviewStore } from "@/stores/preview";
import { useRulesStore } from "@/stores/rules";
import { useUiStore } from "@/stores/ui";

const app = useAppStore();
const activity = useActivityStore();
const rules = useRulesStore();
const preview = usePreviewStore();
const ui = useUiStore();
const { t, language } = useI18n();
const { monitoring, attentions } = storeToRefs(app);
const { sorted, activeCount, sourceCount } = storeToRefs(rules);
const { items, successfulToday, failedToday } = storeToRefs(activity);
const recent = computed(() => items.value.slice(0, 6));
const monitoringHealthy = computed(() => monitoring.value.state === "monitoring");

function fileName(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).at(-1) ?? path;
}

function formatDate(value: string | null): string {
  if (!value) return "—";
  return new Intl.DateTimeFormat(language.value === "en" ? "en-US" : "id-ID", { dateStyle: "medium", timeStyle: "short" }).format(new Date(value));
}

function statusTone(status: string): string {
  const value = status.toLowerCase();
  if (["completed", "undone", "success"].includes(value)) return "success";
  if (["failed", "error"].includes(value)) return "failed";
  return "pending";
}

async function toggleMonitoring(): Promise<void> {
  try {
    await app.setMonitoring(!monitoringHealthy.value);
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  }
}

async function previewAll(): Promise<void> {
  try {
    await preview.create();
  } catch (cause) {
    ui.notify(t("operationFailed", { message: String(cause) }), "error");
  }
}

function activityDescription(item: ActivityDto): string {
  if (item.status.toLowerCase() === "undone") return `Dikembalikan ke ${item.sourcePath}`;
  if (item.status.toLowerCase() === "failed") return item.errorMessage ?? "Operasi gagal";
  return `Dipindahkan ke ${item.destinationPath}`;
}
</script>

<template>
  <section class="page-stack">
    <div class="page-heading">
      <div><span class="eyebrow">OVERVIEW</span><h2>{{ t("dashboardTitle") }}</h2><p>{{ t("dashboardSubtitle") }}</p></div>
      <button class="button button--secondary" type="button" :disabled="!activeCount" @click="previewAll">
        {{ t("previewAll") }}
      </button>
    </div>

    <section class="monitoring-hero" :class="{ 'monitoring-hero--paused': !monitoringHealthy }">
      <div class="monitoring-visual" aria-hidden="true">
        <div class="radar-ring radar-ring--one" /><div class="radar-ring radar-ring--two" /><div class="monitoring-folder">
          ⌂
        </div>
      </div>
      <div class="monitoring-copy">
        <span class="status-kicker"><i />{{ monitoringHealthy ? t("monitoringActive") : t("monitoringPaused") }}</span>
        <h3>{{ monitoringHealthy ? "Folder Anda sedang dijaga secara otomatis" : "Monitoring menunggu untuk dilanjutkan" }}</h3>
        <p v-if="monitoring.lastError">
          {{ monitoring.lastError }}
        </p>
        <p v-else>
          {{ sourceCount }} folder sumber dipantau oleh {{ activeCount }} aturan aktif. Semua pemrosesan tetap berada di perangkat ini.
        </p>
        <button class="text-button text-button--light" type="button" @click="toggleMonitoring">
          {{ monitoringHealthy ? t("pause") : t("resume") }} monitoring →
        </button>
      </div>
      <dl class="monitoring-metrics">
        <div><dt>{{ t("queuedJobs") }}</dt><dd>{{ monitoring.queueDepth }}</dd></div>
        <div><dt>{{ t("activeJobs") }}</dt><dd>{{ monitoring.activeJobs }}</dd></div>
        <div><dt>Sumber tersedia</dt><dd>{{ monitoring.watchedSourceCount }}</dd></div>
      </dl>
    </section>

    <div class="stat-grid">
      <article class="stat-card">
        <span class="stat-icon stat-icon--violet">⌁</span><div><span>{{ t("activeRules") }}</span><strong>{{ activeCount }}</strong><small>dari {{ sorted.length }} total aturan</small></div>
      </article>
      <article class="stat-card">
        <span class="stat-icon stat-icon--blue">⌂</span><div><span>{{ t("watchedFolders") }}</span><strong>{{ sourceCount }}</strong><small>folder sumber unik</small></div>
      </article>
      <article class="stat-card">
        <span class="stat-icon stat-icon--green">✓</span><div><span>{{ t("successToday") }}</span><strong>{{ successfulToday }}</strong><small>operasi selesai</small></div>
      </article>
      <article class="stat-card">
        <span class="stat-icon stat-icon--red">!</span><div><span>{{ t("failedToday") }}</span><strong>{{ failedToday }}</strong><small>perlu ditinjau</small></div>
      </article>
    </div>

    <div class="dashboard-columns">
      <section class="panel activity-panel">
        <header class="panel-header">
          <div><h3>{{ t("recentActivity") }}</h3><p>Perubahan terbaru dari operasi manual dan realtime.</p></div><RouterLink class="text-button" to="/activity">
            Lihat semua →
          </RouterLink>
        </header>
        <div v-if="recent.length" class="activity-list">
          <article v-for="item in recent" :key="item.id" class="activity-row">
            <span class="activity-status" :class="`activity-status--${statusTone(item.status)}`">{{ statusTone(item.status) === "success" ? "✓" : statusTone(item.status) === "failed" ? "!" : "…" }}</span>
            <div class="activity-main">
              <strong>{{ fileName(item.sourcePath) }}</strong><span :title="activityDescription(item)">{{ activityDescription(item) }}</span><small>{{ item.ruleName ?? "Tanpa aturan" }} · {{ item.trigger }}</small>
            </div>
            <time :datetime="item.completedAt ?? item.detectedAt">{{ formatDate(item.completedAt ?? item.detectedAt) }}</time>
          </article>
        </div>
        <div v-else class="empty-panel empty-panel--compact">
          <span class="empty-icon" aria-hidden="true">↗</span><h3>{{ t("noActivity") }}</h3><p>Jalankan preview untuk melihat file yang dapat dirapikan.</p>
        </div>
      </section>

      <aside class="panel attention-panel">
        <header class="panel-header">
          <div><h3>{{ t("attentionTitle") }}</h3><p>Hal yang membutuhkan keputusan Anda.</p></div>
        </header>
        <div v-if="attentions.length" class="attention-list">
          <article v-for="attention in attentions" :key="`${attention.code}-${attention.ruleId ?? ''}`" class="attention-item">
            <span aria-hidden="true">!</span><div><strong>{{ attention.code }}</strong><p>{{ attention.message }}</p></div>
          </article>
        </div>
        <div v-else class="all-good">
          <span aria-hidden="true">✓</span><strong>{{ t("allGood") }}</strong><p>Organizer siap bekerja saat ada file baru.</p>
        </div>
        <RouterLink class="button button--secondary button--full" to="/rules">
          {{ t("openRules") }}
        </RouterLink>
      </aside>
    </div>
  </section>
</template>
