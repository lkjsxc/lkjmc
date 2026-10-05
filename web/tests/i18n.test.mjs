import { test } from "node:test";
import assert from "node:assert/strict";
import vm from "node:vm";
import { readFile } from "node:fs/promises";
import ts from "../node_modules/typescript/lib/typescript.js";
async function harness(saved) {
  const subscriptions = new Set();
  const context = vm.createContext({
    document: { documentElement: { lang: "" } },
    window: { addEventListener: () => {} },
    localStorage: { getItem: () => saved, setItem: () => {} },
    navigator: { language: "ja-JP" },
  });
  const react = new vm.SyntheticModule(["useSyncExternalStore"], function () {
    this.setExport("useSyncExternalStore", (subscribe, get) => { subscribe(() => subscriptions.add(get())); return get(); });
  }, { context });
  const i18n = new vm.SourceTextModule(ts.transpile(await readFile(new URL("../src/i18n.ts", import.meta.url), "utf8"), { module: ts.ModuleKind.ESNext }), { context });
  await i18n.link(async (specifier) => {
    if (specifier === "react") return react;
    const value = JSON.parse(await readFile(new URL(specifier, new URL("../src/i18n.ts", import.meta.url)), "utf8"));
    return new vm.SyntheticModule(["default"], function () { this.setExport("default", value); }, { context });
  });
  await i18n.evaluate();
  return { i18n: i18n.namespace, subscriptions, context };
}

test("anonymous English default ignores Japanese browser; a root subscription sees every explicit switch", async () => {
  const h = await harness(null);
  assert.equal(h.i18n.getLanguage(), "en");
  h.i18n.useLanguage();
  h.i18n.setLanguage("ja");
  assert.equal(h.context.document.documentElement.lang, "ja");
  assert.equal(h.subscriptions.has("ja"), true);
  h.i18n.setLanguage("en");
  assert.equal(h.subscriptions.has("en"), true);
});
test("open errors, pending operations and dialog messages render in the current language", async () => {
  const h = await harness("en");
  const pending = h.i18n.message("error.login_required");
  const error = { systemMessage: pending };
  assert.equal(h.i18n.messageError(error), "Please sign in.");
  h.i18n.setLanguage("ja");
  assert.equal(h.i18n.messageError(error), "ログインしてください。");
  h.i18n.setLanguage("en");
  assert.equal(h.i18n.renderSystemMessage(pending), "Please sign in.");
});
test("unknown IDs, diagnostics and invalid parameters never leak an arbitrary source language", async () => {
  const h = await harness("en");
  const japanese = "秘密の内部エラー";
  assert.equal(h.i18n.renderSystemMessage(japanese).includes(japanese), false);
  assert.match(h.i18n.renderSystemMessage({ id: "unknown.key", params: {} }), /Reference: message-/);
  assert.match(h.i18n.renderSystemMessage({ id: "error.internal", params: {} }), /could not be verified/);
  assert.match(h.i18n.renderSystemMessage({ id: "error.login_required", params: { extra: "日本語" } }), /could not be verified/);
});
test("named player text is inserted exactly once and is never treated as another template", async () => {
  const h = await harness("en");
  const value = "日本語 {reference} $1 \\";
  assert.equal(h.i18n.renderSystemMessage({ id: "error.internal", params: { reference: value } }), "The action failed. Reference: " + value);
});
test("team context and bounded file notices retain parameters when their display language changes", async () => {
  const h = await harness("en");
  const player = "Player {1} $1";
  const team = "日本語 {0} $2";
  const permissions = h.i18n.message("team.permissions_title", player, team);
  const bound = h.i18n.message("hosting.files.entry_limit", { limit: 200 });
  assert.equal(h.i18n.renderSystemMessage(permissions), `Permissions for ${player} in ${team}`);
  assert.match(h.i18n.renderSystemMessage(bound), /^Showing the first 200 entries\./);
  h.i18n.setLanguage("ja");
  assert.equal(h.i18n.renderSystemMessage(permissions), `${team}での${player}の権限`);
  assert.match(h.i18n.renderSystemMessage(bound), /^最初の200件を表示しています。/);
  h.i18n.setLanguage("en");
  assert.equal(h.i18n.renderSystemMessage(permissions), `Permissions for ${player} in ${team}`);
});
