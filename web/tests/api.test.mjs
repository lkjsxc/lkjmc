import { test } from "node:test";
import assert from "node:assert/strict";
import vm from "node:vm";
import { readFile } from "node:fs/promises";
import ts from "../node_modules/typescript/lib/typescript.js";
import { localeModule } from "./localeHarness.mjs";
async function harness() {
  const requests = [];
  const context = vm.createContext({
    AbortController,
    AbortSignal,
    DOMException,
    Headers,
    FormData,
    Response,
    crypto,
    Intl,
    Date,
    setTimeout,
    clearTimeout,
    fetch: (path, options) =>
      new Promise((resolve) => requests.push({ path, options, resolve })),
  });
  const identity = new vm.SourceTextModule(
    ts.transpile(
      await readFile(new URL("../src/identity.ts", import.meta.url), "utf8"),
      { module: ts.ModuleKind.ESNext },
    ),
    { context },
  );
  const i18n = await localeModule(context);
  const api = new vm.SourceTextModule(
    ts.transpile(
      await readFile(new URL("../src/api.ts", import.meta.url), "utf8"),
      { module: ts.ModuleKind.ESNext },
    ),
    { context },
  );
  await api.link((specifier) => (specifier === "./identity" ? identity : i18n));
  await api.evaluate();
  const respond = (index, body, status = 200) =>
    requests[index].resolve(new Response(JSON.stringify(body), { status }));
  return {
    api: api.namespace,
    identity: identity.namespace,
    requests,
    respond,
  };
}
test("actual API rejects old-session responses and job dedupe cannot cross CSRF identity", async () => {
  const h = await harness();
  const me = h.api.api("/api/v1/me");
  h.respond(0, { account: { id: "same-account" }, csrf: "one" });
  await me;
  const old = h.api.readJob("same-job");
  const rejected = assert.rejects(old, { name: "AbortError" });
  const nextMe = h.api.api("/api/v1/me");
  h.respond(2, { account: { id: "same-account" }, csrf: "two" });
  await nextMe;
  assert.equal(h.requests[1].options.signal.aborted, true);
  const fresh = h.api.readJob("same-job");
  assert.notEqual(fresh, old);
  h.respond(1, { result: { text: "private old result" } });
  await rejected;
  assert.equal(h.api.readJob("same-job"), fresh);
  h.respond(3, { result: { text: "authorized current result" } });
  assert.equal((await fresh).result.text, "authorized current result");
  const send = h.api.command("message_send", { room: "room", body: "message" });
  assert.equal(h.requests[4].options.headers.get("x-csrf-token"), "two");
  h.respond(4, { result: { sent: true } });
  await send;
});
test("401 invalidates the epoch even with an unreadable response; parallel output stays inaccessible", async () => {
  const h = await harness();
  h.identity.acceptIdentity("account", "session");
  const before = h.identity.identityEpoch();
  let resets = 0;
  h.identity.onIdentityReset(() => resets++);
  const privateRead = h.api.api("/api/v1/servers/private");
  const rejected = assert.rejects(privateRead, { name: "AbortError" });
  const expired = h.api.api("/api/v1/me");
  const unauthorized = assert.rejects(expired, { status: 401 });
  h.requests[1].resolve(new Response("not JSON", { status: 401 }));
  await unauthorized;
  assert.equal(h.identity.identityEpoch(), before + 1);
  assert.equal(resets, 1);
  h.respond(0, { text: "old private output" });
  await rejected;
});

test("successful empty logout response clears the session without requiring JSON", async () => {
  const h = await harness();
  h.identity.acceptIdentity("account", "session");
  const signal = h.identity.identitySignal();
  const logout = h.api.api("/auth/logout", { method: "POST" });
  h.requests[0].resolve(new Response(null, { status: 204 }));
  await logout;
  assert.equal(signal.aborted, true);
});
