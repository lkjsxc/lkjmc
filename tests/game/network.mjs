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
import minecraftProtocol from "minecraft-protocol";
import { protocolDatabase, protocolSql } from "./scope.mjs";
import { playerMenuChecks, smpMenuChecks, launcherChecks, sleepingJoinChecks, failedJoinChecks, timeoutJoinChecks, sameWorldPlayerChecks } from "./menu-join.mjs";
import { teamMenuChecks } from "./teams.mjs";
import { teleportChecks } from "./teleports.mjs";
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
// Bind raw role logs to this ux_verify run before any server starts. The CI
// collector accepts only these exact paths from the current failed acceptance.
const evidence = process.env.LKJMC_TEST_EVIDENCE;
if (evidence !== undefined) {
  assert(/^\.local\/ux\/real-[0-9a-f]{12}$/.test(evidence), "owned protocol evidence directory");
  await fs.writeFile(path.join(root, evidence, "network-logs.json"), JSON.stringify({
    schema: 1,
    fixture_id: evidence.slice(-12),
    network_id: tag,
    started_at: Date.now() / 1000,
    logs: ["official", "lobby", "proxy"].map((role) => `.local/game/network-${tag}-${role}.log`),
  }) + "\n", { mode: 0o600, flag: "wx" });
}
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
    tabEntries: new Map(),
  };
  clients.push(c);
  // A profile existing in Mineflayer's player map is not enough: the wire
  // update_listed flag is what actually includes a player in the Tab UI.
  bot._client.on("player_info", packet => {
    for (const item of packet.data) {
      const entry = c.tabEntries.get(item.uuid) ?? { listed:false };
      if (packet.action.add_player) entry.name = item.player.name;
      if (packet.action.update_listed) entry.listed = item.listed === 1 || item.listed === true;
      c.tabEntries.set(item.uuid, entry);
    }
  });
  bot._client.on("player_remove", packet => {
    for (const uuid of packet.players) c.tabEntries.delete(uuid);
  });
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
  return await protocolSql(root, sql);
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
  const ping = await new Promise((resolve,reject) => minecraftProtocol.ping({host:"127.0.0.1",port:25693},(error,result) => error ? reject(error) : resolve(result)));
  assert.equal(typeof ping.description === "string" ? ping.description : ping.description.text, "A Minecraft Server");
  console.log("PASS public Java MOTD is A Minecraft Server");

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
  await playerMenuChecks(a, { until, session });
  a = await launcherChecks(a, { until, consoleCommand, lobby, reconnect });
  const sleeping = await sleepingJoinChecks(a, { until, submit, job, session, ids, fixtureSql, reconnect,
    startOfficial: () => start("official") });
  a = sleeping.client;
  await smpMenuChecks(a, { until });
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
  const beforeHub = new Set((await api("/internal/v1/game/view", {...(await session(a)),view:"home",query:{}})).jobs.map(job=>job.id));
  a.bot.chat("/hub");
  await until(
    async () => (await session(a)).server_id === ids.lobby,
    "signed native departure and hub",
  );
  // Physical arrival can precede the durable confirmation by one proxy poll.
  // Preserve that admission fence rather than racing the next travel command.
  await until(async () => (await api("/internal/v1/game/view", {...(await session(a)),view:"home",query:{}})).jobs.some(job =>
    !beforeHub.has(job.id) && job.kind==="player.join" && job.state==="succeeded" && job.result?.actual_server_id===ids.lobby), "hub travel has durably confirmed arrival");
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
  await teamMenuChecks(a, { until, session, submit, api, fixtureSql });
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
  await sameWorldPlayerChecks(a,b,{until,session,api});
  const c = connect("NetC" + tag);
  await until(() => c.bot.entity, "third teleport requester");
  await move(c, ids.official);
  await until(() => c.world === "minecraft:living", "third requester in SMP");
  await teleportChecks(a,b,c,{until,api,session,submit,job,consoleCommand,official});
  c.bot.quit();

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
  const combatMessageStart = a.messages.length;
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
  const combatMessageId =
    "text.you_cannot_transfer_servers_for_30_seconds_after_pvp";
  assert.equal(
    rejected.error?.id,
    combatMessageId,
    JSON.stringify({
      error: rejected.error,
      messages: a.messages.slice(combatMessageStart),
    }),
  );
  assert.equal(
    sa.language,
    "en",
    "player menu must restore the selected English language",
  );
  const catalog = JSON.parse(
    await fs.readFile(path.join(root, "locales/en.json"), "utf8"),
  );
  assert.equal(typeof catalog[combatMessageId], "string");
  try {
    await until(
      () =>
        a.messages
          .slice(combatMessageStart)
          .some((m) => m.includes(catalog[combatMessageId])),
      "localized native combat departure rejection notice",
      15000,
    );
  } catch (error) {
    error.message +=
      "\n" +
      JSON.stringify({
        error: rejected.error,
        messages: a.messages.slice(combatMessageStart),
      });
    throw error;
  }
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
