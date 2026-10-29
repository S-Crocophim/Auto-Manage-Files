import { defineStore } from "pinia";
import { desktopApi } from "@/ipc/commands";
import type { SettingsDto, Theme } from "@/ipc/generated-types";
import { setLanguage } from "@/i18n";

export function defaultSettings(): SettingsDto {
  return {
    theme: "system",
    language: "id",
    minimizeToTrayOnClose: true,
    autoStart: false,
    notificationsEnabled: true,
    monitoringEnabled: false,
    onboardingComplete: false,
    historyRetentionDays: 30,
    stabilityWindowMs: 1500,
    previewBeforeManual: true,
  };
}

let mediaQuery: MediaQueryList | null = null;
let systemThemeHandler: (() => void) | null = null;

function resolveTheme(theme: Theme): "light" | "dark" {
  if (theme !== "system") return theme;
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function applyTheme(theme: Theme): void {
  document.documentElement.dataset.theme = resolveTheme(theme);
  document.documentElement.dataset.themePreference = theme;

  if (mediaQuery && systemThemeHandler) {
    mediaQuery.removeEventListener("change", systemThemeHandler);
  }
  mediaQuery = null;
  systemThemeHandler = null;

  if (theme === "system") {
    mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    systemThemeHandler = () => {
      document.documentElement.dataset.theme = resolveTheme("system");
    };
    mediaQuery.addEventListener("change", systemThemeHandler);
  }
}

export const useSettingsStore = defineStore("settings", {
  state: () => ({
    values: defaultSettings(),
    saving: false,
  }),
  actions: {
    hydrate(settings: SettingsDto): void {
      this.values = { ...settings };
      setLanguage(settings.language);
      applyTheme(settings.theme);
    },
    previewTheme(theme: Theme): void {
      applyTheme(theme);
    },
    async save(patch: Partial<SettingsDto>): Promise<SettingsDto> {
      this.saving = true;
      try {
        const settings = await desktopApi.updateSettings(patch);
        this.hydrate(settings);
        return settings;
      } finally {
        this.saving = false;
      }
    },
  },
});
