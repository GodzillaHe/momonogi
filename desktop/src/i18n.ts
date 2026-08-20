import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { en } from "./locales/en";
import { zhCN } from "./locales/zh-CN";

export const LANGUAGE_STORAGE_KEY = "momonogi.language";
export type AppLanguage = "en" | "zh-CN";

function normalizeLanguage(value: string | null | undefined): AppLanguage {
  return value?.toLowerCase().startsWith("zh") ? "zh-CN" : "en";
}

function initialLanguage(): AppLanguage {
  const saved = window.localStorage.getItem(LANGUAGE_STORAGE_KEY);
  return saved === "en" || saved === "zh-CN" ? saved : normalizeLanguage(window.navigator.language);
}

void i18n.use(initReactI18next).init({
  resources: { en, "zh-CN": zhCN },
  lng: initialLanguage(),
  fallbackLng: "en",
  interpolation: { escapeValue: false },
  returnNull: false,
});

document.documentElement.lang = normalizeLanguage(i18n.language);

export async function setAppLanguage(language: AppLanguage): Promise<void> {
  await i18n.changeLanguage(language);
  window.localStorage.setItem(LANGUAGE_STORAGE_KEY, language);
  document.documentElement.lang = language;
}

export default i18n;
