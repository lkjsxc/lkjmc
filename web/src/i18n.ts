import { useSyncExternalStore } from "react";
import registry from "../../locales/languages.json";
import english from "../../locales/en.json";
import japanese from "../../locales/ja.json";

export type MessageParameter = string | number | boolean | null;
export type SystemMessage = {
  id: string;
  params: Record<string, MessageParameter>;
};
export type MessageId = keyof typeof english;
export const languages = registry.languages;
const catalogs: Record<string, Record<string, string>> = { en: english, ja: japanese };
const supported = (value: unknown): value is string =>
  typeof value === "string" && languages.some((entry) => entry.code === value);
let language = registry.default;
try {
  const saved = localStorage.getItem("lkjmc.language");
  if (supported(saved)) language = saved;
} catch { /* Storage is optional. */ }
const listeners = new Set<() => void>();
const subscribe = (listener: () => void) => {
  listeners.add(listener);
  return () => { listeners.delete(listener); };
};
export function setLanguage(value: string) {
  const next = supported(value) ? value : registry.default;
  if (typeof document !== "undefined") document.documentElement.lang = next;
  if (next === language) return;
  language = next;
  try { localStorage.setItem("lkjmc.language", next); } catch { /* Optional persistence. */ }
  listeners.forEach((listener) => listener());
}
export function getLanguage() { return language; }
export function getLocale() {
  return languages.find((entry) => entry.code === language)!.locale;
}
/** Subscribe at the application root so anonymous pages and open dialogs rerender. */
export function useLanguage() {
  return useSyncExternalStore(subscribe, getLanguage, () => registry.default);
}
export function message(id: string, ...values: unknown[]): SystemMessage {
  const params = values.length === 1 && values[0] && typeof values[0] === "object" && !Array.isArray(values[0])
    ? values[0] as Record<string, MessageParameter>
    : Object.fromEntries(values.map((value, index) => [String(index), value as MessageParameter]));
  return { id, params };
}
export function isSystemMessage(value: unknown): value is SystemMessage {
  if (!value || typeof value !== "object") return false;
  const item = value as SystemMessage;
  return typeof item.id === "string" && !!item.params && typeof item.params === "object" && !Array.isArray(item.params)
    && Object.values(item.params).every((parameter) => parameter === null || ["string", "number", "boolean"].includes(typeof parameter));
}
function reference(value: unknown) {
  if (isSystemMessage(value) && typeof value.params.reference === "string") return value.params.reference;
  // A stable reference aids support without presenting an arbitrary fallback language.
  let hash = 2166136261;
  const source = typeof value === "string" ? value : JSON.stringify(value) ?? "unknown";
  for (const char of source) hash = Math.imul(hash ^ char.charCodeAt(0), 16777619);
  return "message-" + (hash >>> 0).toString(16).padStart(8, "0");
}
export function renderSystemMessage(value: unknown): string {
  let item = value;
  // System messages stored in historical text columns use their JSON envelope.
  if (typeof item === "string" && item.startsWith("{")) {
    try { item = JSON.parse(item); } catch { /* Invalid system record. */ }
  }
  const template = isSystemMessage(item) ? catalogs[language]?.[item.id] : undefined;
  if (!template || !isSystemMessage(item)) {
    return catalogs[language]["system.unknown"].replace("{reference}", reference(item));
  }
  const required = [...new Set([...template.matchAll(/\{([A-Za-z_0-9]+)\}/g)].map((match) => match[1]))];
  if (required.some((key) => !(key in item.params)) || Object.keys(item.params).some((key) => !required.includes(key))) {
    return catalogs[language]["system.unknown"].replace("{reference}", reference(item));
  }
  // Replacement is one pass: braces inside player-provided parameter text stay literal.
  return template.replace(/\{([A-Za-z_0-9]+)\}/g, (_, key) => String(item.params[key]));
}
export function t(id: string, ...values: unknown[]) {
  return renderSystemMessage(message(id, ...values));
}
/** Only call for system-owned errors; player-authored content stays verbatim. */
export function messageError(value: unknown): string {
  if (value && typeof value === "object" && "systemMessage" in value)
    return renderSystemMessage((value as { systemMessage: unknown }).systemMessage);
  return renderSystemMessage(value);
}
export const translateError = renderSystemMessage;
if (typeof document !== "undefined") document.documentElement.lang = language;
if (typeof window !== "undefined") window.addEventListener("storage", (event) => {
  if (event.key === "lkjmc.language") setLanguage(event.newValue ?? registry.default);
});
