import { useSyncExternalStore } from "react";
import registry from "../../locales/languages.json";
import japanese from "../../locales/ja.json";

export const languages = registry.languages;
const files = import.meta.glob<Record<string, string>>("../../locales/*.json", {
  eager: true,
  import: "default",
});
const catalogs: Record<string, Record<string, string>> = Object.fromEntries(
  languages
    .filter((entry) => entry.code !== "en")
    .map((entry) => [
      entry.code,
      files[`../../locales/${entry.code}.json`] ?? {},
    ]),
);
const supported = (value: unknown): value is string =>
  typeof value === "string" && languages.some((entry) => entry.code === value);
let language = registry.default;
try {
  const saved = localStorage.getItem("lkjmc.language");
  if (supported(saved)) language = saved;
} catch {
  /* Storage may be unavailable in a private browser. */
}
const listeners = new Set<() => void>();
export function setLanguage(value: string) {
  const next = supported(value) ? value : registry.default;
  document.documentElement.lang = next;
  if (next === language) return;
  language = next;
  try {
    localStorage.setItem("lkjmc.language", next);
  } catch {
    /* Optional persistence. */
  }
  listeners.forEach((listener) => listener());
}
export function getLanguage() {
  return language;
}
export function getLocale() {
  return languages.find((entry) => entry.code === language)!.locale;
}
export function useLanguage() {
  return useSyncExternalStore((listener) => {
    listeners.add(listener);
    return () => {
      listeners.delete(listener);
    };
  }, getLanguage);
}
export function t(message: string, ...values: unknown[]) {
  const translated = catalogs[language]?.[message] ?? message;
  return translated.replace(/\{(\d+)\}/g, (match, index) =>
    Number(index) < values.length ? String(values[Number(index)]) : match,
  );
}
// Only structured system errors use this lookup. Player names, chat, and other
// user content are always rendered verbatim.
const errorMessages = new Map(
  Object.entries(japanese).map(([en, ja]) => [ja, en]),
);
export function translateError(message: string) {
  const key = errorMessages.get(message) ?? message;
  if (catalogs[language]?.[key]) return t(key);
  for (const [template, translated] of Object.entries(
    catalogs[language] ?? {},
  )) {
    if (!/\{[A-Za-z_0-9]*\}/.test(template)) continue;
    const names: string[] = [];
    const parts = template.split(/(\{[A-Za-z_0-9]*\})/g);
    const pattern = parts
      .map((part) => {
        if (/^\{[A-Za-z_0-9]*\}$/.test(part)) {
          names.push(part);
          return "(.+?)";
        }
        return part.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      })
      .join("");
    const matched = new RegExp("^" + pattern + "$", "s").exec(message);
    if (matched)
      return translated.replace(
        /\{[A-Za-z_0-9]*\}/g,
        (value) => matched[names.indexOf(value) + 1] ?? value,
      );
  }
  return key;
}
document.documentElement.lang = language;
window.addEventListener("storage", (event) => {
  if (event.key === "lkjmc.language")
    setLanguage(event.newValue ?? registry.default);
});
