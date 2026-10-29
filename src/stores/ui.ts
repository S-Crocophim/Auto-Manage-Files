import { defineStore } from "pinia";

export type ToastTone = "success" | "error" | "info" | "warning";

export interface ToastMessage {
  id: number;
  message: string;
  tone: ToastTone;
}

let nextToastId = 1;

export const useUiStore = defineStore("ui", {
  state: () => ({
    toasts: [] as ToastMessage[],
  }),
  actions: {
    notify(message: string, tone: ToastTone = "info", duration = 4200): void {
      const id = nextToastId++;
      this.toasts.push({ id, message, tone });
      window.setTimeout(() => this.dismiss(id), duration);
    },
    dismiss(id: number): void {
      this.toasts = this.toasts.filter((toast) => toast.id !== id);
    },
  },
});
