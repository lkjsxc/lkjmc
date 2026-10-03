// Runs a private offline fixture through the real Velocity -> modern forwarding -> Paper path.
// Public Java authentication and Bedrock clients are separate acceptance checks.
import fs from "node:fs/promises";
import { createWriteStream } from "node:fs";
import { spawn, execFile } from "node:child_process";
import { promisify } from "node:util";
import crypto from "node:crypto";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { setTimeout as sleep } from "node:timers/promises";
import mineflayer from "mineflayer";
import { protocolDatabase, protocolSql } from "./scope.mjs";
import { launcherChecks, sleepingJoinChecks, failedJoinChecks, timeoutJoinChecks } from "./menu-join.mjs";
const root = fileURLToPath(new URL("../../", import.meta.url)),
  local = path.join(root, ".local/game");
const databaseName = await protocolDatabase(root);
const ids = JSON.parse(await fs.readFile(path.join(local, "ids.json"), "utf8"));
const token = (
  await fs.readFile(path.join(local, "proxy-token"), "utf8")
).trim();
const tag = crypto.randomBytes(3).toString("hex"),
  processes = [],
  clients = [];
async function until(test, label, timeout = 180000) {
  const end = Date.now() + timeout;
  while (Date.now() < end) {
    const value = await test();
    if (value) return value;
    await sleep(100);
  }
  throw new Error("Timeout: " + label);
}
async function api(route, body) {
  const r = await fetch("http://127.0.0.1:18091" + route, {
    method: body ? "POST" : "GET",
    headers: {
      authorization: "Bearer " + token,
      "content-type": "application/json",
    },
    body: body ? JSON.stringify(body) : undefined,
  });
  const v = await r.json();
  if (!r.ok) throw new Error(r.status + ": " + JSON.stringify(v));
  return v;
}
async function start(role) {
  const p = spawn("python3", ["scripts/game_dev.py", role], {
    cwd: root,
    stdio: ["pipe", "pipe", "pipe"],
  });
  processes.push(p);
  p.lines = [];
  let pending = "";
  const log = createWriteStream(
    path.join(local, `network-${tag}-${role}.log`),
    { mode: 0o600 },
  );
  const record = (b) => {
    log.write(b);
    pending += b.toString();
    let end;
    while ((end = pending.indexOf("\n")) >= 0) {
      const line = pending.slice(0, end).replace(/\x1b\[[0-9;]*m/g, "");
      pending = pending.slice(end + 1);
      p.lines.push(line);
      if (/ERROR|SEVERE/.test(line)) console.error(role, line);
    }
  };
  p.stdout.on("data", record);
  p.stderr.on("data", record);
  p.on("close", () => log.end());
  await until(() => {
    if (p.exitCode !== null) throw new Error(role + " exited " + p.exitCode);
    return p.lines.some((l) =>
      l.includes(
        role === "proxy" ? "lkjmc proxy ready" : "lkjmc adapter ready:",
      ),
    );
  }, role + " ready");
  console.log(role + " ready");
  return p;
}
function connect(name, port = 25693) {
  const bot = mineflayer.createBot({
    host: "127.0.0.1",
    port,
    username: name,
    auth: "offline",
    version: "26.1",
    hideErrors: false,
  });
  const c = {
    name,
    bot,
    messages: [],
    positions: [],
    world: null,
    ended: false,
  };
  clients.push(c);
  for (const packet of ["login", "respawn"])
    bot._client.on(packet, (p) => (c.world = p.worldState.name));
  bot._client.on("position", (p) =>
    c.positions.push({ world: c.world, x: p.x, y: p.y, z: p.z }),
  );
  bot.on("messagestr", (m) => c.messages.push(m));
  bot.on("end", () => (c.ended = true));
  bot.on("kicked", (reason) => (c.kicked = reason));
  bot.on("error", (error) => (c.error = error.message));
  return c;
}
async function session(c) {
  // Login Success carries the native UUID before the player-list projection.
  // The latter can arrive later on a constrained offline fixture.
  const native = c.bot._client.uuid;
  assert(native, "The protocol login has not supplied a native UUID");
  return api("/internal/v1/game/profile/" + native);
}
async function submit(c, command) {
  const s = await session(c);
  return (
    await api("/internal/v1/game/command", {
      ...s,
      request_id: crypto.randomUUID(),
      command,
    })
  ).result;
}
async function job(c, id) {
  return api("/internal/v1/game/view", {
    ...(await session(c)),
    view: "job",
    query: { id },
  });
}
async function move(c, target) {
  const request = await submit(c, { type: "server_join", id: target });
  const done = await until(async () => {
    const j = await job(c, request.job_id);
    return ["succeeded", "failed"].includes(j.state) ? j : false;
  }, "move " + c.name);
  assert.equal(done.state, "succeeded", JSON.stringify(done));
  await until(
    async () => (await session(c)).server_id === target,
    "actual server " + target,
  );
}
async function consoleCommand(p, line) {
  p.stdin.write(line + "\n");
  await sleep(250);
}
async function fixtureSql(sql) {
  await protocolSql(root, sql);
}
async function reconnect(c) {
  const native = c.bot._client.uuid;
  c.bot.quit();
  await until(() => c.ended, "fixture disconnect");
  await until(async () => {
    const projection = await api("/internal/v1/projection");
    return !projection.sessions.some((s) => s.native_uuid === native);
  }, "original session disconnected");
  // Respect Velocity's ordinary login throttle during repeated fixture logins.
  await sleep(3500);
  const next = connect(c.name);
  await until(() => {
    assert(!next.ended, "fixture reconnect ended: " + JSON.stringify(next.kicked ?? next.error));
    return next.bot.entity;
  }, "fixture reconnect", 30000);
  await until(async () => (await session(next)).server_id === ids.lobby, "reconnected lobby");
  return next;
}
try {
  // First learn the real adapter capability. An unobserved registration cannot
  // assert proxy compatibility merely because the fixture will eventually run Paper.
  const initialOfficial = await start("official");
  await until(async () => (await api("/internal/v1/projection")).servers.some(s => s.id === ids.official && s.capabilities?.proxy_join), "observed Paper forwarding capability");
  initialOfficial.stdin.write("stop\n");
  await until(() => initialOfficial.exitCode !== null, "initial Paper saved and stopped");
  const lobby = await start("lobby");
  const proxy = await start("proxy");
  let a = connect("NetA" + tag);
  await until(() => a.bot.entity, "first client");
  await until(
    async () => (await session(a)).server_id === ids.lobby,
    "initial lobby",
  );
  assert(Math.hypot(a.bot.entity.position.x, a.bot.entity.position.z) < 30);
  console.log(
    "PASS initial login always enters lobby through modern forwarding",
  );
  const menu = (title) =>
    until(
      () =>
        a.bot.currentWindow &&
        JSON.stringify(a.bot.currentWindow.title).includes(title),
      "menu " + title,
      15000,
    );
  a.bot.chat("/menu");
  await menu("lkjmc");
  assert.equal(a.bot.currentWindow.slots[10]?.name, "compass");
  assert.equal(a.bot.currentWindow.slots[53]?.name, "barrier");
  for (const [slot, title, icon] of [
    [11, "Friends", "player_head"],
    [12, "Chat", "writable_book"],
    [13, "Teams", "white_banner"],
    [14, "Parties", "campfire"],
  ]) {
    assert.equal(a.bot.currentWindow.slots[slot]?.name, icon);
    await a.bot.clickWindow(slot, 0, 0);
    await menu(title);
    if (title === "Teams")
      assert(
        !JSON.stringify(a.bot.currentWindow.slots.slice(0, 45)).includes(
          "Create party",
        ),
      );
    if (title === "Parties")
      assert(
        !JSON.stringify(a.bot.currentWindow.slots.slice(0, 45)).includes(
          "Create team",
        ),
      );
    await a.bot.clickWindow(49, 0, 0);
    await menu("lkjmc");
  }

  await a.bot.clickWindow(19, 0, 0);
  await menu("Language");
  await a.bot.clickWindow(11, 0, 0);
  await until(
    async () => (await session(a)).language === "ja",
    "game language saved",
  );
  await menu("lkjmc");
  await a.bot.clickWindow(19, 0, 0);
  await menu("言語");
  await a.bot.clickWindow(10, 0, 0);
  await until(
    async () => (await session(a)).language === "en",
    "English restored",
  );
  await menu("lkjmc");
  await a.bot.clickWindow(10, 0, 0);
  await menu("Servers");
  assert(a.bot.currentWindow.slots[45]?.name === "arrow");
  await a.bot.clickWindow(45, 0, 0);
  await menu("lkjmc");
  await a.bot.clickWindow(53, 0, 0);
  await until(() => !a.bot.currentWindow, "menu closed");
  console.log(
    "PASS real inventory navigation and shared English/Japanese preference",
  );
  a = await launcherChecks(a, { until, consoleCommand, lobby, reconnect });
  const sleeping = await sleepingJoinChecks(a, { until, submit, job, session, ids, fixtureSql, reconnect,
    startOfficial: () => start("official") });
  a = sleeping.client;
  const official = sleeping.official;
  await failedJoinChecks(a, { fixtureSql, submit, job, until, session, ids, move });
  await timeoutJoinChecks(a, { fixtureSql, submit, job, until, session, ids, move });
  const spoof = connect(a.name, 25691);
  await until(() => spoof.ended, "direct backend rejected", 20000);
  assert(spoof.kicked || spoof.error);
  console.log("PASS direct backend login cannot bypass proxy identity");
  a.bot.chat("/servers");
  await until(
    () => a.messages.some((m) => m.includes("official development")),
    "clickable server list",
  );
  await move(a, ids.official);
  await until(() => a.world === "minecraft:living", "distant initial spawn");
  const first = { ...a.bot.entity.position };
  assert(Math.hypot(first.x, first.z) > 10000);
  assert(
    a.positions
      .filter((p) => p.world === "minecraft:living")
      .every((p) => Math.hypot(p.x, p.z) > 10000),
  );
  console.log(
    "PASS lobby -> isolated waiting -> distant SMP, without default living spawn",
  );
  a.bot.chat("/hub");
  await until(
    async () => (await session(a)).server_id === ids.lobby,
    "signed native departure and hub",
  );
  console.log(
    "PASS SMP departure is saved and confirmed by actual lobby arrival",
  );
  await move(a, ids.official);
  await until(() => a.world === "minecraft:living", "SMP return");
  assert(
    Math.hypot(
      a.bot.entity.position.x - first.x,
      a.bot.entity.position.z - first.z,
    ) < 2,
  );
  console.log("PASS SMP reentry retains its last position");
  const b = connect("NetB" + tag);
  await until(() => b.bot.entity, "second client");
  await until(
    async () => (await session(b)).server_id === ids.lobby,
    "second lobby",
  );
  await move(b, ids.official);
  await until(() => b.world === "minecraft:living", "second distant spawn");
  assert(
    Math.hypot(
      b.bot.entity.position.x - first.x,
      b.bot.entity.position.z - first.z,
    ) >= 10000,
  );
  await consoleCommand(
    official,
    `execute in minecraft:living run tp ${b.name} ${a.name}`,
  );
  await until(
    () =>
      Math.hypot(
        a.bot.entity.position.x - b.bot.entity.position.x,
        a.bot.entity.position.z - b.bot.entity.position.z,
      ) < 2,
    "test players meet intentionally",
  );
  await consoleCommand(
    official,
    `damage ${a.name} 1 minecraft:player_attack by ${b.name}`,
  );
  // Let the authenticated combat event settle, then model a delayed/missing Core projection.
  await until(
    async () => Boolean((await session(a)).combat_until),
    "combat event persisted",
  );
  const sa = await session(a),
    sb = await session(b);
  await fixtureSql(`BEGIN; UPDATE game_sessions SET combat_until=NULL WHERE account_id IN ('${sa.account_id}','${sb.account_id}'); UPDATE accounts SET combat_until=NULL WHERE id IN ('${sa.account_id}','${sb.account_id}'); COMMIT;`);
  const denied = await submit(a, { type: "server_join", id: ids.lobby });
  const rejected = await until(
    async () => {
      const j = await job(a, denied.job_id);
      return j.state === "failed" ? j : false;
    },
    "native combat departure rejection",
    15000,
  );
  assert.equal((await session(a)).server_id, ids.official);
  assert(a.messages.some((m) => m.includes("after PvP")));
  console.log(
    "PASS native combat gate blocks server transfer even when Core combat data is delayed",
  );
  const saved = { ...a.bot.entity.position };
  a.bot.quit();
  await until(() => a.ended, "disconnect");
  await sleep(500);
  const again = connect(a.name);
  await until(() => again.bot.entity, "reconnect");
  await until(
    async () => (await session(again)).server_id === ids.lobby,
    "reconnect lobby",
  );
  assert(
    Math.hypot(again.bot.entity.position.x, again.bot.entity.position.z) < 30,
  );
  console.log(
    "PASS network reconnect returns to lobby, not the previous backend",
  );
  await fixtureSql(`UPDATE game_sessions SET combat_until=now()+interval '30 seconds' WHERE account_id='${sb.account_id}'`);
  await consoleCommand(official, `kick ${b.name} integration_backend_recovery`);
  await until(
    async () => (await session(b)).server_id === ids.lobby,
    "forced backend recovery reaches lobby",
  );
  assert(!b.ended);
  console.log(
    "PASS backend disconnect returns to lobby even during the combat cooldown",
  );
  await fs.writeFile(
    path.join(local, `network-${tag}-result.json`),
    JSON.stringify(
      {
        tag,
        first,
        saved,
        positions: a.positions,
        combatJob: rejected.id,
        offlineFixture: true,
      },
      null,
      2,
    ),
    { mode: 0o600 },
  );
} finally {
  for (const c of clients) if (!c.ended) c.bot.quit();
  await sleep(500);
  for (const p of [...processes].reverse())
    if (p.exitCode === null) {
      p.stdin.write("stop\n");
      try {
        await until(() => p.exitCode !== null, "server stop", 40000);
      } catch (e) {
        p.kill("SIGTERM");
      }
    }
}
