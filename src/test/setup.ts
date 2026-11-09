import { afterEach } from "vitest";
import { enableAutoUnmount } from "@vue/test-utils";

enableAutoUnmount(afterEach);
afterEach(() => {
  document.documentElement.removeAttribute("data-theme");
});
