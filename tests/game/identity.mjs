// Real Paper save files and pets, including unloaded chunks and JVM termination.
// Bedrock authentication is a DB/native-UUID fixture; this is not Geyser or console acceptance.
import fs from "node:fs/promises";
import { createWriteStream } from "node:fs";
import { spawn, execFileSync } from "node:child_process";
import crypto from "node:crypto";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { setTimeout as sleep } from "node:timers/promises";
import { gzipSync } from "node:zlib";
import mineflayer from "mineflayer";
import nbt from "prismarine-nbt";

const root = fileURLToPath(new URL("../../", import.meta.url)),
  local = path.join(root, ".local/game");
const ids = JSON.parse(await fs.readFile(path.join(local, "ids.json"), "utf8"));
const token = (
  await fs.readFile(path.join(local, "proxy-token"), "utf8")
).trim();
const tag = crypto.randomBytes(3).toString("hex"),
  log = createWriteStream(path.join(local, `identity-${tag}.log`), {
    mode: 0o600,
  });
const dataRoot = path.join(local, "official/holding/players"),
  plugin = path.join(local, "official/plugins/Lkjmc");
let server,
  lines = [],
  cursor = 0;
const clients = [];
async function until(test, name, ms = 180000) {
  const end = Date.now() + ms;
  while (Date.now() < end) {
    const value = await test();
    if (value) return value;
    await sleep(200);
  }
  throw new Error("Timeout: " + name);
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
async function api(route, body) {
  const r = await fetch("http://127.0.0.1:18091" + route, {
    method: "POST",
    headers: {
      authorization: "Bearer " + token,
      "content-type": "application/json",
    },
    body: JSON.stringify(body),
  });
  const value = await r.json();
  if (!r.ok) throw new Error(r.status + ": " + JSON.stringify(value));
  return value;
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
function uuidInts(uuid) {
  const b = Buffer.from(uuid.replaceAll("-", ""), "hex");
  return [0, 4, 8, 12].map((o) => b.readInt32BE(o));
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
}
async function consoleCommand(command) {
  const offset = lines.length,
    marker = "IDENTITY_ACK_" + ++cursor;
  server.stdin.write(command + "\n");
  await sleep(30);
  server.stdin.write("say " + marker + "\n");
  await until(
    () => lines.slice(offset).some((l) => l.includes(marker)),
    command,
    30000,
  );
  const output = lines.slice(offset);
  assert(!output.some((l) => l.includes("FIXTURE_ERROR")), output.join("\n"));
  return output;
}
async function stop() {
  if (server && server.exitCode === null) {
    server.stdin.write("stop\n");
    await until(() => server.exitCode !== null, "Paper stopped", 60000);
    assert.equal(server.exitCode, 0);
  }
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
  const entry = { name, session, bot, timer, world: null };
  clients.push(entry);
  for (const packet of ["login", "respawn"])
    bot._client.on(packet, (p) => (entry.world = p.worldState.name));
  bot.on("error", (e) => console.error(name, e.message));
  bot.on("kicked", (e) => console.error(name, "kicked", JSON.stringify(e)));
  bot.on("end", () => {
    clearInterval(timer);
    api("/internal/v1/game/disconnect", session).catch(() => {});
  });
  await until(
    () => bot.entity && entry.world === "minecraft:living",
    name + " living spawn",
  );
  return entry;
}
async function disconnect(entry) {
  clearInterval(entry.timer);
  entry.bot.quit();
  await api("/internal/v1/game/disconnect", entry.session);
  await sleep(1000);
}
async function webSession(account) {
  const file = path.join(
    local,
    `identity-${tag}-${account}-${crypto.randomUUID()}.session`,
  );
  execFileSync("python3", ["scripts/dev.py", "dev-session", account, file], {
    cwd: root,
    stdio: ["ignore", "pipe", "pipe"],
  });
  return JSON.parse(await fs.readFile(file, "utf8"));
}
async function webCommand(session, command) {
  const r = await fetch("http://127.0.0.1:18091/api/v1/commands", {
    method: "POST",
    headers: {
      cookie: "lkjmc_session=" + session.token,
      "x-csrf-token": session.csrf,
      origin: "http://127.0.0.1:18091",
      "content-type": "application/json",
    },
    body: JSON.stringify({ request_id: crypto.randomUUID(), command }),
  });
  const v = await r.json();
  if (!r.ok) throw new Error(r.status + ": " + JSON.stringify(v));
  return v.result;
}
async function link(retained, other, selected) {
  const a = await webSession(retained),
    b = await webSession(other);
  const begin = await webCommand(a, { type: "link_begin" });
  await webCommand(b, { type: "link_present", code: begin.code });
  return webCommand(a, {
    type: "link_confirm",
    id: begin.id,
    selected_profile: selected,
  });
}
function job(id) {
  return JSON.parse(sql(`SELECT row_to_json(j) FROM jobs j WHERE id='${id}'`));
}
async function done(id) {
  return until(() => {
    const j = job(id);
    if (j.state === "failed") throw new Error(JSON.stringify(j));
    return j.state === "succeeded" ? j : null;
  }, "identity job complete");
}
async function crash(boundary) {
  await fs.writeFile(path.join(plugin, "test-crash-once"), boundary, {
    mode: 0o600,
  });
}
async function crashed(id, complete = false) {
  await until(() => server.exitCode !== null, "fault killed JVM");
  assert.equal(server.exitCode, 86);
  assert.equal(job(id).state === "succeeded", complete);
  sql(
    `UPDATE jobs SET lease_until=now()-interval '1 second' WHERE id='${id}' AND state='leased'`,
  );
}
async function inspectPet(point) {
  const output = await consoleCommand(
    `lkjmcfixture inspect ${point.x} 160 ${point.z} 1 2 1`,
  );
  const line = output.find((l) => l.includes("FIXTURE_STATE "));
  assert(line);
  return JSON.parse(line.slice(line.indexOf("FIXTURE_STATE ") + 14)).entities;
}
async function inspectPlayer(entry) {
  const output = await consoleCommand(
    `lkjmcfixture inspect_player 0 0 0 ${entry.name}`,
  );
  const line = output.find((l) => l.includes("FIXTURE_PLAYER_STATE "));
  assert(line);
  return JSON.parse(line.slice(line.indexOf("FIXTURE_PLAYER_STATE ") + 21));
}
async function assertPet(point, owner) {
  await until(async () => {
    const entities = await inspectPet(point);
    assert(entities.length <= 1, "Pet duplicated");
    return entities.length === 1 && entities[0].owner === owner;
  }, "pet owner " + owner);
}
function amount(items, material) {
  return items
    .filter((i) => i.material === material)
    .reduce((n, i) => n + i.amount, 0);
}
async function convertFixture(source, bedrock, xuid) {
  // Stopped, local-only rig: take actual Paper's saved source profile and give it a
  // fixture Bedrock UUID. Production identities can only be supplied by the proxy.
  for (const part of ["data", "advancements", "stats"]) {
    for (const suffix of part === "data" ? [".dat", ".dat_old"] : [".json"]) {
      const original = path.join(dataRoot, part, native(source.name) + suffix),
        target = path.join(dataRoot, part, bedrock + suffix);
      let bytes;
      try {
        bytes = await fs.readFile(original);
      } catch (e) {
        if (e.code === "ENOENT") continue;
        throw e;
      }
      if (part === "data") {
        const { parsed } = await nbt.parse(bytes);
        parsed.value.UUID = { type: "intArray", value: uuidInts(bedrock) };
        bytes = gzipSync(nbt.writeUncompressed(parsed));
      }
      await fs.writeFile(target, bytes);
      await fs.unlink(original);
    }
  }
  await fs.rename(
    path.join(plugin, "player-locations", native(source.name) + ".json"),
    path.join(plugin, "player-locations", bedrock + ".json"),
  );
  sql(
    `BEGIN; UPDATE identities SET issuer='bedrock',subject='${xuid}' WHERE issuer='java' AND account_id='${source.session.account_id}'; UPDATE profiles SET native_uuid='${bedrock}' WHERE id='${source.session.profile_id}'; COMMIT;`,
  );
}
function freshWeb(name) {
  return execFileSync("python3", ["scripts/dev.py", "account", name], {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  })
    .trim()
    .split("\n")
    .at(-1);
}
const result = {
  tag,
  scope:
    "loopback Paper; fixture Bedrock identity; no public client acceptance",
  faults: [],
  checks: [],
};
try {
  await start();
  const java = await connect("J" + tag),
    source = await connect("B" + tag);
  const xuid = String(4000000000000000n + BigInt("0x" + tag)),
    bedrock =
      "00000000-0000-0000-" +
      BigInt(xuid).toString(16).padStart(16, "0").slice(0, 4) +
      "-" +
      BigInt(xuid).toString(16).padStart(16, "0").slice(4);
  await consoleCommand(`give ${java.name} minecraft:gold_ingot 11`);
  await consoleCommand(`give ${source.name} minecraft:diamond 17`);
  await consoleCommand(
    `lkjmcfixture player 0 0 0 ${java.name} 2 5 GOLD_INGOT 9`,
  );
  await consoleCommand(
    `lkjmcfixture player 0 0 0 ${source.name} 7 77 EMERALD 3`,
  );
  await consoleCommand(
    `advancement grant ${java.name} only minecraft:husbandry/complete_catalogue`,
  );
  await consoleCommand(
    `advancement grant ${source.name} only minecraft:adventure/arbalistic`,
  );
  const selectedState = await inspectPlayer(source);
  const position = source.bot.entity.position.clone();
  const base = (parseInt(tag, 16) % 3000) * 16 + 400000;
  const pets = [
    { x: base, z: base, owner: bedrock, loaded: true },
    { x: base + 32, z: base, owner: native(java.name), loaded: true },
    { x: base + 64, z: base, owner: bedrock, loaded: false },
    { x: base + 96, z: base, owner: native(java.name), loaded: false },
  ];
  for (const p of pets) {
    await consoleCommand(
      `execute in minecraft:living run setblock ${p.x} 159 ${p.z} minecraft:stone`,
    );
    await consoleCommand(`lkjmcfixture pet ${p.x} 160 ${p.z} ${p.owner}`);
    if (p.loaded)
      await consoleCommand(
        `execute in minecraft:living run forceload add ${p.x} ${p.z}`,
      );
  }
  await disconnect(java);
  await disconnect(source);
  await stop();
  await convertFixture(source, bedrock, xuid);
  const selectedBefore = await fs.readFile(
    path.join(dataRoot, "data", bedrock + ".dat"),
  );
  await start();
  await crash("identity.prepared");
  const migration = await link(
    java.session.account_id,
    source.session.account_id,
    source.session.profile_id,
  );
  await crashed(migration.job_id);
  result.faults.push("identity.prepared");
  for (const boundary of [
    "identity.policy",
    "identity.playerdata",
    "identity.committed",
    "identity.core_ack",
  ]) {
    await crash(boundary);
    await start();
    await crashed(migration.job_id, boundary === "identity.core_ack");
    result.faults.push(boundary);
  }
  await start();
  const receipt = await done(migration.job_id),
    plan = receipt.payload.native_plan;
  assert.equal(
    JSON.parse(
      await fs.readFile(
        path.join(
          plugin,
          "identity-archives/journal",
          migration.job_id + ".json",
        ),
        "utf8",
      ),
    ).published,
    true,
  );
  assert.equal(receipt.result.native_uuid, native(java.name));
  assert.equal(plan.selected, bedrock);
  const manifest = JSON.parse(
    await fs.readFile(
      path.join(plugin, "identity-archives", migration.job_id, "manifest.json"),
      "utf8",
    ),
  );
  const snapshot = path.join(
    plugin,
    "identity-archives",
    migration.job_id,
    "original",
    bedrock,
    "data",
  );
  assert.deepEqual(await fs.readFile(snapshot), selectedBefore);
  for (const row of manifest.original) {
    if (row.sha256) {
      const bytes = await fs.readFile(
        path.join(
          plugin,
          "identity-archives",
          migration.job_id,
          "original",
          row.path,
        ),
      );
      assert.equal(
        crypto.createHash("sha256").update(bytes).digest("hex"),
        row.sha256,
      );
    }
  }
  for (const suffix of [".dat", ".dat_old"])
    await assert.rejects(
      fs.access(path.join(dataRoot, "data", bedrock + suffix)),
      { code: "ENOENT" },
    );
  result.checks.push(
    "immutable original archives; native source removed; canonical destination verified",
  );
  for (const p of pets)
    await assertPet(
      p,
      p.owner === bedrock ? native(java.name) : plan.archive_owner,
    );
  result.checks.push(
    "loaded and previously unloaded pets retain selected ownership or archive separately",
  );
  const active = await connect(java.name),
    actual = await inspectPlayer(active);
  assert.equal(active.session.profile_id, source.session.profile_id);
  assert.equal(amount(actual.items, "DIAMOND"), 17);
  assert.equal(amount(actual.items, "GOLD_INGOT"), 0);
  assert.equal(amount(actual.ender, "EMERALD"), 3);
  assert.equal(amount(actual.ender, "GOLD_INGOT"), 0);
  assert.equal(actual.level, selectedState.level);
  assert.equal(actual.jumps, 77);
  assert(actual.advancements.includes("minecraft:adventure/arbalistic"));
  assert(
    !actual.advancements.includes("minecraft:husbandry/complete_catalogue"),
  );
  assert(
    active.bot.entity.position.distanceTo(position) < 4,
    "Selected last location did not survive",
  );
  result.checks.push(
    "selected inventory, ender chest, experience, statistics, advancements and last location only",
  );
  const freshPet = { x: base + 128, z: base };
  await consoleCommand(
    `lkjmcfixture pet ${freshPet.x} 160 ${freshPet.z} ${native(java.name)}`,
  );
  await assertPet(freshPet, native(java.name));
  await disconnect(active);
  const web = freshWeb("Web" + tag);
  const noMove = await link(
    web,
    java.session.account_id,
    source.session.profile_id,
  );
  await done(noMove.job_id);
  const after = await connect(java.name);
  assert.equal(after.session.account_id, web);
  assert.equal(amount((await inspectPlayer(after)).items, "DIAMOND"), 17);
  await assertPet(freshPet, native(java.name));
  result.checks.push(
    "web link retains the same native profile and newly tamed pets without a restart",
  );
  const empty = freshWeb("Empty" + tag),
    emptyProfile = sql(
      `SELECT id FROM profiles WHERE account_id='${empty}' AND status='active'`,
    );
  const begin = (
    await api("/internal/v1/game/command", {
      ...after.session,
      request_id: crypto.randomUUID(),
      command: { type: "link_begin" },
    })
  ).result;
  await webCommand(await webSession(empty), {
    type: "link_present",
    code: begin.code,
  });
  const settings = await api("/internal/v1/game/view", {
    ...after.session,
    view: "settings",
    query: {},
  });
  const choice = settings.links
    .find((l) => l.id === begin.id)
    .profiles.findIndex((p) => p.id === emptyProfile);
  assert(choice >= 0);
  const window = (title) =>
    until(
      () =>
        after.bot.currentWindow &&
        JSON.stringify(after.bot.currentWindow.title).includes(title),
      "menu " + title,
      15000,
    );
  after.bot.chat("/menu");
  await window("lkjmc");
  const accountSlot = after.bot.currentWindow.slots.slice(0, 45).findIndex((item) => item?.name === "name_tag");
  assert(accountSlot >= 0, "The contextual menu provides Account");
  await after.bot.clickWindow(accountSlot, 0, 0);
  await window("Account");
  await after.bot.clickWindow(11, 0, 0);
  await window("Account linking");
  await after.bot.clickWindow(12, 0, 0);
  await window("Game data to keep");
  await after.bot.clickWindow(10 + Math.floor(choice / 7) * 9 + choice % 7, 0, 0);
  await window("Confirm game data");
  await after.bot.clickWindow(10, 0, 0);
  const clear = {
    job_id: await until(
      () =>
        sql(
          `SELECT id FROM jobs WHERE kind='identity.migrate' AND payload->>'link_id'='${begin.id}'`,
        ),
      "game menu submitted link",
    ),
  };
  const cleared = await done(clear.job_id);
  assert(
    after.bot._client.ended,
    "Migration did not disconnect the live player",
  );
  result.checks.push(
    "real inventory menu selects a profile, confirms, saves and disconnects before replacement",
  );
  for (const p of [...pets.filter((p) => p.owner === bedrock), freshPet])
    await assertPet(p, cleared.payload.native_plan.archive_owner);
  const reset = await connect(java.name),
    emptyState = await inspectPlayer(reset);
  assert.equal(amount(emptyState.items, "DIAMOND"), 0);
  assert.equal(amount(emptyState.ender, "EMERALD"), 0);
  assert.equal(emptyState.level, 0);
  assert(!emptyState.advancements.includes("minecraft:adventure/arbalistic"));
  assert(reset.bot.entity.position.distanceTo(position) > 10000);
  result.checks.push(
    "choosing a fresh web profile archives prior items and pets, clears native fallback data, and uses a fresh distant start",
  );
  await disconnect(reset);
  for (const p of pets.filter((p) => p.loaded))
    await consoleCommand(
      `execute in minecraft:living run forceload remove ${p.x} ${p.z}`,
    );
  await stop();
  result.pass = true;
  await fs.writeFile(
    path.join(local, `identity-${tag}-result.json`),
    JSON.stringify(result, null, 2),
    { mode: 0o600 },
  );
  console.log(JSON.stringify(result, null, 2));
} finally {
  for (const c of clients) {
    clearInterval(c.timer);
    c.bot.quit();
  }
  await stop();
  await new Promise((resolve) => log.end(resolve));
}
// Mineflayer retains protocol timers after repeated forced disconnects. All assertions,
// evidence writes and the graceful Paper stop above must complete before ending them.
process.exit(0);
