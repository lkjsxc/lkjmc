import { test } from "node:test";
import assert from "node:assert/strict";
import { ReadSession } from "../src/readSession.ts";
import { mergeWindow } from "../src/timelineState.ts";
import { registerReadSessionTests } from "./readSession.test.ts";
registerReadSessionTests(test, assert);
const empty = { items: [], cursor: null, loaded: false, scroll: 42 };
test("timeline keeps prior pages, updates deletions/jobs, and uses the shared ordinal equal-time order", () => {
  const m = (id, time = "2026-10-03T08:00:00Z", extra = {}) => ({
    id,
    created_at: time,
    ...extra,
  });
  let state = mergeWindow(empty, {
    items: [m("message:9"), m("job:opaque")],
    next_cursor: "cursor/1",
  });
  state = mergeWindow(
    state,
    { items: [m("message:2"), m("message:9")], next_cursor: "cursor/2" },
    true,
  );
  state = mergeWindow(state, {
    items: [
      m("message:9", undefined, { deleted_at: "now" }),
      m("job:opaque", undefined, { state: "succeeded" }),
      m("notification:1", "2026-10-03T08:00:01Z"),
    ],
    next_cursor: "wrong/tail-cursor",
  });
  assert.deepEqual(
    state.items.map((i) => i.id),
    ["job:opaque", "message:2", "message:9", "notification:1"],
  );
  assert.equal(state.cursor, "cursor/2");
  assert.equal(state.items[2].deleted_at, "now");
  assert.equal(state.items[0].state, "succeeded");
  assert.equal(state.scroll, 42);
});
test("quiet reads share delayed requests, wait for actual result and throttle the next host submission", async () => {
  let clock = 100,
    submits = 0,
    gets = 0,
    release;
  const gate = new Promise((r) => (release = r));
  const transport = {
    submit: async () => {
      submits++;
      await gate;
      return { job_id: "durable" };
    },
    job: async () => {
      gets++;
      return gets === 1
        ? { state: "leased", progress: { message: "reading" } }
        : { state: "succeeded", result: { lines: ["actual stdout"] } };
    },
  };
  const read = new ReadSession("server_logs", { id: "server" });
  const a = read.advance(transport, () => clock, true),
    b = read.advance(transport, () => clock, true);
  assert.equal(submits, 1);
  assert.equal(read.snapshot().busy, true);
  release();
  await Promise.all([a, b]);
  await read.advance(transport, () => clock, true);
  assert.equal(gets, 0);
  clock += 1500;
  await read.advance(transport, () => clock, true);
  assert.equal(read.snapshot().result, undefined);
  assert.equal(read.snapshot().progress.message, "reading");
  clock += 2000;
  await read.advance(transport, () => clock, true);
  assert.deepEqual(read.snapshot().result, { lines: ["actual stdout"] });
  clock += 14999;
  await read.advance(transport, () => clock, true);
  assert.equal(submits, 1);
  clock++;
  await read.advance(transport, () => clock, true);
  assert.equal(submits, 2);
});
test("uncertain submission retry keeps idempotency and failed GET resumes the same job", async () => {
  let clock = 100,
    keys = [],
    gets = 0;
  const transport = {
    submit: async (_t, _v, key) => {
      keys.push(key);
      if (keys.length === 1) throw Error("network lost");
      return { job_id: "same-job" };
    },
    job: async (id) => {
      assert.equal(id, "same-job");
      if (gets++ === 0) throw Error("status unavailable");
      return { state: "succeeded", result: { path: "documents", entries: [] } };
    },
  };
  const read = new ReadSession("server_files", {
    id: "server",
    path: "documents",
  });
  await read.advance(transport, () => clock);
  assert.equal(read.snapshot().error, "network lost");
  await read.advance(transport, () => clock);
  assert.equal(keys.length, 1);
  read.retry();
  await read.advance(transport, () => clock);
  assert.equal(keys[0], keys[1]);
  clock += 1500;
  await read.advance(transport, () => clock);
  assert.equal(read.snapshot().error, "status unavailable");
  read.retry();
  await read.advance(transport, () => clock);
  assert.equal(keys.length, 2);
  assert.deepEqual(read.snapshot().result, { path: "documents", entries: [] });
  clock += 50000;
  await read.advance(transport, () => clock);
  assert.equal(keys.length, 2);
});
test("authoritative read failures clear previous output and need explicit retry; scoped reads do not share results", async () => {
  let clock = 100,
    gets = 0,
    submitted = 0;
  const transport = {
    submit: async () => ({ job_id: `read-${++submitted}` }),
    job: async () =>
      ++gets === 1
        ? { state: "succeeded", result: { lines: ["history"] } }
        : { state: "failed", error: "Date not found" },
  };
  const history = new ReadSession("server_logs", {
    id: "s",
    date: "2026-10-01",
  });
  await history.advance(transport, () => clock);
  clock += 1500;
  await history.advance(transport, () => clock);
  history.retry();
  await history.advance(transport, () => clock);
  clock += 1500;
  await history.advance(transport, () => clock);
  assert.equal(history.snapshot().error, "Date not found");
  assert.equal(history.snapshot().result, undefined);
  clock += 60000;
  await history.advance(transport, () => clock, true);
  assert.equal(submitted, 2);
  assert.equal(
    new ReadSession("server_logs", { id: "s", date: "2026-10-02" }).snapshot()
      .result,
    undefined,
  );
});

