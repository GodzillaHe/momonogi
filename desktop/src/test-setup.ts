import "@testing-library/jest-dom/vitest";
import { beforeEach } from "vitest";
import i18n from "./i18n";

beforeEach(async () => {
  window.localStorage.clear();
  await i18n.changeLanguage("en");
  document.documentElement.lang = "en";
});

Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => undefined,
    removeListener: () => undefined,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
    dispatchEvent: () => false,
  }),
});
