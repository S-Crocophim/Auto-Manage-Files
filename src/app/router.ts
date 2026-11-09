import { createRouter, createWebHashHistory } from "vue-router";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", redirect: "/dashboard" },
    {
      path: "/onboarding",
      name: "onboarding",
      component: () => import("@/pages/OnboardingPage.vue"),
      meta: { chrome: false },
    },
    {
      path: "/dashboard",
      name: "dashboard",
      component: () => import("@/pages/DashboardPage.vue"),
    },
    {
      path: "/rules",
      name: "rules",
      component: () => import("@/pages/RulesPage.vue"),
    },
    {
      path: "/activity",
      name: "activity",
      component: () => import("@/pages/ActivityPage.vue"),
    },
    {
      path: "/settings",
      name: "settings",
      component: () => import("@/pages/SettingsPage.vue"),
    },
    { path: "/:pathMatch(.*)*", redirect: "/dashboard" },
  ],
});
