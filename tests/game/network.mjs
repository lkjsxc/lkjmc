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
const root = fileURLToPath(new URL("../../", import.meta.url)),
  local = path.join(root, ".local/game");
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
  return api("/internal/v1/game/profile/" + c.bot.player.uuid);
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
try {
  const [official, lobby] = await Promise.all([
    start("official"),
    start("lobby"),
  ]);
  const proxy = await start("proxy");
  const a = connect("NetA" + tag);
  await until(() => a.bot.entity, "first client");
  await until(
    async () => (await session(a)).server_id === ids.lobby,
    "initial lobby",
  );
  assert(Math.hypot(a.bot.entity.position.x, a.bot.entity.position.z) < 30);
  console.log(
    "PASS initial login always enters lobby through modern forwarding",
  );
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
  await promisify(execFile)("docker", [
    "exec",
    "lkjmc-rebuild-dev-postgres",
    "psql",
    "-U",
    "lkjmc",
    "-d",
    "lkjmc_rebuild",
    "-X",
    "-q",
    "-c",
    `BEGIN; UPDATE game_sessions SET combat_until=NULL WHERE account_id IN ('${sa.account_id}','${sb.account_id}'); UPDATE accounts SET combat_until=NULL WHERE id IN ('${sa.account_id}','${sb.account_id}'); COMMIT;`,
  ]);
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
  assert(a.messages.some((m) => m.includes("PvP直後")));
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
  await promisify(execFile)("docker", [
    "exec",
    "lkjmc-rebuild-dev-postgres",
    "psql",
    "-U",
    "lkjmc",
    "-d",
    "lkjmc_rebuild",
    "-X",
    "-q",
    "-c",
    `UPDATE game_sessions SET combat_until=now()+interval '30 seconds' WHERE account_id='${sb.account_id}'`,
  ]);
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
