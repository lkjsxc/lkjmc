// Real Paper private-End lifecycle on the loopback-only development rig.
// Faults terminate the JVM. Only the local test clock and fixture funding bypass gameplay.
import fs from "node:fs/promises";
import { createWriteStream } from "node:fs";
import { spawn, execFileSync } from "node:child_process";
import crypto from "node:crypto";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { setTimeout as sleep } from "node:timers/promises";
import mineflayer from "mineflayer";
import { Vec3 } from "vec3";

const root = fileURLToPath(new URL("../../", import.meta.url)),
  local = path.join(root, ".local/game");
const ids = JSON.parse(await fs.readFile(path.join(local, "ids.json"), "utf8"));
const token = (
  await fs.readFile(path.join(local, "proxy-token"), "utf8")
).trim();
const tag = crypto.randomBytes(3).toString("hex");
const log = createWriteStream(path.join(local, `adventure-${tag}.log`), {
  mode: 0o600,
});
let server,
  lines = [],
  logCursor = 0;
const clients = [];
async function until(test, name, timeout = 180000) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    const value = await test();
    if (value) return value;
    await sleep(150);
  }
  throw new Error("Timeout: " + name);
}
async function api(route, body, allowError = false) {
  const r = await fetch("http://127.0.0.1:18091" + route, {
    method: "POST",
    headers: {
      authorization: "Bearer " + token,
      "content-type": "application/json",
    },
    body: JSON.stringify(body),
  });
  const value = await r.json();
  if (!r.ok && !allowError)
    throw new Error(`${r.status}: ${JSON.stringify(value)}`);
  return allowError ? { status: r.status, value } : value;
}
function native(name) {
  const b = crypto
    .createHash("md5")
    .update("OfflinePlayer:" + name)
    .digest();
  b[6] = (b[6] & 15) | 48;
  b[8] = (b[8] & 63) | 128;
  const h = b.toString("hex");
  return [
    h.slice(0, 8),
    h.slice(8, 12),
    h.slice(12, 16),
    h.slice(16, 20),
    h.slice(20),
  ].join("-");
}
async function start() {
  const offset = lines.length;
  server = spawn("python3", ["scripts/game_dev.py", "official"], {
    cwd: root,
    stdio: ["pipe", "pipe", "pipe"],
  });
  let pending = "";
  const record = (data) => {
    log.write(data);
    pending += data.toString();
    let end;
    while ((end = pending.indexOf("\n")) >= 0) {
      const line = pending.slice(0, end).replace(/\x1b\[[0-9;]*m/g, "");
      pending = pending.slice(end + 1);
      lines.push(line);
      if (/SEVERE|ERROR.*Lkjmc/.test(line)) console.error(line);
    }
  };
  server.stdout.on("data", record);
  server.stderr.on("data", record);
  await until(() => {
    if (server.exitCode !== null)
      throw new Error("Paper exited " + server.exitCode);
    return lines.slice(offset).some((l) => l.includes("lkjmc adapter ready:"));
  }, "Paper ready");
  console.log("Paper ready");
}
async function consoleCommand(command) {
  assert(server && server.exitCode === null);
  const start = lines.length,
    marker = "TEST_ACK_" + ++logCursor;
  server.stdin.write(command + "\n");
  await sleep(30);
  server.stdin.write("say " + marker + "\n");
  await until(
    () => lines.slice(start).some((l) => l.includes(marker)),
    command,
    30000,
  );
  return lines.slice(start);
}
async function connect(name, expected = "minecraft:living") {
  const session = await api("/internal/v1/game/connect", {
    issuer: "java",
    subject: native(name),
    native_uuid: native(name),
    display_name: name,
    session_id: crypto.randomUUID(),
  });
  const heartbeat = () =>
    api("/internal/v1/game/heartbeat", { ...session, server_id: ids.official });
  await heartbeat();
  const timer = setInterval(() => heartbeat().catch(() => {}), 10000);
  const bot = mineflayer.createBot({
    host: "127.0.0.1",
    port: 25691,
    username: name,
    version: "26.1",
    auth: "offline",
    checkTimeoutInterval: 120000,
  });
  const entry = { name, session, bot, timer, world: null, messages: [] };
  clients.push(entry);
  entry.spawns = 0;
  entry.positions = [];
  bot.on("spawn", () => entry.spawns++);
  bot._client.on("position", (p) =>
    entry.positions.push({ world: entry.world, x: p.x, y: p.y, z: p.z }),
  );
  for (const packet of ["login", "respawn"])
    bot._client.on(packet, (p) => (entry.world = p.worldState.name));
  bot.on("messagestr", (m) => entry.messages.push(m));
  bot.on("error", (e) => console.error(name, e.message));
  bot.on("kicked", (e) => console.error(name, "kicked", JSON.stringify(e)));
  bot.on("end", () => {
    clearInterval(timer);
  });
  await until(() => bot.entity && entry.world === expected, name + " spawn");
  return entry;
}
async function disconnect(entry) {
  clearInterval(entry.timer);
  entry.bot.quit();
  await api("/internal/v1/game/disconnect", entry.session).catch(() => {});
}
async function view(entry, name, query = {}) {
  return api("/internal/v1/game/view", { ...entry.session, view: name, query });
}
async function submit(entry, command) {
  return (
    await api("/internal/v1/game/command", {
      ...entry.session,
      request_id: crypto.randomUUID(),
      command,
    })
  ).result;
}
async function done(entry, result) {
  if (!result.job_id) return result;
  return until(async () => {
    const job = await view(entry, "job", { id: result.job_id });
    if (job.state === "failed") throw new Error(job.error);
    return job.state === "succeeded" ? job.result : false;
  }, "job " + result.job_id);
}
async function command(entry, body) {
  return done(entry, await submit(entry, body));
}
async function tp(entry, x, y, z) {
  await consoleCommand(
    `execute in minecraft:living run tp ${entry.name} ${x} ${y} ${z}`,
  );
  await until(
    () => entry.bot.entity.position.distanceTo(new Vec3(x, y, z)) < 1,
    "teleport",
  );
  await sleep(300);
}
async function give(entry, name, amount) {
  await consoleCommand(`give ${entry.name} minecraft:${name} ${amount}`);
  await until(
    () =>
      entry.bot.inventory
        .items()
        .some((i) => i.name === name && i.count >= amount),
    "receive " + name,
  );
}

function sql(statement) {
  return execFileSync(
    "docker",
    [
      "exec",
      "lkjmc-rebuild-dev-postgres",
      "psql",
      "-U",
      "lkjmc",
      "-d",
      "lkjmc_rebuild",
      "-XAt",
      "-v",
      "ON_ERROR_STOP=1",
      "-c",
      statement,
    ],
    { encoding: "utf8" },
  ).trim();
}
function fund(entry) {
  const account = entry.session.account_id,
    transaction = crypto.randomUUID();
  assert.match(account, /^[0-9a-f-]{36}$/);
  sql(
    `BEGIN; INSERT INTO ledger(id,reference,kind,actor,detail) VALUES('${transaction}','adventure-fixture:${transaction}','fixture','${account}','{}'); UPDATE wallets SET balance=balance+6000 WHERE owner='${account}'; INSERT INTO ledger_entries(transaction_id,owner,amount,balance_after) SELECT '${transaction}',owner,6000,balance FROM wallets WHERE owner='${account}'; COMMIT;`,
  );
}
function balance(entry) {
  return JSON.parse(
    sql(
      `SELECT json_build_object('balance',balance,'reserved',reserved) FROM wallets WHERE owner='${entry.session.account_id}'`,
    ),
  );
}
function adventure(id) {
  return JSON.parse(
    sql(`SELECT row_to_json(a) FROM adventures a WHERE id='${id}'`),
  );
}
function count(entry, item) {
  return entry.bot.inventory
    .items()
    .filter((i) => i.name === item)
    .reduce((n, i) => n + i.count, 0);
}
async function arm(boundary) {
  await fs.writeFile(
    path.join(local, "official/plugins/Lkjmc/test-crash-once"),
    boundary,
    { mode: 0o600 },
  );
}
async function crashed() {
  await until(() => server.exitCode !== null, "JVM crash");
  assert.equal(server.exitCode, 86);
}
async function restart(a, b) {
  await disconnect(a);
  await disconnect(b);
  await start();
  return [await connect(a.name), await connect(b.name)];
}

try {
  await start();
  let a = await connect("EndA" + tag),
    b = await connect("EndB" + tag);
  fund(a);
  fund(b);
  await give(a, "ender_eye", 48);
  await give(a, "diamond", 3);
  await arm("adventure.eyes_removed");
  const first = await submit(a, { type: "adventure_create" });
  await crashed();
  [a, b] = await restart(a, b);
  await done(a, first);
  assert.equal(count(a, "ender_eye"), 36);
  assert.deepEqual(balance(a), { balance: 5000, reserved: 0 });
  let record = adventure(first.adventure_id);
  assert.equal(
    (Date.parse(record.expires_at) - Date.parse(record.opens_at)) / 1000,
    10800,
  );
  await command(a, { type: "adventure_join", id: first.adventure_id });
  await until(
    () => a.world === `minecraft:adventure_${first.adventure_id}`,
    "actual private End entry",
  );
  assert.equal(count(a, "diamond"), 3);
  await consoleCommand(
    `execute in minecraft:adventure_${first.adventure_id} run tp ${b.name} 100.5 50 0.5`,
  );
  assert.equal(
    b.world,
    "minecraft:living",
    "unregistered players cannot teleport into someone else's adventure",
  );
  const duplicate = await api(
    "/internal/v1/game/command",
    {
      ...a.session,
      request_id: crypto.randomUUID(),
      command: { type: "adventure_create" },
    },
    true,
  );
  assert.equal(duplicate.status, 409);
  const positions = a.positions.length;
  sql(
    `UPDATE adventures SET expires_at=now()-interval '1 second' WHERE id='${first.adventure_id}'`,
  );
  await until(
    () => a.world === "minecraft:living",
    "expiry returns without a shared spawn",
  );
  await until(
    () => adventure(first.adventure_id).state === "closed",
    "expired world retired",
  );
  assert(
    a.positions.slice(positions).some((p) => p.world === "minecraft:overworld"),
    "fallback went through private holding",
  );
  assert(
    a.positions
      .slice(positions)
      .filter((p) => p.world === "minecraft:living")
      .every((p) => Math.hypot(p.x, p.z) > 10000),
  );
  assert.equal(count(a, "diamond"), 3);
  console.log(
    "PASS physical End creation, cost recovery, admission, inventory, accelerated expiry",
  );

  await arm("adventure.world_ready");
  const second = await submit(a, { type: "adventure_create" });
  await crashed();
  // The verified game session is still leased in Core during this deliberate short crash.
  const cancellation = await submit(a, {
    type: "adventure_cancel",
    id: second.adventure_id,
  });
  [a, b] = await restart(a, b);
  await done(a, cancellation);
  record = adventure(second.adventure_id);
  assert.equal(record.state, "refunded");
  assert(record.material_asset);
  assert.equal(count(a, "ender_eye"), 24);
  assert.deepEqual(balance(a), { balance: 5000, reserved: 0 });
  await command(a, { type: "asset_receive", id: record.material_asset });
  await until(() => count(a, "ender_eye") === 36, "receive refunded eyes once");
  const twice = await api(
    "/internal/v1/game/command",
    {
      ...a.session,
      request_id: crypto.randomUUID(),
      command: { type: "asset_receive", id: record.material_asset },
    },
    true,
  );
  assert.equal(twice.status, 409);
  await assert.rejects(
    fs.access(
      path.join(
        local,
        `official/holding/dimensions/minecraft/adventure_${second.adventure_id}`,
      ),
    ),
  );
  console.log(
    "PASS world-ready crash then cancellation: coins released, exact eyes in one escrow",
  );

  const x = Math.floor(a.bot.entity.position.x),
    z = Math.floor(a.bot.entity.position.z),
    y = 180;
  await consoleCommand(
    `execute in minecraft:living run fill ${x - 3} ${y - 1} ${z - 3} ${x + 4} ${y - 1} ${z + 3} stone`,
  );
  await consoleCommand(`lkjmcfixture bed ${x} ${y} ${z} ${a.name}`);
  const third = await submit(a, { type: "adventure_create" });
  await done(a, third);
  await command(a, { type: "adventure_join", id: third.adventure_id });
  await until(
    () => a.world === `minecraft:adventure_${third.adventure_id}`,
    "third End entry",
  );
  await disconnect(a);
  sql(
    `UPDATE adventures SET expires_at=now()-interval '1 second' WHERE id='${third.adventure_id}'`,
  );
  await until(
    () => adventure(third.adventure_id).state === "closed",
    "offline expiry",
  );
  a = await connect(a.name);
  await until(
    () =>
      Math.hypot(a.bot.entity.position.x - x, a.bot.entity.position.z - z) < 8,
    "valid bed after offline expiry",
  );
  assert.equal(count(a, "diamond"), 3);
  assert.deepEqual(balance(a), { balance: 4000, reserved: 0 });
  console.log(
    "PASS offline expiry invalidates saved End position and returns to a valid bed",
  );
  await fs.writeFile(
    path.join(local, `adventure-${tag}-result.json`),
    JSON.stringify(
      {
        tag,
        first,
        second,
        third,
        checks: [
          "eyes_removed_crash",
          "world_ready_crash_cancel",
          "escrow_refund_once",
          "membership",
          "normal_inventory",
          "accelerated_expiry",
          "offline_bed_return",
        ],
      },
      null,
      2,
    ),
    { mode: 0o600 },
  );
} finally {
  for (const c of clients) await disconnect(c).catch(() => {});
  if (server && server.exitCode === null) {
    server.stdin.write("stop\n");
    await until(() => server.exitCode !== null, "Paper shutdown", 60000).catch(
      () => server.kill("SIGTERM"),
    );
  }
  log.end();
}