test("actual routes redirect old chat and give authorized task pages meaningful destinations", async () => {
  const [{ default: ts }, { default: vm }, { readFile }] = await Promise.all([
    import("../node_modules/typescript/lib/typescript.js"),
    import("node:vm"),
    import("node:fs/promises"),
  ]);
  const source = await readFile(
    new URL("../src/routes.ts", import.meta.url),
    "utf8",
  );
  const context = vm.createContext({
    URL,
    location: { origin: "https://ux.fixture" },
  });
  const module = new vm.SourceTextModule(
    ts.transpile(source, { module: ts.ModuleKind.ESNext }),
    { context },
  );
  await module.link(
    () =>
      new vm.SourceTextModule("export const t = (message) => message;", {
        context,
      }),
  );
  await module.evaluate();
  const { normalize, resolveRoute, topPages, childPages } = module.namespace;
  assert.equal(
    normalize("/chat/00000000-0000-0000-0000-000000000003"),
    "/timeline?room=00000000-0000-0000-0000-000000000003",
  );
  assert.equal(resolveRoute("/chat").component, "timeline");
  assert.equal(resolveRoute("/chat?kind=messages").component, "timeline");
  assert.equal(
    topPages().some((p) => p.id === "account" || p.id === "chat"),
    false,
  );
  assert.equal(topPages().find((p) => p.id === "timeline").path, "/timeline");
  const server = "00000000-0000-0000-0000-000000000001";
  assert.equal(normalize("/home/activity"), "/timeline?kind=events");
  assert.equal(
    normalize(`/manage/servers/${server}/activity`),
    `/manage/servers/${server}`,
  );
  assert.equal(
    childPages(resolveRoute("/home")).some((p) => p.name === "Recent actions"),
    false,
  );
  const logs = resolveRoute(`/manage/servers/${server}/logs`);
  assert.equal(logs.section, "manage-logs");
  assert.equal(logs.api, `/api/v1/servers/${server}?section=manage-console`);
  assert.equal(
    childPages(logs, { can_administer: false }).some(
      (p) => p.path.endsWith("/files") || p.path.endsWith("/members"),
    ),
    false,
  );
  assert.equal(
    childPages(logs, { can_administer: true }).some((p) =>
      p.path.endsWith("/files"),
    ),
    true,
  );
  assert.equal(
    childPages(resolveRoute("/teams"), undefined, { name: "Builders" })
      .map((p) => p.name)
      .join(","),
    "Builders,Members,Settings",
  );
});

test("timeline bounds retained history and applies known-id removals and membership pruning", () => {
  const items = Array.from({ length: 400 }, (_, n) => ({
    id: String(n),
    created_at: new Date(n * 1000).toISOString(),
    type: "message",
    room_id: n % 2 ? "allowed" : "revoked",
  }));
  let state = mergeWindow(empty, { items, next_cursor: "older" });
  assert.equal(state.items.length, 200);
  state = mergeWindow(state, {
    items: [],
    next_cursor: null,
    removed_ids: ["399"],
    updates: [{ ...items[397], deleted_at: "now" }],
    removed_room_ids: ["revoked"],
  });
  assert.equal(
    state.items.some((item) => item.id === "399" || item.room_id === "revoked"),
    false,
  );
  assert.equal(state.items.find((item) => item.id === "397").deleted_at, "now");
});

test("timeline preserves PostgreSQL microsecond order and rebases the retained history boundary", () => {
  const state = mergeWindow(empty, {items: [
    {id:"message:1",created_at:"2026-10-03T08:00:00.123999+00:00",before_cursor:"later"},
    {id:"message:9",created_at:"2026-10-03T08:00:00.123001+00:00",before_cursor:"earlier"},
  ],next_cursor:"initial"});
  assert.deepEqual(state.items.map(i=>i.id),["message:9","message:1"]);
  assert.equal(state.cursor,"earlier");
  const exhausted = mergeWindow(state,{items:[],next_cursor:null},true);
  assert.equal(exhausted.cursor,null);
  assert.equal(mergeWindow(exhausted,{items:[],next_cursor:"tail"}).cursor,null);
});
