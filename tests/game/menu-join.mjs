// Acceptance helpers for network.mjs's explicit private offline fixture only.
// These are protocol bots, not production Java authentication or Bedrock proof.
import assert from "node:assert/strict";
import crypto from "node:crypto";
import net from "node:net";
import { setTimeout as sleep } from "node:timers/promises";

const token = (item) => item && JSON.stringify(item.components ?? item.nbt ?? {}).includes("menu_launcher");
export async function launcherChecks(c, { until, consoleCommand, lobby, reconnect }) {
  const bot = c.bot;
  await until(() => token(bot.inventory.slots[44]), "tagged lobby book installed");
  assert.equal(bot.inventory.slots[44].name, "book");
  bot.setQuickBarSlot(8);
  const opened = () => until(() => bot.currentWindow, "launcher menu opened", 10000);
  const close = async () => {
    bot.closeWindow(bot.currentWindow);
    await until(() => !bot.currentWindow, "launcher menu closed");
    await sleep(400); // Test distinct gestures outside the deliberate debounce.
  };
  let count = 0;
  bot.on("windowOpen", () => count++);
  bot._client.write("arm_animation", { hand: 0 });
  await opened();
  await close();
  const before = count;
  bot.activateItem();
  bot.activateItem();
  await opened();
  await sleep(500);
  assert.equal(count, before + 1, "double right trigger opens exactly one view");
  await close();
  const block = bot.blockAt(bot.entity.position.offset(0, -1, 0));
  assert(block && block.name !== "air", "lobby floor fixture");
  bot._client.write("block_dig", { status: 0, location: block.position, face: 1, sequence: 0 });
  await opened();
  await close();
  bot.activateBlock(block);
  await opened();
  await close();
  assert.equal(bot.blockAt(block.position).name, block.name, "left-block launcher does not break the floor");
  // Existing players may still own the tagged compass from the prior adapter.
  await consoleCommand(lobby, `item replace entity ${c.name} hotbar.8 with minecraft:compass[minecraft:custom_data={PublicBukkitValues:{"lkjmc:menu_launcher":1b}}]`);
  await until(() => bot.inventory.slots[44]?.name === "compass" && token(bot.inventory.slots[44]), "legacy tagged compass fixture");
  bot.activateItem();
  await opened();
  await close();
  const entityTag = "ux_menu_" + c.name;
  await consoleCommand(lobby, `execute at ${c.name} run summon minecraft:cow ~2 ~ ~ {NoAI:1b,Tags:["${entityTag}"]}`);
  const cow = await until(() => Object.values(bot.entities).find((entity) => entity.name === "cow" && entity.position.distanceTo(bot.entity.position) < 4), "menu entity fixture");
  bot.activateEntity(cow);
  await opened();
  await close();
  await consoleCommand(lobby, `kill @e[type=minecraft:cow,tag=${entityTag}]`);
  await consoleCommand(lobby, `item replace entity ${c.name} weapon.offhand from entity ${c.name} hotbar.8`);
  bot.activateItem(true);
  await opened();
  await close();
  await consoleCommand(lobby, `item replace entity ${c.name} weapon.offhand with minecraft:stick`);
  bot._client.write("block_dig", { status: 6, location: { x: 0, y: 0, z: 0 }, face: 0, sequence: 0 });
  await sleep(300);
  assert(token(bot.inventory.slots[44]) && bot.inventory.slots[45]?.name === "stick", "hand swap involving launcher is cancelled");
  await bot.clickWindow(44, 0, 0); // Native player inventory slot, window 0.
  await opened();
  await close();
  await bot.clickWindow(44, 1, 0);
  await opened();
  await close();
  await bot.clickWindow(44, 0, 2); // Number-key swap must not move the owned token.
  await sleep(300);
  assert(token(bot.inventory.slots[44]));
  bot._client.write("block_dig", { status: 4, location: { x: 0, y: 0, z: 0 }, face: 0, sequence: 0 });
  await sleep(300);
  assert(token(bot.inventory.slots[44]), "owned launcher cannot be dropped");
  // A deliberately named ordinary book has no PDC authority.
  await consoleCommand(lobby, `item replace entity ${c.name} hotbar.0 with minecraft:book[minecraft:custom_name='"Game menu"']`);
  bot.setQuickBarSlot(0);
  bot.activateItem();
  await sleep(500);
  assert(!bot.currentWindow, "user-crafted named item is not a launcher");
  // Slot 8 may contain real property. Rejoin must install into an empty slot.
  await consoleCommand(lobby, `item replace entity ${c.name} hotbar.8 with minecraft:diamond 3`);
  const next = await reconnect(c);
  await until(() => next.bot.inventory.slots[44]?.name === "diamond", "ordinary slot restored");
  assert.equal(next.bot.inventory.slots[44].count, 3);
  assert.equal(next.bot.inventory.slots[36]?.name, "book", "ordinary named book survives");
  await until(() => next.bot.inventory.slots.some(token), "launcher uses an empty slot");
  assert.equal(next.bot.inventory.slots.filter(token).length, 1, "exactly one owned token");
  console.log("PASS launcher left/right AIR/BLOCK, inventory left/right, legacy compass, offhand/entity, debounce, protected swap/drop and ordinary-item preservation");
  return next;
}

