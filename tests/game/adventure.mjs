// Real Paper Expedition lifecycle on the loopback-only development rig.
// Faults terminate the JVM. Only the local test clock and fixture funding bypass gameplay.
import fs from "node:fs/promises";
import { createWriteStream } from "node:fs";
import { spawn } from "node:child_process";
import crypto from "node:crypto";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { setTimeout as sleep } from "node:timers/promises";
import mineflayer from "mineflayer";
import { Vec3 } from "vec3";
import { protocolDatabase, protocolSql } from "./scope.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url)),
  local = path.join(root, ".local/game");
const databaseName = await protocolDatabase(root);
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
  // This fixture verifies crash receipts, not unaided survival while a test
  // worker waits for recovery. Protect only this generated offline actor from
  // incidental hostile mobs; keep survival inventories and all travel/combat
  // admission assertions unchanged. No production world setting is modified.
  assert(/^[A-Za-z0-9_]{3,16}$/.test(name));
  await consoleCommand(`effect give ${name} minecraft:resistance infinite 4 true`);
  const resistance = bot.registry.effectsByName.Resistance.id;
  await until(() => bot.entity.effects[resistance]?.amplifier === 4,
    name + " receipt-fixture protection applied", 30000);
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
    if (job.state === "failed") throw new Error(JSON.stringify(job.error));
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
async function safePad(x, y, z, radius = 2) {
  await consoleCommand(
    `execute in minecraft:living run fill ${x-radius} ${y-1} ${z-radius} ${x+radius} ${y-1} ${z+radius} stone`,
  );
  await consoleCommand(
    `execute in minecraft:living run fill ${x-radius} ${y} ${z-radius} ${x+radius} ${y+3} ${z+radius} air`,
  );
}
async function assertSafeOrigin(entry, expected) {
  const standing = ["stone", "air", "air"];
  const blocks = () => [-1, 0, 1].map((dy) =>
    entry.bot.blockAt(new Vec3(Math.floor(expected.x), Math.floor(expected.y)+dy, Math.floor(expected.z)))?.name,
  );
  await until(
    () => entry.world === "minecraft:living"
      && entry.bot.entity.position.distanceTo(expected) < .25
      && blocks().every((name, i) => name === standing[i]),
    "confirmed solid return pad, clear headroom and settled origin",
    30000,
  );
  assert.deepEqual(blocks(), standing, "Return fixture must have solid floor and clear feet/head before entry");
}
async function assertOriginReturn(entry, expected, message) {
  const near = () => entry.world === "minecraft:living"
    && entry.bot.entity.position.distanceTo(expected) < 8;
  try {
    await until(near, message, 30000);
  } catch (cause) {
    let saved;
    try {
      saved = JSON.parse(await fs.readFile(path.join(local, "official/plugins/Lkjmc/player-locations", native(entry.name)+".json"), "utf8"));
    } catch (error) {
      saved = { read_error: error.message };
    }
    throw new assert.AssertionError({
      message: message+": "+JSON.stringify({ expected, actual: { world: entry.world, position: entry.bot.entity.position }, saved, recent_positions: entry.positions.slice(-8) }),
      expected,
      actual: entry.bot.entity.position,
      operator: "distance < 8 in SMP",
      cause,
    });
  }
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

async function sql(statement) {
  return (await protocolSql(root, statement)).stdout.trim();
}
async function fund(entry) {
  const account = entry.session.account_id, transaction = crypto.randomUUID();
  assert.match(account, /^[0-9a-f-]{36}$/);
  await sql(`BEGIN; INSERT INTO ledger(id,reference,kind,actor,detail) VALUES('${transaction}','expedition-fixture:${transaction}','fixture','${account}','{}'); UPDATE wallets SET balance=balance+6000 WHERE owner='${account}'; INSERT INTO ledger_entries(transaction_id,owner,amount,balance_after) SELECT '${transaction}',owner,6000,balance FROM wallets WHERE owner='${account}'; COMMIT;`);
}
async function balance(entry) {
  return JSON.parse(await sql(`COPY (SELECT json_build_object('balance',balance,'reserved',reserved) FROM wallets WHERE owner='${entry.session.account_id}') TO STDOUT`));
}
async function adventure(id) {
  assert.match(id, /^[0-9a-f-]{36}$/);
  return JSON.parse(await sql(`COPY (SELECT row_to_json(a) FROM adventures a WHERE id='${id}') TO STDOUT`));
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
async function restart(a, b, expectedA = "minecraft:living", expectedB = "minecraft:living") {
  await disconnect(a);
  await disconnect(b);
  await start();
  return [await connect(a.name, expectedA), await connect(b.name, expectedB)];
}

try {
  await start();
  let a = await connect("EndA" + tag),
    b = await connect("EndB" + tag);
  await fund(a);
  await fund(b);
  await give(a, "ender_eye", 48);
  await give(a, "diamond", 3);
  const party = await command(a, { type: "party_create", name: "Expedition " + tag });
  const invitation = await command(a, { type: "invite", kind: "party", resource: party.party_id, target: b.session.account_id });
  await command(b, { type: "invite_respond", id: invitation.id, accept: true });
  await command(a, { type: "party_ready", ready: true });
  await command(b, { type: "party_ready", ready: true });
  await arm("adventure.eyes_removed");
  const first = await submit(a, { type: "expedition_prepare" });
  await crashed();
  [a, b] = await restart(a, b);
  await done(a, first);
  assert.equal(count(a, "ender_eye"), 36);
  assert.deepEqual(await balance(a), { balance: 5000, reserved: 0 });
  let record = await adventure(first.expedition_id);
  assert.equal(
    (Date.parse(record.expires_at) - Date.parse(record.opens_at)) / 1000,
    10800,
  );
  await sql(`UPDATE accounts SET combat_until=now()+interval '30 seconds' WHERE id='${a.session.account_id}'`);
  const combatDenied = await api("/internal/v1/game/command", { ...a.session, request_id:crypto.randomUUID(), command:{type:"expedition_enter",id:first.expedition_id} }, true);
  assert.equal(combatDenied.status,409,"Committed admission does not override combat restrictions");
  await sql(`UPDATE accounts SET combat_until=NULL WHERE id='${a.session.account_id}'`);
  const firstOrigin = a.bot.entity.position.clone();
  await arm("expedition.origin_saved");
  const interruptedEntry = await submit(a, { type: "expedition_enter", id: first.expedition_id });
  await crashed();
  [a, b] = await restart(a, b);
  await until(async () => (await view(a,"job",{id:interruptedEntry.job_id})).state === "failed", "stale entry after origin-only crash fails without movement");
  assert.equal(a.world,"minecraft:living");
  assert(a.bot.entity.position.distanceTo(firstOrigin)<8);
  await arm("expedition.entered");
  const entered = await submit(a, { type: "expedition_enter", id: first.expedition_id });
  await crashed();
  [a, b] = await restart(a, b, `minecraft:adventure_${first.expedition_id}`);
  await done(a, entered);

  await until(
    () => a.world === `minecraft:adventure_${first.expedition_id}`,
    "actual temporary End expedition entry",
  );
  assert.equal(count(a, "diamond"), 3);
  const outsider = await connect("EndC" + tag);
  await consoleCommand(
    `execute in minecraft:adventure_${first.expedition_id} run tp ${outsider.name} 100.5 50 0.5`,
  );
  assert.equal(
    outsider.world,
    "minecraft:living",
    "unregistered players cannot teleport into someone else's adventure",
  );
  const duplicate = await api(
    "/internal/v1/game/command",
    {
      ...a.session,
      request_id: crypto.randomUUID(),
      command: { type: "expedition_prepare" },
    },
    true,
  );
  assert.equal(duplicate.status, 409);
  await command(b, { type: "party_ready", ready: false });
  await command(b, { type: "expedition_enter", id: first.expedition_id });
  await until(() => b.world === `minecraft:adventure_${first.expedition_id}`, "committed participant may enter after becoming unready");
  await command(b, { type: "party_leave" });
  await command(a, { type: "party_leave" });
  await sleep(1200);
  assert.equal(a.world, `minecraft:adventure_${first.expedition_id}`);
  assert.equal(b.world, `minecraft:adventure_${first.expedition_id}`, "Party leave/disband does not revoke expedition membership");
  await command(b, { type: "expedition_return", id: first.expedition_id });
  await until(() => b.world === "minecraft:living", "participant return after party closure");
  await disconnect(outsider);
  await arm("expedition.returned");
  const returned = await submit(a, { type: "expedition_return", id: first.expedition_id });
  await crashed();
  [a, b] = await restart(a, b);
  await done(a, returned);
  await assertOriginReturn(a, firstOrigin, "Return receipt replays without random movement");

  await until(() => a.world === "minecraft:living", "explicit expedition return");
  const nextOrigin = a.bot.entity.position.clone();
  nextOrigin.x = Math.floor(nextOrigin.x) + 32;
  nextOrigin.y = Math.floor(nextOrigin.y) + 8;
  nextOrigin.z = Math.floor(nextOrigin.z);
  await safePad(nextOrigin.x,nextOrigin.y,nextOrigin.z);
  await tp(a,nextOrigin.x+.5,nextOrigin.y,nextOrigin.z+.5);
  await assertSafeOrigin(a, new Vec3(nextOrigin.x+.5,nextOrigin.y,nextOrigin.z+.5));
  const reentryOrigin = a.bot.entity.position.clone();
  await command(a, { type: "expedition_enter", id: first.expedition_id });
  await until(() => a.world === `minecraft:adventure_${first.expedition_id}`, "expedition re-entry");
  await sql(
    `UPDATE adventures SET expires_at=now()-interval '1 second' WHERE id='${first.expedition_id}'`,
  );
  await until(
    () => a.world === "minecraft:living",
    "expiry restores the recorded entry origin",
  );
  await until(
    async () => (await adventure(first.expedition_id)).state === "closed",
    "expired world retired",
  );
  await assertOriginReturn(a, reentryOrigin, "Expiry restores this entry's origin without random displacement");
  assert.equal(count(a, "diamond"), 3);
  console.log(
    "PASS physical End creation, cost recovery, admission, inventory, accelerated expiry",
  );

  await arm("adventure.world_ready");
  const second = await submit(a, { type: "expedition_prepare" });
  await crashed();
  // The verified game session is still leased in Core during this deliberate short crash.
  const cancellation = await submit(a, {
    type: "expedition_cancel",
    id: second.expedition_id,
  });
  [a, b] = await restart(a, b);
  await done(a, cancellation);
  record = await adventure(second.expedition_id);
  assert.equal(record.state, "refunded");
  assert(record.material_asset);
  assert.equal(count(a, "ender_eye"), 24);
  assert.deepEqual(await balance(a), { balance: 5000, reserved: 0 });
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
        `official/holding/dimensions/minecraft/adventure_${second.expedition_id}`,
      ),
    ),
  );
  console.log(
    "PASS world-ready crash then cancellation: coins released, exact eyes in one escrow",
  );

  const x = Math.floor(a.bot.entity.position.x),
    z = Math.floor(a.bot.entity.position.z),
    y = 180;
  await safePad(x,y,z,4);
  await consoleCommand(`lkjmcfixture bed ${x} ${y} ${z} ${a.name}`);
  const third = await submit(a, { type: "expedition_prepare" });
  await done(a, third);
  await command(a, { type: "expedition_enter", id: third.expedition_id });
  await until(
    () => a.world === `minecraft:adventure_${third.expedition_id}`,
    "third End entry",
  );
  await disconnect(a);
  await sql(
    `UPDATE adventures SET expires_at=now()-interval '1 second' WHERE id='${third.expedition_id}'`,
  );
  await until(
    async () => (await adventure(third.expedition_id)).state === "closed",
    "offline expiry",
  );
  a = await connect(a.name);
  await until(
    () =>
      Math.hypot(a.bot.entity.position.x - x, a.bot.entity.position.z - z) < 8,
    "recorded entry origin after offline expiry",
  );
  assert.equal(count(a, "diamond"), 3);
  assert.deepEqual(await balance(a), { balance: 4000, reserved: 0 });
  console.log(
    "PASS offline expiry invalidates saved End position and restores the entry origin",
  );
  const unsafeOrigin = a.bot.entity.position.clone();
  const fourth = await submit(a,{type:"expedition_prepare"});
  await done(a,fourth);
  await command(a,{type:"expedition_enter",id:fourth.expedition_id});
  await until(() => a.world === `minecraft:adventure_${fourth.expedition_id}`, "unsafe-origin expedition entry");
  await consoleCommand(`execute in minecraft:living run fill ${Math.floor(unsafeOrigin.x)-2} ${Math.floor(unsafeOrigin.y)-1} ${Math.floor(unsafeOrigin.z)-2} ${Math.floor(unsafeOrigin.x)+2} ${Math.floor(unsafeOrigin.y)-1} ${Math.floor(unsafeOrigin.z)+2} air`);
  await sql(`UPDATE adventures SET expires_at=now()-interval '1 second' WHERE id='${fourth.expedition_id}'`);
  await until(() => a.world === "minecraft:living", "safe respawn fallback when origin is destroyed");
  await until(async () => (await adventure(fourth.expedition_id)).state === "closed", "unsafe-origin world retired");
  assert(Math.hypot(a.bot.entity.position.x-x,a.bot.entity.position.z-z)<8);
  assert(Math.abs(a.bot.entity.position.y-y)<3,"Unsafe origin uses the valid respawn area, not a new random start");
  assert.equal(count(a,"diamond"),3);
  console.log("PASS unsafe origin uses the validated respawn area without random displacement");
  await fs.writeFile(
    path.join(local, `adventure-${tag}-result.json`),
    JSON.stringify(
      {
        tag,
        database: databaseName,
        first,
        second,
        third,
        fourth,
        checks: [
          "eyes_removed_crash",
          "world_ready_crash_cancel",
          "escrow_refund_once",
          "membership",
          "normal_inventory",
          "accelerated_expiry",
          "explicit_return",
          "latest_entry_origin",
          "committed_party_admission",
          "origin_saved_crash_session_gate",
          "entered_crash_receipt",
          "returned_crash_receipt",
          "offline_origin_return",
          "unsafe_origin_respawn_fallback",
          "combat_gate",
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
