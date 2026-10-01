// Real Paper/WorldEdit/native NBT test, on the explicit loopback-only development rig.
// The separate console fixture prepares entities and reads the world; it never settles jobs.
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

const root = fileURLToPath(new URL("../../", import.meta.url)),
  local = path.join(root, ".local/game");
const ids = JSON.parse(await fs.readFile(path.join(local, "ids.json"), "utf8"));
const token = (
  await fs.readFile(path.join(local, "proxy-token"), "utf8")
).trim();
const tag = crypto.randomBytes(3).toString("hex");
const injectCrashes = process.env.LKJMC_CRASH_TESTS !== "none";
const log = createWriteStream(path.join(local, `physical-${tag}.log`), {
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
async function connect(name) {
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
  await until(
    () => bot.entity && entry.world === "minecraft:living",
    name + " spawn",
  );
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
async function place(entry, item, x, y, z) {
  await entry.bot.equip(
    entry.bot.inventory.items().find((i) => i.name === item),
    "hand",
  );
  await entry.bot.placeBlock(
    entry.bot.blockAt(new Vec3(x, y - 1, z)),
    new Vec3(0, 1, 0),
  );
  await sleep(200);
}
async function select(entry, which, x, y, z) {
  await entry.bot.lookAt(new Vec3(x + 0.5, y + 0.5, z + 0.5), true);
  await sleep(200);
  const start = entry.messages.length;
  entry.bot.chat("/lkjmc pos" + which);
  const message = await until(
    () => entry.messages.slice(start).find((m) => m.includes(which + "点目:")),
    "select " + which,
    10000,
  );
  console.log("Selection:", message);
  assert(
    message.includes(`${x}, ${y}, ${z}`),
    "the actual selected block must match the intended corner",
  );
}
async function inspect(x, y, z, w = 3, h = 3, l = 3) {
  const output = await consoleCommand(
    `lkjmcfixture inspect ${x} ${y} ${z} ${w} ${h} ${l}`,
  );
  const row = output.find((l) => l.includes("FIXTURE_STATE "));
  if (!row) throw new Error(output.join("\n"));
  return JSON.parse(row.split("FIXTURE_STATE ")[1]);
}
async function arm(boundary) {
  await fs.writeFile(
    path.join(local, "official/plugins/Lkjmc/test-crash-once"),
    boundary,
    { mode: 0o600 },
  );
}
async function crashed() {
  await until(() => server.exitCode !== null, "deliberate JVM halt");
  assert.equal(server.exitCode, 86);
  console.log("PASS deliberate crash boundary reached");
}
async function reconnect(a, b) {
  await disconnect(a);
  await disconnect(b);
  await start();
  return [await connect(a.name), await connect(b.name)];
}

try {
  await start();
  let a = await connect("PackA" + tag),
    b = await connect("PackB" + tag);
  const x = Math.floor(a.bot.entity.position.x / 16) * 16,
    z = Math.floor(a.bot.entity.position.z / 16) * 16,
    y = Math.min(260, Math.max(160, Math.ceil(a.bot.entity.position.y) + 20));
  const claimA = await submit(a, {
    type: "claim_create",
    name: "Physical source " + tag,
    min_x: x / 16,
    min_z: z / 16,
    max_x: x / 16 + 1,
    max_z: z / 16 + 1,
  });
  await done(a, claimA);
  const claimB = await submit(b, {
    type: "claim_create",
    name: "Physical destination " + tag,
    min_x: x / 16 + 3,
    min_z: z / 16,
    max_x: x / 16 + 3,
    max_z: z / 16,
  });
  await done(b, claimB);
  await consoleCommand("execute in minecraft:living run difficulty peaceful");
  await consoleCommand(
    `execute in minecraft:living run fill ${x} ${y - 1} ${z} ${x + 63} ${y - 1} ${z + 15} minecraft:smooth_stone`,
  );
  const bx = x + 6,
    bz = z + 6;
  await tp(a, bx - 1.5, y, bz + 0.5);
  await give(a, "stone", 2);
  await give(a, "cobblestone", 1);
  await give(a, "chest", 1);
  await place(a, "stone", bx, y, bz);
  await place(a, "cobblestone", bx, y, bz + 1);
  await tp(a, bx + 1.5, y, bz - 1.5);
  await place(a, "chest", bx + 1, y, bz);
  await consoleCommand(
    `execute in minecraft:living run item replace block ${bx + 1} ${y} ${bz} container.0 with minecraft:diamond 3`,
  );
  await consoleCommand(
    `execute in minecraft:living run setblock ${bx + 2} ${y} ${bz} minecraft:gold_block`,
  );
  await consoleCommand(
    `execute in minecraft:living run setblock ${bx + 2} ${y + 2} ${bz + 2} minecraft:glass`,
  );
  await consoleCommand(
    `lkjmcfixture entities ${bx} ${y} ${bz} ${native(b.name)}`,
  );
  await tp(a, bx - 1.5, y, bz + 0.5);
  await select(a, 1, bx, y, bz);
  await tp(a, bx + 2.5, y + 3, bz + 5.5);
  await select(a, 2, bx + 2, y + 2, bz + 2);
  await tp(a, bx + 5.5, y, bz + 5.5);
  const before = await inspect(bx, y, bz);
  assert.equal(before.blocks.filter((b) => b.built).length, 3);
  assert.equal(before.entities.length, 3);
  assert.deepEqual(before.blocks.find((b) => b.material === "CHEST").items, [
    { material: "DIAMOND", amount: 3 },
  ]);
  console.log(
    "PASS real manual provenance and native fixture",
    JSON.stringify(before),
  );
  const captured = await submit(a, {
    type: "asset_capture",
    kind: "building",
    title: "Physical house " + tag,
    selection: { claim_id: claimA.claim_id },
    include_contents: true,
  });
  const pending = await until(async () => {
    const job = await view(a, "job", { id: captured.job_id });
    if (job.state === "failed") throw new Error(job.error);
    const market = await view(b, "market");
    return market.assets.find(
      (row) => row.id === captured.asset_id && row.manifest_sha256,
    );
  }, "pet consent manifest");
  assert(pending.manifest.required_consents.includes(b.session.account_id));
  assert.equal((await inspect(bx, y, bz)).entities.length, 3);
  console.log("PASS capture waits for the actual pet owner");
  if (injectCrashes) await arm("building.flushed");
  await submit(b, {
    type: "asset_consent",
    id: captured.asset_id,
    manifest_sha256: pending.manifest_sha256,
  });
  if (injectCrashes) {
    await crashed();
    [a, b] = await reconnect(a, b);
  }
  await done(a, captured);
  const removed = await inspect(bx, y, bz);
  assert.deepEqual(removed.blocks.map((b) => b.material).sort(), [
    "GLASS",
    "GOLD_BLOCK",
  ]);
  assert.equal(removed.entities.length, 0);
  console.log(
    "PASS capture" +
      (injectCrashes ? " crash recovery" : "") +
      " leaves natural blocks and no duplicate entities/items",
  );
  await give(b, "cobblestone", 20);
  await command(b, { type: "npc_sell", material: "COBBLESTONE", amount: 20 });
  const listing = await submit(a, {
    type: "listing_create",
    asset: captured.asset_id,
    price: 10,
  });
  await submit(b, { type: "listing_buy", id: listing.listing_id });
  const destination = {
    claim_id: claimB.claim_id,
    x: x + 50,
    y,
    z: z + 5,
    rotation: 90,
  };
  const preview = await command(b, {
    type: "asset_place",
    id: captured.asset_id,
    placement: { ...destination, preview: true },
  });
  assert(preview.clear);
  assert.equal(preview.rotation, 90);
  if (injectCrashes) await arm("building.flushed");
  const placing = await submit(b, {
    type: "asset_place",
    id: captured.asset_id,
    placement: {
      ...destination,
      preview: false,
      preview_hash: preview.preview_hash,
    },
  });
  if (injectCrashes) {
    await crashed();
    [a, b] = await reconnect(a, b);
  }
  await done(b, placing);
  const placed = await inspect(destination.x, y, destination.z);
  assert.equal(placed.blocks.length, 3);
  assert(placed.blocks.every((row) => row.built));
  assert.deepEqual(
    placed.blocks.find((row) => row.material === "CHEST").items,
    [{ material: "DIAMOND", amount: 3 }],
  );
  assert.equal(placed.entities.length, 3);
  assert.equal(new Set(placed.entities.map((e) => e.marker)).size, 3);
  assert(placed.entities.every((e) => e.marker));
  const villager = placed.entities.find((e) => e.type === "VILLAGER"),
    source = before.entities.find((e) => e.type === "VILLAGER");
  assert.equal(villager.level, source.level);
  assert.equal(villager.xp, source.xp);
  assert.deepEqual(villager.trades, source.trades);
  assert.equal(
    placed.entities.find((e) => e.type === "WOLF").owner,
    native(b.name),
  );
  assert.deepEqual(
    placed.entities.find((e) => e.type === "ARMOR_STAND").equipment,
    [{ material: "GOLDEN_HELMET", amount: 1 }],
  );
  const duplicate = await api(
    "/internal/v1/game/command",
    {
      ...b.session,
      request_id: crypto.randomUUID(),
      command: {
        type: "asset_place",
        id: captured.asset_id,
        placement: {
          ...destination,
          preview: false,
          preview_hash: preview.preview_hash,
        },
      },
    },
    true,
  );
  assert.equal(duplicate.status, 409);
  console.log(
    "PASS placement" +
      (injectCrashes ? " crash recovery" : "") +
      ": exactly one building, storage, villager trades, pet ownership and decoration",
    JSON.stringify(placed),
  );
  // Inventory and its operation nonce share one native player file. Exercise both sides of its save.
  for (const boundary of injectCrashes
    ? ["inventory.prepared", "inventory.flushed", "inventory.committed"]
    : []) {
    await give(b, "cobblestone", 17);
    const balance = (await view(b, "life")).owners.find(
      (o) => o.id === b.session.account_id,
    ).wallet.balance;
    await arm(boundary);
    const selling = await submit(b, {
      type: "npc_sell",
      material: "COBBLESTONE",
      amount: 17,
    });
    await crashed();
    [a, b] = await reconnect(a, b);
    await done(b, selling);
    assert(
      !b.bot.inventory.items().some((i) => i.name === "cobblestone"),
      "a replay must not return sold items",
    );
    assert.equal(
      (await view(b, "life")).owners.find((o) => o.id === b.session.account_id)
        .wallet.balance,
      balance + 17,
      "the sold inventory must be paid exactly once",
    );
    console.log(
      "PASS inventory crash boundary",
      boundary,
      "job",
      selling.job_id,
    );
  }
  await consoleCommand(`lkjmcfixture bed ${x + 10} ${y} ${z + 10} ${a.name}`);
  const oldSpawns = a.spawns;
  await consoleCommand(`kill ${a.name}`);
  await until(
    () =>
      a.spawns > oldSpawns &&
      a.world === "minecraft:living" &&
      Math.hypot(
        a.bot.entity.position.x - (x + 10),
        a.bot.entity.position.z - (z + 10),
      ) < 8,
    "valid bed respawn",
  );
  console.log("PASS valid bed has priority");
  await tp(a, x + 40.5, y, z + 4.5);
  const land = await submit(a, {
    type: "asset_capture",
    kind: "land",
    title: "In-place land " + tag,
    selection: { claim_id: claimA.claim_id },
    include_contents: true,
  });
  await done(a, land);
  const landListing = await submit(a, {
    type: "listing_create",
    asset: land.asset_id,
    price: 10,
  });
  const quota = await api(
    "/internal/v1/game/command",
    {
      ...b.session,
      request_id: crypto.randomUUID(),
      command: { type: "listing_buy", id: landListing.listing_id },
    },
    true,
  );
  assert.equal(quota.status, 409, "purchased land consumes protection quota");
  await command(b, { type: "claim_release", id: claimB.claim_id });
  await submit(b, { type: "listing_buy", id: landListing.listing_id });
  await until(async () => {
    const life = await view(b, "life");
    return life.claims.some(
      (c) => c.id === claimA.claim_id && c.state === "active",
    );
  }, "in-place land ownership sync");
  const previous = a.spawns,
    positionStart = a.positions.length;
  await consoleCommand(`kill ${a.name}`);
  await until(
    () =>
      a.spawns > previous &&
      a.world === "minecraft:living" &&
      Math.hypot(
        a.bot.entity.position.x - (x + 10),
        a.bot.entity.position.z - (z + 10),
      ) >= 10000,
    "sold bed falls back to a distant safe point",
  );
  assert(
    a.positions
      .slice(positionStart)
      .some((p) => p.world === "minecraft:overworld"),
    "the primary holding world must isolate fallback respawns",
  );
  assert(
    a.positions
      .slice(positionStart)
      .filter((p) => p.world === "minecraft:living")
      .every((p) => Math.hypot(p.x, p.z) > 256),
  );
  console.log(
    "PASS sold land invalidates the old bed, without a common default spawn",
  );
  await fs.writeFile(
    path.join(local, `physical-${tag}-result.json`),
    JSON.stringify(
      {
        tag,
        injectCrashes,
        claimA,
        claimB,
        captured,
        placing,
        before,
        removed,
        placed,
        preview,
        soldBedPositions: a.positions.slice(positionStart),
      },
      null,
      2,
    ),
    { mode: 0o600 },
  );
} finally {
  for (const client of clients) await disconnect(client);
  if (server?.exitCode === null) {
    server.stdin.write("stop\n");
    await until(() => server.exitCode !== null, "Paper stops", 45000);
  }
  log.end();
}
process.exit(0);