export async function sleepingJoinChecks(c, helpers) {
  const { until, submit, job, session, ids, fixtureSql, startOfficial, reconnect } = helpers;
  const name = "official development";
  await fixtureSql(`UPDATE servers SET observed='stopped',desired='stopped',last_observed_at=now(),error=NULL WHERE id='${ids.official}'`);
  const first = await submit(c, { type: "server_join", id: ids.official });
  const duplicate = await submit(c, { type: "server_join", id: ids.official });
  assert.equal(duplicate.job_id, first.job_id, "duplicate sleeping target coalesces");
  await until(() => c.messages.some((m) => m.includes(`Waking ${name}`)), "waking chat feedback");
  c.bot.chat("/go cancel");
  await until(async () => (await job(c, first.job_id)).state === "cancelled", "waiting request cancelled");
  await until(() => c.messages.some((m) => m.includes("Travel cancelled")), "cancellation chat feedback");
  assert.equal((await session(c)).server_id, ids.lobby);
  const old = await submit(c, { type: "server_join", id: ids.official });
  const chosen = await submit(c, { type: "server_join", id: ids.lobby });
  await until(async () => (await job(c, old.job_id)).state === "cancelled", "new target supersedes waiting request");
  await until(async () => (await job(c, chosen.job_id)).state === "succeeded", "new chosen lobby observed");
  const stale = await submit(c, { type: "server_join", id: ids.official });
  const oldSession = (await session(c)).session_id;
  const next = await reconnect(c);
  assert.notEqual((await session(next)).session_id, oldSession);
  await until(async () => (await job(next, stale.job_id)).state === "failed", "old session job fails after reconnect");
  await sleep(6500);
  assert.equal((await session(next)).server_id, ids.lobby, "stale request cannot move the reconnected person");
  const messageStart = next.messages.length;
  const travel = await submit(next, { type: "server_join", id: ids.official });
  await until(() => next.messages.slice(messageStart).some((m) => m.includes(`Waking ${name}`)), "sleeping target waking");
  assert.equal((await session(next)).server_id, ids.lobby);
  // This fixture manually performs the host's start after Core has durably queued it.
  // Paper was absent up to this point; verified readiness comes from the real adapter.
  await fixtureSql(`UPDATE servers SET observed='starting',last_observed_at=now() WHERE id='${ids.official}'`);
  const officialStarting = startOfficial();
  await until(() => next.messages.slice(messageStart).some((m) => m.includes(`Preparing ${name}`)), "preparing transition feedback");
  assert.notEqual((await job(next, travel.job_id)).state, "succeeded", "preparing is not success");
  const official = await officialStarting;
  const done = await until(async () => {
    const status = await job(next, travel.job_id);
    return ["succeeded", "failed"].includes(status.state) && status;
  }, "sleeping server eventual transfer", 610000);
  assert.equal(done.state, "succeeded", JSON.stringify(done));
  assert.equal(done.result.session_id, (await session(next)).session_id);
  await until(() => next.world === "minecraft:living", "actual protocol world arrival");
  await until(() => next.messages.slice(messageStart).some((m) => m.includes(`Arrived at ${name}`)), "actual success chat");
  assert(!next.ended);
  const progress = next.messages.slice(messageStart).filter((m) => /Waking |Preparing |Saving and connecting/.test(m));
  assert(progress.length <= 34, "bounded progress, not every poll");
  console.log("PASS sleeping -> waking -> preparing -> real Paper readiness -> actual destination and chat; duplicate, supersession, cancellation and stale reconnect");
  return { client: next, official };
}

export async function failedJoinChecks(c, { fixtureSql, submit, job, until, session, ids, destinationPort = 25799, destinationName = "unavailable fixture", move }) {
  // A closed loopback destination exercises an ordinary connection failure while
  // the real source Paper performs its signed departure/save and release.
  const target = crypto.randomUUID();
  const player = await session(c);
  await fixtureSql(`INSERT INTO servers(id,name,kind,visibility,desired,observed,version,software,memory_mib,cpu_millis,storage_mib,last_observed_at,capabilities,address) VALUES('${target}','${destinationName}','legacy','public','running','running','fixture','paper',2048,2000,10240,now(),'{"proxy_join":true}', '127.0.0.1:${destinationPort}')`);
  const before = c.messages.length;
  const travel = await submit(c, { type: "server_join", id: target });
  const done = await until(async () => {
    const j = await job(c, travel.job_id);
    return j.state === "failed" && j;
  }, "closed destination fails promptly", 45000);
  assert.equal(done.result.effect, "none");
  assert.equal((await session(c)).session_id, player.session_id);
  const actual = (await session(c)).server_id;
  assert([ids.official, ids.lobby].includes(actual), "failure retains the source or returns safely to the lobby");
  assert(!c.ended, "ordinary connection failure keeps the player connected");
  await until(() => c.messages.slice(before).some((m) => m.includes(`Travel to ${destinationName} failed:`)), "actionable target-specific failure chat");
  const source = c.bot.entity.position.clone();
  c.bot.setControlState("forward", true);
  await until(() => c.bot.entity.position.distanceTo(source) > .2, "signed departure lock released", 10000);
  c.bot.setControlState("forward", false);
  await fixtureSql(`UPDATE servers SET visibility='private' WHERE id='${target}'`);
  if (actual === ids.lobby) await move(c, ids.official);
  console.log("PASS connection failure feedback, source/lobby retained and signed departure released");
}

export async function timeoutJoinChecks(c, helpers) {
  const sockets = new Set();
  const stalled = net.createServer((socket) => {
    sockets.add(socket);
    socket.on("close", () => sockets.delete(socket));
    // Accept TCP but never reply to the Minecraft handshake. Velocity must reach
    // its bounded read timeout and release the source instead of disconnecting it.
  });
  await new Promise((resolve, reject) => {
    stalled.once("error", reject);
    stalled.listen(0, "127.0.0.1", resolve);
  });
  try {
    await failedJoinChecks(c, { ...helpers, destinationPort: stalled.address().port,
      destinationName: "timeout fixture" });
    console.log("PASS stalled destination times out with failure chat and signed release; player remains connected");
  } finally {
    for (const socket of sockets) socket.destroy();
    await new Promise((resolve) => stalled.close(resolve));
  }
}
