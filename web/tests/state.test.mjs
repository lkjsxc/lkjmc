import { test } from "node:test";
import assert from "node:assert/strict";
import { localeModule } from "./localeHarness.mjs";
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
  assert.equal(read.snapshot().error.message, "network lost");
  await read.advance(transport, () => clock);
  assert.equal(keys.length, 1);
  read.retry();
  await read.advance(transport, () => clock);
  assert.equal(keys[0], keys[1]);
  clock += 1500;
  await read.advance(transport, () => clock);
  assert.equal(read.snapshot().error.message, "status unavailable");
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

test("canonical player and hosting routes reject retired aliases and respect authority", async () => {
 const [{default:ts},{default:vm},{readFile}]=await Promise.all([import("../node_modules/typescript/lib/typescript.js"),import("node:vm"),import("node:fs/promises")]);
 const context=vm.createContext({URL,location:{origin:"https://ux.fixture"}});
 const module=new vm.SourceTextModule(ts.transpile(await readFile(new URL("../src/routes.ts",import.meta.url),"utf8"),{module:ts.ModuleKind.ESNext}),{context});
 const i18n=await localeModule(context);await module.link(()=>i18n);await module.evaluate();
 const {normalize,resolveRoute,topPages,childPages}=module.namespace;
 assert.equal(normalize(""),"/play");
 for(const old of ["/home","/servers","/chat","/friends","/teams","/parties","/manage/servers"])assert.equal(resolveRoute(old).component,"missing",old);
 assert.equal(resolveRoute("/play").component,"play-hub");assert.equal(resolveRoute("/timeline").component,"timeline");
 assert.equal(topPages().map(p=>p.id).join(","),"play,worlds,people,timeline,hosting,admin");
 assert.equal(topPages()[0].name,"Play");i18n.namespace.setLanguage("ja");assert.equal(topPages()[0].name,"プレイ");i18n.namespace.setLanguage("en");
 const server="00000000-0000-0000-0000-000000000001";
 const logs=resolveRoute(`/hosting/servers/${server}/logs`);assert.equal(logs.section,"manage-logs");assert.equal(logs.api,`/api/v1/servers/${server}?section=manage-console`);
 assert.equal(childPages(logs,{can_administer:false}).some(p=>p.path.endsWith("/files")||p.path.endsWith("/members")),false);
 assert.equal(childPages(logs,{can_administer:true}).some(p=>p.path.endsWith("/files")),true);
 assert.equal(resolveRoute(`/worlds/${server}/economy?tab=storage`).section,"stored-assets");
 assert.equal(resolveRoute(`/worlds/${server}/world?tab=homes`).section,"homes");
 assert.equal(childPages(resolveRoute("/people/teams"),undefined,{name:"Builders"}).map(p=>p.name).join(","),"Builders,Members,Settings");
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
